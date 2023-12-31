use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateFilterExt, UpdateHandler},
    dptree,
    payloads::SendMessageSetters,
    prelude::{ChatId, Dialogue, Requester},
    types::{Message, ParseMode, Update},
    utils::command::BotCommands,
    Bot,
};

use crate::{runner::*, utils::*};

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
    Accept {
        state: AcceptState,
    },
    Remove {
        state: RemoveState,
    },
    Info {
        state: InfoState,
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
    Confirm { game_id: GameId },
}

#[derive(Clone)]
pub enum JoinState {
    GetId,
}

#[derive(Clone)]
pub enum LeaveState {
    GetId,
}

#[derive(Clone)]
pub enum AcceptState {
    GetGameId,
    GetUserId { game_id: GameId },
}

#[derive(Clone)]
pub enum RemoveState {
    GetGameId,
    GetUserId { game_id: GameId },
}

#[derive(Clone)]
pub enum InfoState {
    GetId,
}

type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "please use this command to register if you haven't!")]
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
    #[command(description = "accept someone to one of your games.")]
    Accept,
    #[command(description = "remove someone from one of your games.")]
    Remove,
    #[command(description = "get info about one of your games.")]
    Info,
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
                .branch(case![Command::Accept].endpoint(accept_cmd))
                .branch(case![Command::Remove].endpoint(remove_cmd))
                .branch(case![Command::Info].endpoint(info_cmd))
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
        .branch(case![State::Accept { state }].endpoint(accept))
        .branch(case![State::Remove { state }].endpoint(remove))
        .branch(case![State::Info { state }].endpoint(info))
        .branch(dptree::endpoint(invalid_state));

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler)
}

async fn start_cmd(bot: Bot, dialogue: MyDialogue, msg: Message, runner: Runner) -> HandlerResult {
    match runner.get_user(&UserId::from(msg.chat.id)) {
        Some(_) => {
            bot.send_message(
                msg.chat.id,
                "It looks like you're already registered.\n\
                You can change your username using /username\n\
                Use /help to get more info.",
            )
            .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Let's start! How should I call you?")
                .await?;

            dialogue
                .update(State::Register {
                    state: RegisterState::GetName,
                })
                .await?;
        }
    }

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

async fn run_cmd(bot: Bot, dialogue: MyDialogue, msg: Message, runner: Runner) -> HandlerResult {
    let mut message = String::from(
        "Please enter id of the game you want to run\n\
        You can /cancel\n\n\
        Here are available options:\n",
    );

    match runner.get_user(&UserId::from(msg.chat.id)) {
        Some(user) => {
            user.admin_games
                .iter()
                .for_each(|elem| match runner.get_game(elem) {
                    Some(game) => {
                        message.push_str(format!("{game}").as_str());
                    }
                    None => {
                        todo!()
                    }
                });
        }
        None => {
            todo!()
        }
    }

    bot.send_message(msg.chat.id, message)
        .parse_mode(ParseMode::MarkdownV2)
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
        "Please enter id of the game you want to leave.\n\
            You can /cancel\n\
            You will be able to rejoin this game.",
    )
    .await?;
    dialogue
        .update(State::Leave {
            state: LeaveState::GetId,
        })
        .await?;
    Ok(())
}

async fn accept_cmd(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Please enter id of the game you want to manage.\n\
            You can /cancel",
    )
    .await?;
    dialogue
        .update(State::Accept {
            state: AcceptState::GetGameId,
        })
        .await?;
    Ok(())
}

async fn remove_cmd(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Please enter id of the game you want to manage.\n\
            You can /cancel",
    )
    .await?;
    dialogue
        .update(State::Remove {
            state: RemoveState::GetGameId,
        })
        .await?;
    Ok(())
}

async fn list_cmd(bot: Bot, _dialogue: MyDialogue, msg: Message, runner: Runner) -> HandlerResult {
    match runner.get_user(&UserId::from(msg.chat.id)) {
        Some(user) => {
            let username = user.username;
            bot.send_message(
                msg.chat.id,
                format!("Hi, {username} Here's list of all your games:"),
            )
            .await?;
            match user.pending_games.len() {
                0 => {
                    bot.send_message(msg.chat.id, "There was no pending games found.")
                        .await?;
                }
                _ => {
                    let mut message: String = String::from("Here are your pending games:\n\n");
                    for pending_game in user.pending_games {
                        match runner.get_game(&pending_game) {
                            Some(game) => {
                                message.push_str(format!("{}\n", game).as_str());
                            }
                            None => {
                                todo!()
                            }
                        }
                    }
                    bot.send_message(msg.chat.id, message.as_str())
                        .parse_mode(ParseMode::MarkdownV2)
                        .await?;
                }
            }
            match user.active_games.len() {
                0 => {
                    bot.send_message(msg.chat.id, "There was no user active found.")
                        .await?;
                }
                _ => {
                    let mut message: String = String::from("Here are your active games:\n\n");
                    for active_game in user.active_games {
                        match runner.get_game(&active_game) {
                            Some(game) => {
                                message.push_str(format!("{}\n", game).as_str());
                            }
                            None => {
                                todo!()
                            }
                        }
                    }
                    bot.send_message(msg.chat.id, message.as_str())
                        .parse_mode(ParseMode::MarkdownV2)
                        .await?;
                }
            }
            match user.admin_games.len() {
                0 => {
                    bot.send_message(msg.chat.id, "There was no admin games found.")
                        .await?;
                }
                _ => {
                    let mut message: String = String::from("Here are your admin games:\n\n");
                    for admin_game in user.admin_games {
                        match runner.get_game(&admin_game) {
                            Some(game) => {
                                message.push_str(format!("{}\n", game).as_str());
                            }
                            None => {
                                todo!()
                            }
                        }
                    }
                    bot.send_message(msg.chat.id, message.as_str())
                        .parse_mode(ParseMode::MarkdownV2)
                        .await?;
                }
            }
        }
        None => {
            bot.send_message(
                msg.chat.id,
                "It looks like you're not registered. Please register with /start",
            )
            .await?;
        }
    }

    Ok(())
}

async fn info_cmd(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Please enter id of the game you want to get info about.\n\
            You can /cancel",
    )
    .await?;
    dialogue
        .update(State::Info {
            state: InfoState::GetId,
        })
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
    runner: Runner,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(name) => {
            runner.new_user(msg.chat.id.into(), name.clone());
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
            bot.send_message(msg.chat.id, "Please use /help").await?;
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
    runner: Runner,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(name) => {
            runner.change_username(&UserId::from(msg.chat.id), name.clone());

            bot.send_message(
                msg.chat.id,
                format! {"You've changed your username to {name}."},
            )
            .await?;

            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please use /help").await?;
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
    runner: Runner,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(game_name) => match runner.new_game(UserId::from(msg.chat.id), game_name.clone()) {
            Ok(game_id) => {
                let id = game_id.0;
                bot.send_message(
                    msg.chat.id,
                    format! {"You've created game named {game_name} with game id `{id}`"},
                )
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
                bot.send_message(
                msg.chat.id,
                format! {"To join game {game_name} you have to use /join after registration and use `{id}`\\."},
            )
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

                dialogue.exit().await?;
            }
            Err(_) => {
                todo!()
            }
        },
        None => {
            bot.send_message(msg.chat.id, "Please use /help").await?;
            dialogue.exit().await?;
        }
    }

    Ok(())
}

async fn run(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    state: RunState,
    runner: Runner,
) -> HandlerResult {
    match state {
        RunState::GetId => match msg.text().map(ToOwned::to_owned) {
            Some(game_id) => {
                let game_id = GameId::from(game_id);
                let user_id = UserId::from(msg.chat.id);

                match runner.get_game(&game_id) {
                    Some(game) => match game.admin == user_id {
                        true => {
                            let id = game_id.0;
                            let name = game.name;
                            bot.send_message(
                                    msg.chat.id,
                                    format! {"Please confirm that you're going to run game `{name}`\n\
                                        This action is irreversible\n\
                                        Messages about who to give the gift to will be sent out instantly\n\n\
                                        To confirm please type `Yes, I do want to run game {id}`\n\
                                        You can /cancel"},
                                ).parse_mode(ParseMode::MarkdownV2)
                                .await?;
                            dialogue
                                .update(State::Run {
                                    state: RunState::Confirm { game_id },
                                })
                                .await?;
                        }
                        false => {
                            bot.send_message(
                                msg.chat.id,
                                "It looks like you're not admin of this game",
                            )
                            .await?;
                            dialogue.exit().await?;
                        }
                    },
                    None => {
                        bot.send_message(msg.chat.id, "It looks like there's no such game")
                            .await?;
                        dialogue.exit().await?;
                    }
                }
            }
            None => {
                bot.send_message(msg.chat.id, "Please use /help").await?;
                dialogue.exit().await?;
            }
        },
        RunState::Confirm { game_id } => match msg.text().map(ToOwned::to_owned) {
            Some(text) => {
                let id = game_id.0;
                match text == format!("Yes, I do want to run game {id}") {
                    true => {
                        bot.send_message(
                            msg.chat.id,
                            "You've successfully ran this game.\n\
                                 Messages will be sent immediately\n\
                                Thanks for using this bot!",
                        )
                        .await?;

                        let messages = runner.run_game(game_id).unwrap();

                        for (recipient, message) in messages {
                            bot.send_message(ChatId(recipient.0), message).await?;
                        }

                        dialogue.exit().await?;
                    }
                    false => {
                        bot.send_message(msg.chat.id, "Text doesn't match confirnation statement.\n Please retry or use /cancel").await?;
                    }
                }
            }
            None => {
                bot.send_message(msg.chat.id, "Please use /help").await?;
                dialogue.exit().await?;
            }
        },
    }

    Ok(())
}

async fn join(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    _state: JoinState,
    runner: Runner,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(game_id) => {
            let game_id = GameId::from(game_id);
            let user_id = UserId::from(msg.chat.id);

            match runner.get_game(&game_id) {
                Some(_) => {
                    runner.add_user_to_pending(&user_id, &game_id);

                    bot.send_message(
                        msg.chat.id,
                        "You're now in the waiting list to this game.\n\
                    Please wait until game administrator confirms you.\n\
                    You can /leave to leave game and /list to list all your games.",
                    )
                    .await?;
                }
                None => {
                    bot.send_message(
                        msg.chat.id,
                        "It looks like there's no such game.\n\
                    Please use /help",
                    )
                    .await?;
                }
            }

            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please use /help").await?;
            dialogue.exit().await?;
        }
    }

    Ok(())
}

async fn leave(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    _state: LeaveState,
    runner: Runner,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(game_id) => {
            let user_id = UserId::from(msg.chat.id);
            let game_id = GameId::from(game_id);

            runner.remove_user_from_game(&user_id, &game_id).unwrap();

            bot.send_message(msg.chat.id, "You've successfully left this game.")
                .await?;
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please use /help").await?;
            dialogue.exit().await?;
        }
    }

    Ok(())
}

async fn accept(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    state: AcceptState,
    runner: Runner,
) -> HandlerResult {
    match state {
        AcceptState::GetGameId => match msg.text().map(ToOwned::to_owned) {
            Some(game_id) => {
                //TODO do proper checks
                let game_id = GameId::from(game_id);

                let mut message = String::from("Here are all pendling users:\n");

                runner
                    .get_game(&game_id)
                    .unwrap()
                    .pending_users
                    .iter()
                    .map(|id| runner.get_user(id).unwrap())
                    .for_each(|user| message.push_str(user.to_string().as_str()));

                bot.send_message(msg.chat.id, message)
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;

                bot.send_message(msg.chat.id, "Please send id of the user to accept.")
                    .await?;

                dialogue
                    .update(State::Accept {
                        state: AcceptState::GetUserId { game_id },
                    })
                    .await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Please use /help").await?;
                dialogue.exit().await?;
            }
        },
        AcceptState::GetUserId { game_id } => match msg.text().map(ToOwned::to_owned) {
            Some(user_id) => {
                let user_id = UserId::from(user_id);

                runner
                    .promote_user_from_pending_to_active(&user_id, &game_id)
                    .unwrap();

                bot.send_message(msg.chat.id, "You've accepted this user to the game.")
                    .await?;
                dialogue.exit().await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Please use /help").await?;
                dialogue.exit().await?;
            }
        },
    }

    Ok(())
}

async fn remove(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    state: RemoveState,
    runner: Runner,
) -> HandlerResult {
    match state {
        RemoveState::GetGameId => match msg.text().map(ToOwned::to_owned) {
            Some(game_id) => {
                //TODO do proper checks
                let game_id = GameId::from(game_id);

                let mut message = String::from("Here are all users:\n\n");

                message.push_str("Pending users:\n\n");

                let game = runner.get_game(&game_id).unwrap();

                game.pending_users
                    .iter()
                    .map(|id| runner.get_user(id).unwrap())
                    .for_each(|user| message.push_str(user.to_string().as_str()));

                message.push_str("Active users:\n\n");

                game.active_users
                    .iter()
                    .map(|id| runner.get_user(id).unwrap())
                    .for_each(|user| message.push_str(user.to_string().as_str()));

                bot.send_message(msg.chat.id, message)
                    .parse_mode(ParseMode::MarkdownV2)
                    .await?;

                bot.send_message(msg.chat.id, "Please send id of the user to remove.")
                    .await?;

                dialogue
                    .update(State::Remove {
                        state: RemoveState::GetUserId { game_id },
                    })
                    .await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Please use /help").await?;
                dialogue.exit().await?;
            }
        },
        RemoveState::GetUserId { game_id } => match msg.text().map(ToOwned::to_owned) {
            Some(user_id) => {
                let user_id = UserId::from(user_id);

                runner.remove_user_from_game(&user_id, &game_id);

                bot.send_message(msg.chat.id, "You've removed this user from the game.")
                    .await?;
                dialogue.exit().await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Please use /help").await?;
                dialogue.exit().await?;
            }
        },
    }

    Ok(())
}

async fn info(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    _state: InfoState,
    runner: Runner,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(game_id) => {
            let game_id = GameId::from(game_id);
            let game = runner.get_game(&game_id).unwrap();

            let mut message = String::from("Here are info about your game:\n\n");

            let game_name = game.name;
            message.push_str(format!("Name: `{game_name}`\n\n").as_str());

            let game_id = game_id.0;
            message.push_str(format!("Id: `{game_id}`\n\n").as_str());

            message.push_str("Active users:\n");

            game.active_users
                .iter()
                .map(|id| runner.get_user(id).unwrap())
                .for_each(|user| message.push_str(user.to_string().as_str()));

            message.push_str("\nPending users:\n");

            game.pending_users
                .iter()
                .map(|id| runner.get_user(id).unwrap())
                .for_each(|user| message.push_str(user.to_string().as_str()));

            bot.send_message(msg.chat.id, message)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;

            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please use /help").await?;
            dialogue.exit().await?;
        }
    }

    Ok(())
}
