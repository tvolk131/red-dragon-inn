use super::player::{Player, PlayerUUID};
use super::{Character, Error};

pub struct GameLogic {
    players: Vec<(PlayerUUID, Player)>,
    current_player_turn: PlayerUUID,
    gambling_round_or: Option<GamblingRound>,
}

impl GameLogic {
    pub fn new(characters: Vec<Character>) -> Self {
        Self {
            players: Vec::new(),
            current_player_turn: PlayerUUID::new(),
            gambling_round_or: None,
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

    pub fn play_card(&mut self, player_uuid: PlayerUUID, card_index: usize) -> Option<Error> {
        match self
            .players
            .iter_mut()
            .find(|(uuid, _)| *uuid == player_uuid)
        {
            Some((player_uuid, player)) => player.play_card_from_hand(card_index),
            None => Some(Error(format!(
                "Player does not exist with player id {}",
                player_uuid.to_string()
            ))),
        }
    }
}

struct GamblingRound {
    active_player_indexes: Vec<i32>,
    pot_amount: i32,
}

impl GamblingRound {
    fn new() -> Self {
        Self {
            active_player_indexes: Vec::new(),
            pot_amount: 0,
        }
    }
}
