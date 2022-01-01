use super::drink::{create_drink_deck, Drink};
use super::player::{Player, PlayerUUID};
use super::player_view::GameViewPlayerData;
use super::{Character, Error};

pub struct GameLogic {
    players: Vec<(PlayerUUID, Player)>,
    drink_deck_draw_pile: Vec<Drink>,
    drink_deck_discard_pile: Vec<Drink>,
    turn_info: TurnInfo,
    gambling_round_or: Option<GamblingRound>,
}

impl GameLogic {
    pub fn new(characters: Vec<(PlayerUUID, Character)>) -> Result<Self, Error> {
        let player_count = characters.len();

        if player_count < 2 || player_count > 8 {
            return Err(Error::new("Must have between 2 and 8 players"));
        }

        Ok(Self {
            players: characters
                .into_iter()
                .map(|(player_uuid, character)| {
                    (
                        player_uuid,
                        Player::create_from_character(
                            character,
                            Self::get_starting_gold_amount_for_player_count(player_count),
                        ),
                    )
                })
                .collect(),
            drink_deck_draw_pile: create_drink_deck(),
            drink_deck_discard_pile: Vec::new(),
            turn_info: TurnInfo {
                player_turn: PlayerUUID::new(),
                turn_phase: TurnPhase::DiscardAndDraw,
            },
            gambling_round_or: None,
        })
    }

    pub fn get_current_player_turn<'a>(&'a self) -> &'a PlayerUUID {
        &self.turn_info.player_turn
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
                need_cheating_card_to_take_control: false,
            });
        }
    }

    pub fn gambling_take_control_of_round(
        &mut self,
        player_uuid: PlayerUUID,
        need_cheating_card_to_take_control: bool,
    ) {
        let gambling_round = match &mut self.gambling_round_or {
            Some(gambling_round) => gambling_round,
            None => return,
        };

        gambling_round.winning_player = player_uuid;
        gambling_round.need_cheating_card_to_take_control = need_cheating_card_to_take_control;
        self.gambling_increment_player_turn();
    }

    pub fn gambling_ante_up(&self) {
        // TODO - Implement
    }

    pub fn gambling_pass(&mut self) {
        self.gambling_increment_player_turn();

        let (winner_or, pot_amount) = {
            let gambling_round = match &self.gambling_round_or {
                Some(gambling_round) => gambling_round,
                None => return,
            };

            let winner_or = if self.is_gambling_turn(&gambling_round.winning_player) {
                Some(gambling_round.winning_player.clone())
            } else {
                None
            };

            (winner_or, gambling_round.pot_amount)
        };

        if let Some(winner) = winner_or {
            self.get_player_by_uuid_mut(&winner)
                .unwrap()
                .add_gold(pot_amount);
            self.gambling_round_or = None;
        }
    }

    pub fn gambling_need_cheating_card_to_take_control(&self) -> bool {
        match &self.gambling_round_or {
            Some(gambling_round) => gambling_round.need_cheating_card_to_take_control,
            None => false,
        }
    }

    fn gambling_increment_player_turn(&mut self) {
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

    pub fn is_gambling_turn(&self, player_uuid: &PlayerUUID) -> bool {
        match &self.gambling_round_or {
            Some(gambling_round) => &gambling_round.current_player_turn == player_uuid,
            None => false,
        }
    }

    pub fn get_game_view_player_data(&self) -> Vec<GameViewPlayerData> {
        self.players.iter().map(|(player_uuid, player)| player.to_game_view_player_data(player_uuid.clone())).collect()
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

    pub fn order_drink(
        &mut self,
        player_uuid: &PlayerUUID,
        other_player_uuid: &PlayerUUID,
    ) -> Option<Error> {
        // TODO - Implement.
        None
    }

    fn get_starting_gold_amount_for_player_count(player_count: usize) -> i32 {
        if player_count <= 2 {
            8
        } else if player_count >= 7 {
            12
        } else {
            10
        }
    }
}

struct GamblingRound {
    active_player_uuids: Vec<PlayerUUID>,
    current_player_turn: PlayerUUID,
    winning_player: PlayerUUID,
    pot_amount: i32,
    need_cheating_card_to_take_control: bool,
}

pub struct TurnInfo {
    player_turn: PlayerUUID,
    turn_phase: TurnPhase,
}

enum TurnPhase {
    DiscardAndDraw,
    Action,
    OrderDrinks,
    Drink,
}
