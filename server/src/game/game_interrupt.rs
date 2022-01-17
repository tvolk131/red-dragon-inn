use super::error::Error;
use super::PlayerUUID;
use super::player_card::PlayerCard;
use std::sync::Arc;

#[derive(Clone)]
pub struct GameInterrupts {
    interrupt_stacks: Vec<Vec<GameInterruptData>>
}

impl GameInterrupts {
    pub fn new() -> Self {
        Self {
            interrupt_stacks: Vec::new()
        }
    }

    pub fn push_new_stack(&mut self, game_interrupt_type: GameInterruptType, card: PlayerCard, card_owner_uuid: PlayerUUID, interrupt_override_player_uuid: PlayerUUID) {
        self.interrupt_stacks.push(vec![
            GameInterruptData {
                game_interrupt_type,
                card: Arc::from(card),
                card_owner_uuid,
                interrupt_override_player_uuid
            }
        ]);
    }

    /// Create multiple consecutive interrupt stacks each targeting a different player.
    /// This is used for cards where multiple players are affected individually, such as
    /// an `I Raise` card, which forces each individual user to ante.
    pub fn push_new_stacks(&mut self, game_interrupt_type: GameInterruptType, card: PlayerCard, card_owner_uuid: &PlayerUUID, interrupt_override_player_uuids: Vec<PlayerUUID>) {
        let card_arc = Arc::from(card);

        for interrupt_override_player_uuid in interrupt_override_player_uuids {
            self.interrupt_stacks.push(vec![
                GameInterruptData {
                    game_interrupt_type,
                    card: card_arc.clone(),
                    card_owner_uuid: card_owner_uuid.clone(),
                    interrupt_override_player_uuid
                }
            ]);
        }
    }

    pub fn push_to_current_stack(&mut self, game_interrupt_type: GameInterruptType, card: PlayerCard, card_owner_uuid: PlayerUUID, interrupt_override_player_uuid: PlayerUUID) {
        let current_stack = match self.interrupt_stacks.first_mut() {
            Some(current_stack) => current_stack,
            None => return self.push_new_stack(game_interrupt_type, card, card_owner_uuid, interrupt_override_player_uuid)
        };

        current_stack.push(
            GameInterruptData {
                game_interrupt_type,
                card: Arc::from(card),
                card_owner_uuid: card_owner_uuid.clone(),
                interrupt_override_player_uuid: interrupt_override_player_uuid.clone()
            }
        );
    }

    pub fn resolve_current_stack(&mut self) -> Result<Vec<(PlayerUUID, PlayerCard)>, Error> {
        if self.interrupt_stacks.is_empty() {
            return Err(Error::new("No stacks to resolve"));
        }
        // The check above will prevent `remove` from panicking.
        let mut current_stack = self.interrupt_stacks.remove(0);

        let mut return_val = Vec::new();

        // TODO - Finish implementing this method.
        while let Some(game_interrupt_data) = current_stack.pop() {
            match game_interrupt_data.card.as_ref() {
                PlayerCard::SimplePlayerCard(simple_player_card) => {},
                PlayerCard::DirectedPlayerCard(directed_player_card) => {},
                PlayerCard::InterruptPlayerCard(interrupt_player_card) => {}
            };

            if let Ok(card) = Arc::try_unwrap(game_interrupt_data.card) {
                return_val.push((game_interrupt_data.card_owner_uuid, card));
            };
        }

        Ok(return_val)
    }

    pub fn get_current_interrupt(&self) -> Option<GameInterruptType> {
        let current_stack = match self.interrupt_stacks.first() {
            Some(current_stack) => current_stack,
            None => return None
        };

        Some(current_stack.last()?.game_interrupt_type)
    }

    pub fn is_empty(&self) -> bool {
        self.interrupt_stacks.is_empty()
    }

    fn prune(&mut self) {
        self.interrupt_stacks.retain(|stack| !stack.is_empty());
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
struct GameInterruptData {
    game_interrupt_type: GameInterruptType,
    card: Arc<PlayerCard>,
    card_owner_uuid: PlayerUUID,
    interrupt_override_player_uuid: PlayerUUID
}

#[derive(Clone, Copy)]
pub struct PlayerCardInfo {
    pub affects_fortitude: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::player_card::gambling_im_in_card;

    #[test]
    fn is_empty_returns_correct_value() {
        let mut game_interrupts = GameInterrupts::new();
        assert_eq!(game_interrupts.is_empty(), true);
        game_interrupts.push_new_stack(GameInterruptType::AboutToDrink, PlayerCard::SimplePlayerCard(gambling_im_in_card()), PlayerUUID::new(), PlayerUUID::new());
        assert_eq!(game_interrupts.is_empty(), false);
        game_interrupts.resolve_current_stack().unwrap();
        assert_eq!(game_interrupts.is_empty(), true);
    }
}
