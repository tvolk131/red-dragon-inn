use super::error::Error;
use super::player_card::PlayerCard;

pub struct GameInterruptStack {
    interrupt_card_stack: Vec<PlayerCard>,
}

impl GameInterruptStack {
    fn push_interrupt_card(&self, card: PlayerCard) -> Option<Error> {
        if let Some(most_recent_interrupt_card) = self.interrupt_card_stack.last() {
            if most_recent_interrupt_card {}
        }

        None
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
