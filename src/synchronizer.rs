use std::sync::Arc;

use crate::{
    models::ApiClients,
    utils::{convert_to_create, convert_to_update, pause, MsData, WooData},
};
use chrono::Timelike;
use rust_woocommerce as woo;
use tokio::sync::mpsc;

pub async fn run(api_clients: ApiClients) {
    loop {
        let now = chrono::Utc::now();
        let hour = now.hour();
        let mins = now.minute();
        if hour == 0 && mins == 0 {
            match MsData::get(api_clients.ms_client.clone()).await {
                Ok(ms_data) => {
                    tokio::spawn(sync(ms_data.clone(), api_clients.safira_woo_client.clone()));
                    tokio::spawn(sync(ms_data, api_clients.lc_woo_client.clone()));
                    pause(24).await;
                }
                Err(e) => {
                    tracing::error!("Ошибка при получении данных из Мой Склад {e:?}");
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        } else {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }
}

async fn sync(ms_data: MsData, woo_client: Arc<woo::ApiClient>) {
    let host = woo_client.base_url();
    loop {
        tracing::info!("Синхронизирую {host} с Мой Склад");
        match WooData::get(woo_client.clone()).await {
            Ok(woo_data) => {
                let mut products_to_create = Vec::new();
                let mut products_to_update = Vec::new();
                let mut products_to_delete = Vec::new();
                let (create_tx, mut create_rx) = mpsc::unbounded_channel();
                let (update_tx, mut update_rx) = mpsc::unbounded_channel();
                let (delete_tx, mut delete_rx) = mpsc::unbounded_channel();
                let ms_data = ms_data.clone();
                tokio::spawn(async move {
                    for (ms_article, ms_product) in ms_data.products.clone() {
                        if let Some(woo_product) = woo_data.products.get(&ms_article) {
                            // update woo product
                            if let Some(converted) =
                                convert_to_update(&ms_product, woo_product, &ms_data, &woo_data)
                            {
                                // products_to_update.push(converted)
                                if let Err(e) = update_tx.send(converted) {
                                    tracing::error!("Error sending {ms_article} {e:?}");
                                }
                            }
                        } else {
                            // create woo product
                            if let Some(converted) =
                                convert_to_create(&ms_product, &ms_data, &woo_data)
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
                    tracing::info!(
                        "Получилось {len} товаров для создания в {host}",
                        len = products_to_create.len()
                    );
                    if let Err(e) = woo_client
                        .batch_create::<woo::Product, _>(products_to_create)
                        .await
                    {
                        tracing::error!("{e:?}");
                        pause(1).await;
                        continue;
                    }
                }
                if !products_to_update.is_empty() {
                    tracing::info!(
                        "Получилось {len} товаров для обновления в {host}",
                        len = products_to_update.len()
                    );
                    if let Err(e) = woo_client
                        .batch_update::<woo::Product, _>(products_to_update)
                        .await
                    {
                        tracing::error!("{e:?}");
                        pause(1).await;
                        continue;
                    }
                }
                if !products_to_delete.is_empty() {
                    tracing::info!(
                        "Получилось {len} товаров для удаления в {host}",
                        len = products_to_delete.len()
                    );
                    if let Err(e) = woo_client
                        .batch_delete::<woo::Product>(products_to_delete)
                        .await
                    {
                        tracing::error!("{e:?}");
                        pause(1).await;
                        continue;
                    }
                }
                break;
            }
            Err(e) => {
                tracing::error!("Ошибка получения данных из WooCommerce '{host} -> {e:?}");
                pause(1).await;
            }
        }
    }
}
