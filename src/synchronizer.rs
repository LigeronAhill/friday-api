use std::sync::Arc;

use crate::{
    models::ApiClients,
    utils::{convert_to_create, convert_to_update, pause, MsData, WooData},
};
use chrono::{Datelike, TimeZone};
use rust_woocommerce as woo;
use tokio::sync::mpsc;

pub async fn run(api_clients: ApiClients) {
    loop {
        let now = chrono::Utc::now();
        tracing::info!("Сейчас {current}", current = now.to_rfc3339());
        if let Some(tommorow) = now.checked_add_days(chrono::Days::new(1)) {
            let midnight = chrono::Utc
                .with_ymd_and_hms(tommorow.year(), tommorow.month(), tommorow.day(), 0, 0, 0)
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
            tracing::info!("Начинаю синхронизацию");
            match MsData::get(api_clients.ms_client.clone()).await {
                Ok(ms_data) => {
                    let ms_data_instance = ms_data.clone();
                    let swc = api_clients.safira_woo_client.clone();
                    tokio::spawn(async move {
                        sync(ms_data_instance, swc).await;
                    });
                    let lwc = api_clients.lc_woo_client.clone();
                    tokio::spawn(async move {
                        sync(ms_data, lwc).await;
                    });
                }
                Err(e) => {
                    tracing::error!("Ошибка при получении данных из Мой Склад {e:?}");
                }
            }
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
