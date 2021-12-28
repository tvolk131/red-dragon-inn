use super::drink::Drink;
use super::player_card::PlayerCard;

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
    pub fn new(gold: i32) -> Self {
        Self {
            alcohol_content: 0,
            fortitude: 20,
            gold,
            hand: Vec::new(),
            draw_pile: Vec::new(),
            discard_pile: Vec::new(),
            drinks: Vec::new(),
        }
    }

    pub fn drink(&mut self, drink: Drink) {
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
