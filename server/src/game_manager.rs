use super::game::player_view::{GameView, ListedGameView, ListedGameViewCollection};
use super::game::{Error, Game, GameUUID, PlayerUUID};
use super::Character;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct GameManager {
    games_by_game_id: HashMap<GameUUID, RwLock<Game>>,
    player_uuids_to_game_id: HashMap<PlayerUUID, GameUUID>,
    player_uuids_to_display_names: HashMap<PlayerUUID, String>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            player_uuids_to_display_names: HashMap::new(),
            games_by_game_id: HashMap::new(),
            player_uuids_to_game_id: HashMap::new(),
        }
    }

    pub fn add_player(
        &mut self,
        player_uuid: PlayerUUID,
        display_name: String,
    ) -> Result<(), Error> {
        if self
            .player_uuids_to_display_names
            .contains_key(&player_uuid)
        {
            return Err(Error::new("Player already exists"));
        }
        self.player_uuids_to_display_names
            .insert(player_uuid, display_name);
        Ok(())
    }

    pub fn remove_player(&mut self, player_uuid: &PlayerUUID) -> Result<(), Error> {
        self.assert_player_exists(player_uuid)?;
        if self.player_is_in_game(player_uuid) {
            self.leave_game(player_uuid)?;
        }
        self.player_uuids_to_display_names.remove(player_uuid);
        Ok(())
    }

    pub fn get_player_display_name(&self, player_uuid: &PlayerUUID) -> Option<&String> {
        self.player_uuids_to_display_names.get(player_uuid)
    }

    pub fn list_games(&self) -> ListedGameViewCollection {
        let mut listed_game_views: Vec<ListedGameView> = self
            .games_by_game_id
            .iter()
            .map(|(game_uuid, game)| game.read().unwrap().get_listed_game_view(game_uuid.clone()))
            .collect();
        listed_game_views.sort();
        ListedGameViewCollection { listed_game_views }
    }

    pub fn create_game(
        &mut self,
        player_uuid: PlayerUUID,
        game_name: String,
    ) -> Result<GameUUID, Error> {
        if self.player_uuids_to_game_id.contains_key(&player_uuid) {
            return Err(Error::new("Player is already in a game"));
        }
        self.assert_player_exists(&player_uuid)?;
        let game_id = GameUUID::new();
        let mut game = Game::new(game_name);
        game.join(player_uuid.clone())?;
        self.games_by_game_id
            .insert(game_id.clone(), RwLock::from(game));
        self.player_uuids_to_game_id
            .insert(player_uuid, game_id.clone());
        Ok(game_id)
    }

    pub fn join_game(&mut self, player_uuid: PlayerUUID, game_id: GameUUID) -> Result<(), Error> {
        self.assert_player_exists(&player_uuid)?;
        if self.player_uuids_to_game_id.contains_key(&player_uuid) {
            return Err(Error::new("Player is already in a game"));
        }
        let game = match self.games_by_game_id.get(&game_id) {
            Some(game) => game,
            None => return Err(Error::new("Game does not exist")),
        };
        game.write().unwrap().join(player_uuid.clone())?;
        self.player_uuids_to_game_id.insert(player_uuid, game_id);
        Ok(())
    }

    fn player_is_in_game(&self, player_uuid: &PlayerUUID) -> bool {
        self.player_uuids_to_game_id.contains_key(player_uuid)
    }

    pub fn leave_game(&mut self, player_uuid: &PlayerUUID) -> Result<(), Error> {
        self.assert_player_exists(player_uuid)?;
        let game_id = match self.player_uuids_to_game_id.get(player_uuid) {
            Some(game_id) => game_id,
            None => return Err(Error::new("Player is not in a game")),
        };
        let game_is_empty = {
            let game = match self.games_by_game_id.get(game_id) {
                Some(game) => game,
                None => return Err(Error::new("Game does not exist")),
            };
            let mut unlocked_game = game.write().unwrap();
            unlocked_game.leave(player_uuid)?;
            unlocked_game.is_empty()
        };
        if game_is_empty {
            self.games_by_game_id.remove(game_id);
        }
        self.player_uuids_to_game_id.remove(player_uuid);
        Ok(())
    }

    pub fn start_game(&self, player_uuid: &PlayerUUID) -> Result<(), Error> {
        let game = match self.get_game_of_player(player_uuid) {
            Ok(game) => game,
            Err(error) => return Err(error),
        };
        game.write().unwrap().start(player_uuid)
    }

    pub fn select_character(
        &self,
        player_uuid: &PlayerUUID,
        character: Character,
    ) -> Result<(), Error> {
        let game = match self.get_game_of_player(player_uuid) {
            Ok(game) => game,
            Err(error) => return Err(error),
        };
        game.write()
            .unwrap()
            .select_character(player_uuid, character)
    }

    fn assert_player_exists(&self, player_uuid: &PlayerUUID) -> Result<(), Error> {
        if !self.player_uuids_to_display_names.contains_key(player_uuid) {
            return Err(Error::new("Player does not exist"));
        }
        Ok(())
    }

    pub fn play_card(
        &self,
        player_uuid: &PlayerUUID,
        other_player_uuid_or: &Option<PlayerUUID>,
        card_index: usize,
    ) -> Result<(), Error> {
        let game = match self.get_game_of_player(player_uuid) {
            Ok(game) => game,
            Err(error) => return Err(error),
        };
        let mut unlocked_game = game.write().unwrap();
        if let Some(other_player_uuid) = other_player_uuid_or {
            if !unlocked_game.player_is_in_game(other_player_uuid) {
                return Err(Error::new(
                    "Other player is not in the same game or does not exist",
                ));
            }
        }
        unlocked_game.play_card(player_uuid, other_player_uuid_or, card_index)
    }

    pub fn discard_cards_and_draw_to_full(
        &self,
        player_uuid: &PlayerUUID,
        card_indices: Vec<usize>,
    ) -> Result<(), Error> {
        let game = match self.get_game_of_player(player_uuid) {
            Ok(game) => game,
            Err(error) => return Err(error),
        };
        game.write()
            .unwrap()
            .discard_cards_and_draw_to_full(player_uuid, card_indices)
    }

    pub fn order_drink(
        &self,
        player_uuid: &PlayerUUID,
        other_player_uuid: &PlayerUUID,
    ) -> Result<(), Error> {
        let game = match self.get_game_of_player(player_uuid) {
            Ok(game) => game,
            Err(error) => return Err(error),
        };
        game.write()
            .unwrap()
            .order_drink(player_uuid, other_player_uuid)
    }

    pub fn pass(&self, player_uuid: &PlayerUUID) -> Result<(), Error> {
        let game = match self.get_game_of_player(player_uuid) {
            Ok(game) => game,
            Err(error) => return Err(error),
        };
        game.write().unwrap().pass(player_uuid)
    }

    pub fn get_game_view(&self, player_uuid: PlayerUUID) -> Result<GameView, Error> {
        let game = self.get_game_of_player(&player_uuid)?;
        game.read()
            .unwrap()
            .get_game_view(player_uuid, &self.player_uuids_to_display_names)
    }

    fn get_game_of_player(&self, player_uuid: &PlayerUUID) -> Result<&RwLock<Game>, Error> {
        self.assert_player_exists(player_uuid)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_add_and_remove_player_without_error() {
        let mut game_manager = GameManager::new();

        let player_uuid = PlayerUUID::new();

        assert!(game_manager
            .add_player(player_uuid.clone(), String::from("Tommy"))
            .is_ok());
        assert!(game_manager.remove_player(&player_uuid).is_ok());
    }

    #[test]
    fn cannot_add_player_twice() {
        let mut game_manager = GameManager::new();

        let player_uuid = PlayerUUID::new();

        game_manager
            .add_player(player_uuid.clone(), String::from("Tommy"))
            .unwrap();
        assert_eq!(
            game_manager
                .add_player(player_uuid, String::from("Tommy"))
                .unwrap_err(),
            Error::new("Player already exists")
        );
    }

    #[test]
    fn cannot_remove_player_that_does_not_exist() {
        let mut game_manager = GameManager::new();

        let player_uuid = PlayerUUID::new();

        assert_eq!(
            game_manager.remove_player(&player_uuid).unwrap_err(),
            Error::new("Player does not exist")
        );

        game_manager
            .add_player(player_uuid.clone(), String::from("Tommy"))
            .unwrap();
        game_manager.remove_player(&player_uuid).unwrap();

        assert_eq!(
            game_manager.remove_player(&player_uuid).unwrap_err(),
            Error::new("Player does not exist")
        );
    }

    #[test]
    fn empty_games_are_removed() {
        let mut game_manager = GameManager::new();

        let player_uuid = PlayerUUID::new();

        game_manager
            .add_player(player_uuid.clone(), String::from("Tommy"))
            .unwrap();
        game_manager
            .create_game(player_uuid.clone(), "Game 1".to_string())
            .unwrap();

        assert_eq!(game_manager.games_by_game_id.len(), 1);
        assert_eq!(game_manager.leave_game(&player_uuid), Ok(()));
        assert_eq!(game_manager.games_by_game_id.len(), 0);
        assert_eq!(
            game_manager.leave_game(&player_uuid),
            Err(Error::new("Player is not in a game"))
        );
    }

    #[test]
    fn cannot_create_game_when_you_are_already_in_one() {
        let mut game_manager = GameManager::new();

        let player_uuid = PlayerUUID::new();

        game_manager
            .add_player(player_uuid.clone(), String::from("Tommy"))
            .unwrap();
        game_manager
            .create_game(player_uuid.clone(), "Game 1".to_string())
            .unwrap();
        assert_eq!(
            game_manager.create_game(player_uuid.clone(), "Game 1".to_string()),
            Err(Error::new("Player is already in a game"))
        );

        assert_eq!(game_manager.games_by_game_id.len(), 1);
    }
}
