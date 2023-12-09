use crate::utils::*;
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct UserRegistrationError {
    pub id: UserId,
    pub username: String,
}

impl fmt::Display for UserRegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "User with id: {} already exists and has username: {}",
            self.id.0, self.username
        )
    }
}

impl Error for UserRegistrationError {
    fn description(&self) -> &str {
        format!("{}", self).as_str()
    }
}

#[derive(Debug)]
pub struct UserDoesNotExistError {
    pub id: UserId,
}

impl fmt::Display for UserDoesNotExistError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "User with id: {} does not exist", self.id.0,)
    }
}

impl Error for UserDoesNotExistError {
    fn description(&self) -> &str {
        format!("{}", self).as_str()
    }
}

#[derive(Debug)]
pub struct GameDoesNotExistError {
    pub id: GameId,
}

impl fmt::Display for GameDoesNotExistError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Game with id: {} does not exist", self.id.0,)
    }
}

impl Error for GameDoesNotExistError {
    fn description(&self) -> &str {
        format!("{}", self).as_str()
    }
}

#[derive(Debug)]
pub struct UserIsNotInPendingError {
    pub user_id: UserId,
    pub game_id: GameId,
}

impl fmt::Display for UserIsNotInPendingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "User with id: {} is not in pending users of game with id: {}",
            self.user_id, self.game_id
        )
    }
}

impl Error for UserIsNotInPendingError {
    fn description(&self) -> &str {
        format!("{}", self).as_str()
    }
}

#[derive(Debug)]
pub struct GameIsNotInPendingError {
    pub user_id: UserId,
    pub game_id: GameId,
}

impl fmt::Display for GameIsNotInPendingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Game with id: {} is not in pending games of user with id: {}",
            self.game_id, self.user_id
        )
    }
}

impl Error for GameIsNotInPendingError {
    fn description(&self) -> &str {
        format!("{}", self).as_str()
    }
}

#[derive(Debug)]
pub struct UserIsAlreadyInGameError {
    pub user_id: UserId,
    pub game_id: GameId,
}

impl fmt::Display for UserIsAlreadyInGameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "User with id: {} is already in game with id: {}",
            self.user_id, self.game_id,
        )
    }
}

impl Error for UserIsAlreadyInGameError {
    fn description(&self) -> &str {
        format!("{}", self).as_str()
    }
}

#[derive(Debug)]
pub struct GameIsAlreadyInUserError {
    pub user_id: UserId,
    pub game_id: GameId,
}

impl fmt::Display for GameIsAlreadyInUserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Game with id: {} is already in user with id: {}",
            self.game_id, self.user_id,
        )
    }
}

impl Error for GameIsAlreadyInUserError {
    fn description(&self) -> &str {
        format!("{}", self).as_str()
    }
}
