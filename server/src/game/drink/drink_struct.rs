use super::super::player::Player;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

type GetStatFn = Arc<dyn Fn(&Player) -> i32 + Send + Sync>;

#[derive(Clone)]
pub struct Drink {
    display_name: String,
    get_alcohol_content_modifier_fn: GetStatFn,
    get_fortitude_modifier_fn: GetStatFn,
    has_chaser: bool,
}

impl Debug for Drink {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_name)
    }
}

impl Drink {
    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn has_chaser(&self) -> bool {
        self.has_chaser
    }

    pub fn get_alcohol_content_modifier(&self, player: &Player) -> i32 {
        (self.get_alcohol_content_modifier_fn)(player)
    }

    pub fn get_fortitude_modifier(&self, player: &Player) -> i32 {
        (self.get_fortitude_modifier_fn)(player)
    }
}

pub fn simple_drink(
    display_name: impl ToString,
    alcohol_content_mod: i32,
    fortitude_mod: i32,
    has_chaser: bool,
) -> Drink {
    Drink {
        display_name: display_name.to_string(),
        get_alcohol_content_modifier_fn: Arc::from(move |_player: &Player| alcohol_content_mod),
        get_fortitude_modifier_fn: Arc::from(move |_player: &Player| fortitude_mod),
        has_chaser,
    }
}

pub fn orcish_rotgut() -> Drink {
    Drink {
        display_name: "Orcish Rotgut".to_string(),
        get_alcohol_content_modifier_fn: Arc::from(
            |player: &Player| {
                if player.is_orc() {
                    2
                } else {
                    0
                }
            },
        ),
        get_fortitude_modifier_fn: Arc::from(
            |player: &Player| {
                if player.is_orc() {
                    0
                } else {
                    -2
                }
            },
        ),
        has_chaser: false,
    }
}

pub fn troll_swill() -> Drink {
    Drink {
        display_name: "Troll Swill".to_string(),
        get_alcohol_content_modifier_fn: Arc::from(
            |player: &Player| {
                if player.is_troll() {
                    2
                } else {
                    1
                }
            },
        ),
        get_fortitude_modifier_fn: Arc::from(
            |player: &Player| {
                if player.is_troll() {
                    0
                } else {
                    -1
                }
            },
        ),
        has_chaser: false,
    }
}
