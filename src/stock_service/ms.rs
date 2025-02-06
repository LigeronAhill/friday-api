use std::sync::Arc;

use rust_moysklad::MoySkladApiClient;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::models::Stock;

const STOCK_ATTRIBUTE_NAME: &str = "Наличие";
const IN_STOCK: &str = "В наличии (2-3 раб. дня)";
const OUT_OF_STOCK: &str = "Под заказ (5-8 недель)";

pub async fn saver(mut rx: UnboundedReceiver<Vec<Stock>>, ms_client: Arc<MoySkladApiClient>) {
    while let Some(stock) = rx.recv().await {
        update_ms_stock_attribute(ms_client.clone(), stock).await;
    }
}
async fn update_ms_stock_attribute(ms_client: Arc<MoySkladApiClient>, stock: Vec<Stock>) {
    match ms_client.get_all::<rust_moysklad::Product>().await {
        Ok(ms_products) => {
            if ms_products.is_empty() {
                tracing::error!("Получен пустой список продуктов из Мой Склад");
                return;
            }
            if let Some(in_stock_attribute) = get_stock_attribute(&ms_products, IN_STOCK) {
                if let Some(out_of_stock_attribute) =
                    get_stock_attribute(&ms_products, OUT_OF_STOCK)
                {
                    let mut items_to_update = Vec::new();
                    for ms_product in ms_products.iter() {
                        let in_stock_in_ms = is_in_stock_in_ms(ms_product);
                        let quantity = is_in_stock(ms_product, &stock);
                        let in_stock = quantity > 2.;
                        if in_stock != in_stock_in_ms {
                            let attribute = if in_stock {
                                in_stock_attribute.clone()
                            } else {
                                out_of_stock_attribute.clone()
                            };
                            let upd = rust_moysklad::Product::update()
                                .meta(ms_product.meta.clone())
                                .attribute(attribute)
                                .build();
                            items_to_update.push(upd);
                        }
                    }
                    if items_to_update.is_empty() {
                        tracing::info!("Нет товаров, которые требуют обновления");
                    } else {
                        tracing::info!(
                            "Получилось '{quantity}' товаров для обновления в Мой Склад",
                            quantity = items_to_update.len()
                        );
                        let result: Vec<rust_moysklad::Product> = ms_client
                            .batch_create_update(items_to_update)
                            .await
                            .unwrap_or_default();
                        if !result.is_empty() {
                            tracing::info!(
                                "Обновлено '{quantity}' атрибутов наличия товаров в Мой Склад",
                                quantity = result.len()
                            );
                        } else {
                            tracing::info!("Получен пустой ответ от Мой Склад на запрос обновления атрибутов остатков");
                        }
                    }
                } else {
                    tracing::error!("Не удалось получить атрибут \"Под заказ (5-8 недель)\"");
                }
            } else {
                tracing::error!("Не удалось получить атрибут \"В наличии\"");
            }
        }
        Err(e) => {
            tracing::error!("Ошибка получения продуктов из Мой Склад:\n{e:?}");
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
fn is_in_stock_in_ms(ms_product: &rust_moysklad::Product) -> bool {
    ms_product
        .attributes
        .clone()
        .unwrap_or_default()
        .into_iter()
        .find(|a| a.name == STOCK_ATTRIBUTE_NAME)
        .map(|a| {
            let val = match a.value.clone() {
                rust_moysklad::AttributeValue::Custom(v) => v.name,
                _ => String::new(),
            };
            val == IN_STOCK
        })
        .unwrap_or(false)
}
fn is_in_stock(ms_product: &rust_moysklad::Product, stock: &[Stock]) -> f64 {
    let Some(sku) = ms_product.article.clone() else {
        return 0.0;
    };
    let mut temp = stock.to_vec();
    for word in sku.split_whitespace() {
        temp = temp
            .into_iter()
            .filter(|s| s.name.to_uppercase().contains(&word.to_uppercase()))
            .collect::<Vec<_>>();
    }
    temp.iter().map(|s| s.stock).sum::<f64>()
}
