use super::deck::AutoShufflingDeck;
use super::drink::{create_drink_deck, Drink};
use super::gambling_manager::GamblingManager;
use super::interrupt_manager::InterruptManager;
use super::player_card::{PlayerCard, RootPlayerCard, ShouldInterrupt, TargetStyle};
use super::player_manager::{NextPlayerUUIDOption, PlayerManager};
use super::player_view::{GameViewPlayerCard, GameViewPlayerData};
use super::uuid::PlayerUUID;
use super::{Character, Error};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone)]
pub struct GameLogic {
    player_manager: PlayerManager,
    gambling_manager: GamblingManager,
    interrupt_manager: InterruptManager,
    drink_deck: AutoShufflingDeck<Box<dyn Drink>>,
    turn_info: TurnInfo,
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
            gambling_manager: GamblingManager::new(),
            interrupt_manager: InterruptManager::new(),
            drink_deck: AutoShufflingDeck::new(create_drink_deck()),
            turn_info: TurnInfo::new(first_player_uuid),
        })
    }

    pub fn get_turn_info(&self) -> &TurnInfo {
        &self.turn_info
    }

    pub fn get_game_view_player_data_of_all_players(&self) -> Vec<GameViewPlayerData> {
        self.player_manager
            .get_game_view_player_data_of_all_players()
    }

    pub fn get_game_view_player_hand(&self, player_uuid: &PlayerUUID) -> Vec<GameViewPlayerCard> {
        match self.player_manager.get_player_by_uuid(player_uuid) {
            Some(player) => player.get_game_view_hand(
                player_uuid,
                &self.gambling_manager,
                &self.interrupt_manager,
                &self.turn_info,
            ),
            None => Vec::new(),
        }
    }

    pub fn get_turn_phase(&self) -> TurnPhase {
        self.turn_info.turn_phase
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

        match self.process_card(card, player_uuid, other_player_uuid_or) {
            Ok(card_or) => {
                if let Some(card) = card_or {
                    self.player_manager
                        .get_player_by_uuid_mut(player_uuid)
                        .unwrap()
                        .discard_card(card);
                }
                Ok(())
            }
            Err((card, err)) => {
                self.player_manager
                    .get_player_by_uuid_mut(player_uuid)
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
        if self.get_turn_info().get_current_player_turn() != player_uuid
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
        if self.get_turn_info().get_current_player_turn() != player_uuid
            || self.turn_info.turn_phase != TurnPhase::OrderDrinks
        {
            return Err(Error::new("Cannot order drinks at this time"));
        }

        // TODO - Handle the unwrap here.
        let drink = self.drink_deck.draw_card().unwrap();
        let other_player = match self
            .player_manager
            .get_player_by_uuid_mut(other_player_uuid)
        {
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

    pub fn pass(&mut self, player_uuid: &PlayerUUID) -> Result<(), Error> {
        if self.interrupt_manager.is_turn_to_interrupt(player_uuid) {
            self.interrupt_manager
                .pass(&mut self.player_manager, &mut self.gambling_manager)?;
            return Ok(());
        }

        if self.gambling_manager.is_turn(player_uuid) {
            self.gambling_manager
                .pass(&mut self.player_manager, &mut self.turn_info);
            return Ok(());
        }

        if self
            .get_turn_info()
            .can_play_action_card(player_uuid, &self.gambling_manager)
        {
            self.skip_action_phase()?;
            return Ok(());
        }

        Err(Error::new("Cannot pass at this time"))
    }

    /// The return type for this method is a bit complex, but was carefully chosen.
    /// If `Ok` is returned, then the wrapped card should be discarded if it exists.
    /// If an error is returned, the card should be returned to the player's hand.
    fn process_card(
        &mut self,
        card: PlayerCard,
        player_uuid: &PlayerUUID,
        other_player_uuid_or: &Option<PlayerUUID>,
    ) -> Result<Option<PlayerCard>, (PlayerCard, Error)> {
        if card.can_play(
            player_uuid,
            &self.gambling_manager,
            &self.interrupt_manager,
            &self.turn_info,
        ) {
            match card {
                PlayerCard::RootPlayerCard(root_player_card) => {
                    match process_root_player_card(
                        root_player_card,
                        player_uuid,
                        other_player_uuid_or,
                        &mut self.player_manager,
                        &mut self.gambling_manager,
                        &mut self.interrupt_manager,
                        &mut self.turn_info,
                    ) {
                        Ok(card_or) => Ok(card_or.map(|card| card.into())),
                        Err((card, err)) => Err((card.into(), err)),
                    }
                }
                PlayerCard::InterruptPlayerCard(interrupt_player_card) => {
                    if other_player_uuid_or.is_some() {
                        Err((
                            interrupt_player_card.into(),
                            Error::new("Cannot direct this card at another player"),
                        ))
                    } else {
                        match self.interrupt_manager.play_interrupt_card(
                            interrupt_player_card,
                            player_uuid.clone(),
                            &mut self.player_manager,
                            &mut self.gambling_manager,
                        ) {
                            Ok(_) => Ok(None),
                            Err((card, error)) => Err((card.into(), error)),
                        }
                    }
                }
            }
        } else {
            Err((card, Error::new("Card cannot be played at this time")))
        }
    }

    fn skip_action_phase(&mut self) -> Result<(), Error> {
        if self.turn_info.turn_phase == TurnPhase::Action {
            self.turn_info.turn_phase = TurnPhase::OrderDrinks;
            Ok(())
        } else {
            Err(Error::new("It is not the player's action phase"))
        }
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
        match self
            .player_manager
            .get_next_alive_player_uuid(&self.turn_info.player_turn)
        {
            NextPlayerUUIDOption::Some(next_player_uuid) => {
                self.turn_info = TurnInfo::new(next_player_uuid.clone())
            }
            NextPlayerUUIDOption::PlayerNotFound => {
                // TODO - Figure out how to handle this. It SHOULD never be hit here. If it is, that means there's a bug.
            }
            NextPlayerUUIDOption::OnlyPlayerLeft => {
                // TODO - Declare this player as the winner.
            }
        };
    }
}

fn process_root_player_card(
    root_player_card: RootPlayerCard,
    player_uuid: &PlayerUUID,
    targeted_player_uuid_or: &Option<PlayerUUID>,
    player_manager: &mut PlayerManager,
    gambling_manager: &mut GamblingManager,
    interrupt_manager: &mut InterruptManager,
    turn_info: &mut TurnInfo,
) -> Result<Option<RootPlayerCard>, (RootPlayerCard, Error)> {
    if !root_player_card.can_play(player_uuid, gambling_manager, interrupt_manager, turn_info) {
        return Err((
            root_player_card,
            Error::new("Cannot play card at this time"),
        ));
    }

    match root_player_card.get_target_style() {
        TargetStyle::SelfPlayer => {
            if targeted_player_uuid_or.is_some() {
                return Err((
                    root_player_card,
                    Error::new("Cannot direct this card at another player"),
                ));
            }

            match root_player_card.pre_interrupt_play(
                player_uuid,
                player_manager,
                gambling_manager,
                turn_info,
            ) {
                ShouldInterrupt::Yes => {
                    if root_player_card.get_interrupt_data_or().is_some() {
                        interrupt_manager.start_single_player_interrupt(
                            root_player_card,
                            player_uuid.clone(),
                            player_uuid.clone(),
                            player_manager,
                            gambling_manager,
                        )?;
                        Ok(None)
                    } else {
                        root_player_card.interrupt_play(
                            player_uuid,
                            player_uuid,
                            player_manager,
                            gambling_manager,
                        );
                        Ok(Some(root_player_card))
                    }
                }
                ShouldInterrupt::No => Ok(Some(root_player_card)),
            }
        }
        TargetStyle::SingleOtherPlayer => {
            if let Some(targeted_player_uuid) = targeted_player_uuid_or {
                match root_player_card.pre_interrupt_play(
                    player_uuid,
                    player_manager,
                    gambling_manager,
                    turn_info,
                ) {
                    ShouldInterrupt::Yes => {
                        if root_player_card.get_interrupt_data_or().is_some() {
                            interrupt_manager.start_single_player_interrupt(
                                root_player_card,
                                player_uuid.clone(),
                                targeted_player_uuid.clone(),
                                player_manager,
                                gambling_manager,
                            )?;
                            Ok(None)
                        } else {
                            root_player_card.interrupt_play(
                                player_uuid,
                                targeted_player_uuid,
                                player_manager,
                                gambling_manager,
                            );
                            Ok(Some(root_player_card))
                        }
                    }
                    ShouldInterrupt::No => Ok(Some(root_player_card)),
                }
            } else {
                Err((
                    root_player_card,
                    Error::new("Must direct this card at another player"),
                ))
            }
        }
        TargetStyle::AllOtherPlayers => {
            if targeted_player_uuid_or.is_some() {
                return Err((
                    root_player_card,
                    Error::new("Cannot direct this card at another player"),
                ));
            }

            match root_player_card.pre_interrupt_play(
                player_uuid,
                player_manager,
                gambling_manager,
                turn_info,
            ) {
                ShouldInterrupt::Yes => {
                    let mut targeted_player_uuids = rotate_player_vec_to_start_with_player(
                        player_manager.clone_uuids_of_all_alive_players(),
                        player_uuid,
                    );
                    // Remove self from list.
                    // TODO - Add check here so that `remove` never panicks.
                    targeted_player_uuids.remove(0);

                    if root_player_card.get_interrupt_data_or().is_some() {
                        interrupt_manager.start_multi_player_interrupt(
                            root_player_card,
                            player_uuid,
                            targeted_player_uuids,
                            player_manager,
                            gambling_manager,
                        )?;
                        Ok(None)
                    } else {
                        for targeted_player_uuid in &targeted_player_uuids {
                            root_player_card.interrupt_play(
                                player_uuid,
                                targeted_player_uuid,
                                player_manager,
                                gambling_manager,
                            );
                        }
                        Ok(Some(root_player_card))
                    }
                }
                ShouldInterrupt::No => Ok(Some(root_player_card)),
            }
        }
        // TODO - This branch is almost identical to the one above. Let's reduce code duplication somehow.
        TargetStyle::AllPlayersIncludingSelf => {
            if targeted_player_uuid_or.is_some() {
                return Err((
                    root_player_card,
                    Error::new("Cannot direct this card at another player"),
                ));
            }

            match root_player_card.pre_interrupt_play(
                player_uuid,
                player_manager,
                gambling_manager,
                turn_info,
            ) {
                ShouldInterrupt::Yes => {
                    let targeted_player_uuids = rotate_player_vec_to_start_with_player(
                        player_manager.clone_uuids_of_all_alive_players(),
                        player_uuid,
                    );

                    if root_player_card.get_interrupt_data_or().is_some() {
                        interrupt_manager.start_multi_player_interrupt(
                            root_player_card,
                            player_uuid,
                            targeted_player_uuids,
                            player_manager,
                            gambling_manager,
                        )?;
                        Ok(None)
                    } else {
                        for targeted_player_uuid in &targeted_player_uuids {
                            root_player_card.interrupt_play(
                                player_uuid,
                                targeted_player_uuid,
                                player_manager,
                                gambling_manager,
                            );
                        }
                        Ok(Some(root_player_card))
                    }
                }
                ShouldInterrupt::No => Ok(Some(root_player_card)),
            }
        }
    }
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

    pub fn set_order_drinks_phase(&mut self) {
        self.turn_phase = TurnPhase::OrderDrinks
    }

    pub fn is_order_drink_phase(&self) -> bool {
        self.turn_phase == TurnPhase::OrderDrinks
    }

    pub fn add_drinks_to_order(&mut self, amount: i32) {
        self.drinks_to_order += amount;
    }

    pub fn get_current_player_turn(&self) -> &PlayerUUID {
        &self.player_turn
    }

    pub fn can_play_action_card(
        &self,
        player_uuid: &PlayerUUID,
        gambling_manager: &GamblingManager,
    ) -> bool {
        self.get_current_player_turn() == player_uuid
            && self.turn_phase == TurnPhase::Action
            && !gambling_manager.round_in_progress()
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
    use super::super::player_card::{
        change_other_player_fortitude_card, gambling_im_in_card,
        ignore_root_card_affecting_fortitude,
    };
    use super::*;

    #[test]
    fn can_handle_simple_gambling_round() {
        let player1_uuid = PlayerUUID::new();
        let player2_uuid = PlayerUUID::new();

        let mut game_logic = GameLogic::new(vec![
            (player1_uuid.clone(), Character::Deirdre),
            (player2_uuid.clone(), Character::Gerki),
        ])
        .unwrap();
        game_logic
            .discard_cards_and_draw_to_full(&player1_uuid, Vec::new())
            .unwrap();

        // Sanity check.
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player1_uuid)
                .unwrap()
                .get_gold(),
            8
        );
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player2_uuid)
                .unwrap()
                .get_gold(),
            8
        );
        assert_eq!(game_logic.gambling_manager.round_in_progress(), false);
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Start gambling round.
        assert!(game_logic
            .process_card(gambling_im_in_card().into(), &player1_uuid, &None)
            .is_ok());

        // Both players choose not to play an interrupt card.
        assert!(game_logic
            .interrupt_manager
            .is_turn_to_interrupt(&player1_uuid));
        game_logic
            .interrupt_manager
            .pass(
                &mut game_logic.player_manager,
                &mut game_logic.gambling_manager,
            )
            .unwrap();
        assert!(game_logic
            .interrupt_manager
            .is_turn_to_interrupt(&player2_uuid));
        game_logic
            .interrupt_manager
            .pass(
                &mut game_logic.player_manager,
                &mut game_logic.gambling_manager,
            )
            .unwrap();
        assert_eq!(game_logic.interrupt_manager.interrupt_in_progress(), false);

        // 1 gold should be subtracted from each player.
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player1_uuid)
                .unwrap()
                .get_gold(),
            7
        );
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player2_uuid)
                .unwrap()
                .get_gold(),
            7
        );
        assert_eq!(game_logic.gambling_manager.round_in_progress(), true);
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 2 does not take control of the gambling round, making player 1 the winner.
        game_logic
            .gambling_manager
            .pass(&mut game_logic.player_manager, &mut game_logic.turn_info);

        // Gambling pot should be given to the winner.
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player1_uuid)
                .unwrap()
                .get_gold(),
            9
        );
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player2_uuid)
                .unwrap()
                .get_gold(),
            7
        );
        assert_eq!(game_logic.gambling_manager.round_in_progress(), false);
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::OrderDrinks);
    }

    #[test]
    fn can_handle_change_other_player_fortitude_card() {
        let player1_uuid = PlayerUUID::new();
        let player2_uuid = PlayerUUID::new();

        let mut game_logic = GameLogic::new(vec![
            (player1_uuid.clone(), Character::Deirdre),
            (player2_uuid.clone(), Character::Gerki),
        ])
        .unwrap();
        game_logic
            .discard_cards_and_draw_to_full(&player1_uuid, Vec::new())
            .unwrap();

        // Sanity check.
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player1_uuid)
                .unwrap()
                .get_gold(),
            8
        );
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player2_uuid)
                .unwrap()
                .get_gold(),
            8
        );
        assert_eq!(game_logic.gambling_manager.round_in_progress(), false);
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        assert!(game_logic
            .process_card(
                change_other_player_fortitude_card("Punch in the face", -2).into(),
                &player1_uuid,
                &Some(player2_uuid.clone())
            )
            .is_ok());

        // Sanity check.
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player2_uuid)
                .unwrap()
                .get_fortitude(),
            20
        );

        // Player 2 choose not to play an interrupt card.
        assert!(game_logic
            .interrupt_manager
            .is_turn_to_interrupt(&player2_uuid));
        game_logic
            .interrupt_manager
            .pass(
                &mut game_logic.player_manager,
                &mut game_logic.gambling_manager,
            )
            .unwrap();
        assert_eq!(game_logic.interrupt_manager.interrupt_in_progress(), false);

        // Fortitude should be reduced.
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player2_uuid)
                .unwrap()
                .get_fortitude(),
            18
        );
    }

    #[test]
    fn can_handle_interrupted_change_other_player_fortitude_card() {
        let player1_uuid = PlayerUUID::new();
        let player2_uuid = PlayerUUID::new();

        let mut game_logic = GameLogic::new(vec![
            (player1_uuid.clone(), Character::Deirdre),
            (player2_uuid.clone(), Character::Gerki),
        ])
        .unwrap();
        game_logic
            .discard_cards_and_draw_to_full(&player1_uuid, Vec::new())
            .unwrap();

        // Sanity check.
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player1_uuid)
                .unwrap()
                .get_gold(),
            8
        );
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player2_uuid)
                .unwrap()
                .get_gold(),
            8
        );
        assert_eq!(game_logic.gambling_manager.round_in_progress(), false);
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        assert!(game_logic
            .process_card(
                change_other_player_fortitude_card("Punch in the face", -2).into(),
                &player1_uuid,
                &Some(player2_uuid.clone())
            )
            .is_ok());

        // Sanity check.
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player2_uuid)
                .unwrap()
                .get_fortitude(),
            20
        );

        // Player 2 plays an interrupt card.
        assert!(game_logic
            .interrupt_manager
            .is_turn_to_interrupt(&player2_uuid));
        assert!(game_logic
            .process_card(
                ignore_root_card_affecting_fortitude("Block punch").into(),
                &player2_uuid,
                &None
            )
            .is_ok());
        // Player 1 chooses not to play a countering interrupt card.
        assert!(game_logic
            .interrupt_manager
            .is_turn_to_interrupt(&player1_uuid));
        game_logic
            .interrupt_manager
            .pass(
                &mut game_logic.player_manager,
                &mut game_logic.gambling_manager,
            )
            .unwrap();
        assert_eq!(game_logic.interrupt_manager.interrupt_in_progress(), false);

        // Fortitude should not be reduced.
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player2_uuid)
                .unwrap()
                .get_fortitude(),
            20
        );
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
