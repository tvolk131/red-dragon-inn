use super::player::Player;

pub trait Drink: Send + Sync {
    fn process(&self, player: &mut Player);
}

pub fn create_drink_deck() -> Vec<Box<dyn Drink>> {
    vec![
        Box::from(DarkAle {}),
        Box::from(DarkAle {}),
        Box::from(DarkAle {}),
        // with_chaser(dark_ale()),
        Box::from(DirtyDishwater {}),
        Box::from(DragonBreathAle {}),
        Box::from(DragonBreathAle {}),
        Box::from(DragonBreathAle {}),
        // drinking_contest(),
        // drinking_contest(),
        Box::from(ElvenWine {}),
        Box::from(ElvenWine {}),
        // with_chaser(elven_wine()),
        Box::from(HolyWater {}),
        Box::from(LightAle {}),
        Box::from(LightAle {}),
        Box::from(LightAle {}),
        // with_chaser(light_ale()),
        // with_chaser(light_ale()),
        Box::from(OrcishRotgut {}),
        // round_on_the_house(),
        // round_on_the_house(),
        // troll_swill(),
        Box::from(Water {}),
        // were_cutting_you_off(),
        Box::from(Wine {}),
        Box::from(Wine {}),
        Box::from(Wine {}),
        // with_chaser(wine()),
        // wizards_brew()
    ]
}

macro_rules! simple_drink {
    ($struct_name:ident, $alcohol_content_mod:expr, $fortitude_mod: expr) => {
        pub struct $struct_name {}

        impl Drink for $struct_name {
            fn process(&self, player: &mut Player) {
                player.change_alcohol_content($alcohol_content_mod);
                player.change_fortitude($fortitude_mod);
            }
        }
    };
}

simple_drink!(ElvenWine, 3, 0);
simple_drink!(Wine, 2, 0);
simple_drink!(DirtyDishwater, 0, -1);
simple_drink!(LightAle, 1, 0);
simple_drink!(DarkAle, 1, 0);
simple_drink!(DragonBreathAle, 4, 0);
simple_drink!(Water, -1, 0);
simple_drink!(HolyWater, 0, 2);

pub struct OrcishRotgut {}

impl Drink for OrcishRotgut {
    fn process(&self, player: &mut Player) {
        if player.is_orc() {
            player.change_alcohol_content(2);
        } else {
            player.change_fortitude(-2);
        }
    }
}
