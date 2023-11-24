use lazy_static::lazy_static;

use teloxide::{dispatching::dialogue::InMemStorage, prelude::*, types::ParseMode};

use sled::Db;

mod utils;

mod scheme;
use scheme::{schema, State};

static DB_PATH: &str = "./database.db";

lazy_static! {
    pub static ref DB_CONNECTION: Db = sled::open(DB_PATH).unwrap();
}

#[derive(Clone)]
struct Parameters {
    db_connection: Db,
}

impl Parameters {
    fn new(db_connection: Db) -> Self {
        Self { db_connection }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting secret santa bot...");

    let bot = Bot::from_env();

    let parameters = Parameters::new(DB_CONNECTION.clone());

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![parameters, InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
