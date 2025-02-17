use std::{collections::HashMap, sync::Arc};

use crate::{
    models::{MsEvent, Stock},
    storage::{EventsStorage, StockStorage},
    utils::{convert_to_create, convert_to_update, pause, MsData, WooData},
    AppError,
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;

pub struct Eventer {
    ms_client: Arc<rust_moysklad::MoySkladApiClient>,
    safira_client: Arc<rust_woocommerce::ApiClient>,
    events_storage: Arc<EventsStorage>,
    stock_storage: Arc<StockStorage>,
}
impl Eventer {
    pub fn new(
        ms_client: Arc<rust_moysklad::MoySkladApiClient>,
        safira_client: Arc<rust_woocommerce::ApiClient>,
        events_storage: Arc<EventsStorage>,
        stock_storage: Arc<StockStorage>,
    ) -> Arc<Self> {
        Arc::new(Self {
            ms_client,
            safira_client,
            events_storage,
            stock_storage,
        })
    }
    pub async fn run(self: Arc<Self>) -> anyhow::Result<()> {
        let (tx, rx) = channel(10);
        tokio::spawn(generator(tx, self.clone().events_storage.clone()));
        tokio::spawn(clean(self.clone().events_storage.clone()));
        self.processor(rx).await?;
        Ok(())
    }
    async fn processor(
        self: Arc<Self>,
        mut rx: Receiver<Vec<MsEvent>>,
    ) -> anyhow::Result<()> {
        while let Some(events) = rx.recv().await {
            let ms_data = self.clone().get_ms_data().await?;
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
            let woo_data = self.clone().get_woo_data().await?;
            process_events(
                &events,
                &ms_data,
                &woo_data,
                self.clone().safira_client.clone(),
                &stock,
                self.clone().events_storage.clone(),
            )
                .await?;
        }
        Ok(())
    }
    async fn get_ms_data(self: Arc<Self>) -> anyhow::Result<MsData> {
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
    async fn ms_currencies(
        self: Arc<Self>,
        tx: oneshot::Sender<Vec<rust_moysklad::Currency>>,
    ) -> anyhow::Result<()> {
        let result = self.ms_client.get_all::<rust_moysklad::Currency>().await?;
        if let Err(e) = tx.send(result) {
            tracing::error!("Ошибка отправки событий в очередь -> {e:?}");
        }
        Ok(())
    }
    async fn ms_countries(
        self: Arc<Self>,
        tx: oneshot::Sender<Vec<rust_moysklad::Country>>,
    ) -> anyhow::Result<()> {
        let result = self.ms_client.get_all::<rust_moysklad::Country>().await?;
        if let Err(e) = tx.send(result) {
            tracing::error!("Ошибка отправки событий в очередь -> {e:?}");
        }
        Ok(())
    }
    async fn ms_uoms(self: Arc<Self>, tx: oneshot::Sender<Vec<rust_moysklad::Uom>>) -> anyhow::Result<()> {
        let result = self.ms_client.get_all::<rust_moysklad::Uom>().await?;
        if let Err(e) = tx.send(result) {
            tracing::error!("Ошибка отправки событий в очередь -> {e:?}");
        }
        Ok(())
    }
    async fn ms_products(
        self: Arc<Self>,
        tx: oneshot::Sender<Vec<rust_moysklad::Product>>,
    ) -> anyhow::Result<()> {
        let result = self.ms_client.get_all::<rust_moysklad::Product>().await?;
        if let Err(e) = tx.send(result) {
            tracing::error!("Ошибка отправки событий в очередь -> {e:?}");
        }
        Ok(())
    }
    async fn get_woo_data(self: Arc<Self>) -> anyhow::Result<WooData> {
        let (products, attributes, categories) = tokio::join!(
            self.clone().woo_products(),
            self.clone().woo_attributes(),
            self.clone().woo_categories(),
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
    async fn woo_products(self: Arc<Self>) -> anyhow::Result<Vec<rust_woocommerce::Product>> {
        let result = self.clone().safira_client.clone().list_all().await?;
        Ok(result)
    }
    async fn woo_attributes(self: Arc<Self>) -> anyhow::Result<Vec<rust_woocommerce::Attribute>> {
        let result = self.clone().safira_client.clone().list_all().await?;
        Ok(result)
    }
    async fn woo_categories(self: Arc<Self>) -> anyhow::Result<Vec<rust_woocommerce::Category>> {
        let result = self.clone().safira_client.clone().list_all().await?;
        Ok(result)
    }
}

async fn process_events(
    events: &[MsEvent],
    ms_data: &MsData,
    woo_data: &WooData,
    woo_client: Arc<rust_woocommerce::ApiClient>,
    stock: &[Stock],
    evens_storage: Arc<EventsStorage>,
) -> crate::Result<()> {
    for event in events {
        process_event(event, ms_data, woo_data, woo_client.clone(), stock).await?;
        evens_storage.process(event.id).await?;
    }
    Ok(())
}

async fn process_event(
    event: &MsEvent,
    ms_data: &MsData,
    woo_data: &WooData,
    woo_client: Arc<rust_woocommerce::ApiClient>,
    stock: &[Stock],
) -> crate::Result<()> {
    let Some(ms_product) = ms_data
        .products
        .iter()
        .find(|(_, p)| p.id == event.product_id)
        .map(|(_, p)| p)
    else {
        return Ok(());
    };
    match event.action.as_str() {
        "CREATE" => {
            if let Some(woo_converted_product_to_create) =
                convert_to_create(ms_product, ms_data, woo_data, stock)
            {
                woo_client
                    .create::<rust_woocommerce::Product>(woo_converted_product_to_create)
                    .await
                    .map_err(|e| AppError::Custom(e.to_string()))?;
            }
        }
        "UPDATE" => {
            if event.fields.len() == 1 && event.fields[0] == "Наличие" {
                return Ok(());
            }
            match woo_data.products.get(
                &ms_product
                    .article
                    .clone()
                    .map(|a| a.to_uppercase())
                    .unwrap_or_default(),
            ) {
                Some(woo_product) => {
                    if let Some(woo_converted_product_to_update) =
                        convert_to_update(ms_product, woo_product, ms_data, woo_data, stock)
                    {
                        woo_client
                            .update::<rust_woocommerce::Product>(
                                woo_product.id,
                                woo_converted_product_to_update,
                            )
                            .await
                            .map_err(|e| AppError::Custom(e.to_string()))?;
                    }
                }
                None => {
                    if let Some(woo_converted_product_to_create) =
                        convert_to_create(ms_product, ms_data, woo_data, stock)
                    {
                        woo_client
                            .create::<rust_woocommerce::Product>(woo_converted_product_to_create)
                            .await
                            .map_err(|e| AppError::Custom(e.to_string()))?;
                    }
                }
            }
        }
        "DELETE" => {
            if let Some(woo_product) = woo_data
                .products
                .get(&ms_product.article.clone().unwrap_or_default())
            {
                let id = woo_product.id;
                woo_client
                    .delete::<rust_woocommerce::Product>(id)
                    .await
                    .map_err(|e| AppError::Custom(e.to_string()))?;
            }
        }
        _ => {}
    }
    Ok(())
}

async fn generator(tx: Sender<Vec<MsEvent>>, events_storage: Arc<EventsStorage>) {
    loop {
        let tx = tx.clone();
        match events_storage.get().await {
            Ok(events) => {
                if !events.is_empty() {
                    if let Err(e) = tx.send(events).await {
                        tracing::warn!("Ошибка отправки событий в очередь -> {e:?}");
                    }
                }
            }
            Err(e) => {
                tracing::error!("Ошибка получения событий из базы данных: {e:?}");
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5 * 60)).await;
    }
}

async fn clean(events_storage: Arc<EventsStorage>) {
    loop {
        match events_storage.remove_processed().await {
            Ok(_) => tracing::info!("Обработанные события удалены из базы данных"),
            Err(e) => tracing::error!("Ошибка удаления обработанных событий из базы данных: {e:?}"),
        }
        pause(1).await;
    }
}