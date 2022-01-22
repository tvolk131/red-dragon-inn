use super::game_interrupt::{GameInterruptType, GameInterrupts, PlayerCardInfo};
use super::uuid::PlayerUUID;
use super::GameLogic;
use std::sync::Arc;

#[derive(Clone)]
pub enum PlayerCard {
    RootPlayerCard(RootPlayerCard),
    InterruptPlayerCard(InterruptPlayerCard),
}

impl PlayerCard {
    pub fn get_display_name(&self) -> &str {
        match &self {
            Self::RootPlayerCard(root_player_card) => root_player_card.get_display_name(),
            Self::InterruptPlayerCard(interrupt_player_card) => {
                interrupt_player_card.get_display_name()
            }
        }
    }

    pub fn can_play(&self, player_uuid: &PlayerUUID, game_logic: &GameLogic) -> bool {
        match &self {
            Self::RootPlayerCard(root_player_card) => {
                root_player_card.can_play(player_uuid, game_logic)
            }
            Self::InterruptPlayerCard(interrupt_player_card) => {
                let current_interrupt = match game_logic.get_current_interrupt() {
                    Some(current_interrupt) => current_interrupt,
                    None => return false,
                };
                interrupt_player_card
                    .get_interrupt_type_input()
                    .variant_eq(current_interrupt)
            }
        }
    }
}

impl From<RootPlayerCard> for PlayerCard {
    fn from(root_player_card: RootPlayerCard) -> PlayerCard {
        PlayerCard::RootPlayerCard(root_player_card)
    }
}

impl From<InterruptPlayerCard> for PlayerCard {
    fn from(interrupt_player_card: InterruptPlayerCard) -> PlayerCard {
        PlayerCard::InterruptPlayerCard(interrupt_player_card)
    }
}

#[derive(Clone)]
pub struct RootPlayerCard {
    display_name: String,
    target_style: TargetStyle,
    can_play_fn: fn(player_uuid: &PlayerUUID, game_logic: &GameLogic) -> bool,
    pre_interrupt_play_fn: Arc<dyn Fn(&PlayerUUID, &mut GameLogic) -> ShouldInterrupt + Send + Sync>,
    interrupt_play_fn: Arc<dyn Fn(&PlayerUUID, &PlayerUUID, &mut GameLogic) + Send + Sync>,
    interrupt_data_or: Option<RootPlayerCardInterruptData>,
}

impl RootPlayerCard {
    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn get_target_style(&self) -> TargetStyle {
        self.target_style
    }

    pub fn can_play(&self, player_uuid: &PlayerUUID, game_logic: &GameLogic) -> bool {
        (self.can_play_fn)(player_uuid, game_logic)
    }

    pub fn get_interrupt_data_or(&self) -> Option<&RootPlayerCardInterruptData> {
        self.interrupt_data_or.as_ref()
    }

    pub fn pre_interrupt_play(
        &self,
        player_uuid: &PlayerUUID,
        game_logic: &mut GameLogic,
    ) -> ShouldInterrupt {
        (self.pre_interrupt_play_fn)(player_uuid, game_logic)
    }

    pub fn interrupt_play(
        &self,
        player_uuid: &PlayerUUID,
        targeted_player_uuid: &PlayerUUID,
        game_logic: &mut GameLogic,
    ) {
        (self.interrupt_play_fn)(player_uuid, targeted_player_uuid, game_logic)
    }
}

#[derive(Clone)]
pub struct RootPlayerCardInterruptData {
    interrupt_style: GameInterruptType,
    post_interrupt_play_fn: Arc<dyn Fn(&PlayerUUID, &mut GameLogic) + Send + Sync>,
}

impl RootPlayerCardInterruptData {
    pub fn get_interrupt_style(&self) -> GameInterruptType {
        self.interrupt_style
    }

    pub fn post_interrupt_play(
        &self,
        player_uuid: &PlayerUUID,
        game_logic: &mut GameLogic,
    ) {
        (self.post_interrupt_play_fn)(player_uuid, game_logic)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum TargetStyle {
    SingleOtherPlayer,
    AllOtherPlayers,
    AllPlayersIncludingSelf,
}

#[derive(Clone)]
pub struct InterruptPlayerCard {
    display_name: String,
    interrupt_type_input: GameInterruptType,
    interrupt_type_output: GameInterruptType,
    interrupt_fn: Arc<dyn Fn(&PlayerUUID, &mut GameInterrupts) + Send + Sync>,
}

impl InterruptPlayerCard {
    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn get_interrupt_type_input(&self) -> GameInterruptType {
        self.interrupt_type_input
    }

    pub fn get_interrupt_type_output(&self) -> GameInterruptType {
        self.interrupt_type_output
    }

    pub fn interrupt(&self, player_uuid: &PlayerUUID, game_interrupts: &mut GameInterrupts) {
        (self.interrupt_fn)(player_uuid, game_interrupts)
    }
}

pub enum ShouldInterrupt {
    Yes,
    No,
}

pub fn gambling_im_in_card() -> RootPlayerCard {
    RootPlayerCard {
        display_name: String::from("Gambling? I'm in!"),
        target_style: TargetStyle::AllPlayersIncludingSelf,
        can_play_fn: |player_uuid: &PlayerUUID, game_logic: &GameLogic| -> bool {
            if game_logic.gambling_round_in_progress() {
                game_logic.is_gambling_turn(player_uuid)
                    && !game_logic.gambling_need_cheating_card_to_take_control()
            } else {
                game_logic.can_play_action_card(player_uuid)
            }
        },
        pre_interrupt_play_fn: Arc::from(|player_uuid: &PlayerUUID, game_logic: &mut GameLogic| {
            if game_logic.gambling_round_in_progress() {
                game_logic.gambling_take_control_of_round(player_uuid.clone(), false);
                ShouldInterrupt::No
            } else {
                game_logic.start_gambling_round(player_uuid.clone());
                ShouldInterrupt::Yes
            }
        }),
        interrupt_play_fn: Arc::from(
            |player_uuid: &PlayerUUID,
             targeted_player_uuid: &PlayerUUID,
             game_logic: &mut GameLogic| {
                game_logic.gambling_ante_up(targeted_player_uuid);
            },
        ),
        interrupt_data_or: Some(RootPlayerCardInterruptData {
            interrupt_style: GameInterruptType::AboutToAnte,
            post_interrupt_play_fn: Arc::from(
                |player_uuid: &PlayerUUID, game_logic: &mut GameLogic| {},
            ),
        }),
    }
}

pub fn i_raise_card() -> RootPlayerCard {
    RootPlayerCard {
        display_name: String::from("I raise!"),
        target_style: TargetStyle::AllPlayersIncludingSelf,
        can_play_fn: |player_uuid: &PlayerUUID, game_logic: &GameLogic| -> bool {
            game_logic.gambling_round_in_progress()
                && game_logic.is_gambling_turn(player_uuid)
                && !game_logic.gambling_need_cheating_card_to_take_control()
        },
        pre_interrupt_play_fn: Arc::from(
            |player_uuid: &PlayerUUID, game_logic: &mut GameLogic| ShouldInterrupt::Yes,
        ),
        interrupt_play_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             targeted_player_uuid: &PlayerUUID,
             game_logic: &mut GameLogic| {
                game_logic.gambling_ante_up(targeted_player_uuid)
            },
        ),
        interrupt_data_or: Some(RootPlayerCardInterruptData {
            interrupt_style: GameInterruptType::AboutToAnte,
            post_interrupt_play_fn: Arc::from(
                |player_uuid: &PlayerUUID, game_logic: &mut GameLogic| {},
            ),
        }),
    }
}

pub fn change_other_player_fortitude(
    display_name: impl ToString,
    amount: i32,
) -> RootPlayerCard {
    RootPlayerCard {
        display_name: display_name.to_string(),
        target_style: TargetStyle::SingleOtherPlayer,
        can_play_fn: |player_uuid: &PlayerUUID, game_logic: &GameLogic| -> bool {
            game_logic.can_play_action_card(player_uuid)
        },
        pre_interrupt_play_fn: Arc::from(
            |player_uuid: &PlayerUUID, game_logic: &mut GameLogic| ShouldInterrupt::Yes,
        ),
        interrupt_play_fn: Arc::from(
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
        interrupt_data_or: Some(RootPlayerCardInterruptData {
            interrupt_style: GameInterruptType::SometimesCardPlayed(PlayerCardInfo {
                affects_fortitude: true,
            }),
            post_interrupt_play_fn: Arc::from(
                |player_uuid: &PlayerUUID, game_logic: &mut GameLogic| {},
            ),
        }),
    }
}
