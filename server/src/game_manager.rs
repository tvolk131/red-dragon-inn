use super::game::player_view::GameView;
use super::game::{Error, Game, PlayerUUID};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct GameManager {
    games_by_game_id: HashMap<GameUUID, RwLock<Game>>,
    player_uuids_to_game_id: HashMap<PlayerUUID, GameUUID>,
    player_ids_to_display_names: HashMap<PlayerUUID, String>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            player_ids_to_display_names: HashMap::new(),
            games_by_game_id: HashMap::new(),
            player_uuids_to_game_id: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, player_uuid: PlayerUUID, display_name: String) -> Option<Error> {
        if self.player_ids_to_display_names.contains_key(&player_uuid) {
            return Some(Error::new("Player already exists"));
        }
        self.player_ids_to_display_names
            .insert(player_uuid, display_name);
        None
    }

    pub fn remove_player(&mut self, player_uuid: &PlayerUUID) -> Option<Error> {
        if let Some(err) = self.assert_player_exists(player_uuid) {
            return Some(err);
        }
        self.leave_game(player_uuid);
        self.player_ids_to_display_names.remove(player_uuid);
        None
    }

    pub fn create_game(
        &mut self,
        player_uuid: PlayerUUID,
        game_name: String,
    ) -> Result<GameUUID, Error> {
        if let Some(err) = self.assert_player_exists(&player_uuid) {
            return Err(err);
        }
        let game_id = GameUUID::new();
        let game = Game::new();
        game.join(player_uuid.clone());
        self.games_by_game_id
            .insert(game_id.clone(), RwLock::from(game));
        self.player_uuids_to_game_id
            .insert(player_uuid, game_id.clone());
        Ok(game_id)
    }

    pub fn join_game(&mut self, player_uuid: PlayerUUID, game_id: GameUUID) -> Option<Error> {
        if let Some(err) = self.assert_player_exists(&player_uuid) {
            return Some(err);
        }
        if self.player_uuids_to_game_id.contains_key(&player_uuid) {
            return Some(Error::new("Player is already in a game"));
        }
        let game = match self.games_by_game_id.get(&game_id) {
            Some(game) => game,
            None => return Some(Error::new("Game does not exist")),
        };
        match game.read().unwrap().join(player_uuid.clone()) {
            Some(err) => return Some(err),
            None => {}
        };
        self.player_uuids_to_game_id.insert(player_uuid, game_id);
        None
    }

    pub fn leave_game(&mut self, player_uuid: &PlayerUUID) -> Option<Error> {
        // TODO - Remove game if empty.
        if let Some(err) = self.assert_player_exists(&player_uuid) {
            return Some(err);
        }
        let game_id = match self.player_uuids_to_game_id.get(&player_uuid) {
            Some(game_id) => game_id,
            None => return Some(Error::new("Player is not in a game")),
        };
        let game_is_empty = {
            let game = match self.games_by_game_id.get(game_id) {
                Some(game) => game,
                None => return Some(Error::new("Game does not exist")),
            };
            let unlocked_game = game.read().unwrap();
            match unlocked_game.leave(player_uuid.clone()) {
                Some(err) => return Some(err),
                None => {}
            };
            unlocked_game.is_empty()
        };
        if game_is_empty {
            self.games_by_game_id.remove(game_id);
        }
        self.player_uuids_to_game_id.remove(player_uuid);
        None
    }

    fn assert_player_exists(&self, player_uuid: &PlayerUUID) -> Option<Error> {
        if !self.player_ids_to_display_names.contains_key(player_uuid) {
            return Some(Error::new("Player does not exist"));
        }
        None
    }

    pub fn play_card(&self, player_uuid: &PlayerUUID, card_index: usize) -> Option<Error> {
        let game = match self.get_game_of_player(player_uuid) {
            Ok(game) => game,
            Err(error) => return Some(error),
        };
        game.write().unwrap().play_card(player_uuid, card_index)
    }

    pub fn discard_cards(&self, player_uuid: &PlayerUUID, card_indices: Vec<i32>) -> Option<Error> {
        let game = match self.get_game_of_player(player_uuid) {
            Ok(game) => game,
            Err(error) => return Some(error),
        };
        game.read()
            .unwrap()
            .discard_cards(player_uuid, card_indices)
    }

    pub fn order_drink(
        &self,
        player_uuid: &PlayerUUID,
        other_player_uuid: &PlayerUUID,
    ) -> Option<Error> {
        let game = match self.get_game_of_player(player_uuid) {
            Ok(game) => game,
            Err(error) => return Some(error),
        };
        game.read()
            .unwrap()
            .order_drink(player_uuid, other_player_uuid)
    }

    pub fn pass(
        &self,
        player_uuid: &PlayerUUID,
    ) -> Option<Error> {
        let game = match self.get_game_of_player(player_uuid) {
            Ok(game) => game,
            Err(error) => return Some(error),
        };
        game.read()
            .unwrap()
            .pass(player_uuid)
    }

    pub fn get_game_view(&self, player_uuid: &PlayerUUID) -> Result<GameView, Error> {
        let game = self.get_game_of_player(player_uuid)?;
        game.read().unwrap().get_game_view(player_uuid)
    }

    fn get_game_of_player(&self, player_uuid: &PlayerUUID) -> Result<&RwLock<Game>, Error> {
        let error = Err(Error::new("Player is not in a game"));
        let game_id = match self.player_uuids_to_game_id.get(player_uuid) {
            Some(game_id) => game_id,
            None => return error,
        };
        match self.games_by_game_id.get(game_id) {
            Some(game) => Ok(game),
            None => error,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct GameUUID(String);

impl GameUUID {
    pub fn new() -> Self {
        // TODO - Should generate actual unique id rather than an empty string.
        Self("".to_string())
    }
}

impl<'a> rocket::request::FromParam<'a> for GameUUID {
    type Error = String;
    fn from_param(param: &'a str) -> Result<Self, String> {
        Ok(Self(String::from(param)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_add_and_remove_player_without_error() {
        let mut game_manager = GameManager::new();

        let player_uuid = PlayerUUID::new();

        assert!(game_manager
            .add_player(player_uuid.clone(), String::from("Tommy"))
            .is_none());
        assert!(game_manager.remove_player(&player_uuid).is_none());
    }

    #[test]
    fn cannot_add_player_twice() {
        let mut game_manager = GameManager::new();

        let player_uuid = PlayerUUID::new();

        game_manager.add_player(player_uuid.clone(), String::from("Tommy"));
        assert_eq!(
            game_manager
                .add_player(player_uuid, String::from("Tommy"))
                .unwrap(),
            Error::new("Player already exists")
        );
    }

    #[test]
    fn cannot_remove_player_that_does_not_exist() {
        let mut game_manager = GameManager::new();

        let player_uuid = PlayerUUID::new();

        assert_eq!(
            game_manager.remove_player(&player_uuid).unwrap(),
            Error::new("Player does not exist")
        );

        game_manager.add_player(player_uuid.clone(), String::from("Tommy"));
        game_manager.remove_player(&player_uuid);

        assert_eq!(
            game_manager.remove_player(&player_uuid).unwrap(),
            Error::new("Player does not exist")
        );
    }
}
