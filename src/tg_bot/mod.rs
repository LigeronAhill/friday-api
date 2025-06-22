mod schema;
mod calculator;
mod price_parser;

use crate::storage::{CurrencyStorage, PriceStorage, StockStorage};
use schema::{router, State};
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;
use tracing::instrument;

pub struct TGBot {
    bot: Bot,
}

impl TGBot {
    #[instrument(name = "initializing bot", skip(token))]
    pub fn new(token: &str) -> Self {
        let bot = Bot::new(token);
        Self { bot }
    }
    #[instrument(name = "starting bot", skip_all)]
    pub async fn run(&self, price_storage: Arc<PriceStorage>, stock_storage: Arc<StockStorage>, currency_storage: Arc<CurrencyStorage>) {
        Dispatcher::builder(self.bot.clone(), router())
            .dependencies(dptree::deps![InMemStorage::<State>::new(), price_storage.clone(), stock_storage.clone(), currency_storage.clone()])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }
}
