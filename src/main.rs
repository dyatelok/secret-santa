use polodb_core::bson::doc;
use polodb_core::Database;
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, utils::command::BotCommands};

#[derive(Debug, Serialize, Deserialize)]
struct Book {
    title: String,
    author: String,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "set your name.")]
    Username(String),
    #[command(description = "create a new secret santa event.")]
    Create,
    #[command(description = "run a secret santa game by id.")]
    Run(String),
    #[command(description = "join a secret santa event by id.")]
    Join(String),
    #[command(description = "list all your secret santa games.")]
    List,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let db = Database::open_file("test-polo.db").unwrap();
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Username(username) => {
            // chat_id.username <- username
            // TODO add username to user
            println!("{}", msg.chat.id);
            bot.send_message(msg.chat.id, format!("Your username is @{username}."))
                .await?
        }
        Command::Create => {
            // chat_id.admin_chats <- game_id
            // game.admin <- chat_id
            // game.name <- name
            //TODO create a new game by name and give user an id to share
            bot.send_message(msg.chat.id, format!("You've created a game!"))
                .await?
        }
        Command::Run(game_id) => {
            // if game_id in chat_id.admin_chats => run game
            //TODO run a game by id
            bot.send_message(msg.chat.id, format!("You ran game"))
                .await?
        }
        Command::Join(game_id) => {
            // game_id.users <- chat_id
            //TODO join a game by id
            bot.send_message(msg.chat.id, format!("You've joined the game"))
                .await?
        }
        Command::List => {
            //TODO list all the games user participates in as admin or user
            bot.send_message(msg.chat.id, format!("Here's the list of all your games:"))
                .await?
        }
    };

    Ok(())
}
