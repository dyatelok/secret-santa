use lazy_static::lazy_static;

use teloxide::{prelude::*, utils::command::BotCommands};
use sled::Db;

mod utils;
use utils::*;

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
        Self {
            db_connection,
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    let parameters = Parameters::new(DB_CONNECTION.clone());

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(commands_handler),
        ).branch(
            dptree::filter(|_: Message| true).endpoint(|msg: Message, bot: Bot| async move {
                    bot.send_message(msg.chat.id, "Message doesn't follow any known pattern. Pleace use /help").await?;
                    respond(())
                }),
        );
    
    Dispatcher::builder(bot, handler)
        // Here you specify initial dependencies that all handlers will receive; they can be
        // database connections, configurations, and other auxiliary arguments. It is similar to
        // `actix_web::Extensions`.
        .dependencies(dptree::deps![parameters])
        // If no handler succeeded to handle an update, this closure will be called.
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:?}", upd);
        })
        // If the dispatcher fails for some reason, execute this handler.
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    Start,
    #[command(description = "display this text.")]
    Help,
    #[command(description = "register.")]
    Register,
    #[command(description = "changes your username.")]
    Username,
    #[command(description = "create a new secret santa event.")]
    Create,
    #[command(description = "run a secret santa game.")]
    Run,
    #[command(description = "join a secret santa event.")]
    Join,
    #[command(description = "leave a secret santa event.")]
    Leave,
    #[command(description = "list all your secret santa events.")]
    List,
}

async fn commands_handler(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Start | Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Register => {
            //TODO register new user
        }
        Command::Username => {
            //TODO change username
        }
        Command::Create => {
            // chat_id.admin_chats <- game_id
            // game.admin <- chat_id
            // game.name <- name
            //TODO create new game by name and give user an id to share
            bot.send_message(msg.chat.id, format!("You've created a game!"))
                .await?;
        }
        Command::Run => {
            // if game_id in chat_id.admin_chats => run game
            //TODO run a game by id
            bot.send_message(msg.chat.id, format!("You ran game"))
                .await?;
        }
        Command::Join => {
            // game_id.users <- chat_id
            //TODO join a game by id
            bot.send_message(msg.chat.id, format!("You've joined the game"))
                .await?;
        }
        Command::Leave => {todo!()}
        Command::List => {
            //TODO list all the games user participates in as admin or user
            bot.send_message(msg.chat.id, format!("Here's the list of all your games:"))
                .await?;
        }
    };

    Ok(())
}

