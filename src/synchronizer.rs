use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    models::Stock,
    storage::StockStorage,
    utils::{convert_to_create, convert_to_update, get_quantity, pause, MsData, WooData},
};
use chrono::{Datelike, TimeZone};
use rust_moysklad as ms;
use rust_woocommerce as woo;
use tokio::sync::oneshot::Sender;
use tokio::sync::{mpsc, oneshot};
const STOCK_ATTRIBUTE_NAME: &str = "Наличие";
const IN_STOCK: &str = "В наличии (2-3 раб. дня)";
const OUT_OF_STOCK: &str = "Под заказ (5-8 недель)";

pub struct Synchronizer {
    ms_client: Arc<ms::MoySkladApiClient>,
    safira_client: Arc<woo::ApiClient>,
    lc_client: Arc<woo::ApiClient>,
    stock_storage: Arc<StockStorage>,
}
impl Synchronizer {
    pub fn new(
        ms_client: Arc<ms::MoySkladApiClient>,
        safira_client: Arc<rust_woocommerce::ApiClient>,
        lc_client: Arc<rust_woocommerce::ApiClient>,
        stock_storage: Arc<StockStorage>,
    ) -> Arc<Self> {
        Arc::new(Self {
            ms_client,
            safira_client,
            lc_client,
            stock_storage,
        })
    }
    async fn sync(self: Arc<Self>) -> Result<()> {
        let mut stock = Vec::new();
        let limit = 500;
        let mut offset = 0;
        loop {
            let temp = self
                .clone()
                .stock_storage
                .clone()
                .get(limit, offset)
                .await?;
            if temp.is_empty() {
                break;
            } else {
                stock.extend(temp);
                offset += limit;
            }
        }
        let (ms_data, safira_data, lc_data) = tokio::join!(
            self.clone().get_ms_data(),
            self.clone().get_woo_data(self.safira_client.base_url()),
            self.clone().get_woo_data(self.lc_client.base_url()),
        );
        let ms_data = ms_data?;
        let products = ms_data.products.values().cloned().collect::<Vec<_>>();
        self.clone().update_ms_stock(&stock, &products).await?;
        let safira_data = safira_data?;
        let lc_data = lc_data?;
        let woos = vec![
            (self.safira_client.base_url(), safira_data),
            (self.lc_client.base_url(), lc_data),
        ];
        let (result_sender, mut result_receiver) = mpsc::unbounded_channel();
        for (base_url, woo_data) in woos {
            let mut products_to_create = Vec::new();
            let mut products_to_update = Vec::new();
            let mut products_to_delete = Vec::new();
            let (create_tx, mut create_rx) = mpsc::unbounded_channel();
            let (update_tx, mut update_rx) = mpsc::unbounded_channel();
            let (delete_tx, mut delete_rx) = mpsc::unbounded_channel();
            let ms_data = ms_data.clone();
            let current_stock = stock.clone();
            tokio::spawn(async move {
                for (ms_article, ms_product) in ms_data.products.clone() {
                    if let Some(woo_product) = woo_data.products.get(&ms_article) {
                        // update woo product
                        if let Some(converted) = convert_to_update(
                            &ms_product,
                            woo_product,
                            &ms_data,
                            &woo_data,
                            &current_stock,
                        ) {
                            // products_to_update.push(converted)
                            if let Err(e) = update_tx.send(converted) {
                                tracing::error!("Error sending {ms_article} {e:?}");
                            }
                        }
                    } else {
                        // create woo product
                        if let Some(converted) =
                            convert_to_create(&ms_product, &ms_data, &woo_data, &current_stock)
                        {
                            // products_to_create.push(converted)
                            if let Err(e) = create_tx.send(converted) {
                                tracing::error!("Error sending {ms_article} {e:?}");
                            }
                        }
                    }
                }
                for (sku, product) in woo_data.products.iter() {
                    if !ms_data.products.contains_key(sku) {
                        // delete woo product
                        if let Err(e) = delete_tx.send(product.id) {
                            tracing::error!("Error sending {sku} {e:?}");
                        }
                    }
                }
            });

            while let Some(create) = create_rx.recv().await {
                products_to_create.push(create)
            }
            while let Some(update) = update_rx.recv().await {
                products_to_update.push(update)
            }
            while let Some(delete) = delete_rx.recv().await {
                products_to_delete.push(delete)
            }
            if !products_to_create.is_empty() {
                let sender = result_sender.clone();
                let woo_client = self.clone().woo_client(base_url.clone());
                tokio::spawn(async move {
                    let result = woo_client
                        .batch_create::<woo::Product, _>(products_to_create)
                        .await;
                    if let Err(e) = sender.send(result) {
                        tracing::error!("Error sending {e:?}");
                    }
                });
            }
            if !products_to_update.is_empty() {
                let sender = result_sender.clone();
                let woo_client = self.clone().woo_client(base_url.clone());
                tokio::spawn(async move {
                    let result = woo_client.batch_update(products_to_update).await;
                    if let Err(e) = sender.send(result) {
                        tracing::error!("Error sending {e:?}");
                    }
                });
            }
            if !products_to_delete.is_empty() {
                let sender = result_sender.clone();
                let woo_client = self.clone().woo_client(base_url);
                tokio::spawn(async move {
                    let result = woo_client
                        .batch_delete::<woo::Product>(products_to_delete)
                        .await;
                    if let Err(e) = sender.send(result) {
                        tracing::error!("Error sending {e:?}");
                    }
                });
            }
        }
        drop(result_sender);
        while let Some(result) = result_receiver.recv().await {
            if let Err(e) = result {
                tracing::error!("{e:?}");
            }
        }
        Ok(())
    }
    async fn update_ms_stock(
        self: Arc<Self>,
        stock: &[Stock],
        products: &[rust_moysklad::Product],
    ) -> Result<()> {
        let in_stock_attribute = get_stock_attribute(products, IN_STOCK)
            .ok_or(anyhow::anyhow!("Не найден атрибут В наличии"))?;
        let out_of_stock_attribute = get_stock_attribute(products, OUT_OF_STOCK)
            .ok_or(anyhow::anyhow!("Не найден атрибут Нет в наличии"))?;
        let products_to_update = products
            .iter()
            .flat_map(|ms_product| {
                let ms_sku = ms_product.article.clone()?;
                let meta = ms_product.meta.clone();
                let ms_attributes = ms_product.attributes.clone()?;
                let stock_attr = ms_attributes
                    .iter()
                    .find(|a| a.name == STOCK_ATTRIBUTE_NAME)?;
                let value = match stock_attr.value.clone() {
                    rust_moysklad::AttributeValue::Custom(v) => v.name,
                    _ => String::new(),
                };
                let is_in_stock = get_quantity(&ms_sku, stock) > 2.0;
                if !is_in_stock && value != OUT_OF_STOCK {
                    let upd = rust_moysklad::Product::update()
                        .meta(meta)
                        .attribute(out_of_stock_attribute.clone())
                        .build();
                    Some(upd)
                } else if is_in_stock && value != IN_STOCK {
                    let upd = rust_moysklad::Product::update()
                        .meta(meta)
                        .attribute(in_stock_attribute.clone())
                        .build();
                    Some(upd)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if !products_to_update.is_empty() {
            tracing::info!(
                "Получилось {} продуктов для обновления в Мой Склад",
                products_to_update.len()
            );
            let updated: Vec<rust_moysklad::Product> = self
                .clone()
                .ms_client
                .clone()
                .batch_create_update(products_to_update)
                .await?;
            tracing::info!("Обновлено {} продуктов в Мой Склад", updated.len());
        } else {
            tracing::info!("Наличие в Мой Склад актуально")
        }
        Ok(())
    }
    async fn get_ms_data(self: Arc<Self>) -> Result<MsData> {
        let (currencies_sender, currencies_receiver) = oneshot::channel();
        let (countries_sender, countries_receiver) = oneshot::channel();
        let (uoms_sender, uoms_receiver) = oneshot::channel();
        let (products_sender, products_receiver) = oneshot::channel();
        tokio::spawn(self.clone().ms_currencies(currencies_sender));
        tokio::spawn(self.clone().ms_countries(countries_sender));
        tokio::spawn(self.clone().ms_uoms(uoms_sender));
        tokio::spawn(self.clone().ms_products(products_sender));
        let currencies = currencies_receiver.await?;
        let countries = countries_receiver.await?;
        let uoms = uoms_receiver.await?;
        let products_vec = products_receiver.await?;
        let mut products = HashMap::new();
        for product in products_vec {
            if let Some(sku) = product.article.clone() {
                products.insert(sku.to_uppercase(), product.clone());
            }
        }
        let result = MsData {
            currencies,
            countries,
            uoms,
            products,
        };
        Ok(result)
    }
    async fn get_woo_data(self: Arc<Self>, base_url: String) -> Result<WooData> {
        let (products, attributes, categories) = tokio::join!(
            self.clone().woo_products(base_url.clone()),
            self.clone().woo_attributes(base_url.clone()),
            self.clone().woo_categories(base_url),
        );
        let products_vec = products?;
        let attributes_vec = attributes?;
        let categories_vec = categories?;
        let mut products = HashMap::new();
        for product in products_vec {
            products.insert(product.sku.to_uppercase(), product);
        }
        let mut attributes = HashMap::new();
        for attribute in attributes_vec {
            attributes.insert(attribute.name.clone(), attribute);
        }
        let mut categories = HashMap::new();
        for category in categories_vec {
            categories.insert(category.name.clone(), category);
        }
        Ok(WooData {
            products,
            attributes,
            categories,
        })
    }
    fn woo_client(self: Arc<Self>, base_url: String) -> Arc<woo::ApiClient> {
        let lc = self.clone().lc_client.clone();
        if lc.base_url() == base_url {
            lc
        } else {
            self.clone().safira_client.clone()
        }
    }
    async fn woo_products(self: Arc<Self>, base_url: String) -> Result<Vec<woo::Product>> {
        let client = self.woo_client(base_url);
        let result = client.list_all().await?;
        Ok(result)
    }
    async fn woo_attributes(self: Arc<Self>, base_url: String) -> Result<Vec<woo::Attribute>> {
        let client = self.woo_client(base_url);
        let result = client.list_all().await?;
        Ok(result)
    }
    async fn woo_categories(self: Arc<Self>, base_url: String) -> Result<Vec<woo::Category>> {
        let client = self.woo_client(base_url);
        let result = client.list_all().await?;
        Ok(result)
    }
    async fn ms_currencies(self: Arc<Self>, tx: Sender<Vec<ms::Currency>>) -> Result<()> {
        let result = self.ms_client.get_all::<ms::Currency>().await?;
        tx.send(result).unwrap();
        Ok(())
    }
    async fn ms_countries(self: Arc<Self>, tx: Sender<Vec<ms::Country>>) -> Result<()> {
        let result = self.ms_client.get_all::<ms::Country>().await?;
        tx.send(result).unwrap();
        Ok(())
    }
    async fn ms_uoms(self: Arc<Self>, tx: Sender<Vec<ms::Uom>>) -> Result<()> {
        let result = self.ms_client.get_all::<ms::Uom>().await?;
        tx.send(result).unwrap();
        Ok(())
    }
    async fn ms_products(self: Arc<Self>, tx: Sender<Vec<ms::Product>>) -> Result<()> {
        let result = self.ms_client.get_all::<ms::Product>().await?;
        tx.send(result).unwrap();
        Ok(())
    }
    pub async fn run(self: Arc<Self>) {
        if let Err(e) = self.clone().sync().await {
            tracing::error!("{e:?}");
        } else {
            tracing::info!("Сайты синхронизированы");
        }
        let now = chrono::Utc::now();
        tracing::info!("Сейчас {current}", current = now.to_rfc3339());
        let tomorrow = now.checked_add_days(chrono::Days::new(1)).unwrap();
        let midnight = chrono::Utc
            .with_ymd_and_hms(tomorrow.year(), tomorrow.month(), tomorrow.day(), 0, 0, 0)
            .unwrap();
        tracing::info!("Ближайшая полночь: {m}", m = midnight.to_rfc3339());
        let delta = midnight - now;
        tracing::info!("Дельта: {delta:?}");
        let duration = delta.num_seconds();
        let h = duration / 60 / 60;
        let m = duration / 60 % 60;
        let s = duration % 60;
        tracing::info!("Буду ждать {h} часов {m} минут {s} секунд");
        tokio::time::sleep(tokio::time::Duration::from_secs(duration as u64)).await;
        loop {
            tracing::info!("Начинаю синхронизацию");
            if let Err(e) = self.clone().sync().await {
                tracing::error!("{e:?}");
                pause(1).await;
            } else {
                tracing::info!("Сайты синхронизированы");
                pause(24).await;
            }
        }
    }
}
fn get_stock_attribute(
    ms_products: &[rust_moysklad::Product],
    needed_value: &str,
) -> Option<rust_moysklad::Attribute> {
    ms_products
        .iter()
        .find(|p| {
            p.attributes.clone().is_some_and(|attributes| {
                attributes
                    .iter()
                    .find(|a| a.name == STOCK_ATTRIBUTE_NAME)
                    .is_some_and(|a| {
                        let val = match a.value.clone() {
                            rust_moysklad::AttributeValue::Custom(v) => v.name,
                            _ => String::new(),
                        };
                        val == needed_value
                    })
            })
        })
        .and_then(|p| {
            p.attributes
                .clone()?
                .into_iter()
                .find(|a| a.name == STOCK_ATTRIBUTE_NAME)
        })
}
