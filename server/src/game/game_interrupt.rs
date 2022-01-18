use super::error::Error;
use super::PlayerUUID;
use super::player_card::{PlayerCard, InterruptPlayerCard, DirectedPlayerCard};
use std::sync::Arc;
use super::GameLogic;

#[derive(Clone)]
pub struct GameInterrupts {
    interrupt_stacks: Vec<GameInterruptStack>
}

impl GameInterrupts {
    pub fn new() -> Self {
        Self {
            interrupt_stacks: Vec::new()
        }
    }

    pub fn push_new_stack(&mut self, game_interrupt_type: GameInterruptType, card: DirectedPlayerCard, card_owner_uuid: PlayerUUID, targeted_player_uuid: PlayerUUID) {
        self.interrupt_stacks.push(GameInterruptStack {
            root_card: Arc::from(card),
            root_card_interrupt_type: game_interrupt_type,
            root_card_owner_uuid: card_owner_uuid,
            targeted_player_uuid,
            interrupt_cards: Vec::new()
        });
    }

    /// Create multiple consecutive interrupt stacks each targeting a different player.
    /// This is used for cards where multiple players are affected individually, such as
    /// an `I Raise` card, which forces each individual user to ante.
    pub fn push_new_stacks(&mut self, game_interrupt_type: GameInterruptType, card: DirectedPlayerCard, card_owner_uuid: &PlayerUUID, targeted_player_uuids: Vec<PlayerUUID>) {
        let card_arc = Arc::from(card);

        for targeted_player_uuid in targeted_player_uuids {
            self.interrupt_stacks.push(GameInterruptStack {
                root_card: card_arc.clone(),
                root_card_interrupt_type: game_interrupt_type,
                root_card_owner_uuid: card_owner_uuid.clone(),
                targeted_player_uuid,
                interrupt_cards: Vec::new()
            });
        }
    }

    pub fn push_to_current_stack(&mut self, game_interrupt_type: GameInterruptType, card: InterruptPlayerCard, card_owner_uuid: PlayerUUID) -> Result<(), InterruptPlayerCard> {
        let current_stack = match self.interrupt_stacks.first_mut() {
            Some(current_stack) => current_stack,
            None => return Err(card)
        };

        current_stack.interrupt_cards.push(
            GameInterruptData {
                card,
                card_interrupt_type: game_interrupt_type,
                card_owner_uuid
            }
        );

        Ok(())
    }

    pub fn resolve_current_stack(&mut self, game_logic: &mut GameLogic) -> Result<Vec<(PlayerUUID, PlayerCard)>, Error> {
        if self.interrupt_stacks.is_empty() {
            return Err(Error::new("No stacks to resolve"));
        }
        // The check above will prevent `remove` from panicking.
        let mut current_stack = self.interrupt_stacks.remove(0);

        let mut spent_cards = Vec::new();

        // TODO - Finish implementing this method.
        while let Some(game_interrupt_data) = current_stack.interrupt_cards.pop() {
            game_interrupt_data.card.interrupt(&game_interrupt_data.card_owner_uuid, self);
            spent_cards.push((game_interrupt_data.card_owner_uuid, game_interrupt_data.card.into()));
        }

        current_stack.root_card.play(&current_stack.root_card_owner_uuid, &current_stack.targeted_player_uuid, game_logic);

        if let Ok(card) = Arc::try_unwrap(current_stack.root_card) {
            spent_cards.push((current_stack.root_card_owner_uuid, card.into()));
        };

        Ok(spent_cards)
    }

    pub fn get_current_interrupt(&self) -> Option<GameInterruptType> {
        let current_stack = match self.interrupt_stacks.first() {
            Some(current_stack) => current_stack,
            None => return None
        };

        Some(match current_stack.interrupt_cards.last() {
            Some(most_recent_interrupt_data) => most_recent_interrupt_data.card_interrupt_type,
            None => current_stack.root_card_interrupt_type
        })
    }

    pub fn is_empty(&self) -> bool {
        self.interrupt_stacks.is_empty()
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
    root_card: Arc<DirectedPlayerCard>,
    root_card_interrupt_type: GameInterruptType,
    root_card_owner_uuid: PlayerUUID,
    targeted_player_uuid: PlayerUUID, // The player that the root card is targeting.
    interrupt_cards: Vec<GameInterruptData>
}

#[derive(Clone)]
struct GameInterruptData {
    card: InterruptPlayerCard,
    card_interrupt_type: GameInterruptType,
    card_owner_uuid: PlayerUUID
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
