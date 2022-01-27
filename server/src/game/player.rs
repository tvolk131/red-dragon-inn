use super::deck::AutoShufflingDeck;
use super::drink::Drink;
use super::gambling_manager::GamblingManager;
use super::game_logic::TurnInfo;
use super::interrupt_manager::InterruptManager;
use super::player_card::PlayerCard;
use super::player_view::{GameViewPlayerCard, GameViewPlayerData};
use super::uuid::PlayerUUID;
use super::Character;
use std::borrow::Borrow;

#[derive(Clone)]
pub struct Player {
    alcohol_content: i32,
    fortitude: i32,
    gold: i32,
    hand: Vec<PlayerCard>,
    deck: AutoShufflingDeck<PlayerCard>,
    drinks: Vec<Box<dyn Drink>>,
    is_orc: bool,
}

impl Player {
    pub fn create_from_character(character: Character, gold: i32) -> Self {
        Self::new(gold, character.create_deck(), character.is_orc())
    }

    fn new(gold: i32, deck: Vec<PlayerCard>, is_orc: bool) -> Self {
        let mut player = Self {
            alcohol_content: 0,
            fortitude: 20,
            gold,
            hand: Vec::new(),
            deck: AutoShufflingDeck::new(deck),
            drinks: Vec::new(),
            is_orc,
        };
        player.draw_to_full();
        player
    }

    pub fn to_game_view_player_data(&self, player_uuid: PlayerUUID) -> GameViewPlayerData {
        GameViewPlayerData {
            player_uuid,
            draw_pile_size: self.deck.draw_pile_size(),
            discard_pile_size: self.deck.discard_pile_size(),
            drink_deck_size: self.drinks.len(),
            alcohol_content: self.alcohol_content,
            fortitude: self.fortitude,
            gold: self.gold,
            is_dead: self.is_out_of_game(),
        }
    }

    pub fn get_game_view_hand(
        &self,
        player_uuid: &PlayerUUID,
        gambling_manager: &GamblingManager,
        interrupt_manager: &InterruptManager,
        turn_info: &TurnInfo,
    ) -> Vec<GameViewPlayerCard> {
        self.hand
            .iter()
            .map(|card| GameViewPlayerCard {
                card_name: card.get_display_name().to_string(),
                is_playable: card.can_play(
                    player_uuid,
                    gambling_manager,
                    interrupt_manager,
                    turn_info,
                ),
            })
            .collect()
    }

    pub fn draw_to_full(&mut self) {
        while self.hand.len() < 7 {
            self.hand.push(self.deck.draw_card().unwrap());
        }
    }

    pub fn pop_card_from_hand(&mut self, card_index: usize) -> Option<PlayerCard> {
        // This check may look unnecessary, but it's here because Vec::remove() doesn't
        // return `Option<T>` but instead returns `T` and panics if the index is out of bounds.
        if self.hand.get(card_index).is_none() {
            None
        } else {
            Some(self.hand.remove(card_index))
        }
    }

    pub fn return_card_to_hand(&mut self, card: PlayerCard, mut card_index: usize) {
        if card_index > self.hand.len() {
            card_index = self.hand.len();
        }
        // Will never panic due to the check above.
        self.hand.insert(card_index, card);
    }

    pub fn discard_card(&mut self, card: PlayerCard) {
        self.deck.discard_card(card);
    }

    pub fn is_orc(&self) -> bool {
        self.is_orc
    }

    pub fn add_drink_to_drink_pile(&mut self, drink: Box<dyn Drink>) {
        self.drinks.push(drink);
    }

    pub fn drink_from_drink_pile(&mut self) -> Option<Box<dyn Drink>> {
        if let Some(drink) = self.drinks.pop() {
            self.drink(drink.borrow());
            Some(drink)
        } else {
            None
        }
    }

    pub fn drink(&mut self, drink: &dyn Drink) {
        drink.process(self);
    }

    pub fn change_alcohol_content(&mut self, amount: i32) {
        self.alcohol_content += amount;
        if self.alcohol_content > 20 {
            self.alcohol_content = 20;
        } else if self.alcohol_content < 0 {
            self.alcohol_content = 0;
        }
    }

    pub fn get_fortitude(&self) -> i32 {
        self.fortitude
    }

    pub fn change_fortitude(&mut self, amount: i32) {
        self.fortitude += amount;
        if self.fortitude > 20 {
            self.fortitude = 20;
        } else if self.fortitude < 0 {
            self.fortitude = 0;
        }
    }

    pub fn get_gold(&self) -> i32 {
        self.gold
    }

    pub fn change_gold(&mut self, amount: i32) {
        self.gold += amount
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
