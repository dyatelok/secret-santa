use crate::scheme::HandlerResult;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sled::{Db, IVec};

use teloxide::{prelude::*, types::ChatId};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct UserId(pub i64);

impl From<ChatId> for UserId {
    fn from(value: ChatId) -> Self {
        UserId(value.0)
    }
}

impl From<String> for UserId {
    fn from(value: String) -> Self {
        Self(value.parse::<i64>().unwrap())
    }
}

impl UserId {
    pub fn to_key(self) -> String {
        ron::to_string(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct GameId(pub u64);

impl From<String> for GameId {
    fn from(value: String) -> Self {
        Self(value.parse::<u64>().unwrap())
    }
}

impl GameId {
    pub fn to_key(self) -> String {
        ron::to_string(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub admin: Vec<GameId>,
    pub user: Vec<GameId>,
    pub pending: Vec<GameId>,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name: `{}`\n Id: `{}`\n\n", self.username, self.id.0)
    }
}

impl From<IVec> for User {
    fn from(value: IVec) -> Self {
        ron::from_str(std::str::from_utf8(value.as_ref()).unwrap()).unwrap()
    }
}

impl User {
    pub fn register(db: &Db, id: UserId, username: String) {
        let user = Self {
            id,
            username,
            admin: vec![],
            user: vec![],
            pending: vec![],
        };
        let _ = db.insert(id.to_key(), user.to_ron().as_str());
    }
    fn to_ron(&self) -> String {
        ron::to_string(&self).unwrap()
    }
    pub fn get(db: &Db, id: &UserId) -> Option<User> {
        db.get(id.to_key()).unwrap().map(User::from)
    }
    pub fn change_username(db: &Db, user_id: UserId, new_username: String) {
        match User::get(db, &user_id) {
            Some(user) => {
                let new_user = Self {
                    id: user.id,
                    username: new_username,
                    admin: user.admin,
                    user: user.user,
                    pending: user.pending,
                };
                let _ = db.insert(user_id.to_key(), new_user.to_ron().as_str());
            }
            None => {
                //TODO throw an error
            }
        }
    }
    pub fn add_pending(db: &Db, user_id: UserId, game_id: GameId) {
        match User::get(db, &user_id) {
            Some(user) => {
                let mut pending = user.pending;
                pending.push(game_id);

                let new_user = Self {
                    id: user.id,
                    username: user.username,
                    admin: user.admin,
                    user: user.user,
                    pending,
                };
                let _ = db.insert(user_id.to_key(), new_user.to_ron().as_str());
            }
            None => {
                //TODO throw an error
            }
        }
    }
    pub fn add_admin(db: &Db, user_id: UserId, game_id: GameId) {
        match User::get(db, &user_id) {
            Some(user) => {
                let mut admin = user.admin;
                admin.push(game_id);

                let new_user = Self {
                    id: user.id,
                    username: user.username,
                    admin,
                    user: user.user,
                    pending: user.pending,
                };
                let _ = db.insert(user_id.to_key(), new_user.to_ron().as_str());
            }
            None => {
                //TODO throw an error
            }
        }
    }
    pub fn promote(db: &Db, user_id: UserId, game_id: GameId) {
        match User::get(db, &user_id) {
            Some(user) => {
                //TODO throw an error if not in pending
                let mut pending = user.pending;
                pending.retain(|elem| *elem != game_id);
                let mut user_games = user.user;
                user_games.push(game_id);

                let new_user = Self {
                    id: user.id,
                    username: user.username,
                    admin: user.admin,
                    user: user_games,
                    pending,
                };
                let _ = db.insert(user_id.to_key(), new_user.to_ron().as_str());
            }
            None => {
                //TODO throw an error
            }
        }
    }
    pub fn remove(db: &Db, user_id: UserId, game_id: GameId) {
        match User::get(db, &user_id) {
            Some(user) => {
                let mut pending = user.pending;
                pending.retain(|elem| *elem != game_id);

                let mut user_games = user.user;
                user_games.retain(|elem| *elem != game_id);

                let new_user = Self {
                    id: user.id,
                    username: user.username,
                    admin: user.admin,
                    user: user_games,
                    pending,
                };
                let _ = db.insert(user_id.to_key(), new_user.to_ron().as_str());
            }
            None => {
                //TODO throw an error
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub id: GameId,
    pub name: String,
    pub admin: UserId,
    pub users: Vec<UserId>,
    pub pending: Vec<UserId>,
}

use std::fmt;

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name: `{}`\n Id: `{}`\n\n", self.name, self.id.0)
    }
}

impl From<IVec> for Game {
    fn from(value: IVec) -> Self {
        ron::from_str(std::str::from_utf8(value.as_ref()).unwrap()).unwrap()
    }
}

impl Game {
    pub async fn run(db: &Db, game_id: GameId, bot: &Bot) -> HandlerResult {
        let game = Game::get(db, &game_id).unwrap();

        let _ = db.remove(game_id.to_key());

        let (name, admin, users) = (game.name, game.admin, game.users);

        for user_id in users.clone() {
            User::remove(db, user_id, game_id);
        }

        if users.len() <= 1 {
            for user in users {
                bot.send_message(
                    ChatId(user.0),
                    "It looks like there's only one participant :(",
                )
                .await?;
            }
            bot.send_message(ChatId(admin.0), "All messages have been sent successfully!")
                .await?;
        } else {
            let (users, shuffled) = self_ans_shuffle(users);

            let users = users.iter().map(|id| User::get(db, id).unwrap());
            let shuffled = shuffled.iter().map(|id| User::get(db, id).unwrap());

            for (sender, recipient) in zip(users, shuffled) {
                let sender_name = sender.username;
                let recepient_name = recipient.username;
                bot.send_message(
                                ChatId(sender.id.0),
                                format!(
                                    "Ho Ho Ho, {sender_name}!\n\n\
                                     As a result of participating in game {name}. It looks like you have to prepare a present for {recepient_name}!\n\n\
                                     Have a happy new year, your secret santa."
                                ),
                            )
                            .await?;
            }
            bot.send_message(ChatId(admin.0), "All messages have been sent successfully!")
                .await?;
        }

        Ok(())
    }
    pub fn new(db: &Db, admin: UserId, name: String) -> Self {
        let mut rng = rand::thread_rng();
        let mut id = GameId(rng.gen());
        loop {
            if db.contains_key(ron::to_string(&id).unwrap()).unwrap() {
                id = GameId(rng.gen());
            } else {
                break;
            }
        }
        let game = Self {
            id,
            name,
            admin,
            users: vec![],
            pending: vec![],
        };

        User::add_admin(db, admin, game.id);

        let _ = db.insert(id.to_key(), game.to_ron().as_str());

        game
    }
    pub fn to_ron(&self) -> String {
        ron::to_string(&self).unwrap()
    }
    pub fn get(db: &Db, id: &GameId) -> Option<Game> {
        db.get(id.to_key()).unwrap().map(Game::from)
    }
    pub fn add_pending(db: &Db, game_id: GameId, user_id: UserId) {
        match Game::get(db, &game_id) {
            Some(game) => {
                let mut pending = game.pending;
                pending.push(user_id);

                let new_game = Self {
                    id: game.id,
                    name: game.name,
                    admin: game.admin,
                    users: game.users,
                    pending,
                };
                let _ = db.insert(game_id.to_key(), new_game.to_ron().as_str());
            }
            None => {
                //TODO throw an error
            }
        }
    }
    pub fn promote(db: &Db, game_id: GameId, user_id: UserId) {
        match Game::get(db, &game_id) {
            Some(game) => {
                //TODO throw error if not in pending
                let mut pending = game.pending;
                pending.retain(|elem| *elem != user_id);
                let mut users = game.users;
                users.push(user_id);

                let new_game = Self {
                    id: game.id,
                    name: game.name,
                    admin: game.admin,
                    users,
                    pending,
                };
                let _ = db.insert(game_id.to_key(), new_game.to_ron().as_str());
            }
            None => {
                //TODO throw an error
            }
        }
    }
    pub fn remove(db: &Db, game_id: GameId, user_id: UserId) {
        match Game::get(db, &game_id) {
            Some(game) => {
                let mut pending = game.pending;
                pending.retain(|elem| *elem != user_id);

                let mut users = game.users;
                users.retain(|elem| *elem != user_id);

                let new_game = Self {
                    id: game.id,
                    name: game.name,
                    admin: game.admin,
                    users,
                    pending,
                };
                let _ = db.insert(game_id.to_key(), new_game.to_ron().as_str());
            }
            None => {
                //TODO throw an error
            }
        }
    }
}

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::iter::zip;

pub fn self_ans_shuffle(users: Vec<UserId>) -> (Vec<UserId>, Vec<UserId>) {
    let mut rng = thread_rng();
    let mut shuffled;
    loop {
        shuffled = users.clone();
        shuffled.shuffle(&mut rng);

        if !zip(users.clone(), shuffled.clone()).any(|(a, b)| a == b) {
            break;
        }
    }
    (users, shuffled)
}
