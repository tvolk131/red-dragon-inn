use super::drink::Drink;
use super::player_card::PlayerCard;
use std::sync::{Arc, Mutex};
use super::Error;

#[derive(Clone, PartialEq, Eq)]
pub struct PlayerUUID(String);

impl PlayerUUID {
    pub fn new() -> Self {
        // TODO - Should generate actual unique id rather than an empty string.
        Self("".to_string())
    }
}

impl std::string::ToString for PlayerUUID {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

pub struct Player {
    alcohol_content: i32,
    fortitude: i32,
    gold: i32,
    hand: Vec<Box<dyn PlayerCard>>,
    draw_pile: Vec<Box<dyn PlayerCard>>,
    discard_pile: Vec<Box<dyn PlayerCard>>,
    drinks: Vec<Drink>,
}

impl Player {
    pub fn new(gold: i32, deck: Vec<Box<dyn PlayerCard>>) -> Self {
        let mut player = Self {
            alcohol_content: 0,
            fortitude: 20,
            gold,
            hand: Vec::new(),
            draw_pile: deck,
            discard_pile: Vec::new(),
            drinks: Vec::new(),
        };
        player.draw_to_full();
        player
    }

    pub fn draw_to_full(&mut self) {
        while self.hand.len() < 7 {
            self.hand.push(self.draw_pile.pop().unwrap());
        }
    }

    pub fn play_card_from_hand(&mut self, card_index: usize) -> Option<Error> {
        if self.hand.get(card_index).is_none() {
            return Some(Error("Card does not exist".to_string()));
        }
        let card = self.hand.remove(card_index);

        return if card.can_play() {
            card.play();
            None
        } else {
            Some(Error("Card cannot be played at this time".to_string()))
        };
    }

    pub fn drink_from_drink_pile(&mut self) -> Option<Drink> {
        if let Some(drink) = self.drinks.pop() {
            self.drink(&drink);
            return Some(drink);
        } else {
            return None;
        }
    }

    pub fn drink(&mut self, drink: &Drink) {
        self.alcohol_content += drink.get_alcohol_content_modifier();
        self.fortitude += drink.get_fortitude_modifier();
    }

    pub fn is_out_of_game(&self) -> bool {
        self.is_broke() || self.is_passed_out()
    }

    fn is_broke(&self) -> bool {
        self.gold <= 0
    }

    fn is_passed_out(&self) -> bool {
        self.alcohol_content >= self.fortitude
    }
}
