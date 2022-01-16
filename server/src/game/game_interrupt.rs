use super::error::Error;
use super::PlayerUUID;
use super::player_card::PlayerCard;

pub struct GameInterrupts {
    interrupt_stacks: Vec<Vec<(PlayerUUID, PlayerCard)>>
}

impl GameInterrupts {
    pub fn new() -> Self {
        Self {
            interrupt_stacks: Vec::new()
        }
    }

    pub fn push_new_stack(&mut self, player_uuid: PlayerUUID, card: PlayerCard) {
        self.interrupt_stacks.push(vec![(player_uuid, card)]);
    }

    pub fn can_push_to_current_stack(&self, player_uuid: &PlayerUUID, card: &PlayerCard) -> bool {
        let current_stack = match self.interrupt_stacks.first() {
            Some(current_stack) => current_stack,
            None => return false
        };

        if let Some((_, most_recent_interrupt_card)) = current_stack.last() {
            if let Some(interrupt_type) = most_recent_interrupt_card.get_interrupt_type_output_or() {
                // TODO - Finish implementing this statement. It should not be hardcoded to `false`.
                false
            } else {
                // Card is uninterruptable.
                false
            }
        } else {
            // This line should never be hit. If it is, that
            // means the struct isn't being pruned properly.
            true
        }
    }

    pub fn push_to_current_stack(&mut self, player_uuid: PlayerUUID, card: PlayerCard) -> Result<(), Error> {
        let current_stack = match self.interrupt_stacks.first_mut() {
            Some(current_stack) => current_stack,
            None => return Err(Error::new("No interrupt stack to push to"))
        };

        current_stack.push((player_uuid, card));
        Ok(())
    }

    pub fn peek(&self) -> Option<&(PlayerUUID, PlayerCard)> {
        Some(self.interrupt_stacks.first()?.last()?)
    }

    pub fn pop(&mut self) -> Option<(PlayerUUID, PlayerCard)> {
        let val = self.interrupt_stacks.first_mut()?.pop()?;
        self.prune();
        Some(val)
    }

    pub fn is_empty(&self) -> bool {
        self.interrupt_stacks.is_empty()
    }

    fn prune(&mut self) {
        self.interrupt_stacks.retain(|stack| !stack.is_empty());
    }
}

#[derive(Clone)]
pub enum GameInterruptType {
    AboutToAnte,
    AboutToSpendGold,
    DirectedActionCardPlayed(PlayerCardInfo),
    SometimesCardPlayed(PlayerCardInfo),
    AboutToDrink,
}

#[derive(Clone)]
pub struct PlayerCardInfo {
    pub affects_fortitude: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::player_card::gambling_im_in_card;

    #[test]
    fn read_methods_behave_when_empty() {
        let mut game_interrupts = GameInterrupts::new();
        assert_eq!(game_interrupts.is_empty(), true);
        assert!(game_interrupts.peek().is_none());
        assert!(game_interrupts.pop().is_none());
        assert_eq!(game_interrupts.can_push_to_current_stack(&PlayerUUID::new(), &PlayerCard::SimplePlayerCard(gambling_im_in_card())), false);
    }

    #[test]
    fn read_methods_behave_when_not_empty() {
        let mut game_interrupts = GameInterrupts::new();
        game_interrupts.push_new_stack(PlayerUUID::new(), PlayerCard::SimplePlayerCard(gambling_im_in_card()));
        assert_eq!(game_interrupts.is_empty(), false);
        assert!(game_interrupts.peek().is_some());
        assert!(game_interrupts.pop().is_some());
        assert_eq!(game_interrupts.can_push_to_current_stack(&PlayerUUID::new(), &PlayerCard::SimplePlayerCard(gambling_im_in_card())), true);
    }
}
