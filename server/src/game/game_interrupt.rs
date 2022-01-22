use super::error::Error;
use super::player_card::{RootPlayerCard, InterruptPlayerCard, PlayerCard, ShouldCancelPreviousCard};
use super::GameLogic;
use super::PlayerUUID;
use std::sync::Arc;

#[derive(Clone)]
pub struct GameInterrupts {
    interrupt_stacks: Vec<GameInterruptStack>,
}

impl GameInterrupts {
    pub fn new() -> Self {
        Self {
            interrupt_stacks: Vec::new(),
        }
    }

    pub fn push_new_stack(
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
    pub fn push_new_stacks(
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

    fn can_push_to_current_stack(&self, game_interrupt_type: GameInterruptType) -> bool {
        match self.get_current_interrupt() {
            Some(current_interrupt) => {
                if !game_interrupt_type.variant_eq(current_interrupt) {
                    return false;
                }
            },
            None => return false
        };

        !self.interrupt_stacks.is_empty()
    }

    pub fn push_to_current_stack(
        &mut self,
        game_interrupt_type: GameInterruptType,
        card: InterruptPlayerCard,
        card_owner_uuid: PlayerUUID,
    ) -> Result<(), InterruptPlayerCard> {
        if !self.can_push_to_current_stack(game_interrupt_type) {
            return Err(card);
        }

        let current_stack = match self.interrupt_stacks.first_mut() {
            Some(current_stack) => current_stack,
            None => return Err(card),
        };

        current_stack.interrupt_cards.push(GameInterruptData {
            card,
            card_interrupt_type: game_interrupt_type,
            card_owner_uuid,
        });

        Ok(())
    }

    pub fn resolve_current_stack(
        &mut self,
        game_logic: &mut GameLogic,
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
                .interrupt(&game_interrupt_data.card_owner_uuid, self) {
                    ShouldCancelPreviousCard::Negate => {
                        if let Some(game_interrupt_data) = current_stack.interrupt_cards.pop() {
                            spent_cards.push((game_interrupt_data.card_owner_uuid, game_interrupt_data.card.into()));
                        } else {
                            should_cancel_root_card = ShouldCancelPreviousCard::Negate;
                        }
                    },
                    ShouldCancelPreviousCard::Ignore => {
                        if let Some(game_interrupt_data) = current_stack.interrupt_cards.pop() {
                            spent_cards.push((game_interrupt_data.card_owner_uuid, game_interrupt_data.card.into()));
                        } else {
                            should_cancel_root_card = ShouldCancelPreviousCard::Ignore;
                        }
                    },
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
                    if Arc::ptr_eq(&self.interrupt_stacks.get(i).unwrap().root_card, &current_stack.root_card) {
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
            },
            ShouldCancelPreviousCard::Ignore => {
                if let Ok(root_card) = Arc::try_unwrap(current_stack.root_card) {
                    spent_cards.push((current_stack.root_card_owner_uuid, root_card.into()));
                };
            },
            ShouldCancelPreviousCard::No => {
                current_stack.root_card.interrupt_play(
                    &current_stack.root_card_owner_uuid,
                    &current_stack.targeted_player_uuid,
                    game_logic,
                );
        
                if let Ok(root_card) = Arc::try_unwrap(current_stack.root_card) {
                    // TODO - Handle this unwrap.
                    root_card.get_interrupt_data_or().unwrap().post_interrupt_play(&current_stack.root_card_owner_uuid, game_logic);
                    spent_cards.push((current_stack.root_card_owner_uuid, root_card.into()));
                };
            }
        };

        Ok(spent_cards)
    }

    pub fn get_current_interrupt(&self) -> Option<GameInterruptType> {
        let current_stack = self.interrupt_stacks.first()?;

        Some(match current_stack.interrupt_cards.last() {
            Some(most_recent_interrupt_data) => most_recent_interrupt_data.card_interrupt_type,
            None => current_stack.root_card_interrupt_type,
        })
    }

    pub fn get_last_player_to_play_on_current_stack(&self) -> Option<&PlayerUUID> {
        let current_stack = self.interrupt_stacks.first()?;

        Some(match current_stack.interrupt_cards.last() {
            Some(most_recent_interrupt_data) => &most_recent_interrupt_data.card_owner_uuid,
            None => &current_stack.root_card_owner_uuid,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.interrupt_stacks.is_empty()
    }
}

impl Default for GameInterrupts {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone)]
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
            cards.push((game_interrupt_data.card_owner_uuid, game_interrupt_data.card.into()));
        }

        if let Ok(root_card) = Arc::try_unwrap(self.root_card) {
            cards.push((self.root_card_owner_uuid, root_card.into()));
        };

        cards
    }
}

#[derive(Clone)]
struct GameInterruptData {
    card: InterruptPlayerCard,
    card_interrupt_type: GameInterruptType,
    card_owner_uuid: PlayerUUID,
}

#[derive(Clone, Copy)]
pub struct PlayerCardInfo {
    pub affects_fortitude: bool,
}

// TODO - Uncomment and fix this test module.
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use super::super::player_card::change_other_player_fortitude;

//     #[test]
//     fn is_empty_returns_correct_value() {
//         let mut game_interrupts = GameInterrupts::new();
//         assert_eq!(game_interrupts.is_empty(), true);
//         game_interrupts.push_new_stack(GameInterruptType::AboutToDrink, change_other_player_fortitude("Face punch", 2).into(), PlayerUUID::new(), PlayerUUID::new());
//         assert_eq!(game_interrupts.is_empty(), false);
//         game_interrupts.resolve_current_stack().unwrap();
//         assert_eq!(game_interrupts.is_empty(), true);
//     }
// }
