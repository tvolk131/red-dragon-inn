mod drink;
mod player;
mod player_card;

use player::Player;

pub struct Game {
    players: Vec<Player>,
    gambling_round_or: Option<GamblingRound>,
}

impl Game {
    pub fn gambling_round_in_progress(&self) -> bool {
        self.gambling_round_or.is_some()
    }

    pub fn start_gambling_round(&mut self) {
        if self.gambling_round_or.is_none() {
            self.gambling_round_or = Some(GamblingRound::new());
        }
    }

    pub fn take_control_of_gambling_round(&self) {}

    pub fn gambling_ante_up(&self) {}
}

struct GamblingRound {
    active_players: Vec<&Player>,
    pot_amount: i32
}

impl GamblingRound {
    fn new() -> Self {
        Self {}
    }
}
