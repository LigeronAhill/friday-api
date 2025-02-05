use std::sync::Arc;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    models::{ApiClients, MsEvent},
    storage::EventsStorage,
    utils::{convert_to_create, convert_to_update, pause, MsData, WooData},
    AppError,
};

pub async fn run(api_clients: ApiClients, events_storage: Arc<EventsStorage>) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    // generator
    tokio::spawn(generator(tx, events_storage.clone()));
    // cleaner
    tokio::spawn(clean(events_storage.clone()));
    // processors
    processor(rx, api_clients, events_storage.clone()).await;
}
async fn processor(
    mut rx: UnboundedReceiver<Vec<MsEvent>>,
    api_clients: ApiClients,
    events_storage: Arc<EventsStorage>,
) {
    while let Some(events) = rx.recv().await {
        let mut has_errors = false;
        match MsData::get(api_clients.ms_client.clone()).await {
            Ok(ms_data) => match WooData::get(api_clients.safira_woo_client.clone()).await {
                Ok(safira_woo_data) => {
                    if let Err(e) = process_events(
                        &events,
                        &ms_data,
                        &safira_woo_data,
                        api_clients.safira_woo_client.clone(),
                    )
                    .await
                    {
                        has_errors = true;
                        tracing::error!("{e:?}");
                    }
                }
                Err(e) => {
                    has_errors = true;
                    tracing::error!("{e:?}");
                }
            },
            Err(e) => {
                has_errors = true;
                tracing::error!("{e:?}");
            }
        }
        if !has_errors {
            for event in events {
                match events_storage.process(event.id).await {
                    Ok(_) => tracing::info!("Событие {id} обработано", id = event.id),
                    Err(e) => {
                        tracing::error!("Ошибка при обработке события {id}: {e:?}", id = event.id)
                    }
                }
            }
        }
    }
}

async fn process_events(
    events: &[MsEvent],
    ms_data: &MsData,
    woo_data: &WooData,
    woo_client: Arc<rust_woocommerce::ApiClient>,
) -> crate::Result<()> {
    for event in events {
        process_event(event, ms_data, woo_data, woo_client.clone()).await?;
    }
    Ok(())
}

async fn process_event(
    event: &MsEvent,
    ms_data: &MsData,
    woo_data: &WooData,
    woo_client: Arc<rust_woocommerce::ApiClient>,
) -> crate::Result<()> {
    let Some(ms_product) = ms_data
        .products
        .iter()
        .find(|(_, p)| p.id == event.product_id)
        .map(|(_, p)| p)
    else {
        return Err(AppError::Custom(
            "Нет такого продукта в Мой Склад".to_string(),
        ));
    };
    match event.action.as_str() {
        "CREATE" => {
            if let Some(woo_converted_product_to_create) =
                convert_to_create(ms_product, ms_data, woo_data)
            {
                woo_client
                    .create::<rust_woocommerce::Product>(woo_converted_product_to_create)
                    .await
                    .map_err(|e| AppError::Custom(e.to_string()))?;
            }
        }
        "UPDATE" => {
            match woo_data.products.get(
                &ms_product
                    .article
                    .clone()
                    .map(|a| a.to_uppercase())
                    .unwrap_or_default(),
            ) {
                Some(woo_product) => {
                    if let Some(woo_converted_product_to_update) =
                        convert_to_update(ms_product, woo_product, ms_data, woo_data)
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
                        convert_to_create(ms_product, ms_data, woo_data)
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

async fn generator(tx: UnboundedSender<Vec<MsEvent>>, events_storage: Arc<EventsStorage>) {
    loop {
        match events_storage.get().await {
            Ok(events) => {
                if !events.is_empty() && tx.send(events).is_err() {
                    tracing::error!("Ошибка отправки событий в очередь");
                }
            }
            Err(e) => {
                tracing::error!("Ошибка получения событий из базы данных: {e:?}");
            }
        }
        // pause(1).await;
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}

async fn clean(events_storage: Arc<EventsStorage>) {
    loop {
        match events_storage.remove_processed().await {
            Ok(_) => tracing::info!("Обработанные события удалены из базы данных"),
            Err(e) => tracing::error!("Ошибка удаления обработанных событий из базы данных: {e:?}"),
        }
        pause(24).await;
    }
}
