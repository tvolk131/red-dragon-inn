use super::game_logic::TurnInfo;
use super::player_manager::PlayerManager;
use super::uuid::PlayerUUID;
use super::Error;
use std::default::Default;

#[derive(Clone, Debug)]
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

    pub fn start_round(&mut self, player_uuid: PlayerUUID, player_manager: &mut PlayerManager) {
        if self.gambling_round_or.is_none() {
            self.gambling_round_or = Some(GamblingRound {
                active_player_uuids: player_manager.clone_uuids_of_all_alive_players(),
                current_player_turn: player_uuid.clone(),
                winning_player: player_uuid.clone(),
                pot_amount: 0,
                need_cheating_card_to_take_next_control: false,
            });
            self.ante_up(&player_uuid, player_manager);
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

        gambling_round.winning_player = player_uuid.clone();
        gambling_round.need_cheating_card_to_take_next_control =
            need_cheating_card_to_take_next_control;
        gambling_round.current_player_turn = player_uuid;
        gambling_round.increment_player_turn();
    }

    /// Forces a player to ante up. Fails silently if...
    /// 1. A gambling round is not running.
    /// 2. The player uuid doesn't map to an existing player in the game.
    ///
    /// This method can be considered atomic, since both conditions are verified before any modification.
    pub fn ante_up(&mut self, player_uuid: &PlayerUUID, player_manager: &mut PlayerManager) {
        let player = match player_manager.get_player_by_uuid_mut(player_uuid) {
            Some(player) => player,
            None => return,
        };

        let gambling_round = match &mut self.gambling_round_or {
            Some(gambling_round) => gambling_round,
            None => return,
        };

        player.change_gold(-1);
        gambling_round.pot_amount += 1;
    }

    pub fn pass(&mut self, player_manager: &mut PlayerManager, turn_info: &mut TurnInfo) {
        let (winner_or, pot_amount) = {
            {
                let gambling_round = match &mut self.gambling_round_or {
                    Some(gambling_round) => gambling_round,
                    None => return,
                };

                gambling_round.increment_player_turn();
            }

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
            self.end_round_and_discard_gold(turn_info);
        }
    }

    pub fn need_cheating_card_to_take_next_control(&self) -> bool {
        match &self.gambling_round_or {
            Some(gambling_round) => gambling_round.need_cheating_card_to_take_next_control,
            None => false,
        }
    }

    pub fn end_round_and_discard_gold(&mut self, turn_info: &mut TurnInfo) {
        self.gambling_round_or = None;
        turn_info.set_order_drinks_phase();
    }

    pub fn clone_uuids_of_all_active_players(&self) -> Vec<PlayerUUID> {
        match &self.gambling_round_or {
            Some(gambling_round) => gambling_round.active_player_uuids.clone(),
            None => Vec::new(),
        }
    }

    pub fn leave_gambling_round(&mut self, player_uuid: &PlayerUUID) -> Result<(), Error> {
        if let Some(gambling_round) = &mut self.gambling_round_or {
            // The last player in a gambling round can't leave
            if gambling_round.active_player_uuids.len() < 2 {
                return Err(Error::new(
                    "Last player in gambling round cannot leave the round",
                ));
            }

            if &gambling_round.current_player_turn == player_uuid {
                gambling_round.increment_player_turn();
            }

            gambling_round
                .active_player_uuids
                .retain(|active_player_uuid| active_player_uuid != player_uuid);

            Ok(())
        } else {
            Err(Error::new("Gambling round not running"))
        }
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

#[derive(Clone, Debug)]
struct GamblingRound {
    active_player_uuids: Vec<PlayerUUID>,
    current_player_turn: PlayerUUID,
    winning_player: PlayerUUID,
    pot_amount: i32,
    need_cheating_card_to_take_next_control: bool,
}

impl GamblingRound {
    fn increment_player_turn(&mut self) {
        let current_player_gambling_round_index_or = self
            .active_player_uuids
            .iter()
            .position(|player_uuid| player_uuid == &self.current_player_turn);

        let next_player_gambling_round_index = match current_player_gambling_round_index_or {
            Some(current_player_gambling_round_index) => {
                if current_player_gambling_round_index < self.active_player_uuids.len() - 1 {
                    current_player_gambling_round_index + 1
                } else {
                    0
                }
            }
            None => 0,
        };

        self.current_player_turn = self
            .active_player_uuids
            .get(next_player_gambling_round_index)
            .unwrap()
            .clone();
    }
}
