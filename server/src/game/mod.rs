mod drink;
mod player;
mod player_card;

use player::{Player, PlayerUUID};

pub struct Game {
    game_logic: GameLogic
}

impl Game {
    pub fn new(characters: Vec<Character>) -> Self {
        Self {
            game_logic: GameLogic::new(characters)
        }
    }

    /// Plays a card from the given player's hand.
    ///
    /// Accepts a zero-based card index which refers to a card in the player's hand.
    /// Returns an error if the card cannot currently be played.
    pub fn play_card(&self, player_uuid: PlayerUUID, card_index: i32) {}

    /// Discards any number of cards from the given player's hand.
    ///
    /// The values in `card_indices` represent cards in the player's hand.
    /// This must be called at the beginning of every player's turn.
    /// If the player doesn't want to discard anything, an empty vector
    /// should be passed in for `card_indices`.
    pub fn discard_cards(&self, player_uuid: PlayerUUID, card_indices: Vec<i32>) {}

    /// Order a drink for another player.
    ///
    /// This must be called after the player's action phase is over.
    /// If the player has more than one drink to order, this must
    /// be called repeatedly until all drinks are handed out.
    pub fn order_drink(&self, player_uuid: PlayerUUID, other_player_uuid: PlayerUUID) {}
}

pub enum Character {
    Fiona,
    Zot,
    Deirdre,
    Gerki
}

pub struct GameLogic {
    players: Vec<Player>,
    current_player_turn: PlayerUUID,
    gambling_round_or: Option<GamblingRound>,
}

impl GameLogic {
    fn new(characters: Vec<Character>) -> Self {
        Self {
            players: Vec::new(),
            current_player_turn: PlayerUUID::new(),
            gambling_round_or: None
        }
    }

    pub fn get_current_player_turn<'a>(&'a self) -> &'a PlayerUUID {
        &self.current_player_turn
    }

    pub fn gambling_round_in_progress(&self) -> bool {
        self.gambling_round_or.is_some()
    }

    pub fn start_gambling_round(&mut self) {
        if self.gambling_round_or.is_none() {
            self.gambling_round_or = Some(GamblingRound::new());
        }
    }

    pub fn take_control_of_gambling_round(&self) {}

    pub fn gambling_ante_up(&self) {}
}

struct GamblingRound {
    active_player_indexes: Vec<i32>,
    pot_amount: i32
}

impl GamblingRound {
    fn new() -> Self {
        Self {
            active_player_indexes: Vec::new(),
            pot_amount: 0
        }
    }
}
