#[must_use = "this `Drink` may be unintentionally discarded"]
pub struct Drink {
    name: String,
    alcohol_content_modifier: i32,
    fortitude_modifier: i32,
    has_chaser: bool,
}

impl Drink {
    pub fn get_alcohol_content_modifier(&self) -> i32 {
        self.alcohol_content_modifier
    }

    pub fn get_fortitude_modifier(&self) -> i32 {
        self.fortitude_modifier
    }
}

pub fn create_drink_deck() -> Vec<Drink> {
    vec![
        dark_ale(),
        dark_ale(),
        dark_ale(),
        with_chaser(dark_ale()),
        dirty_dishwater(),
        dragon_breath_ale(),
        dragon_breath_ale(),
        dragon_breath_ale(),
        // drinking_contest(),
        // drinking_contest(),
        elven_wine(),
        elven_wine(),
        with_chaser(elven_wine()),
        holy_water(),
        light_ale(),
        light_ale(),
        light_ale(),
        with_chaser(light_ale()),
        with_chaser(light_ale()),
        orcish_rotgut(),
        // round_on_the_house(),
        // round_on_the_house(),
        // troll_swill(),
        water(),
        // were_cutting_you_off(),
        wine(),
        wine(),
        wine(),
        with_chaser(wine()),
        // wizards_brew()
    ]
}

fn elven_wine() -> Drink {
    Drink {
        name: String::from("Elven Wine"),
        alcohol_content_modifier: 3,
        fortitude_modifier: 0,
        has_chaser: false,
    }
}

fn wine() -> Drink {
    Drink {
        name: String::from("Wine"),
        alcohol_content_modifier: 2,
        fortitude_modifier: 0,
        has_chaser: false,
    }
}

fn dirty_dishwater() -> Drink {
    Drink {
        name: String::from("Dirty Dishwater"),
        alcohol_content_modifier: 0,
        fortitude_modifier: -1,
        has_chaser: false,
    }
}

fn light_ale() -> Drink {
    Drink {
        name: String::from("Light Ale"),
        alcohol_content_modifier: 1,
        fortitude_modifier: 0,
        has_chaser: false,
    }
}

fn dark_ale() -> Drink {
    Drink {
        name: String::from("Dark Ale"),
        alcohol_content_modifier: 1,
        fortitude_modifier: 0,
        has_chaser: false,
    }
}

fn dragon_breath_ale() -> Drink {
    Drink {
        name: String::from("Dragon Breath Ale"),
        alcohol_content_modifier: 4,
        fortitude_modifier: 0,
        has_chaser: false,
    }
}

// TODO - Orcish Rotgut should instead be +2 alcohol if player is an orc.
fn orcish_rotgut() -> Drink {
    Drink {
        name: String::from("Orcish Rotgut"),
        alcohol_content_modifier: 0,
        fortitude_modifier: -2,
        has_chaser: false,
    }
}

fn water() -> Drink {
    Drink {
        name: String::from("Water"),
        alcohol_content_modifier: -1,
        fortitude_modifier: 0,
        has_chaser: false,
    }
}

fn holy_water() -> Drink {
    Drink {
        name: String::from("Holy Water"),
        alcohol_content_modifier: 0,
        fortitude_modifier: 2,
        has_chaser: false,
    }
}

fn with_chaser(mut drink: Drink) -> Drink {
    drink.has_chaser = true;
    drink
}
