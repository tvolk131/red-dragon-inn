use super::gambling_manager::GamblingManager;
use super::game_logic::TurnInfo;
use super::player_card::{
    InterruptPlayerCard, PlayerCard, RootPlayerCard, ShouldCancelPreviousCard,
};
use super::player_manager::{NextPlayerUUIDOption, PlayerManager};
use super::uuid::PlayerUUID;
use super::Error;
use std::default::Default;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct InterruptManager {
    interrupt_stacks: Vec<GameInterruptStack>,
    current_interrupt_turn_or: Option<PlayerUUID>,
}

impl InterruptManager {
    pub fn new() -> Self {
        Self {
            interrupt_stacks: Vec::new(),
            current_interrupt_turn_or: None,
        }
    }

    pub fn get_current_interrupt(&self) -> Option<GameInterruptType> {
        let current_stack = self.interrupt_stacks.first()?;

        Some(match current_stack.interrupt_cards.last() {
            Some(most_recent_interrupt_data) => most_recent_interrupt_data.card_interrupt_type,
            None => current_stack.root_card_interrupt_type,
        })
    }

    pub fn start_single_player_interrupt(
        &mut self,
        card: RootPlayerCard,
        card_owner_uuid: PlayerUUID,
        targeted_player_uuid: PlayerUUID,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
        turn_info: &mut TurnInfo,
    ) -> Result<(), (RootPlayerCard, Error)> {
        if self.interrupt_in_progress() {
            return Err((card, Error::new("An interrupt is already in progress")));
        }

        if let Some(interrupt_data) = card.get_interrupt_data_or() {
            self.current_interrupt_turn_or = Some(card_owner_uuid.clone());
            if let Err(err) =
                self.increment_player_turn(player_manager, gambling_manager, turn_info)
            {
                self.current_interrupt_turn_or = None;
                return Err((card, err));
            }
            self.push_new_stack(
                interrupt_data.get_interrupt_style(),
                card,
                card_owner_uuid,
                targeted_player_uuid,
            );
            Ok(())
        } else {
            Err((card, Error::new("Card is not interruptable")))
        }
    }

    pub fn start_multi_player_interrupt(
        &mut self,
        card: RootPlayerCard,
        card_owner_uuid: &PlayerUUID,
        targeted_player_uuids: Vec<PlayerUUID>,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
        turn_info: &mut TurnInfo,
    ) -> Result<(), (RootPlayerCard, Error)> {
        if self.interrupt_in_progress() {
            return Err((card, Error::new("An interrupt is already in progress")));
        }

        if let Some(interrupt_data) = card.get_interrupt_data_or() {
            self.current_interrupt_turn_or = Some(card_owner_uuid.clone());
            if let Err(err) =
                self.increment_player_turn(player_manager, gambling_manager, turn_info)
            {
                self.current_interrupt_turn_or = None;
                return Err((card, err));
            }
            self.push_new_stacks(
                interrupt_data.get_interrupt_style(),
                card,
                card_owner_uuid,
                targeted_player_uuids,
            );
            Ok(())
        } else {
            Err((card, Error::new("Card is not interruptable")))
        }
    }

    pub fn play_interrupt_card(
        &mut self,
        card: InterruptPlayerCard,
        player_uuid: PlayerUUID,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
        turn_info: &mut TurnInfo,
    ) -> Result<(), (InterruptPlayerCard, Error)> {
        if !self.is_turn_to_interrupt(&player_uuid) {
            return Err((
                card,
                Error::new("It is not your turn to play an interrupt card"),
            ));
        }
        match self.push_to_current_stack(card, player_uuid) {
            Ok(_) => {
                self.increment_player_turn(player_manager, gambling_manager, turn_info)
                    .unwrap();
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn interrupt_in_progress(&self) -> bool {
        !self.interrupt_stacks.is_empty()
    }

    pub fn is_turn_to_interrupt(&self, player_uuid: &PlayerUUID) -> bool {
        Some(player_uuid) == self.current_interrupt_turn_or.as_ref()
    }

    pub fn pass(
        &mut self,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
        turn_info: &mut TurnInfo,
    ) -> Result<(), Error> {
        self.increment_player_turn(player_manager, gambling_manager, turn_info)
    }

    fn increment_player_turn(
        &mut self,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
        turn_info: &mut TurnInfo,
    ) -> Result<(), Error> {
        if let Some(current_interrupt_turn) = &self.current_interrupt_turn_or {
            match player_manager.get_next_alive_player_uuid(current_interrupt_turn) {
                NextPlayerUUIDOption::Some(next_player_uuid) => {
                    // If, after incrementing the player turn, the interrupt turn has
                    // looped back around to the last player who played a card, then
                    // that ends the interrupt stack since that player was uninterrupted.
                    if Some(next_player_uuid) == self.get_last_player_to_play_on_current_stack() {
                        self.resolve_current_stack(player_manager, gambling_manager, turn_info)?;
                        match self.interrupt_stacks.first() {
                            Some(first_interrupt_stack) => {
                                self.current_interrupt_turn_or =
                                    Some(first_interrupt_stack.root_card_owner_uuid.clone());
                                self.increment_player_turn(
                                    player_manager,
                                    gambling_manager,
                                    turn_info,
                                )?;
                            }
                            None => {
                                self.current_interrupt_turn_or = None;
                            }
                        }
                    } else {
                        self.current_interrupt_turn_or = Some(next_player_uuid.clone());
                    }
                }
                _ => {} // TODO - Return an error here.
            };
            Ok(())
        } else {
            Err(Error::new("It is not anyone's turn to interrupt"))
        }
    }

    fn resolve_current_stack(
        &mut self,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
        turn_info: &mut TurnInfo,
    ) -> Result<Vec<(PlayerUUID, PlayerCard)>, Error> {
        if self.interrupt_stacks.is_empty() {
            return Err(Error::new("No stacks to resolve"));
        }
        // The check above will prevent `remove` from panicking.
        let mut current_stack = self.interrupt_stacks.remove(0);

        let mut spent_cards = Vec::new();

        let mut should_cancel_root_card = ShouldCancelPreviousCard::No;

        while let Some(game_interrupt_data) = current_stack.interrupt_cards.pop() {
            match game_interrupt_data
                .card
                .interrupt(&game_interrupt_data.card_owner_uuid, self)
            {
                ShouldCancelPreviousCard::Negate => {
                    if let Some(game_interrupt_data) = current_stack.interrupt_cards.pop() {
                        spent_cards.push((
                            game_interrupt_data.card_owner_uuid,
                            game_interrupt_data.card.into(),
                        ));
                    } else {
                        should_cancel_root_card = ShouldCancelPreviousCard::Negate;
                    }
                }
                ShouldCancelPreviousCard::Ignore => {
                    if let Some(game_interrupt_data) = current_stack.interrupt_cards.pop() {
                        spent_cards.push((
                            game_interrupt_data.card_owner_uuid,
                            game_interrupt_data.card.into(),
                        ));
                    } else {
                        should_cancel_root_card = ShouldCancelPreviousCard::Ignore;
                    }
                }
                ShouldCancelPreviousCard::No => {}
            };
            spent_cards.push((
                game_interrupt_data.card_owner_uuid,
                game_interrupt_data.card.into(),
            ));
        }

        match should_cancel_root_card {
            ShouldCancelPreviousCard::Negate => {
                // TODO - use `drain_filter` instead of while loop. At time of writing, it is only availabe in nightly release.
                let mut i = 0;
                while i < self.interrupt_stacks.len() {
                    if Arc::ptr_eq(
                        &self.interrupt_stacks.get(i).unwrap().root_card,
                        &current_stack.root_card,
                    ) {
                        let stack = self.interrupt_stacks.remove(i);
                        for (player_uuid, card) in stack.drain_all_cards() {
                            spent_cards.push((player_uuid, card));
                        }
                    } else {
                        i += 1;
                    }
                }

                if let Ok(root_card) = Arc::try_unwrap(current_stack.root_card) {
                    spent_cards.push((current_stack.root_card_owner_uuid, root_card.into()));
                };
            }
            ShouldCancelPreviousCard::Ignore => {
                if let Ok(root_card) = Arc::try_unwrap(current_stack.root_card) {
                    spent_cards.push((current_stack.root_card_owner_uuid, root_card.into()));
                };
            }
            ShouldCancelPreviousCard::No => {
                current_stack.root_card.interrupt_play(
                    &current_stack.root_card_owner_uuid,
                    &current_stack.targeted_player_uuid,
                    player_manager,
                    gambling_manager,
                );

                if let Ok(root_card) = Arc::try_unwrap(current_stack.root_card) {
                    // TODO - Handle this unwrap.
                    root_card
                        .get_interrupt_data_or()
                        .unwrap()
                        .post_interrupt_play(
                            &current_stack.root_card_owner_uuid,
                            player_manager,
                            gambling_manager,
                            turn_info,
                        );
                    spent_cards.push((current_stack.root_card_owner_uuid, root_card.into()));
                };
            }
        };

        Ok(spent_cards)
    }

    fn push_new_stack(
        &mut self,
        game_interrupt_type: GameInterruptType,
        card: RootPlayerCard,
        card_owner_uuid: PlayerUUID,
        targeted_player_uuid: PlayerUUID,
    ) {
        self.interrupt_stacks.push(GameInterruptStack {
            root_card: Arc::from(card),
            root_card_interrupt_type: game_interrupt_type,
            root_card_owner_uuid: card_owner_uuid,
            targeted_player_uuid,
            interrupt_cards: Vec::new(),
        });
    }

    /// Create multiple consecutive interrupt stacks each targeting a different player.
    /// This is used for cards where multiple players are affected individually, such as
    /// an `I Raise` card, which forces each individual user to ante.
    fn push_new_stacks(
        &mut self,
        game_interrupt_type: GameInterruptType,
        card: RootPlayerCard,
        card_owner_uuid: &PlayerUUID,
        targeted_player_uuids: Vec<PlayerUUID>,
    ) {
        let card_arc = Arc::from(card);

        for targeted_player_uuid in targeted_player_uuids {
            self.interrupt_stacks.push(GameInterruptStack {
                root_card: card_arc.clone(),
                root_card_interrupt_type: game_interrupt_type,
                root_card_owner_uuid: card_owner_uuid.clone(),
                targeted_player_uuid,
                interrupt_cards: Vec::new(),
            });
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

        current_stack.interrupt_cards.push(GameInterruptData {
            card_interrupt_type: card.get_interrupt_type_output(),
            card,
            card_owner_uuid,
        });

        Ok(())
    }

    fn can_push_to_current_stack(&self, card: &InterruptPlayerCard) -> Result<(), Error> {
        match self.get_current_interrupt() {
            Some(current_interrupt) => {
                if !card
                    .get_interrupt_type_input()
                    .variant_eq(current_interrupt)
                {
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

        Some(match current_stack.interrupt_cards.last() {
            Some(most_recent_interrupt_data) => &most_recent_interrupt_data.card_owner_uuid,
            None => &current_stack.root_card_owner_uuid,
        })
    }
}

impl Default for InterruptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GameInterruptType {
    AboutToAnte,
    AboutToSpendGold,
    DirectedActionCardPlayed(PlayerCardInfo),
    SometimesCardPlayed(PlayerCardInfo),
    AboutToDrink,
}

impl GameInterruptType {
    pub fn variant_eq(&self, other: Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(&other)
    }
}

#[derive(Clone, Debug)]
struct GameInterruptStack {
    root_card: Arc<RootPlayerCard>,
    root_card_interrupt_type: GameInterruptType,
    root_card_owner_uuid: PlayerUUID,
    targeted_player_uuid: PlayerUUID, // The player that the root card is targeting.
    interrupt_cards: Vec<GameInterruptData>,
}

impl GameInterruptStack {
    fn drain_all_cards(mut self) -> Vec<(PlayerUUID, PlayerCard)> {
        let mut cards = Vec::new();

        while let Some(game_interrupt_data) = self.interrupt_cards.pop() {
            cards.push((
                game_interrupt_data.card_owner_uuid,
                game_interrupt_data.card.into(),
            ));
        }

        if let Ok(root_card) = Arc::try_unwrap(self.root_card) {
            cards.push((self.root_card_owner_uuid, root_card.into()));
        };

        cards
    }
}

#[derive(Clone, Debug)]
struct GameInterruptData {
    card: InterruptPlayerCard,
    card_interrupt_type: GameInterruptType,
    card_owner_uuid: PlayerUUID,
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerCardInfo {
    pub affects_fortitude: bool,
}
