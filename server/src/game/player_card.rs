use super::uuid::PlayerUUID;
use super::GameLogic;

pub enum PlayerCard {
    SimplePlayerCard(Box<dyn SimplePlayerCard>),
    DirectedPlayerCard(Box<dyn DirectedPlayerCard>)
}

impl PlayerCard {
    pub fn as_generic_player_card(&self) -> &dyn GenericPlayerCard {
        match &self {
            Self::SimplePlayerCard(simple_player_card) => simple_player_card.as_generic_player_card(),
            Self::DirectedPlayerCard(directed_player_card) => directed_player_card.as_generic_player_card()
        }
    }
}

pub trait GenericPlayerCard: Send + Sync {
    fn get_display_name(&self) -> String;
    fn can_play(&self, player_uuid: &PlayerUUID, game: &GameLogic) -> bool;
    fn as_generic_player_card(&self) -> &dyn GenericPlayerCard;
}

pub trait SimplePlayerCard: GenericPlayerCard {
    fn play(&self, player_uuid: &PlayerUUID, game: &mut GameLogic);
}

pub trait DirectedPlayerCard: GenericPlayerCard {
    fn play(&self, player_uuid: &PlayerUUID, targeted_player_uuid: &PlayerUUID, game: &mut GameLogic);
}

pub struct GamblingImInPlayerCard {}

impl GenericPlayerCard for GamblingImInPlayerCard {
    fn get_display_name(&self) -> String {
        String::from("Gambling? I'm in!")
    }

    fn can_play(&self, player_uuid: &PlayerUUID, game: &GameLogic) -> bool {
        if game.gambling_round_in_progress() {
            game.is_gambling_turn(player_uuid)
                && !game.gambling_need_cheating_card_to_take_control()
        } else {
            game.get_current_player_turn() == player_uuid
        }
    }

    fn as_generic_player_card(&self) -> &dyn GenericPlayerCard {
        self
    }
}

impl SimplePlayerCard for GamblingImInPlayerCard {
    fn play(&self, player_uuid: &PlayerUUID, game: &mut GameLogic) {
        if game.gambling_round_in_progress() {
            game.gambling_take_control_of_round(player_uuid.clone(), false);
        } else {
            game.start_gambling_round();
        }
    }
}

pub struct IRaiseCard {}

impl GenericPlayerCard for IRaiseCard {
    fn get_display_name(&self) -> String {
        String::from("I raise!")
    }

    fn can_play(&self, player_uuid: &PlayerUUID, game: &GameLogic) -> bool {
        game.gambling_round_in_progress() && game.is_gambling_turn(player_uuid) && !game.gambling_need_cheating_card_to_take_control()
    }

    fn as_generic_player_card(&self) -> &dyn GenericPlayerCard {
        self
    }
}

impl SimplePlayerCard for IRaiseCard {
    fn play(&self, _: &PlayerUUID, game: &mut GameLogic) {
        game.gambling_ante_up()
    }
}

pub struct ChangeOtherPlayerFortitude {
    display_name: String,
    fortitude_modifier: i32
}

impl GenericPlayerCard for ChangeOtherPlayerFortitude {
    fn get_display_name(&self) -> String {
        self.display_name.clone()
    }

    fn can_play(&self, player_uuid: &PlayerUUID, game: &GameLogic) -> bool {
        game.can_play_action_card(player_uuid)
    }

    fn as_generic_player_card(&self) -> &dyn GenericPlayerCard {
        self
    }
}

impl DirectedPlayerCard for ChangeOtherPlayerFortitude {
    fn play(&self, player_uuid: &PlayerUUID, targeted_player_uuid: &PlayerUUID, game: &mut GameLogic) {
        if let Some(targeted_player) = game.get_player_by_uuid_mut(targeted_player_uuid) {
            targeted_player.change_fortitude(self.fortitude_modifier);
        }
    }
}

pub struct ChangeSimplePlayerStats {
    display_name: String,
    alcohol_content_modifier: i32,
    fortitude_modifier: i32
}

impl GenericPlayerCard for ChangeSimplePlayerStats {
    fn get_display_name(&self) -> String {
        self.display_name.clone()
    }

    fn can_play(&self, player_uuid: &PlayerUUID, game: &GameLogic) -> bool {
        game.can_play_action_card(player_uuid)
    }

    fn as_generic_player_card(&self) -> &dyn GenericPlayerCard {
        self
    }
}

impl SimplePlayerCard for ChangeSimplePlayerStats {
    fn play(&self, player_uuid: &PlayerUUID, game: &mut GameLogic) {
        if let Some(player) = game.get_player_by_uuid_mut(player_uuid) {
            player.change_alcohol_content(self.alcohol_content_modifier);
            player.change_fortitude(self.fortitude_modifier);
        }
    }
}
