use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::{
    models::Stock,
    storage::StockStorage,
    utils::{convert_to_create, convert_to_update, get_quantity, MsData, WooData},
};
use rust_moysklad as ms;
use rust_woocommerce as woo;
const STOCK_ATTRIBUTE_NAME: &str = "Наличие";
const IN_STOCK: &str = "В наличии (2-3 раб. дня)";
const OUT_OF_STOCK: &str = "Под заказ (5-8 недель)";

pub struct Synchronizer {
    ms_client: Arc<ms::MoySkladApiClient>,
    safira_client: Arc<woo::ApiClient>,
    stock_storage: Arc<StockStorage>,
}
impl Synchronizer {
    pub fn new(
        ms_client: Arc<ms::MoySkladApiClient>,
        safira_client: Arc<rust_woocommerce::ApiClient>,
        stock_storage: Arc<StockStorage>,
    ) -> Arc<Self> {
        Arc::new(Self {
            ms_client,
            safira_client,
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
        info!("Получаю данные из Мой Склад");
        let ms_data = self.clone().get_ms_data().await?;
        let safira_data = self.clone().get_woo_data().await?;
        let products = ms_data.products.values().cloned().collect::<Vec<_>>();
        info!(
            "Получено {len} продуктов из Мой Склад для обновления",
            len = products.len()
        );
        self.clone().update_ms_stock(&stock, &products).await?;
        info!("Получаю данные от safira.club");
        info!(
            "Получено {len} продуктов из Сафира для обновления",
            len = safira_data.products.len()
        );
        info!("Данные от safira.club получены успешно");
        info!("Синхронизирую safira.club");
        let mut products_to_create = Vec::new();
        let mut products_to_update = Vec::new();
        let mut products_to_delete = Vec::new();
        let current_stock = stock.clone();
        for (ms_article, ms_product) in ms_data.products.clone() {
            if let Some(woo_product) = safira_data.products.get(&ms_article) {
                // update woo product
                if let Some(converted) = convert_to_update(
                    &ms_product,
                    woo_product,
                    &ms_data,
                    &safira_data,
                    &current_stock,
                ) {
                    products_to_update.push(converted)
                }
            } else {
                // create woo product
                if let Some(converted) =
                    convert_to_create(&ms_product, &ms_data, &safira_data, &current_stock)
                {
                    products_to_create.push(converted)
                }
            }
        }
        for (sku, product) in safira_data.products.iter() {
            if !ms_data.products.contains_key(sku)
                || ms_data
                    .products
                    .get(sku.as_str())
                    .as_ref()
                    .is_some_and(|p| p.archived.as_ref().is_some_and(|a| *a))
            {
                // delete woo product
                products_to_delete.push(product.id);
            }
        }

        let mut count = 0;

        if !products_to_create.is_empty() {
            info!(
                "Получено {} позиций для создания в safira.club",
                products_to_create.len()
            );
            let result = self
                .safira_client
                .batch_create::<woo::Product, _>(products_to_create)
                .await?;
            info!("Создано {len} позиций в safira.club", len = result.len());
            count += result.len();
        } else {
            info!("Нет позиций для создания в safira.club");
        }
        if !products_to_update.is_empty() {
            info!(
                "Получено {} позиций для обновления в safira.club",
                products_to_update.len()
            );
            let result: Vec<woo::Product> =
                self.safira_client.batch_update(products_to_update).await?;
            info!("Обновлено {len} позиций в safira.club", len = result.len());
            count += result.len();
        } else {
            info!("Нет позиций для обновления в safira.club");
        }
        if !products_to_delete.is_empty() {
            info!(
                "Получено {} позиций для удаления в safira.club",
                products_to_delete.len()
            );
            let result = self
                .safira_client
                .batch_delete::<woo::Product>(products_to_delete)
                .await?;
            info!("Удалено {len} позиций в safira.club", len = result.len());
            count += result.len();
        } else {
            info!("Нет позиций для удаления в safira.club");
        }
        info!("Всего позиций синхронизировано: {count}");

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
        let currencies = self.ms_currencies().await?;
        let countries = self.ms_countries().await?;
        let uoms = self.ms_uoms().await?;
        let products_vec = self.ms_products().await?;
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
    async fn get_woo_data(self: Arc<Self>) -> Result<WooData> {
        let products_vec = self.clone().woo_products().await?;
        let attributes_vec = self.clone().woo_attributes().await?;
        let categories_vec = self.clone().woo_categories().await?;
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
    async fn woo_products(self: Arc<Self>) -> Result<Vec<woo::Product>> {
        let result = self.safira_client.list_all().await?;
        Ok(result)
    }
    async fn woo_attributes(self: Arc<Self>) -> Result<Vec<woo::Attribute>> {
        let result = self.safira_client.list_all().await?;
        Ok(result)
    }
    async fn woo_categories(self: Arc<Self>) -> Result<Vec<woo::Category>> {
        let result = self.safira_client.list_all().await?;
        Ok(result)
    }
    async fn ms_currencies(&self) -> Result<Vec<ms::Currency>> {
        let result = self.ms_client.get_all::<ms::Currency>().await?;
        Ok(result)
    }
    async fn ms_countries(&self) -> Result<Vec<ms::Country>> {
        let result = self.ms_client.get_all::<ms::Country>().await?;
        Ok(result)
    }
    async fn ms_uoms(&self) -> Result<Vec<ms::Uom>> {
        let result = self.ms_client.get_all::<ms::Uom>().await?;
        Ok(result)
    }
    async fn ms_products(&self) -> Result<Vec<ms::Product>> {
        let result = self.ms_client.get_all::<ms::Product>().await?;
        Ok(result)
    }
    // async fn updated_ms_products(&self, last_update: chrono::NaiveDateTime) -> Result<Vec<ms::Product>> {
    //     let lu = last_update.to_string();
    //     let fo = ms::FilterOperator::GreaterThan;
    //     let result = self.ms_client.filter::<ms::Product>("updated", fo, lu).await?;
    //     Ok(result)
    // }
    pub async fn run(self: Arc<Self>) {
        while let Err(e) = self.clone().sync().await {
            tracing::error!("Ошибка синхронизации: --> {e:?}");
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
        loop {
            tracing::info!("Начинаю синхронизацию");
            if let Err(e) = self.clone().sync().await {
                tracing::error!("{e:?}");
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            } else {
                tracing::info!("Сайт синхронизирован");
                tokio::time::sleep(tokio::time::Duration::from_secs(6 * 60 * 60)).await;
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
