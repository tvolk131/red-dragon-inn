use super::super::player::Player;
use super::{Drink, DrinkCard};

#[derive(Clone, Debug)]
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

    pub fn get_display_name(&self) -> String {
        // TODO - I'm pretty sure this will end up with a comma at the end of the last element. Let's fix that.
        format!(
            "[{}]",
            self.drinks.iter().fold(String::new(), |acc, drink| acc
                + drink.get_display_name()
                + ", ")
        )
    }

    pub fn process(&self, player: &mut Player) {
        let alcohol_content_modifier = self.get_combined_alcohol_content_modifier(player);
        let fortitude_modifier = self.get_combined_fortitude_modifier(player);

        player.change_alcohol_content(alcohol_content_modifier);
        player.change_fortitude(fortitude_modifier);
    }

    fn get_combined_alcohol_content_modifier(&self, player: &Player) -> i32 {
        let mut modifier = 0;
        for drink in &self.drinks {
            modifier += drink.get_alcohol_content_modifier(player);
        }
        modifier
    }

    fn get_combined_fortitude_modifier(&self, player: &Player) -> i32 {
        let mut modifier = 0;
        for drink in &self.drinks {
            modifier += drink.get_fortitude_modifier(player);
        }
        modifier
    }
}
