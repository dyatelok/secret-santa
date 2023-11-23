use rand::Rng;
use serde::{Deserialize, Serialize};
use sled::Db;

#[derive(Serialize, Deserialize)]
pub struct UserId(i64);

#[derive(Serialize, Deserialize)]
pub struct GameId(i64);

#[derive(Serialize, Deserialize)]
pub struct User {
    id: UserId,
    username: String,
    admin: Vec<GameId>,
    user: Vec<GameId>,
}

impl User {
    pub fn new(db: &Db, username: String) -> Self {
        Self {
            id: new_userid(db),
            username,
            admin: vec![],
            user: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Game {
    id: GameId,
    name: String,
    admin: UserId,
    users: Vec<UserId>,
    pending: Vec<UserId>,
}

impl Game {
    pub fn new(db: &Db, userid: UserId, name: String) -> Self {
        Self {
            id: new_gameid(db),
            name,
            admin: userid,
            users: vec![],
            pending: vec![],
        }
    }
}

fn new_userid(db: &Db) -> UserId {
    let mut rng = rand::thread_rng();
    let mut id = UserId(rng.gen());
    loop {
        if db.contains_key(ron::to_string(&id).unwrap()).unwrap() {
            id = UserId(rng.gen());
        } else {
            break;
        }
    }
    id
}

fn new_gameid(db: &Db) -> GameId {
    let mut rng = rand::thread_rng();
    let mut id = GameId(rng.gen());
    loop {
        if db.contains_key(ron::to_string(&id).unwrap()).unwrap() {
            id = GameId(rng.gen());
        } else {
            break;
        }
    }
    id
}

