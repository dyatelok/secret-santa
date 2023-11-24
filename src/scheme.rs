use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::ParseMode,
    utils::command::BotCommands,
};

#[derive(Clone, Default)]
pub enum State {
    #[default]
    None,
    Register {
        state: RegisterState,
    },
    Username {
        state: UsernameState,
    },
    Create {
        state: CreateState,
    },
    Run {
        state: RunState,
    },
    Join {
        state: JoinState,
    },
    Leave {
        state: LeaveState,
    },
}

#[derive(Clone)]
pub enum RegisterState {
    GetName,
}

#[derive(Clone)]
pub enum UsernameState {
    GetName,
}

#[derive(Clone)]
pub enum CreateState {
    GetName,
}

#[derive(Clone)]
pub enum RunState {
    GetId,
    Confirm,
}

#[derive(Clone)]
pub enum JoinState {
    GetId,
}

#[derive(Clone)]
pub enum LeaveState {
    GetId,
}

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    Start,
    #[command(description = "display this text.")]
    Help,
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
    #[command(description = "cancel operation.")]
    Cancel,
}

pub fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            // catches all cases where there's no already started dialog
            case![State::None]
                .branch(case![Command::Start].endpoint(start_cmd))
                .branch(case![Command::Help].endpoint(help_cmd))
                .branch(case![Command::Username].endpoint(username_cmd))
                .branch(case![Command::Create].endpoint(create_cmd))
                .branch(case![Command::Run].endpoint(run_cmd))
                .branch(case![Command::Join].endpoint(join_cmd))
                .branch(case![Command::Leave].endpoint(leave_cmd))
                .branch(case![Command::List].endpoint(list_cmd)),
        )
        // catch case if user wants to leave
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::Register { state }].endpoint(register))
        .branch(case![State::Username { state }].endpoint(username))
        .branch(case![State::Create { state }].endpoint(create))
        .branch(case![State::Run { state }].endpoint(run))
        .branch(case![State::Join { state }].endpoint(join))
        .branch(case![State::Leave { state }].endpoint(leave))
        .branch(dptree::endpoint(invalid_state));

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler)
}

async fn start_cmd(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Let's start! How should I call you?")
        .await?;
    dialogue
        .update(State::Register {
            state: RegisterState::GetName,
        })
        .await?;
    Ok(())
}

async fn help_cmd(bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn username_cmd(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Please enter a new username.\n\
        You can /cancel",
    )
    .await?;
    dialogue
        .update(State::Username {
            state: UsernameState::GetName,
        })
        .await?;
    Ok(())
}

async fn create_cmd(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Please enter name of the game.\n\
        You can /cancel",
    )
    .await?;
    dialogue
        .update(State::Create {
            state: CreateState::GetName,
        })
        .await?;
    Ok(())
}

async fn run_cmd(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Please enter id of the game you want to run.\n\
        You can /cancel",
    )
    .await?;
    dialogue
        .update(State::Run {
            state: RunState::GetId,
        })
        .await?;
    Ok(())
}

async fn join_cmd(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Please enter id of the game you want to join.\n\
        You can /cancel",
    )
    .await?;
    dialogue
        .update(State::Join {
            state: JoinState::GetId,
        })
        .await?;
    Ok(())
}

async fn leave_cmd(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        format!(
            "Please enter id of the game you want to leave.\n\
            You can /cancel\n\
            You will be able to rejoin this game."
        ),
    )
    .await?;
    dialogue
        .update(State::Leave {
            state: LeaveState::GetId,
        })
        .await?;
    Ok(())
}

async fn list_cmd(bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    //TODO list all the games
    bot.send_message(msg.chat.id, "Here's list of all your games:")
        .await?;

    Ok(())
}

async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Some(cont) = dialogue.get().await.ok().unwrap() {
        match cont {
            State::Register { state: _ } => {
                bot.send_message(
                    msg.chat.id,
                    "It's not possible to cancel the registration process..",
                )
                .await?;
            }
            _ => {
                bot.send_message(msg.chat.id, "Cancelling the dialogue.")
                    .await?;
                dialogue.exit().await?;
            }
        }
    }
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Unable to handle the message. Type /help to see the usage.",
    )
    .await?;
    Ok(())
}

async fn register(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    _state: RegisterState,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(name) => {
            //TODO register the user
            // if the user is already registered -> cancel registration

            bot.send_message(
                msg.chat.id,
                format! {"Thanks for completing the registration, {name}.\n\
                    You can change your username using /username\n\
                Use /help to get more info."},
            )
            .await?;

            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please use /help.").await?;
            dialogue.exit().await?;
        }
    }

    Ok(())
}

async fn username(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    _state: UsernameState,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(name) => {
            //TODO change username

            bot.send_message(
                msg.chat.id,
                format! {"You've changed your username to {name}."},
            )
            .await?;

            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please use /help.").await?;
            dialogue.exit().await?;
        }
    }

    Ok(())
}

async fn create(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    _state: CreateState,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(name) => {
            //TODO create new game

            let game_id = 123;
            bot.send_message(
                msg.chat.id,
                format! {"You've created game named {name} with game id `{game_id}`"},
            )
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
            bot.send_message(
                msg.chat.id,
                format! {"To join game {name} you have to use `/join` after registration and use {game_id}\\."},
            )
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please use /help.").await?;
            dialogue.exit().await?;
        }
    }

    Ok(())
}

async fn run(bot: Bot, dialogue: MyDialogue, msg: Message, state: RunState) -> HandlerResult {
    match state {
        RunState::GetId => {
            match msg.text().map(ToOwned::to_owned) {
                Some(game_id) => {
                    //TODO run game by id
                    let name = "GameName";
                    bot.send_message(
                        msg.chat.id,
                        format! {"To run game {name} with id {game_id} you have to confirm your it \
                        by typig `run name {name} id {game_id}`\n\n\
                        You can use /cancel to cancel this operation\\."},
                    )
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;

                    dialogue
                        .update(State::Run {
                            state: RunState::Confirm,
                        })
                        .await?;
                }
                None => {
                    bot.send_message(msg.chat.id, "Please use /help.").await?;
                    dialogue.exit().await?;
                }
            }
        }
        RunState::Confirm => {
            match msg.text().map(ToOwned::to_owned) {
                Some(text) => {
                    //TODO confirm text
                    let name = "GameName"; /*
                                           bot.send_message(
                                               msg.chat.id,
                                               format! {"To run game {name} with id {game_id} you have to confirm your it by typig `run game name={name} id={game_id}`\\."},
                                           )
                                           .parse_mode(ParseMode::MarkdownV2)
                                           .await?;*/
                    dialogue.exit().await?;
                }
                None => {
                    bot.send_message(msg.chat.id, "Please use /help.").await?;
                    dialogue.exit().await?;
                }
            }
        }
    }

    Ok(())
}

async fn join(bot: Bot, dialogue: MyDialogue, msg: Message, _state: JoinState) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(game_id) => {
            //TODO join game by id
            bot.send_message(
                msg.chat.id,
                format! {"You're now in the waiting list to this game.\n\
                Please wait until game administrator confirms you.\n\
                You can /leave to leave game and /list to list all your games."},
            )
            .await?;
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please use /help.").await?;
            dialogue.exit().await?;
        }
    }

    Ok(())
}

async fn leave(bot: Bot, dialogue: MyDialogue, msg: Message, _state: LeaveState) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(game_id) => {
            //TODO leave game by id
            bot.send_message(msg.chat.id, format! {"You've left this game."})
                .await?;
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please use /help.").await?;
            dialogue.exit().await?;
        }
    }

    Ok(())
}
