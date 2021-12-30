use super::player::{Player, PlayerUUID};
use super::player_view::GameView;
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
            // TODO - Currently this is dummy data. Properly implement it.
            self.gambling_round_or = Some(GamblingRound {
                active_player_uuids: Vec::new(),
                current_player_turn: PlayerUUID::new(),
                winning_player: PlayerUUID::new(),
                pot_amount: 0,
            });
        }
    }

    pub fn gambling_take_control_of_round(&self, player_uuid: PlayerUUID) {
        let gambling_round = match self.gambling_round_or {
            Some(gambling_round) => gambling_round,
            None => return,
        };

        gambling_round.winning_player = player_uuid;
        self.gambling_increment_player_turn();
    }

    pub fn gambling_ante_up(&self) {
        // TODO - Implement
    }

    pub fn gambling_pass(&self) {
        self.gambling_increment_player_turn();

        let gambling_round = match self.gambling_round_or {
            Some(gambling_round) => gambling_round,
            None => return,
        };

        if gambling_round.winning_player == gambling_round.current_player_turn {
            self.get_player_by_uuid_mut(&gambling_round.winning_player)
                .unwrap()
                .add_gold(gambling_round.pot_amount);
            self.gambling_round_or = None;
        }
    }

    fn gambling_increment_player_turn(&self) {
        let gambling_round = match self.gambling_round_or {
            Some(gambling_round) => gambling_round,
            None => return,
        };

        let current_player_gambling_round_index_or = gambling_round
            .active_player_uuids
            .iter()
            .position(|player_uuid| player_uuid == &gambling_round.current_player_turn);

        let next_player_gambling_round_index = match current_player_gambling_round_index_or {
            Some(current_player_gambling_round_index) => {
                if current_player_gambling_round_index
                    < gambling_round.active_player_uuids.len() - 1
                {
                    current_player_gambling_round_index + 1
                } else {
                    0
                }
            }
            None => 0,
        };

        gambling_round.current_player_turn = gambling_round
            .active_player_uuids
            .get(next_player_gambling_round_index)
            .unwrap()
            .clone();
    }

    pub fn is_gambling_turn(&self, player_uuid: &PlayerUUID) -> bool {
        match &self.gambling_round_or {
            Some(gambling_round) => &gambling_round.current_player_turn == player_uuid,
            None => false,
        }
    }

    pub fn get_game_view(&self, player_uuid: &PlayerUUID) -> Result<GameView, Error> {
        // TODO - Implement this method.
        Err(Error::new("Method is not yet implemented"))
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
                return Some(Error::new(format!(
                    "Player does not exist with player id {}",
                    player_uuid.to_string()
                )))
            }
        };

        let card = match card_or {
            Some(card) => card,
            None => return Some(Error::new("Card does not exist")),
        };

        let return_val = if card.can_play(&player_uuid, self) {
            card.play(&player_uuid, self);
            None
        } else {
            Some(Error::new("Card cannot be played at this time"))
        };

        if let Some(player) = self.get_player_by_uuid_mut(player_uuid) {
            player.discard_card(card);
        }

        return_val
    }
}

struct GamblingRound {
    active_player_uuids: Vec<PlayerUUID>,
    current_player_turn: PlayerUUID,
    winning_player: PlayerUUID,
    pot_amount: i32,
}
