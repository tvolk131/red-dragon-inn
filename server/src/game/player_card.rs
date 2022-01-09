use super::uuid::PlayerUUID;
use super::GameLogic;
use std::sync::Arc;

#[derive(Clone)]
pub enum PlayerCard {
    SimplePlayerCard(SimplePlayerCard),
    DirectedPlayerCard(DirectedPlayerCard),
}

impl PlayerCard {
    pub fn get_display_name(&self) -> &str {
        match &self {
            Self::SimplePlayerCard(simple_player_card) => simple_player_card.get_display_name(),
            Self::DirectedPlayerCard(directed_player_card) => {
                directed_player_card.get_display_name()
            }
        }
    }
    pub fn can_play(&self, player_uuid: &PlayerUUID, game: &GameLogic) -> bool {
        match &self {
            Self::SimplePlayerCard(simple_player_card) => {
                simple_player_card.can_play(player_uuid, game)
            }
            Self::DirectedPlayerCard(directed_player_card) => {
                directed_player_card.can_play(player_uuid, game)
            }
        }
    }
}

#[derive(Clone)]
pub struct SimplePlayerCard {
    display_name: String,
    can_play_fn: fn(player_uuid: &PlayerUUID, game_logic: &GameLogic) -> bool,
    play_fn: Arc<dyn Fn(&PlayerUUID, &mut GameLogic) + Send + Sync>,
}

impl SimplePlayerCard {
    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn can_play(&self, player_uuid: &PlayerUUID, game: &GameLogic) -> bool {
        (self.can_play_fn)(player_uuid, game)
    }

    pub fn play(&self, player_uuid: &PlayerUUID, game_logic: &mut GameLogic) {
        (self.play_fn)(player_uuid, game_logic)
    }
}

#[derive(Clone)]
pub struct DirectedPlayerCard {
    display_name: String,
    can_play_fn: fn(player_uuid: &PlayerUUID, game_logic: &GameLogic) -> bool,
    play_fn: Arc<dyn Fn(&PlayerUUID, &PlayerUUID, &mut GameLogic) + Send + Sync>,
}

impl DirectedPlayerCard {
    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn can_play(&self, player_uuid: &PlayerUUID, game: &GameLogic) -> bool {
        (self.can_play_fn)(player_uuid, game)
    }

    pub fn play(
        &self,
        player_uuid: &PlayerUUID,
        targeted_player_uuid: &PlayerUUID,
        game_logic: &mut GameLogic,
    ) {
        (self.play_fn)(player_uuid, targeted_player_uuid, game_logic)
    }
}

pub fn gambling_im_in_card() -> SimplePlayerCard {
    SimplePlayerCard {
        display_name: String::from("Gambling? I'm in!"),
        can_play_fn: |player_uuid: &PlayerUUID, game_logic: &GameLogic| -> bool {
            if game_logic.gambling_round_in_progress() {
                game_logic.is_gambling_turn(player_uuid)
                    && !game_logic.gambling_need_cheating_card_to_take_control()
            } else {
                game_logic.can_play_action_card(player_uuid)
            }
        },
        play_fn: Arc::from(|player_uuid: &PlayerUUID, game_logic: &mut GameLogic| {
            if game_logic.gambling_round_in_progress() {
                game_logic.gambling_take_control_of_round(player_uuid.clone(), false);
            } else {
                game_logic.start_gambling_round(player_uuid.clone());
            }
        }),
    }
}

pub fn i_raise_card() -> SimplePlayerCard {
    SimplePlayerCard {
        display_name: String::from("Gambling? I'm in!"),
        can_play_fn: |player_uuid: &PlayerUUID, game_logic: &GameLogic| -> bool {
            game_logic.gambling_round_in_progress()
                && game_logic.is_gambling_turn(player_uuid)
                && !game_logic.gambling_need_cheating_card_to_take_control()
        },
        play_fn: Arc::from(|_player_uuid: &PlayerUUID, game_logic: &mut GameLogic| {
            game_logic.gambling_ante_up()
        }),
    }
}

pub fn change_other_player_fortitude(
    display_name: impl ToString,
    amount: i32,
) -> DirectedPlayerCard {
    DirectedPlayerCard {
        display_name: display_name.to_string(),
        can_play_fn: |player_uuid: &PlayerUUID, game_logic: &GameLogic| -> bool {
            game_logic.can_play_action_card(player_uuid)
        },
        play_fn: Arc::from(
            move |_player_uuid: &PlayerUUID,
                  targeted_player_uuid: &PlayerUUID,
                  game_logic: &mut GameLogic| {
                if let Some(targeted_player) =
                    game_logic.get_player_by_uuid_mut(targeted_player_uuid)
                {
                    targeted_player.change_fortitude(amount);
                }
            },
        ),
    }
}
