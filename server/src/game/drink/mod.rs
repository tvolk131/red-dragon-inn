mod drink_struct;
mod drink_with_possible_chasers;

use drink_struct::{orcish_rotgut, simple_drink, troll_swill, Drink};
pub use drink_with_possible_chasers::DrinkWithPossibleChasers;
use std::fmt::Debug;

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

#[derive(Clone, Debug)]
pub enum DrinkEvent {
    DrinkingContest,
    RoundOnTheHouse,
}

pub enum RevealedDrink {
    DrinkWithPossibleChasers(DrinkWithPossibleChasers),
    DrinkEvent(DrinkEvent),
}

#[cfg(test)]
pub fn create_simple_ale_test_drink(has_chaser: bool) -> Drink {
    simple_drink("Test Ale", 1, 0, has_chaser)
}

pub fn create_drink_deck() -> Vec<DrinkCard> {
    vec![
        simple_drink("Dark Ale", 1, 0, false).into(),
        simple_drink("Dark Ale", 1, 0, false).into(),
        simple_drink("Dark Ale", 1, 0, false).into(),
        simple_drink("Dark Ale with a Chaser", 1, 0, true).into(),
        simple_drink("Dirty Dishwater", 0, -1, false).into(),
        simple_drink("Dragon Breath Ale", 4, 0, false).into(),
        simple_drink("Dragon Breath Ale", 4, 0, false).into(),
        simple_drink("Dragon Breath Ale", 4, 0, false).into(),
        simple_drink("Elven Wine", 3, 0, false).into(),
        simple_drink("Elven Wine", 3, 0, false).into(),
        simple_drink("Elven Wine with a Chaser", 3, 0, true).into(),
        simple_drink("Holy Water", 0, 2, false).into(),
        simple_drink("Light Ale", 1, 0, false).into(),
        simple_drink("Light Ale", 1, 0, false).into(),
        simple_drink("Light Ale", 1, 0, false).into(),
        simple_drink("Light Ale with a Chaser", 1, 0, true).into(),
        simple_drink("Light Ale with a Chaser", 1, 0, true).into(),
        simple_drink("Wine", 2, 0, false).into(),
        simple_drink("Wine", 2, 0, false).into(),
        simple_drink("Wine", 2, 0, false).into(),
        simple_drink("Wine with a Chaser", 2, 0, true).into(),
        simple_drink("Wizard's Brew", 2, 2, false).into(),
        simple_drink("Water", 0, 0, false).into(),
        simple_drink("We're Cutting You Off!", -1, 0, false).into(),
        orcish_rotgut().into(),
        troll_swill().into(),
        DrinkCard::DrinkEvent(DrinkEvent::DrinkingContest),
        DrinkCard::DrinkEvent(DrinkEvent::DrinkingContest),
        DrinkCard::DrinkEvent(DrinkEvent::RoundOnTheHouse),
        DrinkCard::DrinkEvent(DrinkEvent::RoundOnTheHouse),
    ]
}

pub trait DrinkDeck {
    fn get_next_drink_card_or(&mut self) -> Option<DrinkCard>;
}

pub fn get_revealed_drink(drink_deck: &mut impl DrinkDeck) -> Option<RevealedDrink> {
    Some(match drink_deck.get_next_drink_card_or()? {
        DrinkCard::Drink(drink) => {
            if !drink.has_chaser() {
                RevealedDrink::DrinkWithPossibleChasers(DrinkWithPossibleChasers::new(
                    vec![drink],
                    None,
                ))
            } else {
                match push_drink_to_vec_or(drink_deck, vec![drink]) {
                    Ok(drinks) => RevealedDrink::DrinkWithPossibleChasers(
                        DrinkWithPossibleChasers::new(drinks, None),
                    ),
                    Err((drinks, discarded_drink_card)) => RevealedDrink::DrinkWithPossibleChasers(
                        DrinkWithPossibleChasers::new(drinks, Some(discarded_drink_card)),
                    ),
                }
            }
        }
        DrinkCard::DrinkEvent(drink_event) => RevealedDrink::DrinkEvent(drink_event),
    })
}

pub fn get_drink_with_possible_chasers_skipping_drink_events(
    drink_deck: &mut impl DrinkDeck,
) -> Option<(DrinkWithPossibleChasers, Vec<DrinkEvent>)> {
    let mut drink_events = Vec::new();
    loop {
        match get_revealed_drink(drink_deck) {
            Some(revealed_drink) => {
                match revealed_drink {
                    RevealedDrink::DrinkWithPossibleChasers(drink) => {
                        return Some((drink, drink_events))
                    }
                    RevealedDrink::DrinkEvent(drink_event) => drink_events.push(drink_event),
                };
            }
            None => return None,
        };
    }
}

fn push_drink_to_vec_or(
    drink_deck: &mut impl DrinkDeck,
    mut drinks: Vec<Drink>,
) -> Result<Vec<Drink>, (Vec<Drink>, DrinkCard)> {
    match drink_deck.get_next_drink_card_or() {
        Some(next_drink_card) => match next_drink_card {
            DrinkCard::Drink(drink) => {
                drinks.push(drink);
                push_drink_to_vec_or(drink_deck, drinks)
            }
            DrinkCard::DrinkEvent(drink_event) => Err((drinks, DrinkCard::DrinkEvent(drink_event))),
        },
        None => Ok(drinks),
    }
}
