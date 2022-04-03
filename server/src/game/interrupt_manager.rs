use super::drink::{DrinkCard, DrinkWithPossibleChasers};
use super::gambling_manager::GamblingManager;
use super::game_logic::TurnInfo;
use super::player_card::{
    InterruptPlayerCard, PlayerCard, RootPlayerCard, ShouldCancelPreviousCard,
};
use super::player_manager::{NextPlayerUUIDOption, PlayerManager};
use super::player_view::{
    GameViewInterruptData, GameViewInterruptStack, GameViewInterruptStackRootItem,
};
use super::uuid::PlayerUUID;
use super::Error;
use std::default::Default;

#[derive(Clone, Debug)]
pub struct InterruptManager {
    interrupt_stacks: Vec<GameInterruptStack>,
}

impl InterruptManager {
    pub fn new() -> Self {
        Self {
            interrupt_stacks: Vec::new(),
        }
    }

    pub fn get_current_interrupt(&self) -> Option<GameInterruptType> {
        self.interrupt_stacks.first()?.get_current_interrupt()
    }

    fn get_current_interrupt_turn_or(&self) -> Option<&PlayerUUID> {
        Some(self.interrupt_stacks.first()?.get_current_interrupt_turn())
    }

    pub fn get_game_view_interrupt_data_or(&self) -> Option<GameViewInterruptData> {
        let current_interrupt_turn = match self.get_current_interrupt_turn_or() {
            Some(current_interrupt_turn) => current_interrupt_turn.clone(),
            None => return None,
        };

        let mut interrupts = Vec::new();
        for interrupt_stack in &self.interrupt_stacks {
            let interrupt_card_names = match interrupt_stack.sessions.last() {
                Some(first_session) => first_session
                    .interrupt_cards
                    .iter()
                    .map(|interrupt_card| interrupt_card.card.get_display_name().to_string())
                    .collect(),
                None => Vec::new(),
            };
            interrupts.push(GameViewInterruptStack {
                root_item: match &interrupt_stack.root {
                    InterruptRoot::RootPlayerCard(root_player_card_with_owner) => {
                        GameViewInterruptStackRootItem {
                            name: root_player_card_with_owner
                                .root_card
                                .get_display_name()
                                .to_string(),
                            item_type: String::from("rootPlayerCard"),
                        }
                    }
                    InterruptRoot::Drink(drink_with_owner) => GameViewInterruptStackRootItem {
                        name: drink_with_owner.drink.get_display_name(),
                        item_type: String::from("drinkEvent"),
                    },
                },
                interrupt_card_names,
            });
        }

        Some(GameViewInterruptData {
            interrupts,
            current_interrupt_turn,
        })
    }

    pub fn start_single_player_root_player_card_interrupt(
        &mut self,
        root_card: RootPlayerCard,
        root_card_owner_uuid: PlayerUUID,
        targeted_player_uuid: PlayerUUID,
    ) -> Result<(), (RootPlayerCard, Error)> {
        if self.interrupt_in_progress() {
            return Err((root_card, Error::new("An interrupt is already in progress")));
        }

        if let Some(interrupt_data) = root_card.get_interrupt_data_or() {
            let root_card_interrupt_type = interrupt_data.get_interrupt_type_output();
            self.interrupt_stacks.push(GameInterruptStack {
                root: InterruptRoot::RootPlayerCard(RootPlayerCardWithInterruptData {
                    root_card,
                    root_card_owner_uuid,
                }),
                current_interrupt_turn: targeted_player_uuid.clone(),
                sessions: vec![GameInterruptStackSession {
                    root_card_interrupt_type,
                    targeted_player_uuid,
                    interrupt_cards: Vec::new(),
                    only_targeted_player_can_interrupt: true,
                }],
            });
            Ok(())
        } else {
            Err((root_card, Error::new("Card is not interruptable")))
        }
    }

    pub fn start_single_player_drink_interrupt(
        &mut self,
        drink: DrinkWithPossibleChasers,
        targeted_player_uuid: PlayerUUID,
    ) -> Result<(), (DrinkWithPossibleChasers, Error)> {
        if self.interrupt_in_progress() {
            return Err((drink, Error::new("An interrupt is already in progress")));
        }

        self.interrupt_stacks.push(GameInterruptStack {
            root: InterruptRoot::Drink(DrinkWithInterruptData { drink }),
            current_interrupt_turn: targeted_player_uuid.clone(),
            sessions: vec![
                GameInterruptStackSession {
                    root_card_interrupt_type: GameInterruptType::AboutToDrink,
                    targeted_player_uuid: targeted_player_uuid.clone(),
                    interrupt_cards: Vec::new(),
                    only_targeted_player_can_interrupt: true,
                },
                GameInterruptStackSession {
                    root_card_interrupt_type: GameInterruptType::ModifyDrink,
                    targeted_player_uuid,
                    interrupt_cards: Vec::new(),
                    only_targeted_player_can_interrupt: false,
                },
            ],
        });
        Ok(())
    }

    /// Create multiple consecutive interrupt stacks each targeting a different player.
    /// This is used for cards where multiple players are affected individually, such as
    /// an `I Raise` card, which forces each individual user to ante.
    pub fn start_multi_player_root_player_card_interrupt(
        &mut self,
        root_card: RootPlayerCard,
        root_card_owner_uuid: PlayerUUID,
        targeted_player_uuids: Vec<PlayerUUID>,
    ) -> Result<(), (RootPlayerCard, Error)> {
        if self.interrupt_in_progress() {
            return Err((root_card, Error::new("An interrupt is already in progress")));
        }

        if targeted_player_uuids.is_empty() {
            return Err((
                root_card,
                Error::new("Cannot start an interrupt with no targeted players"),
            ));
        }

        if let Some(interrupt_data) = root_card.get_interrupt_data_or() {
            let root_card_interrupt_type = interrupt_data.get_interrupt_type_output();
            let mut sessions = Vec::new();

            let current_interrupt_turn = targeted_player_uuids.first().unwrap().clone(); // TODO - Handle this unwrap.

            for targeted_player_uuid in targeted_player_uuids.into_iter().rev() {
                sessions.push(GameInterruptStackSession {
                    root_card_interrupt_type,
                    targeted_player_uuid,
                    interrupt_cards: Vec::new(),
                    only_targeted_player_can_interrupt: true,
                });
            }

            self.interrupt_stacks.push(GameInterruptStack {
                root: InterruptRoot::RootPlayerCard(RootPlayerCardWithInterruptData {
                    root_card,
                    root_card_owner_uuid,
                }),
                current_interrupt_turn,
                sessions,
            });
            Ok(())
        } else {
            Err((root_card, Error::new("Card is not interruptable")))
        }
    }

    pub fn play_interrupt_card(
        &mut self,
        card: InterruptPlayerCard,
        player_uuid: PlayerUUID,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
        turn_info: &mut TurnInfo,
    ) -> Result<Option<InterruptStackResolveData>, (InterruptPlayerCard, Error)> {
        if !self.is_turn_to_interrupt(&player_uuid) {
            return Err((
                card,
                Error::new("It is not your turn to play an interrupt card"),
            ));
        }
        match self.push_to_current_stack(card, player_uuid) {
            Ok(_) => Ok(self
                .increment_player_turn(player_manager, gambling_manager, turn_info, false)
                .unwrap()),
            Err(err) => Err(err),
        }
    }

    pub fn interrupt_in_progress(&self) -> bool {
        !self.interrupt_stacks.is_empty()
    }

    pub fn is_turn_to_interrupt(&self, player_uuid: &PlayerUUID) -> bool {
        Some(player_uuid) == self.get_current_interrupt_turn_or()
    }

    pub fn pass(
        &mut self,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
        turn_info: &mut TurnInfo,
    ) -> Result<Option<InterruptStackResolveData>, Error> {
        self.increment_player_turn(player_manager, gambling_manager, turn_info, true)
    }

    fn increment_player_turn(
        &mut self,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
        turn_info: &mut TurnInfo,
        is_passing: bool,
    ) -> Result<Option<InterruptStackResolveData>, Error> {
        let current_stack_session_is_only_interruptable_by_targeted_player =
            if let Some(current_stack) = self.interrupt_stacks.first() {
                if let Some(current_session) = current_stack.get_current_session() {
                    current_session.only_targeted_player_can_interrupt
                } else {
                    false
                }
            } else {
                false
            };

        if let Some(current_interrupt_turn) = self.get_current_interrupt_turn_or() {
            if current_stack_session_is_only_interruptable_by_targeted_player {
                let current_stack = match self.interrupt_stacks.first() {
                    Some(current_stack) => current_stack,
                    None => return Err(Error::new("No interrupts are running")),
                };
                let current_session = match current_stack.get_current_session() {
                    Some(current_session) => current_session,
                    None => return Err(Error::new("No interrupts are running")),
                };
                if is_passing && current_interrupt_turn == &current_session.targeted_player_uuid {
                    return match self.resolve_current_stack_session(
                        player_manager,
                        gambling_manager,
                        turn_info,
                    ) {
                        Ok(interrupt_stack_resolve_data) => Ok(Some(interrupt_stack_resolve_data)),
                        Err(err) => Err(err),
                    };
                }
            }

            match player_manager.get_next_alive_player_uuid(current_interrupt_turn) {
                NextPlayerUUIDOption::Some(next_player_uuid) => {
                    // If, after incrementing the player turn, the interrupt turn has
                    // looped back around to the last player who played a card, then
                    // that ends the interrupt stack since that player was uninterrupted.
                    if Some(next_player_uuid) == self.get_last_player_to_play_on_current_stack() {
                        match self.resolve_current_stack_session(player_manager, gambling_manager, turn_info) {
                            Ok(interrupt_stack_resolve_data) => Ok(Some(interrupt_stack_resolve_data)),
                            Err(err) => Err(err)
                        }
                    } else {
                        if let Some(current_stack) = self.interrupt_stacks.first_mut() {
                            current_stack.current_interrupt_turn = next_player_uuid.clone();
                        }
                        Ok(None)
                    }
                }
                NextPlayerUUIDOption::PlayerNotFound => {
                    Err(Error::new("Uh oh! Failed to increment player turn. This is an internal error, due to some sort of bug."))
                },
                NextPlayerUUIDOption::OnlyPlayerLeft => {
                    match self.resolve_current_stack_session(player_manager, gambling_manager, turn_info) {
                        Ok(interrupt_stack_resolve_data) => Ok(Some(interrupt_stack_resolve_data)),
                        Err(err) => Err(err)
                    }
                }

            }
        } else {
            Err(Error::new("It is not anyone's turn to interrupt"))
        }
    }

    fn resolve_current_stack_session(
        &mut self,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
        turn_info: &mut TurnInfo,
    ) -> Result<InterruptStackResolveData, Error> {
        if self.interrupt_stacks.is_empty() {
            return Err(Error::new("No stacks to resolve"));
        }
        // The check above will prevent `remove` from panicking.
        let mut current_stack = self.interrupt_stacks.remove(0);

        let mut spent_interrupt_cards = Vec::new();

        let mut should_cancel_root_card = ShouldCancelPreviousCard::No;

        let mut session = current_stack.sessions.pop().unwrap(); // TODO - Handle this unwrap.

        while let Some(game_interrupt_data) = session.interrupt_cards.pop() {
            match game_interrupt_data.card.interrupt(
                &game_interrupt_data.card_owner_uuid,
                self,
                gambling_manager,
            ) {
                ShouldCancelPreviousCard::Negate => {
                    if let Some(game_interrupt_data) = session.interrupt_cards.pop() {
                        spent_interrupt_cards.push((
                            game_interrupt_data.card_owner_uuid,
                            game_interrupt_data.card,
                        ));
                    } else {
                        should_cancel_root_card = ShouldCancelPreviousCard::Negate;
                    }
                }
                ShouldCancelPreviousCard::Ignore => {
                    if let Some(game_interrupt_data) = session.interrupt_cards.pop() {
                        spent_interrupt_cards.push((
                            game_interrupt_data.card_owner_uuid,
                            game_interrupt_data.card,
                        ));
                    } else {
                        should_cancel_root_card = ShouldCancelPreviousCard::Ignore;
                    }
                }
                ShouldCancelPreviousCard::No => {}
            };
            spent_interrupt_cards.push((
                game_interrupt_data.card_owner_uuid,
                game_interrupt_data.card,
            ));
        }

        match should_cancel_root_card {
            ShouldCancelPreviousCard::Negate => {
                let mut interrupt_stack_resolve_data = current_stack.drain_all_cards();
                interrupt_stack_resolve_data
                    .interrupt_cards
                    .append(&mut spent_interrupt_cards);
                Ok(interrupt_stack_resolve_data)
            }
            ShouldCancelPreviousCard::Ignore => {
                if let Some(next_session) = current_stack.sessions.last() {
                    current_stack.current_interrupt_turn =
                        next_session.targeted_player_uuid.clone();
                    self.interrupt_stacks.insert(0, current_stack);
                    Ok(InterruptStackResolveData {
                        root_card_with_owner_or: None,
                        interrupt_cards: spent_interrupt_cards,
                        drink_or: None,
                    })
                } else {
                    Ok(match current_stack.root {
                        InterruptRoot::RootPlayerCard(root_player_card_with_interrupt_data) => {
                            InterruptStackResolveData {
                                root_card_with_owner_or: Some((
                                    root_player_card_with_interrupt_data.root_card,
                                    root_player_card_with_interrupt_data.root_card_owner_uuid,
                                )),
                                interrupt_cards: spent_interrupt_cards,
                                drink_or: None,
                            }
                        }
                        InterruptRoot::Drink(drink_with_interrupt_data) => {
                            InterruptStackResolveData {
                                root_card_with_owner_or: None,
                                interrupt_cards: spent_interrupt_cards,
                                drink_or: Some(drink_with_interrupt_data.drink),
                            }
                        }
                    })
                }
            }
            ShouldCancelPreviousCard::No => {
                match &current_stack.root {
                    InterruptRoot::RootPlayerCard(root_player_card_with_interrupt_data) => {
                        root_player_card_with_interrupt_data
                            .root_card
                            .interrupt_play(
                                &root_player_card_with_interrupt_data.root_card_owner_uuid,
                                &session.targeted_player_uuid,
                                player_manager,
                                gambling_manager,
                            );

                        if let Some(interrupt_data) = root_player_card_with_interrupt_data
                            .root_card
                            .get_interrupt_data_or()
                        {
                            interrupt_data.post_interrupt_play(
                                &root_player_card_with_interrupt_data.root_card_owner_uuid,
                                player_manager,
                                gambling_manager,
                                turn_info,
                            );
                        }
                    }
                    InterruptRoot::Drink(drink_with_interrupt_data) => {
                        if let Some(targeted_player) =
                            player_manager.get_player_by_uuid_mut(&session.targeted_player_uuid)
                        {
                            if session.root_card_interrupt_type == GameInterruptType::AboutToDrink {
                                drink_with_interrupt_data.drink.process(targeted_player);
                            }
                        };
                    }
                };

                if let Some(next_session) = current_stack.sessions.last() {
                    current_stack.current_interrupt_turn =
                        next_session.targeted_player_uuid.clone();
                    self.interrupt_stacks.insert(0, current_stack);
                    Ok(InterruptStackResolveData {
                        root_card_with_owner_or: None,
                        interrupt_cards: spent_interrupt_cards,
                        drink_or: None,
                    })
                } else {
                    Ok(match current_stack.root {
                        InterruptRoot::RootPlayerCard(root_player_card_with_interrupt_data) => {
                            InterruptStackResolveData {
                                root_card_with_owner_or: Some((
                                    root_player_card_with_interrupt_data.root_card,
                                    root_player_card_with_interrupt_data.root_card_owner_uuid,
                                )),
                                interrupt_cards: spent_interrupt_cards,
                                drink_or: None,
                            }
                        }
                        InterruptRoot::Drink(drink_with_interrupt_data) => {
                            InterruptStackResolveData {
                                root_card_with_owner_or: None,
                                interrupt_cards: spent_interrupt_cards,
                                drink_or: Some(drink_with_interrupt_data.drink),
                            }
                        }
                    })
                }
            }
        }
    }

    fn push_to_current_stack(
        &mut self,
        card: InterruptPlayerCard,
        card_owner_uuid: PlayerUUID,
    ) -> Result<(), (InterruptPlayerCard, Error)> {
        if let Err(err) = self.can_push_to_current_stack(&card) {
            return Err((card, err));
        };

        let current_stack = match self.interrupt_stacks.first_mut() {
            Some(current_stack) => current_stack,
            None => return Err((card, Error::new("No card to interrupt"))),
        };

        if let Err((game_interrupt_data, err)) = current_stack
            .push_game_interrupt_data_to_current_stack(GameInterruptData {
                card_interrupt_type: card.get_interrupt_type_output(),
                card,
                card_owner_uuid,
            })
        {
            return Err((game_interrupt_data.card, err));
        }

        Ok(())
    }

    fn can_push_to_current_stack(&self, card: &InterruptPlayerCard) -> Result<(), Error> {
        match self.get_current_interrupt() {
            Some(current_interrupt) => {
                if !card.can_interrupt(current_interrupt) {
                    return Err(Error::new(
                        "This card cannot interrupt the last played card",
                    ));
                }
            }
            None => return Err(Error::new("No card to interrupt")),
        };

        Ok(())
    }

    fn get_last_player_to_play_on_current_stack(&self) -> Option<&PlayerUUID> {
        let current_stack = self.interrupt_stacks.first()?;

        Some(
            match current_stack.sessions.last()?.get_last_player_to_play() {
                Some(player_uuid) => player_uuid,
                None => match &current_stack.root {
                    InterruptRoot::RootPlayerCard(root_player_card_with_interrupt_data) => {
                        &root_player_card_with_interrupt_data.root_card_owner_uuid
                    }
                    InterruptRoot::Drink(_) => {
                        &current_stack.get_current_session()?.targeted_player_uuid
                    }
                },
            },
        )
    }
}

impl Default for InterruptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameInterruptType {
    AboutToAnte,
    DirectedActionCardPlayed(PlayerCardInfo),
    SometimesCardPlayed(PlayerCardInfo),
    ModifyDrink,
    AboutToDrink,
}

#[derive(Clone, Debug)]
struct RootPlayerCardWithInterruptData {
    root_card: RootPlayerCard,
    root_card_owner_uuid: PlayerUUID,
}

#[derive(Clone, Debug)]
struct DrinkWithInterruptData {
    drink: DrinkWithPossibleChasers,
}

#[derive(Clone, Debug)]
enum InterruptRoot {
    RootPlayerCard(RootPlayerCardWithInterruptData),
    Drink(DrinkWithInterruptData),
}

#[derive(Clone, Debug)]
struct GameInterruptStack {
    root: InterruptRoot,
    current_interrupt_turn: PlayerUUID,
    sessions: Vec<GameInterruptStackSession>,
}

impl GameInterruptStack {
    fn get_current_session(&self) -> Option<&GameInterruptStackSession> {
        self.sessions.last()
    }
    fn get_current_session_mut(&mut self) -> Option<&mut GameInterruptStackSession> {
        self.sessions.last_mut()
    }

    fn get_current_interrupt(&self) -> Option<GameInterruptType> {
        let current_session = self.get_current_session()?;

        Some(match current_session.interrupt_cards.last() {
            Some(current_interrupt_data) => current_interrupt_data.card_interrupt_type,
            None => current_session.root_card_interrupt_type,
        })
    }

    fn get_current_interrupt_turn(&self) -> &PlayerUUID {
        &self.current_interrupt_turn
    }

    fn push_game_interrupt_data_to_current_stack(
        &mut self,
        game_interrupt_data: GameInterruptData,
    ) -> Result<(), (GameInterruptData, Error)> {
        let current_session = match self.get_current_session_mut() {
            Some(current_session) => current_session,
            None => return Err((
                game_interrupt_data,
                Error::new(
                    "Game interrupt stack has no session to push to - this is an internal error",
                ),
            )),
        };

        current_session.interrupt_cards.push(game_interrupt_data);

        Ok(())
    }

    fn drain_all_cards(mut self) -> InterruptStackResolveData {
        let mut interrupt_cards = Vec::new();

        for session in &mut self.sessions {
            while let Some(game_interrupt_data) = session.interrupt_cards.pop() {
                interrupt_cards.push((
                    game_interrupt_data.card_owner_uuid,
                    game_interrupt_data.card,
                ));
            }
        }

        match self.root {
            InterruptRoot::RootPlayerCard(root_player_card_with_interrupt_data) => {
                InterruptStackResolveData {
                    root_card_with_owner_or: Some((
                        root_player_card_with_interrupt_data.root_card,
                        root_player_card_with_interrupt_data.root_card_owner_uuid,
                    )),
                    interrupt_cards,
                    drink_or: None,
                }
            }
            InterruptRoot::Drink(drink_with_interrupt_data) => InterruptStackResolveData {
                root_card_with_owner_or: None,
                interrupt_cards,
                drink_or: Some(drink_with_interrupt_data.drink),
            },
        }
    }
}

#[derive(Clone, Debug)]
struct GameInterruptStackSession {
    root_card_interrupt_type: GameInterruptType,
    targeted_player_uuid: PlayerUUID, // The player that the root card is targeting.
    interrupt_cards: Vec<GameInterruptData>,
    only_targeted_player_can_interrupt: bool,
}

impl GameInterruptStackSession {
    fn get_last_player_to_play(&self) -> Option<&PlayerUUID> {
        Some(&self.interrupt_cards.last()?.card_owner_uuid)
    }
}

#[derive(Clone, Debug)]
struct GameInterruptData {
    card: InterruptPlayerCard,
    card_interrupt_type: GameInterruptType,
    card_owner_uuid: PlayerUUID,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerCardInfo {
    pub affects_fortitude: bool,
    pub is_i_dont_think_so_card: bool,
}

pub struct InterruptStackResolveData {
    root_card_with_owner_or: Option<(RootPlayerCard, PlayerUUID)>,
    interrupt_cards: Vec<(PlayerUUID, InterruptPlayerCard)>,
    drink_or: Option<DrinkWithPossibleChasers>,
}

impl InterruptStackResolveData {
    pub fn current_user_action_phase_is_over(&self) -> bool {
        if let Some((root_card, _)) = &self.root_card_with_owner_or {
            root_card.is_action_card() && !root_card.is_gambling_card()
        } else {
            false
        }
    }

    pub fn take_all_player_cards(self) -> (Vec<(PlayerUUID, PlayerCard)>, Vec<DrinkCard>) {
        let mut cards = Vec::new();
        if let Some((root_card, root_card_owner_uuid)) = self.root_card_with_owner_or {
            cards.push((root_card_owner_uuid, root_card.into()));
        }
        for (card_owner_uuid, card) in self.interrupt_cards {
            cards.push((card_owner_uuid, card.into()));
        }
        (
            cards,
            if let Some(drink) = self.drink_or {
                drink.take_all_discardable_drink_cards()
            } else {
                Vec::new()
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::player_card::change_other_player_fortitude_card;
    use super::super::Character;
    use super::*;

    #[test]
    fn player_root_player_card_interrupt_ends_after_targeted_player_passes_2_player_game() {
        let player1_uuid = PlayerUUID::new();
        let player2_uuid = PlayerUUID::new();
        let mut interrupt_manager = InterruptManager::new();
        let mut player_manager = PlayerManager::new(vec![
            (player1_uuid.clone(), Character::Gerki),
            (player2_uuid.clone(), Character::Deirdre),
        ]);
        let mut gambling_manager = GamblingManager::new();
        let mut turn_info = TurnInfo::new_test(player1_uuid.clone());

        assert!(interrupt_manager
            .start_single_player_root_player_card_interrupt(
                change_other_player_fortitude_card("Test card", -1),
                player1_uuid,
                player2_uuid.clone()
            )
            .is_ok());
        assert!(interrupt_manager.is_turn_to_interrupt(&player2_uuid));
        assert!(interrupt_manager
            .pass(&mut player_manager, &mut gambling_manager, &mut turn_info)
            .is_ok());
        assert!(!interrupt_manager.interrupt_in_progress());
    }

    #[test]
    fn player_root_player_card_interrupt_ends_after_targeted_player_passes_3_player_game() {
        let player1_uuid = PlayerUUID::new();
        let player2_uuid = PlayerUUID::new();
        let player3_uuid = PlayerUUID::new();
        let mut interrupt_manager = InterruptManager::new();
        let mut player_manager = PlayerManager::new(vec![
            (player1_uuid.clone(), Character::Gerki),
            (player2_uuid, Character::Deirdre),
            (player3_uuid.clone(), Character::Zot),
        ]);
        let mut gambling_manager = GamblingManager::new();
        let mut turn_info = TurnInfo::new_test(player1_uuid.clone());

        assert!(interrupt_manager
            .start_single_player_root_player_card_interrupt(
                change_other_player_fortitude_card("Test card", -1),
                player1_uuid,
                player3_uuid.clone()
            )
            .is_ok());
        assert!(interrupt_manager.is_turn_to_interrupt(&player3_uuid));
        assert!(interrupt_manager
            .pass(&mut player_manager, &mut gambling_manager, &mut turn_info)
            .is_ok());
        assert!(!interrupt_manager.interrupt_in_progress());
    }

    #[test]
    fn drink_interrupt_ends_after_everyone_passes_2_player_game() {
        let player1_uuid = PlayerUUID::new();
        let player2_uuid = PlayerUUID::new();
        let mut interrupt_manager = InterruptManager::new();
        let mut player_manager = PlayerManager::new(vec![
            (player1_uuid.clone(), Character::Gerki),
            (player2_uuid.clone(), Character::Deirdre),
        ]);
        let mut gambling_manager = GamblingManager::new();
        let mut turn_info = TurnInfo::new_test(player1_uuid.clone());

        assert!(interrupt_manager
            .start_single_player_drink_interrupt(
                DrinkWithPossibleChasers::new(vec![], None),
                player1_uuid.clone()
            )
            .is_ok());
        // All players pass on the chance to modify the drink.
        assert!(interrupt_manager.is_turn_to_interrupt(&player1_uuid));
        assert!(interrupt_manager
            .pass(&mut player_manager, &mut gambling_manager, &mut turn_info)
            .is_ok());
        assert!(interrupt_manager.is_turn_to_interrupt(&player2_uuid));
        assert!(interrupt_manager
            .pass(&mut player_manager, &mut gambling_manager, &mut turn_info)
            .is_ok());
        // Targeted player passes on the chance to interrupt the drink.
        assert!(interrupt_manager.is_turn_to_interrupt(&player1_uuid));
        assert!(interrupt_manager
            .pass(&mut player_manager, &mut gambling_manager, &mut turn_info)
            .is_ok());

        assert!(!interrupt_manager.interrupt_in_progress());
    }

    #[test]
    fn drink_interrupt_ends_after_everyone_passes_3_player_game() {
        let player1_uuid = PlayerUUID::new();
        let player2_uuid = PlayerUUID::new();
        let player3_uuid = PlayerUUID::new();
        let mut interrupt_manager = InterruptManager::new();
        let mut player_manager = PlayerManager::new(vec![
            (player1_uuid.clone(), Character::Gerki),
            (player2_uuid.clone(), Character::Deirdre),
            (player3_uuid.clone(), Character::Zot),
        ]);
        let mut gambling_manager = GamblingManager::new();
        let mut turn_info = TurnInfo::new_test(player1_uuid.clone());

        assert!(interrupt_manager
            .start_single_player_drink_interrupt(
                DrinkWithPossibleChasers::new(vec![], None),
                player1_uuid.clone()
            )
            .is_ok());
        // All players pass on the chance to modify the drink.
        assert!(interrupt_manager.is_turn_to_interrupt(&player1_uuid));
        assert!(interrupt_manager
            .pass(&mut player_manager, &mut gambling_manager, &mut turn_info)
            .is_ok());
        assert!(interrupt_manager.is_turn_to_interrupt(&player2_uuid));
        assert!(interrupt_manager
            .pass(&mut player_manager, &mut gambling_manager, &mut turn_info)
            .is_ok());
        assert!(interrupt_manager.is_turn_to_interrupt(&player3_uuid));
        assert!(interrupt_manager
            .pass(&mut player_manager, &mut gambling_manager, &mut turn_info)
            .is_ok());
        // Targeted player passes on the chance to interrupt the drink.
        assert!(interrupt_manager.is_turn_to_interrupt(&player1_uuid));
        assert!(interrupt_manager
            .pass(&mut player_manager, &mut gambling_manager, &mut turn_info)
            .is_ok());

        assert!(!interrupt_manager.interrupt_in_progress());
    }
}
