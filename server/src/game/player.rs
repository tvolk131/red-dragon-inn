use super::super::auth::SESSION_COOKIE_NAME;
use super::drink::Drink;
use super::player_card::PlayerCard;
use super::Character;
use super::Error;
use serde::Serialize;

#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub struct PlayerUUID(String);

impl PlayerUUID {
    pub fn new() -> Self {
        // TODO - Should generate actual unique id rather than an empty string.
        Self("".to_string())
    }

    pub fn from_cookie_jar(cookie_jar: &rocket::http::CookieJar) -> Result<Self, Error> {
        match cookie_jar.get(SESSION_COOKIE_NAME) {
            Some(cookie) => Ok(Self(String::from(cookie.value()))),
            None => Err(Error::new("User is not signed in")),
        }
    }

    pub fn to_cookie_jar(&self, cookie_jar: &rocket::http::CookieJar) {
        cookie_jar.add(rocket::http::Cookie::new(
            SESSION_COOKIE_NAME,
            self.to_string(),
        ))
    }
}

impl std::string::ToString for PlayerUUID {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl<'a> rocket::request::FromParam<'a> for PlayerUUID {
    type Error = String;
    fn from_param(param: &'a str) -> Result<Self, String> {
        Ok(Self(String::from(param)))
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
    pub fn create_from_character(character: Character, gold: i32) -> Self {
        // TODO - Create deck for given character.
        Self::new(gold, Vec::new())
    }

    fn new(gold: i32, deck: Vec<Box<dyn PlayerCard>>) -> Self {
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

    pub fn pop_card_from_hand(
        &mut self,
        player_uuid: &PlayerUUID,
        card_index: usize,
    ) -> Option<Box<dyn PlayerCard>> {
        // This check may look unnecessary, but it's here because Vec::remove() doesn't
        // return `Option<T>` but instead returns `T` and panics if the index is out of bounds.
        if self.hand.get(card_index).is_none() {
            None
        } else {
            Some(self.hand.remove(card_index))
        }
    }

    pub fn discard_card(&mut self, card: Box<dyn PlayerCard>) {
        self.discard_pile.push(card);
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

    pub fn add_gold(&mut self, amount: i32) {
        if amount > 0 {
            self.gold += amount
        }
    }

    pub fn remove_gold(&mut self, amount: i32) {
        if amount > 0 {
            self.gold -= amount
        }
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
