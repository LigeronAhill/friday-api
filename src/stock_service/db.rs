use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{error, info};

use crate::{models::Stock, storage::StockStorage};

pub async fn saver(mut rx: UnboundedReceiver<Vec<Stock>>, storage: Arc<StockStorage>) {
    while let Some(stock) = rx.recv().await {
        match storage.update(&stock).await {
            Ok((deleted, inserted)) => {
                info!("Удалено {deleted} строк остатков, добавлено {inserted} строк остатков")
            }
            Err(e) => error!("Ошибка обновления стока в базе данных:\n{e:?}"),
        }
    }
}
