use super::game_logic::TurnInfo;
use super::player_manager::PlayerManager;
use super::uuid::PlayerUUID;
use std::default::Default;

#[derive(Clone)]
pub struct GamblingManager {
    gambling_round_or: Option<GamblingRound>,
}

impl GamblingManager {
    pub fn new() -> Self {
        Self {
            gambling_round_or: None,
        }
    }

    pub fn round_in_progress(&self) -> bool {
        self.gambling_round_or.is_some()
    }

    pub fn start_round(&mut self, player_uuid: PlayerUUID, player_manager: &PlayerManager) {
        if self.gambling_round_or.is_none() {
            self.gambling_round_or = Some(GamblingRound {
                active_player_uuids: player_manager.clone_uuids_of_all_alive_players(),
                current_player_turn: player_uuid.clone(),
                winning_player: player_uuid,
                pot_amount: 0,
                need_cheating_card_to_take_next_control: false,
            });
        }
    }

    pub fn take_control_of_round(
        &mut self,
        player_uuid: PlayerUUID,
        need_cheating_card_to_take_next_control: bool,
    ) {
        let gambling_round = match &mut self.gambling_round_or {
            Some(gambling_round) => gambling_round,
            None => return,
        };

        gambling_round.winning_player = player_uuid;
        gambling_round.need_cheating_card_to_take_next_control =
            need_cheating_card_to_take_next_control;
        self.increment_player_turn();
    }

    pub fn ante_up(&mut self, player_uuid: &PlayerUUID, player_manager: &mut PlayerManager) {
        if !self.is_turn(player_uuid) {
            return;
        }

        match &mut self.gambling_round_or {
            Some(gambling_round) => {
                gambling_round.pot_amount += gambling_round.active_player_uuids.len() as i32
            }
            None => return,
        };

        player_manager
            .get_player_by_uuid_mut(player_uuid)
            .unwrap()
            .change_gold(-1);

        self.increment_player_turn();
    }

    pub fn pass(&mut self, player_manager: &mut PlayerManager, turn_info: &mut TurnInfo) {
        self.increment_player_turn();

        let (winner_or, pot_amount) = {
            let gambling_round = match &self.gambling_round_or {
                Some(gambling_round) => gambling_round,
                None => return,
            };

            let winner_or = if self.is_turn(&gambling_round.winning_player) {
                Some(gambling_round.winning_player.clone())
            } else {
                None
            };

            (winner_or, gambling_round.pot_amount)
        };

        if let Some(winner) = winner_or {
            player_manager
                .get_player_by_uuid_mut(&winner)
                .unwrap()
                .change_gold(pot_amount);
            self.gambling_round_or = None;
            turn_info.set_order_drinks_phase();
        }
    }

    pub fn need_cheating_card_to_take_next_control(&self) -> bool {
        match &self.gambling_round_or {
            Some(gambling_round) => gambling_round.need_cheating_card_to_take_next_control,
            None => false,
        }
    }

    fn increment_player_turn(&mut self) {
        let gambling_round = match &mut self.gambling_round_or {
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

    pub fn is_turn(&self, player_uuid: &PlayerUUID) -> bool {
        match &self.gambling_round_or {
            Some(gambling_round) => &gambling_round.current_player_turn == player_uuid,
            None => false,
        }
    }
}

impl Default for GamblingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct GamblingRound {
    active_player_uuids: Vec<PlayerUUID>,
    current_player_turn: PlayerUUID,
    winning_player: PlayerUUID,
    pot_amount: i32,
    need_cheating_card_to_take_next_control: bool,
}