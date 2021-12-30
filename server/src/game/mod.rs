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

pub struct Game {
    game_logic: GameLogic,
}

impl Game {
    pub fn new(characters: Vec<Character>) -> Self {
        Self {
            game_logic: GameLogic::new(characters),
        }
    }

    /// Plays a card from the given player's hand.
    ///
    /// Accepts a zero-based card index which refers to a card in the player's hand.
    /// Returns an error if the card cannot currently be played or does not exist with given index or if the player does not exist.
    pub fn play_card(&mut self, player_uuid: &PlayerUUID, card_index: usize) -> Option<Error> {
        self.game_logic.play_card(player_uuid, card_index)
    }

    /// Discards any number of cards from the given player's hand.
    ///
    /// The values in `card_indices` represent cards in the player's hand.
    /// This must be called at the beginning of every player's turn.
    /// If the player doesn't want to discard anything, an empty vector
    /// should be passed in for `card_indices`.
    pub fn discard_cards(&self, player_uuid: &PlayerUUID, card_indices: Vec<i32>) -> Option<Error> {
        None
    }

    /// Order a drink for another player.
    ///
    /// This must be called after the player's action phase is over.
    /// If the player has more than one drink to order, this must
    /// be called repeatedly until all drinks are handed out.
    pub fn order_drink(
        &self,
        player_uuid: &PlayerUUID,
        other_player_uuid: &PlayerUUID,
    ) -> Option<Error> {
        None
    }

    pub fn get_game_view(&self, player_uuid: &PlayerUUID) -> Result<GameView, Error> {
        self.game_logic.get_game_view(player_uuid)
    }
}

pub enum Character {
    Fiona,
    Zot,
    Deirdre,
    Gerki,
}
