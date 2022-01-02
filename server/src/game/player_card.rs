use super::player::PlayerUUID;
use super::GameLogic;

pub trait PlayerCard: Send + Sync {
    fn can_play(&self, player_uuid: &PlayerUUID, game: &GameLogic) -> bool;
    fn play(&self, player_uuid: &PlayerUUID, game: &mut GameLogic);
}

struct GamblingImInPlayerCard {}

impl PlayerCard for GamblingImInPlayerCard {
    fn can_play(&self, player_uuid: &PlayerUUID, game: &GameLogic) -> bool {
        if game.gambling_round_in_progress() {
            game.is_gambling_turn(player_uuid)
                && !game.gambling_need_cheating_card_to_take_control()
        } else {
            game.get_current_player_turn() == player_uuid
        }
    }

    fn play(&self, player_uuid: &PlayerUUID, game: &mut GameLogic) {
        if game.gambling_round_in_progress() {
            game.gambling_take_control_of_round(player_uuid.clone(), false);
        } else {
            game.start_gambling_round();
        }
    }
}

struct IRaiseCard {}

impl PlayerCard for IRaiseCard {
    fn can_play(&self, player_uuid: &PlayerUUID, game: &GameLogic) -> bool {
        game.gambling_round_in_progress() && game.is_gambling_turn(player_uuid) && !game.gambling_need_cheating_card_to_take_control()
    }

    fn play(&self, player_uuid: &PlayerUUID, game: &mut GameLogic) {
        game.gambling_ante_up()
    }
}
