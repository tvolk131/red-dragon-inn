use super::interrupt_manager::{GameInterruptType, InterruptManager, PlayerCardInfo};
use super::uuid::PlayerUUID;
use std::sync::Arc;
use super::player_manager::PlayerManager;
use super::gambling_manager::GamblingManager;
use super::game_logic::TurnInfo;

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

    pub fn can_play(&self, player_uuid: &PlayerUUID, gambling_manager: &GamblingManager, interrupt_manager: &InterruptManager, turn_info: &TurnInfo) -> bool {
        match &self {
            Self::RootPlayerCard(root_player_card) => {
                root_player_card.can_play(player_uuid, gambling_manager, turn_info)
            }
            Self::InterruptPlayerCard(interrupt_player_card) => {
                let current_interrupt = match interrupt_manager.get_current_interrupt() {
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
    can_play_fn: fn(player_uuid: &PlayerUUID, gambling_manager: &GamblingManager, turn_info: &TurnInfo) -> bool,
    pre_interrupt_play_fn_or: Option<Arc<dyn Fn(&PlayerUUID, &mut PlayerManager, &mut GamblingManager) -> ShouldInterrupt + Send + Sync>>,
    interrupt_play_fn: Arc<dyn Fn(&PlayerUUID, &PlayerUUID, &mut PlayerManager, &mut GamblingManager) + Send + Sync>,
    interrupt_data_or: Option<RootPlayerCardInterruptData>,
}

impl RootPlayerCard {
    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn get_target_style(&self) -> TargetStyle {
        self.target_style
    }

    pub fn can_play(&self, player_uuid: &PlayerUUID, gambling_manager: &GamblingManager, turn_info: &TurnInfo) -> bool {
        (self.can_play_fn)(player_uuid, gambling_manager, turn_info)
    }

    pub fn get_interrupt_data_or(&self) -> Option<&RootPlayerCardInterruptData> {
        self.interrupt_data_or.as_ref()
    }

    pub fn pre_interrupt_play(
        &self,
        player_uuid: &PlayerUUID,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager
    ) -> ShouldInterrupt {
        if let Some(pre_interrupt_play_fn) = &self.pre_interrupt_play_fn_or {
            (pre_interrupt_play_fn)(player_uuid, player_manager, gambling_manager)
        } else {
            ShouldInterrupt::Yes
        }
    }

    pub fn interrupt_play(
        &self,
        player_uuid: &PlayerUUID,
        targeted_player_uuid: &PlayerUUID,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager
    ) {
        (self.interrupt_play_fn)(player_uuid, targeted_player_uuid, player_manager, gambling_manager)
    }
}

pub enum ShouldInterrupt {
    Yes,
    No,
}

#[derive(Clone)]
pub struct RootPlayerCardInterruptData {
    interrupt_style: GameInterruptType,
    post_interrupt_play_fn_or: Option<Arc<dyn Fn(&PlayerUUID, &mut PlayerManager, &mut GamblingManager) + Send + Sync>>,
}

impl RootPlayerCardInterruptData {
    pub fn get_interrupt_style(&self) -> GameInterruptType {
        self.interrupt_style
    }

    pub fn post_interrupt_play(
        &self,
        player_uuid: &PlayerUUID,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager
    ) {
        if let Some(post_interrupt_play_fn) = &self.post_interrupt_play_fn_or {
            (post_interrupt_play_fn)(player_uuid, player_manager, gambling_manager)
        }
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
    interrupt_fn: Arc<dyn Fn(&PlayerUUID, &InterruptManager) -> ShouldCancelPreviousCard + Send + Sync>,
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

    pub fn interrupt(&self, player_uuid: &PlayerUUID, game_interrupts: &mut InterruptManager) -> ShouldCancelPreviousCard {
        (self.interrupt_fn)(player_uuid, game_interrupts)
    }
}

pub enum ShouldCancelPreviousCard {
    Negate,
    Ignore,
    No
}

pub fn gambling_im_in_card() -> RootPlayerCard {
    RootPlayerCard {
        display_name: String::from("Gambling? I'm in!"),
        target_style: TargetStyle::AllPlayersIncludingSelf,
        can_play_fn: |player_uuid: &PlayerUUID, gambling_manager: &GamblingManager, turn_info: &TurnInfo| -> bool {
            if gambling_manager.round_in_progress() {
                gambling_manager.is_turn(player_uuid)
                    && !gambling_manager.need_cheating_card_to_take_next_control()
            } else {
                turn_info.can_play_action_card(player_uuid, gambling_manager)
            }
        },
        pre_interrupt_play_fn_or: Some(Arc::from(|player_uuid: &PlayerUUID, player_manager: &mut PlayerManager, gambling_manager: &mut GamblingManager| {
            if gambling_manager.round_in_progress() {
                gambling_manager.take_control_of_round(player_uuid.clone(), false);
                ShouldInterrupt::No
            } else {
                gambling_manager.start_round(player_uuid.clone(), player_manager);
                ShouldInterrupt::Yes
            }
        })),
        interrupt_play_fn: Arc::from(
            |_player_uuid: &PlayerUUID, targeted_player_uuid: &PlayerUUID, player_manager: &mut PlayerManager, gambling_manager: &mut GamblingManager| {
                gambling_manager.ante_up(targeted_player_uuid, player_manager);
            },
        ),
        interrupt_data_or: Some(RootPlayerCardInterruptData {
            interrupt_style: GameInterruptType::AboutToAnte,
            post_interrupt_play_fn_or: None,
        }),
    }
}

pub fn i_raise_card() -> RootPlayerCard {
    RootPlayerCard {
        display_name: String::from("I raise!"),
        target_style: TargetStyle::AllPlayersIncludingSelf,
        can_play_fn: |player_uuid: &PlayerUUID, gambling_manager: &GamblingManager, _turn_info: &TurnInfo| -> bool {
            gambling_manager.round_in_progress()
                && gambling_manager.is_turn(player_uuid)
                && !gambling_manager.need_cheating_card_to_take_next_control()
        },
        pre_interrupt_play_fn_or: None,
        interrupt_play_fn: Arc::from(
            |_player_uuid: &PlayerUUID, targeted_player_uuid: &PlayerUUID, player_manager: &mut PlayerManager, gambling_manager: &mut GamblingManager| {
                gambling_manager.ante_up(targeted_player_uuid, player_manager)
            },
        ),
        interrupt_data_or: Some(RootPlayerCardInterruptData {
            interrupt_style: GameInterruptType::AboutToAnte,
            post_interrupt_play_fn_or: None,
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
        can_play_fn: |player_uuid: &PlayerUUID, gambling_manager: &GamblingManager, turn_info: &TurnInfo| -> bool {
            turn_info.can_play_action_card(player_uuid, gambling_manager)
        },
        pre_interrupt_play_fn_or: None,
        interrupt_play_fn: Arc::from(
            move |_player_uuid: &PlayerUUID, targeted_player_uuid: &PlayerUUID, player_manager: &mut PlayerManager, _gambling_manager: &mut GamblingManager| {
                if let Some(targeted_player) =
                    player_manager.get_player_by_uuid_mut(targeted_player_uuid)
                {
                    targeted_player.change_fortitude(amount);
                }
            },
        ),
        interrupt_data_or: Some(RootPlayerCardInterruptData {
            interrupt_style: GameInterruptType::SometimesCardPlayed(PlayerCardInfo {
                affects_fortitude: true,
            }),
            post_interrupt_play_fn_or: None,
        }),
    }
}
