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
        // orcish_rotgut(),
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
    }
}

simple_drink!(ElvenWine, 3, 0);
simple_drink!(Wine, 2, 0);
simple_drink!(DirtyDishwater, 0, -1);
simple_drink!(LightAle, 1, 0);
simple_drink!(DarkAle, 1, 0);
simple_drink!(DragonBreathAle, 4, 0);
simple_drink!(Water, -1, 0);
simple_drink!(HolyWater, 0, 2);

// TODO - Orcish Rotgut should instead be +2 alcohol if player is an orc.
// fn orcish_rotgut() -> Drink {
//     Drink {
//         name: String::from("Orcish Rotgut"),
//         alcohol_content_modifier: 0,
//         fortitude_modifier: -2,
//         has_chaser: false,
//     }
// }
