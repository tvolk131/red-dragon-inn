use super::game::player_view::GameView;
use super::game::{Error, Game, PlayerUUID};
use std::collections::HashMap;

pub struct GameManager {
    // TODO - Game ID should be a UUID struct similar to PlayerUUID rather than just a string.
    games_by_game_id: HashMap<String, Game>,
    player_uuids_to_game_id: HashMap<PlayerUUID, String>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            games_by_game_id: HashMap::new(),
            player_uuids_to_game_id: HashMap::new(),
        }
    }

    pub fn play_card(&self, player_uuid: &PlayerUUID) -> Option<Error> {
        None
    }

    pub fn discard_cards(&self, player_uuid: &PlayerUUID, card_indices: Vec<i32>) -> Option<Error> {
        None
    }

    pub fn order_drink(
        &self,
        player_uuid: &PlayerUUID,
        other_player_uuid: &PlayerUUID,
    ) -> Option<Error> {
        None
    }

    pub fn get_game_view(&self, player_uuid: &PlayerUUID) -> Result<GameView, Error> {
        Err(Error::new("Get game view method is unimplemented"))
    }
}
