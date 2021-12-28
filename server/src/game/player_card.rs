use super::Game;

pub trait PlayerCard {
    fn can_play(&self, game: &Game) -> bool;
    fn play(&self, game: &mut Game);
}

struct GamblingImInPlayerCard {}

impl PlayerCard for GamblingImInPlayerCard {
    fn can_play(&self, game: &Game) -> bool {
        false
    }

    fn play(&self, game: &mut Game) {
        if game.gambling_round_in_progress() {
            game.take_control_of_gambling_round();
        } else {
            game.start_gambling_round();
        }
    }
}
