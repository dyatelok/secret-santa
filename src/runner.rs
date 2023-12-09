use crate::errors::*;
use crate::utils::{UserId, *};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use sled::{Batch, Db};
use std::{error::Error, iter::zip};

#[derive(Clone)]
pub struct Runner {
    database: Db,
}

impl Runner {
    pub fn new(db_path: &str) -> Result<Self, Box<dyn Error>> {
        let database = sled::open(db_path)?;
        Ok(Self { database })
    }
    pub fn new_user(&self, id: UserId, username: String) -> Result<(), Box<dyn Error>> {
        // There's no need to check for id collision since telegram already does it
        let response = self.database.get(id.to_key())?;

        if let Some(value) = response {
            let user = User::from(value);
            return Err(Box::new(UserRegistrationError {
                id: user.id,
                username: user.username,
            }));
        }

        let user = User {
            id,
            username,
            admin_games: vec![],
            active_games: vec![],
            pending_games: vec![],
        };

        match self.database.insert(id.to_key(), user.to_ron().as_str()) {
            Ok(_) => Ok(()),
            Err(error) => Err(Box::new(error)),
        }
    }
    pub fn new_game(&self, admin: UserId, name: String) -> Result<GameId, Box<dyn Error>> {
        let mut rng = rand::thread_rng();
        let mut id = GameId(rng.gen());
        while !self
            .database
            .contains_key(ron::to_string(&id).unwrap())
            .unwrap()
        {
            id = GameId(rng.gen());
        }

        let game = Game {
            id,
            name,
            admin,
            active_users: vec![],
            pending_users: vec![],
        };

        self.user_add_admin_game(&admin, &game.id)?;

        // we don't check for existing game since we've already checked for it previously
        self.database.insert(id.to_key(), game.to_ron().as_str())?;

        Ok(id)
    }
    fn user_add_admin_game(
        &self,
        user_id: &UserId,
        game_id: &GameId,
    ) -> Result<(), Box<dyn Error>> {
        match self.get_user(user_id) {
            Some(user) => {
                let mut admin_games = user.admin_games;
                admin_games.push(*game_id);

                let new_user = User {
                    id: user.id,
                    username: user.username,
                    admin_games,
                    active_games: user.active_games,
                    pending_games: user.pending_games,
                };
                self.database
                    .insert(user_id.to_key(), new_user.to_ron().as_str())?;
                Ok(())
            }
            None => Err(Box::new(UserDoesNotExistError { id: *user_id })),
        }
    }
    pub fn get_user(&self, id: &UserId) -> Option<User> {
        self.database.get(id.to_key()).unwrap().map(User::from)
    }
    pub fn get_game(&self, id: &GameId) -> Option<Game> {
        self.database.get(id.to_key()).unwrap().map(Game::from)
    }
    pub fn change_username(
        &self,
        user_id: &UserId,
        new_username: String,
    ) -> Result<(), Box<dyn Error>> {
        match self.get_user(user_id) {
            Some(user) => {
                let new_user = User {
                    id: user.id,
                    username: new_username,
                    admin_games: user.admin_games,
                    active_games: user.active_games,
                    pending_games: user.pending_games,
                };
                self.database
                    .insert(user_id.to_key(), new_user.to_ron().as_str())?;
                Ok(())
            }
            None => Err(Box::new(UserDoesNotExistError { id: *user_id })),
        }
    }
    fn remove_game_from_user(
        &self,
        user_id: &UserId,
        game_id: &GameId,
    ) -> Result<(), Box<dyn Error>> {
        match self.get_user(user_id) {
            Some(user) => {
                let mut admin_games = user.admin_games;
                admin_games.retain(|id| id != game_id);
                let mut active_games = user.active_games;
                active_games.retain(|id| id != game_id);
                let mut pending_games = user.pending_games;
                pending_games.retain(|id| id != game_id);

                let new_user = User {
                    id: user.id,
                    username: user.username,
                    admin_games,
                    active_games,
                    pending_games,
                };
                self.database
                    .insert(user_id.to_key(), new_user.to_ron().as_str())?;
                Ok(())
            }
            None => Err(Box::new(UserDoesNotExistError { id: *user_id })),
        }
    }
    /// returns vector of pairs (userid, message to send)
    pub fn run_game(&self, game_id: GameId) -> Result<Vec<(UserId, String)>, Box<dyn Error>> {
        let Some(game) = self.get_game(&game_id) else {
            return Err(Box::new(GameDoesNotExistError { id: game_id }));
        };

        self.database.remove(game_id.to_key())?;

        self.remove_game_from_user(&game.admin, &game_id)?;
        for user in game.active_users {
            self.remove_game_from_user(&user, &game_id)?;
        }
        for user in game.pending_users {
            self.remove_game_from_user(&user, &game_id)?;
        }

        let game_name = game.name;
        let users = game.active_users;
        let mut messages = Vec::new();

        if users.len() <= 1 {
            for user_id in users {
                messages.push((
                    user_id,
                    "It looks like there's only one participant :(\
                    We can't run this game."
                        .to_owned(),
                ));
            }
            messages.push((
                game.admin,
                "All messages have been sent successfully!\
                It seems like there was less than 2 players so there'll be no presents :("
                    .to_owned(),
            ));
        } else {
            for (presenter, recipient) in Self::distribute_presents(users) {
                let Some(presenter) = self.get_user(&presenter) else {
                    return Err(Box::new(UserDoesNotExistError { id: presenter }));
                };
                let presenter_name = presenter.username;
                let Some(recipient) = self.get_user(&recipient) else {
                    return Err(Box::new(UserDoesNotExistError { id: recipient }));
                };
                let recipient_name = recipient.username;

                messages.push((
                    presenter.id,
                                format!(
                                    "Ho Ho Ho, {presenter_name}!\n\n\
                                     As a result of participating in game {game_name}. It looks like you have to prepare a present for {recipient_name}!\n\n\
                                     Have a happy new year, your secret santa bot."
                                ),
                ));
            }
            messages.push((
                game.admin,
                "All messages have been sent successfully!".to_owned(),
            ));
        }

        Ok(messages)
    }

    fn distribute_presents(users: Vec<UserId>) -> Vec<(UserId, UserId)> {
        let mut rng = thread_rng();
        let mut shuffled;
        loop {
            shuffled = users.clone();
            shuffled.shuffle(&mut rng);

            if !zip(users.clone(), shuffled.clone()).any(|(a, b)| a == b) {
                break;
            }
        }
        zip(users, shuffled).collect()
    }

    pub fn add_user_to_pending(
        &self,
        user_id: &UserId,
        game_id: &GameId,
    ) -> Result<(), Box<dyn Error>> {
        match self.get_game(game_id) {
            Some(game) => match self.get_user(user_id) {
                Some(user) => {
                    if game.pending_users.contains(user_id) || game.active_users.contains(user_id) {
                        return Err(Box::new(UserIsAlreadyInGameError {
                            user_id: *user_id,
                            game_id: *game_id,
                        }));
                    }
                    if user.pending_games.contains(game_id) || user.active_games.contains(game_id) {
                        return Err(Box::new(GameIsAlreadyInUserError {
                            user_id: *user_id,
                            game_id: *game_id,
                        }));
                    }

                    let mut pending_games = user.pending_games;
                    pending_games.push(*game_id);

                    let new_user = User {
                        id: user.id,
                        username: user.username,
                        admin_games: user.admin_games,
                        active_games: user.active_games,
                        pending_games,
                    };

                    let mut pending_users = game.pending_users;
                    pending_users.push(*user_id);

                    let new_game = Game {
                        id: game.id,
                        name: game.name,
                        admin: game.admin,
                        active_users: game.active_users,
                        pending_users,
                    };

                    let mut batch = Batch::default();
                    batch.insert(user_id.to_key().as_str(), new_user.to_ron().as_str());
                    batch.insert(game_id.to_key().as_str(), new_game.to_ron().as_str());

                    self.database.apply_batch(batch)?;

                    Ok(())
                }
                None => Err(Box::new(UserDoesNotExistError { id: *user_id })),
            },
            None => Err(Box::new(GameDoesNotExistError { id: *game_id })),
        }
    }
    pub fn promote_user_from_pending_to_active(
        &self,
        user_id: &UserId,
        game_id: &GameId,
    ) -> Result<(), Box<dyn Error>> {
        match self.get_game(game_id) {
            Some(game) => match self.get_user(user_id) {
                Some(user) => {
                    if !game.pending_users.contains(user_id) {
                        return Err(Box::new(UserIsNotInPendingError {
                            user_id: *user_id,
                            game_id: *game_id,
                        }));
                    }
                    if !user.pending_games.contains(game_id) {
                        return Err(Box::new(GameIsNotInPendingError {
                            user_id: *user_id,
                            game_id: *game_id,
                        }));
                    }

                    let mut pending_games = user.pending_games;
                    pending_games.retain(|id| id != game_id);
                    let mut active_games = user.active_games;
                    active_games.push(*game_id);

                    let new_user = User {
                        id: user.id,
                        username: user.username,
                        admin_games: user.admin_games,
                        active_games,
                        pending_games,
                    };

                    let mut pending_users = game.pending_users;
                    pending_users.retain(|id| id != user_id);
                    let mut active_users = game.active_users;
                    active_users.push(*user_id);

                    let new_game = Game {
                        id: game.id,
                        name: game.name,
                        admin: game.admin,
                        active_users,
                        pending_users,
                    };

                    let mut batch = Batch::default();
                    batch.insert(user_id.to_key().as_str(), new_user.to_ron().as_str());
                    batch.insert(game_id.to_key().as_str(), new_game.to_ron().as_str());

                    self.database.apply_batch(batch)?;

                    Ok(())
                }
                None => Err(Box::new(UserDoesNotExistError { id: *user_id })),
            },
            None => Err(Box::new(GameDoesNotExistError { id: *game_id })),
        }
    }
    // doesn't throw an error if user is not in game or othervise
    pub fn remove_user_from_game(
        &self,
        user_id: &UserId,
        game_id: &GameId,
    ) -> Result<(), Box<dyn Error>> {
        match self.get_game(game_id) {
            Some(game) => match self.get_user(user_id) {
                Some(user) => {
                    let mut pending_games = user.pending_games;
                    pending_games.retain(|id| id != game_id);
                    let mut active_games = user.active_games;
                    active_games.retain(|id| id != game_id);

                    let new_user = User {
                        id: user.id,
                        username: user.username,
                        admin_games: user.admin_games,
                        active_games,
                        pending_games,
                    };

                    let mut pending_users = game.pending_users;
                    pending_users.retain(|id| id != user_id);
                    let mut active_users = game.active_users;
                    active_users.retain(|id| id != user_id);

                    let new_game = Game {
                        id: game.id,
                        name: game.name,
                        admin: game.admin,
                        active_users,
                        pending_users,
                    };

                    let mut batch = Batch::default();
                    batch.insert(user_id.to_key().as_str(), new_user.to_ron().as_str());
                    batch.insert(game_id.to_key().as_str(), new_game.to_ron().as_str());

                    self.database.apply_batch(batch)?;

                    Ok(())
                }
                None => Err(Box::new(UserDoesNotExistError { id: *user_id })),
            },
            None => Err(Box::new(GameDoesNotExistError { id: *game_id })),
        }
    }
}
