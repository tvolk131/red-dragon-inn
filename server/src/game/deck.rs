use super::drink::{DrinkCard, DrinkDeck};
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Clone, Debug)]
pub struct AutoShufflingDeck<T> {
    draw_pile: Vec<T>,
    discard_pile: Vec<T>,
}

impl<T> AutoShufflingDeck<T> {
    pub fn new(mut items: Vec<T>) -> Self {
        items.shuffle(&mut thread_rng());

        Self {
            draw_pile: items,
            discard_pile: Vec::new(),
        }
    }

    pub fn draw_card(&mut self) -> Option<T> {
        if self.draw_pile.is_empty() {
            self.discard_pile
                .drain(..)
                .for_each(|card| self.draw_pile.push(card));
            self.draw_pile.shuffle(&mut thread_rng());
        }
        self.draw_pile.pop()
    }

    pub fn discard_card(&mut self, card: T) {
        self.discard_pile.push(card);
    }

    pub fn draw_pile_size(&self) -> usize {
        self.draw_pile.len()
    }

    pub fn discard_pile_size(&self) -> usize {
        self.discard_pile.len()
    }
}

impl DrinkDeck for AutoShufflingDeck<DrinkCard> {
    fn get_next_drink_card_or(&mut self) -> Option<DrinkCard> {
        self.draw_card()
    }
}
