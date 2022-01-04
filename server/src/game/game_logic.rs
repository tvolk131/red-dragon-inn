use super::deck::AutoShufflingDeck;
use super::drink::{create_drink_deck, Drink};
use super::player::Player;
use super::player_card::PlayerCard;
use super::player_view::GameViewPlayerData;
use super::uuid::PlayerUUID;
use super::{Character, Error};
use std::collections::HashSet;

pub struct GameLogic {
    players: Vec<(PlayerUUID, Player)>,
    drink_deck: AutoShufflingDeck<Box<dyn Drink>>,
    turn_info: TurnInfo,
    gambling_round_or: Option<GamblingRound>,
}

impl GameLogic {
    pub fn new(characters: Vec<(PlayerUUID, Character)>) -> Result<Self, Error> {
        let player_count = characters.len();

        if !(2..=8).contains(&player_count) {
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
            drink_deck: AutoShufflingDeck::new(create_drink_deck()),
            // TODO - Set this to the player who should go first.
            turn_info: TurnInfo::new(PlayerUUID::new()),
            gambling_round_or: None,
        })
    }

    pub fn get_current_player_turn(&self) -> &PlayerUUID {
        &self.turn_info.player_turn
    }

    pub fn can_play_action_card(&self, player_uuid: &PlayerUUID) -> bool {
        self.get_current_player_turn() == player_uuid
            && self.turn_info.turn_phase == TurnPhase::Action
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
        self.players
            .iter()
            .map(|(player_uuid, player)| player.to_game_view_player_data(player_uuid.clone()))
            .collect()
    }

    pub fn get_player_by_uuid_mut(&mut self, player_uuid: &PlayerUUID) -> Option<&mut Player> {
        match self
            .players
            .iter_mut()
            .find(|(uuid, _)| uuid == player_uuid)
        {
            Some((_, player)) => Some(player),
            None => None,
        }
    }

    pub fn play_card(
        &mut self,
        player_uuid: &PlayerUUID,
        other_player_uuid_or: &Option<PlayerUUID>,
        card_index: usize,
    ) -> Option<Error> {
        let card_or = match self.get_player_by_uuid_mut(player_uuid) {
            Some(player) => player.pop_card_from_hand(card_index),
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

        let return_val = if card.as_generic_player_card().can_play(player_uuid, self) {
            match &card {
                PlayerCard::SimplePlayerCard(simple_card) => {
                    if other_player_uuid_or.is_some() {
                        return Some(Error::new("Cannot direct this card at another player"));
                    }
                    simple_card.play(player_uuid, self);
                }
                PlayerCard::DirectedPlayerCard(directed_card) => {
                    let other_player_uuid = match other_player_uuid_or {
                        Some(other_player_uuid) => other_player_uuid,
                        None => return Some(Error::new("Must direct this card at another player")),
                    };
                    directed_card.play(player_uuid, other_player_uuid, self);
                }
            };
            None
        } else {
            Some(Error::new("Card cannot be played at this time"))
        };

        if let Some(player) = self.get_player_by_uuid_mut(player_uuid) {
            player.discard_card(card);
        }

        return_val
    }

    pub fn discard_cards_and_draw_to_full(
        &mut self,
        player_uuid: &PlayerUUID,
        mut card_indices: Vec<usize>,
    ) -> Option<Error> {
        if self.get_current_player_turn() != player_uuid
            || self.turn_info.turn_phase != TurnPhase::DiscardAndDraw
        {
            return Some(Error::new("Cannot discard cards at this time"));
        }

        let player = match self
            .players
            .iter_mut()
            .find(|(uuid, _)| uuid == player_uuid)
        {
            Some((_, player)) => player,
            None => return Some(Error::new("Player is not in the game")),
        };

        if card_indices.len()
            > card_indices
                .iter()
                .cloned()
                .collect::<HashSet<usize>>()
                .len()
        {
            return Some(Error::new("Cannot discard the same card twice"));
        }

        // Sort and reverse so that we can iterate backwards and pop all cards.
        // If we pop the cards in any other order, we some indices will have moved by the time we get to them.
        card_indices.sort_unstable();
        card_indices.reverse();

        for card_index in card_indices {
            let card = match player.pop_card_from_hand(card_index) {
                Some(card) => card,
                // Since we're iterating through the card indices in reverse order, and
                // the indices can't be negative since we're using `usize` to represent
                // them, this error will either be thrown on the first iteration of the
                // loop or not at all. So we can guarantee that this method will always
                // behave atomically.
                None => {
                    return Some(Error::new(
                        "Card indices do not all correspond to cards in the player's hand",
                    ))
                }
            };
            player.discard_card(card);
        }
        player.draw_to_full();
        self.turn_info.turn_phase = TurnPhase::Action;
        None
    }

    pub fn order_drink(
        &mut self,
        player_uuid: &PlayerUUID,
        other_player_uuid: &PlayerUUID,
    ) -> Option<Error> {
        if self.get_current_player_turn() != player_uuid
            || self.turn_info.turn_phase != TurnPhase::OrderDrinks
        {
            return Some(Error::new("Cannot order drinks at this time"));
        }

        // TODO - Handle the unwrap here.
        let drink = self.drink_deck.draw_card().unwrap();
        let other_player = match self.get_player_by_uuid_mut(other_player_uuid) {
            Some(other_player) => other_player,
            None => {
                return Some(Error::new(format!(
                    "Player does not exist with player id {}",
                    player_uuid.to_string()
                )))
            }
        };
        other_player.add_drink_to_drink_pile(drink);

        self.turn_info.drinks_to_order -= 1;
        if self.turn_info.drinks_to_order == 0 {
            self.turn_info.turn_phase = TurnPhase::Drink;
            // TODO - Automatically initiate drink phase.
        }
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
    drinks_to_order: i32,
}

impl TurnInfo {
    fn new(player_uuid: PlayerUUID) -> Self {
        Self {
            player_turn: player_uuid,
            turn_phase: TurnPhase::DiscardAndDraw,
            drinks_to_order: 1,
        }
    }
}

#[derive(PartialEq)]
enum TurnPhase {
    DiscardAndDraw,
    Action,
    OrderDrinks,
    Drink,
}
