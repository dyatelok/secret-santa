use serde::{Deserialize, Serialize};
use sled::IVec;
use std::fmt;
use teloxide::types::ChatId;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
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

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl UserId {
    pub fn to_key(self) -> String {
        ron::to_string(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct GameId(pub u64);

impl From<String> for GameId {
    fn from(value: String) -> Self {
        Self(value.parse::<u64>().unwrap())
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl GameId {
    pub fn to_key(self) -> String {
        ron::to_string(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub admin_games: Vec<GameId>,
    pub active_games: Vec<GameId>,
    pub pending_games: Vec<GameId>,
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
    pub fn to_ron(&self) -> String {
        ron::to_string(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub id: GameId,
    pub name: String,
    pub admin: UserId,
    pub active_users: Vec<UserId>,
    pub pending_users: Vec<UserId>,
}

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
    pub fn to_ron(&self) -> String {
        ron::to_string(&self).unwrap()
    }
}
