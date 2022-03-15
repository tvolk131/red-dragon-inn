use super::deck::AutoShufflingDeck;
use super::drink::{create_drink_deck, DrinkCard};
use super::gambling_manager::GamblingManager;
use super::interrupt_manager::InterruptManager;
use super::player_card::{PlayerCard, RootPlayerCard, ShouldInterrupt, TargetStyle};
use super::player_manager::{NextPlayerUUIDOption, PlayerManager};
use super::player_view::{GameViewInterruptData, GameViewPlayerCard, GameViewPlayerData};
use super::uuid::PlayerUUID;
use super::{Character, Error};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct GameLogic {
    player_manager: PlayerManager,
    gambling_manager: GamblingManager,
    interrupt_manager: InterruptManager,
    drink_deck: AutoShufflingDeck<DrinkCard>,
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

    pub fn get_game_view_interrupt_data_or(&self) -> Option<GameViewInterruptData> {
        self.interrupt_manager.get_game_view_interrupt_data_or()
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

        if player_uuid == other_player_uuid {
            return Err(Error::new("Cannot order drink for yourself"));
        }

        if let Some(drink) = self.drink_deck.draw_card() {
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
        };

        self.turn_info.drinks_to_order -= 1;
        if self.turn_info.drinks_to_order == 0 {
            self.perform_drink_phase(player_uuid)?;
        }

        Ok(())
    }

    pub fn player_can_pass(&self, player_uuid: &PlayerUUID) -> bool {
        self.clone().pass(player_uuid).is_ok()
    }

    pub fn pass(&mut self, player_uuid: &PlayerUUID) -> Result<(), Error> {
        if self.interrupt_manager.interrupt_in_progress() {
            if self.interrupt_manager.is_turn_to_interrupt(player_uuid) {
                let spent_cards_or = self.interrupt_manager.pass(
                    &mut self.player_manager,
                    &mut self.gambling_manager,
                    &mut self.turn_info,
                )?;
                if let Some(spent_cards) = spent_cards_or {
                    if spent_cards.current_user_action_phase_is_over() {
                        self.skip_action_phase()?;
                    }
                    self.player_manager
                        .discard_cards(spent_cards.take_all_player_cards())
                        .unwrap();
                }
                return Ok(());
            } else {
                return Err(Error::new("Cannot pass at this time"));
            }
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
                        self,
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
                            &mut self.turn_info,
                        ) {
                            Ok(spent_cards_or) => {
                                if let Some(spent_cards) = spent_cards_or {
                                    if spent_cards.current_user_action_phase_is_over() {
                                        self.skip_action_phase().unwrap();
                                    }
                                    self.player_manager
                                        .discard_cards(spent_cards.take_all_player_cards())
                                        .unwrap();
                                }
                                Ok(None)
                            }
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

        for drink_card in player.drink_from_drink_pile() {
            self.drink_deck.discard_card(drink_card);
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
    game_logic: &mut GameLogic,
) -> Result<Option<RootPlayerCard>, (RootPlayerCard, Error)> {
    if !root_player_card.can_play(
        player_uuid,
        &game_logic.gambling_manager,
        &game_logic.interrupt_manager,
        &game_logic.turn_info,
    ) {
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
                &mut game_logic.player_manager,
                &mut game_logic.gambling_manager,
                &mut game_logic.turn_info,
            ) {
                ShouldInterrupt::Yes => {
                    if root_player_card.get_interrupt_data_or().is_some() {
                        game_logic.interrupt_manager.start_single_player_interrupt(
                            root_player_card,
                            player_uuid.clone(),
                            player_uuid.clone(),
                        )?;
                        Ok(None)
                    } else {
                        root_player_card.interrupt_play(
                            player_uuid,
                            player_uuid,
                            &mut game_logic.player_manager,
                            &mut game_logic.gambling_manager,
                        );
                        Ok(Some(root_player_card))
                    }
                }
                ShouldInterrupt::No => Ok(Some(root_player_card)),
            }
        }
        TargetStyle::SingleOtherPlayer => {
            if let Some(targeted_player_uuid) = targeted_player_uuid_or {
                if player_uuid == targeted_player_uuid {
                    return Err((
                        root_player_card,
                        Error::new("Must not direct this card at yourself"),
                    ));
                }

                match root_player_card.pre_interrupt_play(
                    player_uuid,
                    &mut game_logic.player_manager,
                    &mut game_logic.gambling_manager,
                    &mut game_logic.turn_info,
                ) {
                    ShouldInterrupt::Yes => {
                        if root_player_card.get_interrupt_data_or().is_some() {
                            game_logic.interrupt_manager.start_single_player_interrupt(
                                root_player_card,
                                player_uuid.clone(),
                                targeted_player_uuid.clone(),
                            )?;
                            Ok(None)
                        } else {
                            root_player_card.interrupt_play(
                                player_uuid,
                                targeted_player_uuid,
                                &mut game_logic.player_manager,
                                &mut game_logic.gambling_manager,
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
            let mut targeted_player_uuids = rotate_player_vec_to_start_with_player(
                game_logic.player_manager.clone_uuids_of_all_alive_players(),
                player_uuid,
            );

            // This check is here because `remove` panicks if the index does not exist.
            if !targeted_player_uuids.is_empty() {
                // Remove self from list.
                targeted_player_uuids.remove(0);
            }

            target_root_card_at_list_of_players(
                player_uuid,
                targeted_player_uuid_or,
                targeted_player_uuids,
                root_player_card,
                game_logic,
            )
        }
        TargetStyle::AllGamblingPlayersIncludingSelf => target_root_card_at_list_of_players(
            player_uuid,
            targeted_player_uuid_or,
            rotate_player_vec_to_start_with_player(
                game_logic
                    .gambling_manager
                    .clone_uuids_of_all_active_players(),
                player_uuid,
            ),
            root_player_card,
            game_logic,
        ),
    }
}

fn target_root_card_at_list_of_players(
    player_uuid: &PlayerUUID,
    targeted_player_uuid_or: &Option<PlayerUUID>,
    targeted_player_uuids: Vec<PlayerUUID>,
    root_player_card: RootPlayerCard,
    game_logic: &mut GameLogic,
) -> Result<Option<RootPlayerCard>, (RootPlayerCard, Error)> {
    if targeted_player_uuid_or.is_some() {
        return Err((
            root_player_card,
            Error::new("Cannot direct this card at another player"),
        ));
    }

    match root_player_card.pre_interrupt_play(
        player_uuid,
        &mut game_logic.player_manager,
        &mut game_logic.gambling_manager,
        &mut game_logic.turn_info,
    ) {
        ShouldInterrupt::Yes => {
            if root_player_card.get_interrupt_data_or().is_some() {
                game_logic.interrupt_manager.start_multi_player_interrupt(
                    root_player_card,
                    player_uuid.clone(),
                    targeted_player_uuids,
                )?;
                Ok(None)
            } else {
                for targeted_player_uuid in &targeted_player_uuids {
                    root_player_card.interrupt_play(
                        player_uuid,
                        targeted_player_uuid,
                        &mut game_logic.player_manager,
                        &mut game_logic.gambling_manager,
                    );
                }
                Ok(Some(root_player_card))
            }
        }
        ShouldInterrupt::No => Ok(Some(root_player_card)),
    }
}

#[derive(Clone, Debug)]
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
    use super::super::drink::create_simple_ale_test_drink;
    use super::super::player_card::{
        change_other_player_fortitude_card, gain_fortitude_anytime_card, gambling_cheat_card,
        gambling_im_in_card, i_raise_card, ignore_root_card_affecting_fortitude,
        wench_bring_some_drinks_for_my_friends_card, winning_hand_card,
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
        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 1 starts gambling round.
        assert!(game_logic
            .process_card(gambling_im_in_card().into(), &player1_uuid, &None)
            .is_ok());

        // Player 2 chooses not to play an interrupt card.
        assert!(game_logic
            .interrupt_manager
            .is_turn_to_interrupt(&player2_uuid));
        assert!(!game_logic.player_can_pass(&player1_uuid));
        assert!(game_logic.player_can_pass(&player2_uuid));
        game_logic.pass(&player2_uuid).unwrap();
        assert!(!game_logic.interrupt_manager.interrupt_in_progress());

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
        assert!(game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 2 does not take control of the gambling round, making player 1 the winner.
        assert!(game_logic.gambling_manager.is_turn(&player2_uuid));
        assert!(!game_logic.player_can_pass(&player1_uuid));
        assert!(game_logic.player_can_pass(&player2_uuid));
        game_logic.pass(&player2_uuid).unwrap();

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
        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::OrderDrinks);
    }

    #[test]
    fn raise_in_gambling_round() {
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
        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 1 starts gambling round.
        assert!(game_logic
            .process_card(gambling_im_in_card().into(), &player1_uuid, &None)
            .is_ok());

        // Player 2 chooses not to play an interrupt card.
        assert!(game_logic
            .interrupt_manager
            .is_turn_to_interrupt(&player2_uuid));
        assert!(!game_logic.player_can_pass(&player1_uuid));
        assert!(game_logic.player_can_pass(&player2_uuid));
        game_logic.pass(&player2_uuid).unwrap();
        assert!(!game_logic.interrupt_manager.interrupt_in_progress());

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
        assert!(game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 2 raises.
        assert!(game_logic.gambling_manager.is_turn(&player2_uuid));
        assert!(!game_logic.player_can_pass(&player1_uuid));
        assert!(game_logic.player_can_pass(&player2_uuid));
        assert!(game_logic
            .process_card(i_raise_card().into(), &player2_uuid, &None)
            .is_ok());

        // Player 2 chooses not to interrupt their ante.
        assert!(!game_logic.player_can_pass(&player1_uuid));
        assert!(game_logic.player_can_pass(&player2_uuid));
        game_logic.pass(&player2_uuid).unwrap();
        // Player 1 chooses not to interrupt their ante.
        assert!(game_logic.player_can_pass(&player1_uuid));
        assert!(!game_logic.player_can_pass(&player2_uuid));
        game_logic.pass(&player1_uuid).unwrap();

        // 1 more gold should be subtracted from each player.
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player1_uuid)
                .unwrap()
                .get_gold(),
            6
        );
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player2_uuid)
                .unwrap()
                .get_gold(),
            6
        );

        // Player 1 does not take control of the gambling round, making player 2 the winner.
        assert!(game_logic.gambling_manager.is_turn(&player1_uuid));
        assert!(game_logic.player_can_pass(&player1_uuid));
        assert!(!game_logic.player_can_pass(&player2_uuid));
        game_logic.pass(&player1_uuid).unwrap();

        // Gambling pot should be given to the winner.
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player1_uuid)
                .unwrap()
                .get_gold(),
            6
        );
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player2_uuid)
                .unwrap()
                .get_gold(),
            10
        );
        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::OrderDrinks);
    }

    #[test]
    fn cheat_in_gambling_round() {
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
        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 1 starts gambling round.
        assert!(game_logic
            .process_card(gambling_im_in_card().into(), &player1_uuid, &None)
            .is_ok());

        // Player 2 chooses not to play an interrupt card.
        assert!(game_logic
            .interrupt_manager
            .is_turn_to_interrupt(&player2_uuid));
        assert!(!game_logic.player_can_pass(&player1_uuid));
        assert!(game_logic.player_can_pass(&player2_uuid));
        game_logic.pass(&player2_uuid).unwrap();
        assert!(!game_logic.interrupt_manager.interrupt_in_progress());

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
        assert!(game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 2 plays a winning hand card.
        assert!(game_logic
            .process_card(winning_hand_card().into(), &player2_uuid, &None)
            .is_ok());

        // Player 1 attempts to play a regular gambling card.
        assert_eq!(
            game_logic
                .process_card(gambling_im_in_card().into(), &player1_uuid, &None)
                .unwrap_err()
                .1,
            Error::new("Card cannot be played at this time")
        );

        // Player 1 plays a cheating card.
        assert!(game_logic
            .process_card(
                gambling_cheat_card("Card up the sleeve").into(),
                &player1_uuid,
                &None
            )
            .is_ok());

        // Player 2 does not take control of the gambling round, making player 1 the winner.
        assert!(game_logic.gambling_manager.is_turn(&player2_uuid));
        assert!(!game_logic.player_can_pass(&player1_uuid));
        assert!(game_logic.player_can_pass(&player2_uuid));
        game_logic.pass(&player2_uuid).unwrap();

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
        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::OrderDrinks);
    }

    #[test]
    fn cannot_play_gambling_cards_during_game_interrupts() {
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
        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Start gambling round.
        assert!(game_logic
            .process_card(gambling_im_in_card().into(), &player1_uuid, &None)
            .is_ok());

        // Other player can choose to interrupt their ante (but doesn't yet).
        assert!(game_logic
            .interrupt_manager
            .is_turn_to_interrupt(&player2_uuid));

        // Neither player can play other gambling cards.
        assert!(!i_raise_card().can_play(
            &player1_uuid,
            &game_logic.gambling_manager,
            &game_logic.interrupt_manager,
            &game_logic.turn_info
        ));
        assert!(!i_raise_card().can_play(
            &player2_uuid,
            &game_logic.gambling_manager,
            &game_logic.interrupt_manager,
            &game_logic.turn_info
        ));
        assert!(!gambling_im_in_card().can_play(
            &player1_uuid,
            &game_logic.gambling_manager,
            &game_logic.interrupt_manager,
            &game_logic.turn_info
        ));
        assert!(!gambling_im_in_card().can_play(
            &player2_uuid,
            &game_logic.gambling_manager,
            &game_logic.interrupt_manager,
            &game_logic.turn_info
        ));

        // Player 2 passes and antes.
        game_logic.pass(&player2_uuid).unwrap();

        // Player 2 can now play a gambling card.
        assert!(!i_raise_card().can_play(
            &player1_uuid,
            &game_logic.gambling_manager,
            &game_logic.interrupt_manager,
            &game_logic.turn_info
        ));
        assert!(i_raise_card().can_play(
            &player2_uuid,
            &game_logic.gambling_manager,
            &game_logic.interrupt_manager,
            &game_logic.turn_info
        ));
        assert!(!gambling_im_in_card().can_play(
            &player1_uuid,
            &game_logic.gambling_manager,
            &game_logic.interrupt_manager,
            &game_logic.turn_info
        ));
        assert!(gambling_im_in_card().can_play(
            &player2_uuid,
            &game_logic.gambling_manager,
            &game_logic.interrupt_manager,
            &game_logic.turn_info
        ));
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
        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 1 attempts to hurt player 2.
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
        assert!(game_logic.interrupt_manager.interrupt_in_progress());

        // Player 2 chooses not to play an interrupt card.
        assert!(game_logic
            .interrupt_manager
            .is_turn_to_interrupt(&player2_uuid));
        game_logic.pass(&player2_uuid).unwrap();
        assert!(!game_logic.interrupt_manager.interrupt_in_progress());

        // Fortitude should be reduced.
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player2_uuid)
                .unwrap()
                .get_fortitude(),
            18
        );

        // Should proceed to player 1's order drink phase.
        assert_eq!(game_logic.get_turn_phase(), TurnPhase::OrderDrinks);
    }

    #[test]
    fn cannot_play_directed_card_on_self() {
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

        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 1 attempts to hurt self.
        assert_eq!(
            game_logic
                .process_card(
                    change_other_player_fortitude_card("Punch in the face", -2).into(),
                    &player1_uuid,
                    &Some(player1_uuid.clone())
                )
                .unwrap_err()
                .1,
            Error::new("Must not direct this card at yourself")
        );

        // Should stay at player 1's action phase.
        assert_eq!(game_logic.get_turn_phase(), TurnPhase::Action);
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
        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Reduce player 2's fortitude to ensure that it is properly restored.
        game_logic
            .player_manager
            .get_player_by_uuid_mut(&player2_uuid)
            .unwrap()
            .change_fortitude(-2);

        assert!(game_logic
            .process_card(
                change_other_player_fortitude_card("Punch in the face", -2).into(),
                &player1_uuid,
                &Some(player2_uuid.clone())
            )
            .is_ok());

        assert!(gain_fortitude_anytime_card("Heal", 1).can_play(
            &player1_uuid,
            &game_logic.gambling_manager,
            &game_logic.interrupt_manager,
            &game_logic.turn_info
        ));
        assert!(game_logic
            .process_card(
                gain_fortitude_anytime_card("Heal", 1).into(),
                &player1_uuid,
                &None
            )
            .is_ok());
    }

    #[test]
    fn can_gain_fortitude_during_game_interrupt() {
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

        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        assert!(game_logic
            .process_card(
                change_other_player_fortitude_card("Punch in the face", -2).into(),
                &player1_uuid,
                &Some(player2_uuid.clone())
            )
            .is_ok());

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
        game_logic.pass(&player1_uuid).unwrap();
        assert!(!game_logic.interrupt_manager.interrupt_in_progress());

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
    fn can_order_drinks_after_action_phase() {
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

        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 1 skips their action phase.
        assert!(game_logic.pass(&player1_uuid).is_ok());

        // Should proceed to player 1's order drink phase.
        assert_eq!(game_logic.get_turn_phase(), TurnPhase::OrderDrinks);

        assert!(game_logic.order_drink(&player1_uuid, &player2_uuid).is_ok());

        // Should proceed to player 2's discard phase.
        assert_eq!(game_logic.get_turn_phase(), TurnPhase::DiscardAndDraw);
    }

    #[test]
    fn can_order_multiple_drinks() {
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

        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 1 skips their action phase.
        assert!(game_logic.pass(&player1_uuid).is_ok());

        // Should proceed to player 1's order drink phase.
        assert_eq!(game_logic.get_turn_phase(), TurnPhase::OrderDrinks);

        assert!(game_logic
            .process_card(
                wench_bring_some_drinks_for_my_friends_card().into(),
                &player1_uuid,
                &None
            )
            .is_ok());

        assert!(game_logic.order_drink(&player1_uuid, &player2_uuid).is_ok());
        assert!(game_logic.order_drink(&player1_uuid, &player2_uuid).is_ok());
        assert!(game_logic.order_drink(&player1_uuid, &player2_uuid).is_ok());

        // Should proceed to player 2's discard phase.
        assert_eq!(game_logic.get_turn_phase(), TurnPhase::DiscardAndDraw);
    }

    #[test]
    fn player_drinks_top_drink_after_ordering_drinks() {
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

        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 1 skips their action phase.
        assert!(game_logic.pass(&player1_uuid).is_ok());

        // Should proceed to player 1's order drink phase.
        assert_eq!(game_logic.get_turn_phase(), TurnPhase::OrderDrinks);

        // Player should drink the top card of their drink deck after ordering drinks for other players.
        game_logic
            .player_manager
            .get_player_by_uuid_mut(&player1_uuid)
            .unwrap()
            .add_drink_to_drink_pile(create_simple_ale_test_drink(false).into());
        let player1_drink_me_pile_size = game_logic
            .player_manager
            .get_player_by_uuid(&player1_uuid)
            .unwrap()
            .to_game_view_player_data(player1_uuid.clone())
            .drink_me_pile_size;
        let player1_alcohol_content = game_logic
            .player_manager
            .get_player_by_uuid(&player1_uuid)
            .unwrap()
            .to_game_view_player_data(player1_uuid.clone())
            .alcohol_content;
        assert!(game_logic.order_drink(&player1_uuid, &player2_uuid).is_ok());
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player1_uuid)
                .unwrap()
                .to_game_view_player_data(player1_uuid.clone())
                .drink_me_pile_size,
            player1_drink_me_pile_size - 1
        );
        assert_eq!(
            game_logic
                .player_manager
                .get_player_by_uuid(&player1_uuid)
                .unwrap()
                .to_game_view_player_data(player1_uuid.clone())
                .alcohol_content,
            player1_alcohol_content + 1
        );

        // Should proceed to player 2's discard phase.
        assert_eq!(game_logic.get_turn_phase(), TurnPhase::DiscardAndDraw);
    }

    #[test]
    fn cannot_order_drinks_for_self() {
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

        assert!(!game_logic.gambling_manager.round_in_progress());
        assert_eq!(game_logic.turn_info.turn_phase, TurnPhase::Action);

        // Player 1 skips their action phase.
        assert!(game_logic.pass(&player1_uuid).is_ok());

        // Should proceed to player 1's order drink phase.
        assert_eq!(game_logic.get_turn_phase(), TurnPhase::OrderDrinks);

        assert_eq!(
            game_logic
                .order_drink(&player1_uuid, &player1_uuid)
                .unwrap_err(),
            Error::new("Cannot order drink for yourself")
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
            vec![player1_uuid, player2_uuid, player3_uuid, player4_uuid,]
        );
    }
}
