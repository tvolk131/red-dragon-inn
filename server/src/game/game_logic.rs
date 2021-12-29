use super::player::{Player, PlayerUUID};
use super::{Character, Error};
use super::player_view::GameView;

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

    pub fn is_gambling_turn(&self, player_uuid: &PlayerUUID) -> bool {
        match &self.gambling_round_or {
            Some(gambling_round) => &gambling_round.current_player_turn == player_uuid,
            None => false,
        }
    }

    pub fn get_game_view(&self, player_uuid: &PlayerUUID) -> Result<GameView, Error> {
        // TODO - Implement this method.
        Err(Error("Method is not yet implemented".to_string()))
    }

    fn get_player_by_uuid_mut(&mut self, player_uuid: &PlayerUUID) -> Option<&mut Player> {
        match self
            .players
            .iter_mut()
            .find(|(uuid, _)| uuid == player_uuid)
        {
            Some((_, player)) => Some(player),
            None => None,
        }
    }

    pub fn play_card(&mut self, player_uuid: &PlayerUUID, card_index: usize) -> Option<Error> {
        let card_or = match self.get_player_by_uuid_mut(player_uuid) {
            Some(player) => player.pop_card_from_hand(&player_uuid, card_index),
            None => {
                return Some(Error(format!(
                    "Player does not exist with player id {}",
                    player_uuid.to_string()
                )))
            }
        };

        let card = match card_or {
            Some(card) => card,
            None => return Some(Error("Card does not exist".to_string())),
        };

        let return_val = if card.can_play(&player_uuid, self) {
            card.play(&player_uuid, self);
            None
        } else {
            Some(Error("Card cannot be played at this time".to_string()))
        };

        if let Some(player) = self.get_player_by_uuid_mut(player_uuid) {
            player.discard_card(card);
        }

        return_val
    }
}

struct GamblingRound {
    active_player_indexes: Vec<PlayerUUID>,
    current_player_turn: PlayerUUID,
    pot_amount: i32,
}

impl GamblingRound {
    fn new() -> Self {
        Self {
            active_player_indexes: Vec::new(),
            current_player_turn: PlayerUUID::new(),
            pot_amount: 0,
        }
    }
}
