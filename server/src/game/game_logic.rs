use super::deck::AutoShufflingDeck;
use super::drink::{create_drink_deck, Drink};
use super::player::Player;
use super::player_card::PlayerCard;
use super::player_view::{GameViewPlayerCard, GameViewPlayerData};
use super::uuid::PlayerUUID;
use super::{Character, Error};
use std::collections::HashSet;

#[derive(Clone)]
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

        // TODO - Set the first player to a random player (or whatever official RDI rules say).
        let first_player_uuid = characters.first().unwrap().0.clone();

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
            turn_info: TurnInfo::new(first_player_uuid),
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

    pub fn start_gambling_round(&mut self, player_uuid: PlayerUUID) {
        if self.gambling_round_or.is_none() {
            self.gambling_round_or = Some(GamblingRound {
                active_player_uuids: self.players.iter().map(|(uuid, _)| uuid).cloned().collect(),
                current_player_turn: player_uuid.clone(),
                winning_player: player_uuid,
                pot_amount: 0,
                need_cheating_card_to_take_control: false,
            });
        }
        self.gambling_ante_up();
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

    /// Forces all players that are still in the current gambling round to each
    /// put one more gold in the gambling pot. Then passes the gambling turn to
    /// the next player still in the gambling round.
    pub fn gambling_ante_up(&mut self) {
        let active_gambling_players = match &mut self.gambling_round_or {
            Some(gambling_round) => {
                gambling_round.pot_amount += gambling_round.active_player_uuids.len() as i32;
                gambling_round.active_player_uuids.clone()
            }
            None => return,
        };
        for player_uuid in active_gambling_players.iter() {
            self.get_player_by_uuid_mut(player_uuid)
                .unwrap()
                .change_gold(-1);
        }
        self.gambling_increment_player_turn();
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
                .change_gold(pot_amount);
            self.gambling_round_or = None;
            self.turn_info.turn_phase = TurnPhase::OrderDrinks
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

    pub fn get_game_view_player_hand(&self, player_uuid: &PlayerUUID) -> Vec<GameViewPlayerCard> {
        match self.get_player_by_uuid(player_uuid) {
            Some(player) => player.get_game_view_hand(player_uuid, self),
            None => Vec::new(),
        }
    }

    fn get_player_by_uuid(&self, player_uuid: &PlayerUUID) -> Option<&Player> {
        match self.players.iter().find(|(uuid, _)| uuid == player_uuid) {
            Some((_, player)) => Some(player),
            None => None,
        }
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

    pub fn is_action_phase(&self) -> bool {
        self.turn_info.turn_phase == TurnPhase::Action
    }

    pub fn skip_action_phase(&mut self) -> Option<Error> {
        if self.turn_info.turn_phase == TurnPhase::Action {
            self.turn_info.turn_phase = TurnPhase::OrderDrinks;
            None
        } else {
            Some(Error::new("It is not the player's action phase"))
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

        let return_val = if card.can_play(player_uuid, self) {
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
            self.perform_drink_phase(player_uuid)?;
        }
        None
    }

    fn perform_drink_phase(&mut self, player_uuid: &PlayerUUID) -> Option<Error> {
        let player = match self.get_player_by_uuid_mut(player_uuid) {
            Some(player) => player,
            None => {
                return Some(Error::new(format!(
                    "Player does not exist with player id {}",
                    player_uuid.to_string()
                )))
            }
        };

        if let Some(drink) = player.drink_from_drink_pile() {
            self.drink_deck.discard_card(drink);
        }
        self.start_next_player_turn();
        None
    }

    fn start_next_player_turn(&mut self) {
        let current_player_index = self
            .players
            .iter()
            .position(|(player_uuid, _)| player_uuid == &self.turn_info.player_turn)
            .unwrap();
        let mut next_player_index = current_player_index + 1;
        if next_player_index == self.players.len() {
            next_player_index = 0;
        }

        let entry = self.players.get(next_player_index).unwrap();
        let mut next_player_uuid = &entry.0;
        let mut next_player = &entry.1;

        while next_player.is_out_of_game() {
            next_player_index += 1;
            if next_player_index == self.players.len() {
                next_player_index = 0;
            }

            let entry = self.players.get(next_player_index).unwrap();
            next_player_uuid = &entry.0;
            next_player = &entry.1;

            if next_player_index == current_player_index {
                // TODO - Break from loop and declare this player as the winner.
            }
        }

        self.turn_info = TurnInfo::new(next_player_uuid.clone());
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

#[derive(Clone)]
struct GamblingRound {
    active_player_uuids: Vec<PlayerUUID>,
    current_player_turn: PlayerUUID,
    winning_player: PlayerUUID,
    pot_amount: i32,
    need_cheating_card_to_take_control: bool,
}

#[derive(Clone)]
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

#[derive(Clone, PartialEq, Debug)]
enum TurnPhase {
    DiscardAndDraw,
    Action,
    OrderDrinks,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_simple_gambling_round() {
        let player1_uuid = PlayerUUID::new();
        let player2_uuid = PlayerUUID::new();

        let mut game_logic = GameLogic::new(vec![(player1_uuid.clone(), Character::Deirdre), (player2_uuid, Character::Gerki)]).unwrap();
        game_logic.discard_cards_and_draw_to_full(&player1_uuid, Vec::new());

        // Sanity check.
        assert_eq!(game_logic.players.first().unwrap().1.get_gold(), 8);
        assert_eq!(game_logic.players.last().unwrap().1.get_gold(), 8);        
        assert!(game_logic.gambling_round_or.is_none());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        game_logic.start_gambling_round(player1_uuid);

        assert_eq!(game_logic.players.first().unwrap().1.get_gold(), 7);
        assert_eq!(game_logic.players.last().unwrap().1.get_gold(), 7);
        assert!(game_logic.gambling_round_or.is_some());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        game_logic.gambling_pass();

        assert!(game_logic.gambling_round_or.is_none());
        assert_eq!(game_logic.players.first().unwrap().1.get_gold(), 9);
        assert_eq!(game_logic.players.last().unwrap().1.get_gold(), 7);
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::OrderDrinks);
    }
}
