mod schema;
mod calculator;
mod price_parser;

use schema::{router, State};
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
    #[instrument(name = "starting bot", skip(self))]
    pub async fn run(&self) {
        Dispatcher::builder(self.bot.clone(), router())
            .dependencies(dptree::deps![InMemStorage::<State>::new()])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }
}
