use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

mod errors;

mod utils;

mod runner;
use runner::*;

mod scheme;
use scheme::{schema, State};

const DB_PATH: &str = "./database.db";

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting secret santa bot...");

    let bot = Bot::from_env();

    let runner = Runner::new(DB_PATH).unwrap();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![runner.clone(), InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
