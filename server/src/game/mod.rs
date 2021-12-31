mod drink;
mod error;
mod game_logic;
mod player;
mod player_card;
pub mod player_view;

pub use error::Error;
pub use player::PlayerUUID;

use game_logic::GameLogic;
use player_view::GameView;
use std::collections::HashSet;

pub struct Game {
    players: HashSet<PlayerUUID>,
    // Is `Some` if game is running, otherwise is `None`.
    game_logic_or: Option<GameLogic>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            players: HashSet::new(),
            game_logic_or: None,
        }
    }

    pub fn join(&mut self, player_uuid: PlayerUUID) -> Option<Error> {
        if !self.players.insert(player_uuid) {
            Some(Error::new("Player is already in this game"))
        } else {
            None
        }
    }

    pub fn leave(&mut self, player_uuid: &PlayerUUID) -> Option<Error> {
        if !self.players.remove(player_uuid) {
            Some(Error::new("Player is not in this game"))
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    /// Plays a card from the given player's hand.
    ///
    /// Accepts a zero-based card index which refers to a card in the player's hand.
    /// Returns an error if the card cannot currently be played or does not exist with given index or if the player does not exist.
    pub fn play_card(&mut self, player_uuid: &PlayerUUID, card_index: usize) -> Option<Error> {
        let game_logic = match self.get_mut_game_logic() {
            Ok(game_logic) => game_logic,
            Err(err) => return Some(err),
        };
        game_logic.play_card(player_uuid, card_index)
    }

    /// Discards any number of cards from the given player's hand.
    ///
    /// The values in `card_indices` represent cards in the player's hand.
    /// This must be called at the beginning of every player's turn.
    /// If the player doesn't want to discard anything, an empty vector
    /// should be passed in for `card_indices`.
    pub fn discard_cards(&self, player_uuid: &PlayerUUID, card_indices: Vec<i32>) -> Option<Error> {
        // TODO - Implement.
        None
    }

    /// Order a drink for another player.
    ///
    /// This must be called after the player's action phase is over.
    /// If the player has more than one drink to order, this must
    /// be called repeatedly until all drinks are handed out.
    pub fn order_drink(
        &mut self,
        player_uuid: &PlayerUUID,
        other_player_uuid: &PlayerUUID,
    ) -> Option<Error> {
        let game_logic = match self.get_mut_game_logic() {
            Ok(game_logic) => game_logic,
            Err(err) => return Some(err),
        };
        game_logic.order_drink(player_uuid, other_player_uuid)
    }

    pub fn pass(&self, player_uuid: &PlayerUUID) -> Option<Error> {
        // TODO - Implement.
        None
    }

    pub fn get_game_view(&self, player_uuid: &PlayerUUID) -> Result<GameView, Error> {
        self.get_game_logic()?.get_game_view(player_uuid)
    }

    fn get_game_logic(&self) -> Result<&GameLogic, Error> {
        match &self.game_logic_or {
            Some(game_logic) => Ok(game_logic),
            None => Err(Error::new("Game is not currently running")),
        }
    }

    fn get_mut_game_logic(&mut self) -> Result<&mut GameLogic, Error> {
        match &mut self.game_logic_or {
            Some(game_logic) => Ok(game_logic),
            None => Err(Error::new("Game is not currently running")),
        }
    }
}

pub enum Character {
    Fiona,
    Zot,
    Deirdre,
    Gerki,
}
