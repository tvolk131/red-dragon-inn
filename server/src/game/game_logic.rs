use super::deck::AutoShufflingDeck;
use super::drink::{create_drink_deck, Drink};
use super::game_interrupt::{GameInterruptType, GameInterrupts};
use super::player_card::{ShouldInterrupt, PlayerCard, RootPlayerCard, TargetStyle};
use super::player_view::{GameViewPlayerCard, GameViewPlayerData};
use super::uuid::PlayerUUID;
use super::{Character, Error};
use super::player_manager::{PlayerManager, NextPlayerUUIDOption};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone)]
pub struct GameLogic {
    player_manager: PlayerManager,
    drink_deck: AutoShufflingDeck<Box<dyn Drink>>,
    turn_info: TurnInfo,
    gambling_round_or: Option<GamblingRound>,
    interrupts: GameInterrupts,
    current_interrupt_turn_or: Option<PlayerUUID>
}

impl GameLogic {
    pub fn new(players_with_characters: Vec<(PlayerUUID, Character)>) -> Result<Self, Error> {
        if !(2..=8).contains(&players_with_characters.len()) {
            return Err(Error::new("Must have between 2 and 8 players"));
        }

        // TODO - Set the first player to a random player (or whatever official RDI rules say).
        let first_player_uuid = players_with_characters.first().unwrap().0.clone();

        Ok(Self {
            player_manager: PlayerManager::new(players_with_characters),
            drink_deck: AutoShufflingDeck::new(create_drink_deck()),
            turn_info: TurnInfo::new(first_player_uuid),
            gambling_round_or: None,
            interrupts: GameInterrupts::new(),
            current_interrupt_turn_or: None
        })
    }

    pub fn get_current_player_turn(&self) -> &PlayerUUID {
        &self.turn_info.player_turn
    }

    pub fn can_play_action_card(&self, player_uuid: &PlayerUUID) -> bool {
        self.get_current_player_turn() == player_uuid
            && self.turn_info.turn_phase == TurnPhase::Action
            && self.gambling_round_or.is_none()
    }

    pub fn get_player_manager_mut(&mut self) -> &mut PlayerManager {
        &mut self.player_manager
    }

    pub fn gambling_round_in_progress(&self) -> bool {
        self.gambling_round_or.is_some()
    }

    pub fn start_gambling_round(&mut self, player_uuid: PlayerUUID) {
        if self.gambling_round_or.is_none() {
            self.gambling_round_or = Some(GamblingRound {
                active_player_uuids: self.player_manager.clone_uuids_of_all_alive_players(),
                current_player_turn: player_uuid.clone(),
                winning_player: player_uuid,
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

    pub fn gambling_ante_up(&mut self, player_uuid: &PlayerUUID) {
        if !self.is_gambling_turn(player_uuid) {
            return
        }

        match &mut self.gambling_round_or {
            Some(gambling_round) => gambling_round.pot_amount += gambling_round.active_player_uuids.len() as i32,
            None => return,
        };

        self.player_manager.get_player_by_uuid_mut(player_uuid)
            .unwrap()
            .change_gold(-1);

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
            self.player_manager.get_player_by_uuid_mut(&winner)
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

    pub fn get_game_view_player_data_of_all_players(&self) -> Vec<GameViewPlayerData> {
        self.player_manager.get_game_view_player_data_of_all_players()
    }

    pub fn get_game_view_player_hand(&self, player_uuid: &PlayerUUID) -> Vec<GameViewPlayerCard> {
        match self.player_manager.get_player_by_uuid(player_uuid) {
            Some(player) => player.get_game_view_hand(player_uuid, self),
            None => Vec::new(),
        }
    }

    pub fn get_current_interrupt(&self) -> Option<GameInterruptType> {
        self.interrupts.get_current_interrupt()
    }

    pub fn is_action_phase(&self) -> bool {
        self.turn_info.turn_phase == TurnPhase::Action
    }

    pub fn get_turn_phase(&self) -> TurnPhase {
        self.turn_info.turn_phase
    }

    pub fn skip_action_phase(&mut self) -> Result<(), Error> {
        if self.turn_info.turn_phase == TurnPhase::Action {
            self.turn_info.turn_phase = TurnPhase::OrderDrinks;
            Ok(())
        } else {
            Err(Error::new("It is not the player's action phase"))
        }
    }

    pub fn play_card(
        &mut self,
        player_uuid: &PlayerUUID,
        other_player_uuid_or: &Option<PlayerUUID>,
        card_index: usize,
    ) -> Result<(), Error> {
        let card_or = match self.player_manager.get_player_by_uuid_mut(player_uuid) {
            Some(player) => player.pop_card_from_hand(card_index),
            None => {
                return Err(Error::new(format!(
                    "Player does not exist with player id {}",
                    player_uuid.to_string()
                )))
            }
        };

        // This must be discarded before the functions ends. So
        // there should be no early returns after this statement.
        let card = match card_or {
            Some(card) => card,
            None => return Err(Error::new("Card does not exist")),
        };

        // This is a bit complex, but the type of `return_val` was carefully chosen. If `Ok` is returned, then the wrapped card should be discarded if it exists.
        // If an error is returned, the card should always be returned to the player's hand.
        let return_val: Result<Option<PlayerCard>, (PlayerCard, Error)> = if card
            .can_play(player_uuid, self)
        {
            match card {
                PlayerCard::RootPlayerCard(root_player_card) => {
                    process_root_player_card(root_player_card, player_uuid, other_player_uuid_or, self)
                }
                PlayerCard::InterruptPlayerCard(interrupt_player_card) => {
                    if other_player_uuid_or.is_some() {
                        Err((
                            interrupt_player_card.into(),
                            Error::new("Cannot direct this card at another player"),
                        ))
                    } else if self.interrupts.is_empty() {
                        Err((
                            interrupt_player_card.into(),
                            Error::new("Cannot play an interrupt card at this time"),
                        ))
                    } else if let Err(interrupt_player_card) =
                        self.interrupts.push_to_current_stack(
                            interrupt_player_card.get_interrupt_type_output(),
                            interrupt_player_card,
                            player_uuid.clone(),
                        )
                    {
                        Err((
                            interrupt_player_card.into(),
                            Error::new("Cannot play this card at this time"),
                        ))
                    } else {
                        self.increment_current_interrupt_player_turn();
                        Ok(None)
                    }
                }
            }
        } else {
            Err((card, Error::new("Card cannot be played at this time")))
        };

        match return_val {
            Ok(card_or) => {
                if let Some(card) = card_or {
                    self.player_manager.get_player_by_uuid_mut(player_uuid)
                        .unwrap()
                        .discard_card(card);
                }
                Ok(())
            }
            Err((card, err)) => {
                self.player_manager.get_player_by_uuid_mut(player_uuid)
                    .unwrap()
                    .return_card_to_hand(card, card_index);
                Err(err)
            }
        }
    }

    pub fn discard_cards_and_draw_to_full(
        &mut self,
        player_uuid: &PlayerUUID,
        mut card_indices: Vec<usize>,
    ) -> Result<(), Error> {
        if self.get_current_player_turn() != player_uuid
            || self.turn_info.turn_phase != TurnPhase::DiscardAndDraw
        {
            return Err(Error::new("Cannot discard cards at this time"));
        }

        let player = match self.player_manager.get_player_by_uuid_mut(player_uuid) {
            Some(player) => player,
            None => return Err(Error::new("Player is not in the game")),
        };

        if card_indices.len()
            > card_indices
                .iter()
                .cloned()
                .collect::<HashSet<usize>>()
                .len()
        {
            return Err(Error::new("Cannot discard the same card twice"));
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
                    return Err(Error::new(
                        "Card indices do not all correspond to cards in the player's hand",
                    ))
                }
            };
            player.discard_card(card);
        }
        player.draw_to_full();
        self.turn_info.turn_phase = TurnPhase::Action;
        Ok(())
    }

    pub fn order_drink(
        &mut self,
        player_uuid: &PlayerUUID,
        other_player_uuid: &PlayerUUID,
    ) -> Result<(), Error> {
        if self.get_current_player_turn() != player_uuid
            || self.turn_info.turn_phase != TurnPhase::OrderDrinks
        {
            return Err(Error::new("Cannot order drinks at this time"));
        }

        // TODO - Handle the unwrap here.
        let drink = self.drink_deck.draw_card().unwrap();
        let other_player = match self.player_manager.get_player_by_uuid_mut(other_player_uuid) {
            Some(other_player) => other_player,
            None => {
                return Err(Error::new(format!(
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
        Ok(())
    }

    fn perform_drink_phase(&mut self, player_uuid: &PlayerUUID) -> Result<(), Error> {
        let player = match self.player_manager.get_player_by_uuid_mut(player_uuid) {
            Some(player) => player,
            None => {
                return Err(Error::new(format!(
                    "Player does not exist with player id {}",
                    player_uuid.to_string()
                )))
            }
        };

        if let Some(drink) = player.drink_from_drink_pile() {
            self.drink_deck.discard_card(drink);
        }
        self.start_next_player_turn();
        Ok(())
    }

    fn start_next_player_turn(&mut self) {
        match self.player_manager.get_next_alive_player_uuid(&self.turn_info.player_turn) {
            NextPlayerUUIDOption::Some(next_player_uuid) => self.turn_info = TurnInfo::new(next_player_uuid.clone()),
            NextPlayerUUIDOption::PlayerNotFound => {
                // TODO - Figure out how to handle this. It SHOULD never be hit here. If it is, that means there's a bug.
            },
            NextPlayerUUIDOption::OnlyPlayerLeft => {
                // TODO - Declare this player as the winner.
            }
        };
    }

    fn increment_current_interrupt_player_turn(&mut self) {
        if let Some(current_interrupt_turn) = &self.current_interrupt_turn_or {
            match self.player_manager.get_next_alive_player_uuid(current_interrupt_turn) {
                NextPlayerUUIDOption::Some(next_player_uuid) => {
                    // If, after incrementing the player turn, the interrupt turn has
                    // looped back around to the last player who played a card, then
                    // that ends the interrupt stack since that player was uninterrupted.
                    if Some(next_player_uuid) == self.interrupts.get_last_player_to_play_on_current_stack() {
                        self.interrupts.resolve_current_stack(self);
                        self.current_interrupt_turn_or = None;
                    } else {
                        self.current_interrupt_turn_or = Some(next_player_uuid.clone());
                    }
                },
                _ => {} // TODO - Return an error here.
            };
        }
    }
}

fn process_root_player_card(root_player_card: RootPlayerCard, player_uuid: &PlayerUUID, targeted_player_uuid_or: &Option<PlayerUUID>, game_logic: &mut GameLogic) -> Result<Option<PlayerCard>, (PlayerCard, Error)> {
    if !root_player_card.can_play(player_uuid, game_logic) {
        return Err((root_player_card.into(), Error::new("Cannot play card at this time")));
    }

    match root_player_card.get_target_style() {
        TargetStyle::SingleOtherPlayer => {
            if let Some(targeted_player_uuid) = targeted_player_uuid_or {
                match root_player_card.pre_interrupt_play(player_uuid, game_logic) {
                    ShouldInterrupt::Yes => {
                        match root_player_card.get_interrupt_data_or() {
                            Some(interrupt_data) => {
                                game_logic.interrupts.push_new_stack(interrupt_data.get_interrupt_style(), root_player_card, player_uuid.clone(), targeted_player_uuid.clone());
                                game_logic.current_interrupt_turn_or = Some(player_uuid.clone());
                                game_logic.increment_current_interrupt_player_turn();
                                Ok(None)
                            },
                            None => {
                                root_player_card.interrupt_play(player_uuid, targeted_player_uuid, game_logic);
                                Ok(Some(root_player_card.into()))
                            }
                        }
                    },
                    ShouldInterrupt::No => Ok(Some(root_player_card.into()))
                }
            } else {
                Err((root_player_card.into(), Error::new("Must direct this card at another player")))
            }
        },
        TargetStyle::AllOtherPlayers => {
            if targeted_player_uuid_or.is_some() {
                return Err((
                    root_player_card.into(),
                    Error::new("Cannot direct this card at another player"),
                ));
            }

            match root_player_card.pre_interrupt_play(player_uuid, game_logic) {
                ShouldInterrupt::Yes => {
                    let mut targeted_player_uuids =
                        rotate_player_vec_to_start_with_player(
                            game_logic.player_manager.clone_uuids_of_all_alive_players(),
                            player_uuid,
                        );
                    // Remove self from list.
                    // TODO - Add check here so that `remove` never panicks.
                    targeted_player_uuids.remove(0);

                    match root_player_card.get_interrupt_data_or() {
                        Some(interrupt_data) => {
                            game_logic.interrupts.push_new_stacks(interrupt_data.get_interrupt_style(), root_player_card, player_uuid, targeted_player_uuids);
                            game_logic.current_interrupt_turn_or = Some(player_uuid.clone());
                            game_logic.increment_current_interrupt_player_turn();
                            Ok(None)
                        },
                        None => {
                            for targeted_player_uuid in &targeted_player_uuids {
                                root_player_card.interrupt_play(player_uuid, targeted_player_uuid, game_logic);
                            }
                            Ok(Some(root_player_card.into()))
                        }
                    }
                },
                ShouldInterrupt::No => Ok(Some(root_player_card.into()))
            }
        },
        // TODO - This branch is almost identical to the one above. Let's reduce code duplication somehow.
        TargetStyle::AllPlayersIncludingSelf => {
            if targeted_player_uuid_or.is_some() {
                return Err((
                    root_player_card.into(),
                    Error::new("Cannot direct this card at another player"),
                ));
            }

            match root_player_card.pre_interrupt_play(player_uuid, game_logic) {
                ShouldInterrupt::Yes => {
                    let targeted_player_uuids =
                        rotate_player_vec_to_start_with_player(
                            game_logic.player_manager.clone_uuids_of_all_alive_players(),
                            player_uuid,
                        );

                    match root_player_card.get_interrupt_data_or() {
                        Some(interrupt_data) => {
                            game_logic.interrupts.push_new_stacks(interrupt_data.get_interrupt_style(), root_player_card, player_uuid, targeted_player_uuids);
                            game_logic.current_interrupt_turn_or = Some(player_uuid.clone());
                            game_logic.increment_current_interrupt_player_turn();
                            Ok(None)
                        },
                        None => {
                            for targeted_player_uuid in &targeted_player_uuids {
                                root_player_card.interrupt_play(player_uuid, targeted_player_uuid, game_logic);
                            }
                            Ok(Some(root_player_card.into()))
                        }
                    }
                },
                ShouldInterrupt::No => Ok(Some(root_player_card.into()))
            }
        },
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

#[derive(Clone, Copy, PartialEq, Debug, Serialize)]
pub enum TurnPhase {
    DiscardAndDraw,
    Action,
    OrderDrinks,
}

fn rotate_player_vec_to_start_with_player(
    mut players: Vec<PlayerUUID>,
    starting_player_uuid: &PlayerUUID,
) -> Vec<PlayerUUID> {
    let player_index = players
        .iter()
        .position(|player_uuid| player_uuid == starting_player_uuid)
        .unwrap_or(0);
    players.rotate_left(player_index);
    players
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_simple_gambling_round() {
        let player1_uuid = PlayerUUID::new();
        let player2_uuid = PlayerUUID::new();

        let mut game_logic = GameLogic::new(vec![
            (player1_uuid.clone(), Character::Deirdre),
            (player2_uuid, Character::Gerki),
        ])
        .unwrap();
        game_logic
            .discard_cards_and_draw_to_full(&player1_uuid, Vec::new())
            .unwrap();

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

    #[test]
    fn test_rotate_player_vec_to_start_with_player() {
        let player1_uuid = PlayerUUID::new();
        let player2_uuid = PlayerUUID::new();
        let player3_uuid = PlayerUUID::new();
        let player4_uuid = PlayerUUID::new();

        let player_uuids = vec![
            player1_uuid.clone(),
            player2_uuid.clone(),
            player3_uuid.clone(),
            player4_uuid.clone(),
        ];

        assert_eq!(
            rotate_player_vec_to_start_with_player(player_uuids.clone(), &player1_uuid),
            vec![
                player1_uuid.clone(),
                player2_uuid.clone(),
                player3_uuid.clone(),
                player4_uuid.clone()
            ]
        );

        assert_eq!(
            rotate_player_vec_to_start_with_player(player_uuids.clone(), &player2_uuid),
            vec![
                player2_uuid.clone(),
                player3_uuid.clone(),
                player4_uuid.clone(),
                player1_uuid.clone(),
            ]
        );

        assert_eq!(
            rotate_player_vec_to_start_with_player(player_uuids.clone(), &player3_uuid),
            vec![
                player3_uuid.clone(),
                player4_uuid.clone(),
                player1_uuid.clone(),
                player2_uuid.clone(),
            ]
        );

        assert_eq!(
            rotate_player_vec_to_start_with_player(player_uuids.clone(), &player4_uuid),
            vec![
                player4_uuid.clone(),
                player1_uuid.clone(),
                player2_uuid.clone(),
                player3_uuid.clone(),
            ]
        );

        assert_eq!(
            rotate_player_vec_to_start_with_player(player_uuids, &PlayerUUID::new()),
            vec![
                player1_uuid.clone(),
                player2_uuid.clone(),
                player3_uuid.clone(),
                player4_uuid.clone(),
            ]
        );
    }
}
