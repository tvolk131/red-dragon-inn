use super::player::Player;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Debug)]
pub enum DrinkCard {
    Drink(Drink),
    DrinkEvent(DrinkEvent),
}

impl From<Drink> for DrinkCard {
    fn from(drink: Drink) -> DrinkCard {
        DrinkCard::Drink(drink)
    }
}

impl From<DrinkEvent> for DrinkCard {
    fn from(drink_event: DrinkEvent) -> DrinkCard {
        DrinkCard::DrinkEvent(drink_event)
    }
}

#[derive(Clone)]
pub struct Drink {
    display_name: String,
    process_fn: fn(player: &mut Player),
    has_chaser: bool,
}

impl Debug for Drink {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_name)
    }
}

impl Drink {
    pub fn process(&self, player: &mut Player) {
        (self.process_fn)(player)
    }

    pub fn has_chaser(&self) -> bool {
        self.has_chaser
    }
}

#[derive(Clone, Debug)]
pub enum DrinkEvent {
    DrinkingContest,
    RoundOnTheHouse,
}

pub struct DrinkWithPossibleChasers {
    drinks: Vec<Drink>,
    ignored_card_or: Option<DrinkCard>,
}

impl DrinkWithPossibleChasers {
    pub fn new(drinks: Vec<Drink>, ignored_card_or: Option<DrinkCard>) -> Self {
        Self {
            drinks,
            ignored_card_or,
        }
    }

    pub fn get_drinks(&self) -> &Vec<Drink> {
        &self.drinks
    }

    pub fn take_all_discardable_drink_cards(self) -> Vec<DrinkCard> {
        let mut discardable_drink_cards = Vec::new();
        for drink in self.drinks {
            discardable_drink_cards.push(drink.into());
        }
        if let Some(ignored_card) = self.ignored_card_or {
            discardable_drink_cards.push(ignored_card);
        }
        discardable_drink_cards
    }
}

pub enum RevealedDrink {
    DrinkWithPossibleChasers(DrinkWithPossibleChasers),
    DrinkEvent(DrinkEvent),
}

macro_rules! simple_drink {
    ($display_name:expr, $alcohol_content_mod:expr, $fortitude_mod:expr, $has_chaser:expr) => {
        Drink {
            display_name: $display_name.to_string(),
            process_fn: |player: &mut Player| {
                player.change_alcohol_content($alcohol_content_mod);
                player.change_fortitude($fortitude_mod);
            },
            has_chaser: $has_chaser,
        }
    };
}

#[cfg(test)]
pub fn create_simple_ale_test_drink(has_chaser: bool) -> Drink {
    simple_drink!("Test Ale", 1, 0, has_chaser)
}

fn orcish_rotgut() -> Drink {
    Drink {
        display_name: "Orcish Rotgut".to_string(),
        process_fn: |player: &mut Player| {
            if player.is_orc() {
                player.change_alcohol_content(2);
            } else {
                player.change_fortitude(-2);
            }
        },
        has_chaser: false,
    }
}

fn troll_swill() -> Drink {
    Drink {
        display_name: "Troll Swill".to_string(),
        process_fn: |player: &mut Player| {
            if player.is_troll() {
                player.change_alcohol_content(2);
            } else {
                player.change_alcohol_content(1);
                player.change_fortitude(-1);
            }
        },
        has_chaser: false,
    }
}

pub fn create_drink_deck() -> Vec<DrinkCard> {
    vec![
        simple_drink!("Dark Ale", 1, 0, false).into(),
        simple_drink!("Dark Ale", 1, 0, false).into(),
        simple_drink!("Dark Ale", 1, 0, false).into(),
        simple_drink!("Dark Ale with a Chaser", 1, 0, true).into(),
        simple_drink!("Dirty Dishwater", 0, -1, false).into(),
        simple_drink!("Dragon Breath Ale", 4, 0, false).into(),
        simple_drink!("Dragon Breath Ale", 4, 0, false).into(),
        simple_drink!("Dragon Breath Ale", 4, 0, false).into(),
        simple_drink!("Elven Wine", 3, 0, false).into(),
        simple_drink!("Elven Wine", 3, 0, false).into(),
        simple_drink!("Elven Wine with a Chaser", 3, 0, true).into(),
        simple_drink!("Holy Water", 0, 2, false).into(),
        simple_drink!("Light Ale", 1, 0, false).into(),
        simple_drink!("Light Ale", 1, 0, false).into(),
        simple_drink!("Light Ale", 1, 0, false).into(),
        simple_drink!("Light Ale with a Chaser", 1, 0, true).into(),
        simple_drink!("Light Ale with a Chaser", 1, 0, true).into(),
        simple_drink!("Wine", 2, 0, false).into(),
        simple_drink!("Wine", 2, 0, false).into(),
        simple_drink!("Wine", 2, 0, false).into(),
        simple_drink!("Wine with a Chaser", 2, 0, true).into(),
        simple_drink!("Wizard's Brew", 2, 2, false).into(),
        orcish_rotgut().into(),
        troll_swill().into(),
        simple_drink!("Water", 0, 0, false).into(),
        simple_drink!("We're Cutting You Off!", -1, 0, false).into(),
        DrinkCard::DrinkEvent(DrinkEvent::DrinkingContest),
        DrinkCard::DrinkEvent(DrinkEvent::DrinkingContest),
        DrinkCard::DrinkEvent(DrinkEvent::RoundOnTheHouse),
        DrinkCard::DrinkEvent(DrinkEvent::RoundOnTheHouse),
    ]
}

macro_rules! impl_get_revealed_drink {
    ($struct_name:ty, $get_next_card:expr) => {
        impl $struct_name {
            pub fn get_revealed_drink(&mut self) -> Option<RevealedDrink> {
                Some(match $get_next_card(self)? {
                    DrinkCard::Drink(drink) => {
                        if !drink.has_chaser() {
                            RevealedDrink::DrinkWithPossibleChasers(DrinkWithPossibleChasers::new(
                                vec![drink],
                                None,
                            ))
                        } else {
                            match self.push_drink_to_vec_or(vec![drink]) {
                                Ok(drinks) => RevealedDrink::DrinkWithPossibleChasers(
                                    DrinkWithPossibleChasers::new(drinks, None),
                                ),
                                Err((drinks, discarded_drink_card)) => {
                                    RevealedDrink::DrinkWithPossibleChasers(
                                        DrinkWithPossibleChasers::new(
                                            drinks,
                                            Some(discarded_drink_card),
                                        ),
                                    )
                                }
                            }
                        }
                    }
                    DrinkCard::DrinkEvent(drink_event) => RevealedDrink::DrinkEvent(drink_event),
                })
            }

            fn push_drink_to_vec_or(
                &mut self,
                mut drinks: Vec<Drink>,
            ) -> Result<Vec<Drink>, (Vec<Drink>, DrinkCard)> {
                match $get_next_card(self) {
                    Some(next_drink_card) => match next_drink_card {
                        DrinkCard::Drink(drink) => {
                            drinks.push(drink);
                            self.push_drink_to_vec_or(drinks)
                        }
                        DrinkCard::DrinkEvent(drink_event) => {
                            Err((drinks, DrinkCard::DrinkEvent(drink_event)))
                        }
                    },
                    None => Ok(drinks),
                }
            }
        }
    };
}

pub(crate) use impl_get_revealed_drink;
