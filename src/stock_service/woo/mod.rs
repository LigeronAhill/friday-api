use std::sync::Arc;

use rust_woocommerce as woo;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::models::Stock;
pub async fn saver(
    mut sku_stock_receiver: UnboundedReceiver<Vec<Stock>>,
    safira_woo_client: Arc<woo::ApiClient>,
) {
    while let Some(result) = sku_stock_receiver.recv().await {
        let safira_client = safira_woo_client.clone();
        let stock = result.clone();
        tokio::spawn(update_stock(safira_client, stock));
    }
}

async fn update_stock(client: Arc<woo::ApiClient>, stock: Vec<Stock>) {
    let host = client.base_url();
    let woo_products: Vec<woo::Product> = client.list_all().await.unwrap_or_default();
    let mut products_to_update = Vec::new();
    for product in woo_products {
        let sku = product.sku.clone();
        let quantity = get_quantity(&sku, &stock) as i32;
        let upd = woo::Product::builder()
            .id(product.id)
            .stock_quantity(quantity)
            .build();
        products_to_update.push(upd);
    }
    if products_to_update.is_empty() {
        tracing::info!("Нет остатков для обновления на сайте {host}");
    } else {
        let result: Vec<woo::Product> = client
            .batch_update(products_to_update)
            .await
            .unwrap_or_default();
        tracing::info!(
            "Обновлено {len} остатков продуктов на сайте {host}",
            len = result.len()
        );
    }
}
fn get_quantity(sku: &str, stock: &[Stock]) -> f64 {
    let mut temp = stock.to_vec();
    for word in sku.split_whitespace() {
        temp = temp
            .into_iter()
            .filter(|s| s.name.to_uppercase().contains(&word.to_uppercase()))
            .collect::<Vec<_>>();
    }
    temp.iter().map(|s| s.stock).sum::<f64>()
}
