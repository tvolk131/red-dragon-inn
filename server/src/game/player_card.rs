use super::player::PlayerUUID;
use super::GameLogic;

pub trait PlayerCard {
    fn can_play(&self, player_uuid: PlayerUUID, game: &GameLogic) -> bool;
    fn play(&self, player_uuid: PlayerUUID, game: &mut GameLogic);
}

struct GamblingImInPlayerCard {}

impl PlayerCard for GamblingImInPlayerCard {
    fn can_play(&self, player_uuid: PlayerUUID, game: &GameLogic) -> bool {
        false
    }

    fn play(&self, player_uuid: PlayerUUID, game: &mut GameLogic) {
        if game.gambling_round_in_progress() {
            game.take_control_of_gambling_round();
        } else {
            game.start_gambling_round();
        }
    }
}
