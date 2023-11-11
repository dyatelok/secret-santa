use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
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
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::Username(username) => {
            //TODO save a username
            println!("{}",msg.chat.id);
            bot.send_message(msg.chat.id, format!("Your username is @{username}.")).await?
        }
        Command::Create => {
            //TODO create a new game by name and give user an id to share
            todo!()
        }
        Command::Run(game_id) => {
            //TODO run a game by id
            todo!()
        }
        Command::Join(game_id) => {
            //TODO join a game by id
            todo!()
        }
        Command::List => {
            //TODO list all the games user participates in as admin or 
            todo!()
        }
    };

    Ok(())
}























