use super::game::player_view::GameView;
use super::game::{Error, Game, PlayerUUID};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct GameManager {
    // TODO - Game ID should be a UUID struct similar to PlayerUUID rather than just a string.
    games_by_game_id: HashMap<String, RwLock<Game>>,
    player_uuids_to_game_id: HashMap<PlayerUUID, String>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            games_by_game_id: HashMap::new(),
            player_uuids_to_game_id: HashMap::new(),
        }
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
        game.read().unwrap().discard_cards(player_uuid, card_indices)
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
        game.read().unwrap().order_drink(player_uuid, other_player_uuid)
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
