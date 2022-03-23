use super::gambling_manager::GamblingManager;
use super::game_logic::TurnInfo;
use super::interrupt_manager::{GameInterruptType, InterruptManager, PlayerCardInfo};
use super::player_manager::PlayerManager;
use super::uuid::PlayerUUID;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

#[derive(Clone, Debug)]
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

    pub fn can_play(
        &self,
        player_uuid: &PlayerUUID,
        gambling_manager: &GamblingManager,
        interrupt_manager: &InterruptManager,
        turn_info: &TurnInfo,
    ) -> bool {
        match &self {
            Self::RootPlayerCard(root_player_card) => root_player_card.can_play(
                player_uuid,
                gambling_manager,
                interrupt_manager,
                turn_info,
            ),
            Self::InterruptPlayerCard(interrupt_player_card) => {
                let current_interrupt = match interrupt_manager.get_current_interrupt() {
                    Some(current_interrupt) => current_interrupt,
                    None => return false,
                };

                if let GameInterruptType::SometimesCardPlayed(player_card_info) = current_interrupt
                {
                    if player_card_info.is_i_dont_think_so_card
                        && !interrupt_player_card.is_i_dont_think_so_card
                    {
                        return false;
                    }
                }

                interrupt_player_card.can_interrupt(current_interrupt)
                    && interrupt_manager.is_turn_to_interrupt(player_uuid)
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

type PreInterruptPlayFn = Arc<
    dyn Fn(&PlayerUUID, &mut PlayerManager, &mut GamblingManager, &mut TurnInfo) -> ShouldInterrupt
        + Send
        + Sync,
>;

type InterruptPlayFn =
    Arc<dyn Fn(&PlayerUUID, &PlayerUUID, &mut PlayerManager, &mut GamblingManager) + Send + Sync>;

type PostInterruptPlayFn =
    Arc<dyn Fn(&PlayerUUID, &mut PlayerManager, &mut GamblingManager, &mut TurnInfo) + Send + Sync>;

#[derive(Clone)]
pub struct RootPlayerCard {
    display_name: String,
    card_type: RootPlayerCardType,
    target_style: TargetStyle,
    can_play_fn: fn(
        player_uuid: &PlayerUUID,
        gambling_manager: &GamblingManager,
        interrupt_manager: &InterruptManager,
        turn_info: &TurnInfo,
    ) -> bool,
    pre_interrupt_play_fn_or: Option<PreInterruptPlayFn>,
    interrupt_play_fn: InterruptPlayFn,
    interrupt_data_or: Option<RootPlayerCardInterruptData>,
}

impl Debug for RootPlayerCard {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_name)
    }
}

impl RootPlayerCard {
    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn get_target_style(&self) -> TargetStyle {
        self.target_style
    }

    pub fn is_action_card(&self) -> bool {
        match self.card_type {
            RootPlayerCardType::Action => true,
            RootPlayerCardType::ActionGambling => true,
            RootPlayerCardType::Anytime => false,
            RootPlayerCardType::Gambling => false,
            RootPlayerCardType::Cheating => false,
            RootPlayerCardType::Sometimes => false,
        }
    }

    pub fn is_gambling_card(&self) -> bool {
        match self.card_type {
            RootPlayerCardType::Action => false,
            RootPlayerCardType::ActionGambling => true,
            RootPlayerCardType::Anytime => false,
            RootPlayerCardType::Gambling => true,
            RootPlayerCardType::Cheating => false,
            RootPlayerCardType::Sometimes => false,
        }
    }

    pub fn can_play(
        &self,
        player_uuid: &PlayerUUID,
        gambling_manager: &GamblingManager,
        interrupt_manager: &InterruptManager,
        turn_info: &TurnInfo,
    ) -> bool {
        if (self.card_type != RootPlayerCardType::Anytime
            && self.card_type != RootPlayerCardType::Sometimes)
            && interrupt_manager.interrupt_in_progress()
        {
            false
        } else {
            (self.can_play_fn)(player_uuid, gambling_manager, interrupt_manager, turn_info)
        }
    }

    pub fn get_interrupt_data_or(&self) -> Option<&RootPlayerCardInterruptData> {
        self.interrupt_data_or.as_ref()
    }

    pub fn pre_interrupt_play(
        &self,
        player_uuid: &PlayerUUID,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
        turn_info: &mut TurnInfo,
    ) -> ShouldInterrupt {
        if let Some(pre_interrupt_play_fn) = &self.pre_interrupt_play_fn_or {
            (pre_interrupt_play_fn)(player_uuid, player_manager, gambling_manager, turn_info)
        } else {
            ShouldInterrupt::Yes
        }
    }

    pub fn interrupt_play(
        &self,
        player_uuid: &PlayerUUID,
        targeted_player_uuid: &PlayerUUID,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
    ) {
        (self.interrupt_play_fn)(
            player_uuid,
            targeted_player_uuid,
            player_manager,
            gambling_manager,
        )
    }
}

#[derive(Clone, PartialEq)]
pub enum RootPlayerCardType {
    Action,
    ActionGambling,
    Anytime,
    Gambling,
    Cheating,
    Sometimes,
}

pub enum ShouldInterrupt {
    Yes,
    No,
}

#[derive(Clone)]
pub struct RootPlayerCardInterruptData {
    interrupt_type_output: GameInterruptType,
    post_interrupt_play_fn_or: Option<PostInterruptPlayFn>,
}

impl RootPlayerCardInterruptData {
    pub fn get_interrupt_type_output(&self) -> GameInterruptType {
        self.interrupt_type_output
    }

    pub fn post_interrupt_play(
        &self,
        player_uuid: &PlayerUUID,
        player_manager: &mut PlayerManager,
        gambling_manager: &mut GamblingManager,
        turn_info: &mut TurnInfo,
    ) {
        if let Some(post_interrupt_play_fn) = &self.post_interrupt_play_fn_or {
            (post_interrupt_play_fn)(player_uuid, player_manager, gambling_manager, turn_info)
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum TargetStyle {
    SelfPlayer,
    SingleOtherPlayer,
    AllOtherPlayers,
    AllGamblingPlayersIncludingSelf,
}

#[derive(Clone)]
pub struct InterruptPlayerCard {
    display_name: String,
    can_interrupt_fn: fn(GameInterruptType) -> bool,
    interrupt_type_output: GameInterruptType,
    interrupt_fn:
        Arc<dyn Fn(&PlayerUUID, &InterruptManager) -> ShouldCancelPreviousCard + Send + Sync>,
    is_i_dont_think_so_card: bool,
}

impl Debug for InterruptPlayerCard {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_name)
    }
}

impl InterruptPlayerCard {
    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn can_interrupt(&self, current_interrupt: GameInterruptType) -> bool {
        (self.can_interrupt_fn)(current_interrupt)
    }

    pub fn get_interrupt_type_output(&self) -> GameInterruptType {
        self.interrupt_type_output
    }

    pub fn interrupt(
        &self,
        player_uuid: &PlayerUUID,
        interrupt_manager: &mut InterruptManager,
    ) -> ShouldCancelPreviousCard {
        (self.interrupt_fn)(player_uuid, interrupt_manager)
    }
}

pub enum ShouldCancelPreviousCard {
    Negate,
    Ignore,
    No,
}

pub fn gambling_im_in_card() -> RootPlayerCard {
    RootPlayerCard {
        display_name: String::from("Gambling? I'm in!"),
        card_type: RootPlayerCardType::ActionGambling,
        target_style: TargetStyle::AllOtherPlayers,
        can_play_fn: |player_uuid: &PlayerUUID,
                      gambling_manager: &GamblingManager,
                      _interrupt_manager: &InterruptManager,
                      turn_info: &TurnInfo|
         -> bool {
            if gambling_manager.round_in_progress() {
                gambling_manager.is_turn(player_uuid)
                    && !gambling_manager.need_cheating_card_to_take_next_control()
            } else {
                turn_info.can_play_action_card(player_uuid, gambling_manager)
            }
        },
        pre_interrupt_play_fn_or: Some(Arc::from(
            |player_uuid: &PlayerUUID,
             player_manager: &mut PlayerManager,
             gambling_manager: &mut GamblingManager,
             _turn_info: &mut TurnInfo| {
                if gambling_manager.round_in_progress() {
                    gambling_manager.take_control_of_round(player_uuid.clone(), false);
                    ShouldInterrupt::No
                } else {
                    gambling_manager.start_round(player_uuid.clone(), player_manager);
                    ShouldInterrupt::Yes
                }
            },
        )),
        interrupt_play_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             targeted_player_uuid: &PlayerUUID,
             player_manager: &mut PlayerManager,
             gambling_manager: &mut GamblingManager| {
                gambling_manager.ante_up(targeted_player_uuid, player_manager);
            },
        ),
        interrupt_data_or: Some(RootPlayerCardInterruptData {
            interrupt_type_output: GameInterruptType::AboutToAnte,
            post_interrupt_play_fn_or: Some(Arc::from(
                |_player_uuid: &PlayerUUID,
                 player_manager: &mut PlayerManager,
                 gambling_manager: &mut GamblingManager,
                 turn_info: &mut TurnInfo| {
                    gambling_manager.pass(player_manager, turn_info);
                },
            )),
        }),
    }
}

pub fn i_raise_card() -> RootPlayerCard {
    RootPlayerCard {
        display_name: String::from("I raise!"),
        card_type: RootPlayerCardType::Gambling,
        target_style: TargetStyle::AllGamblingPlayersIncludingSelf,
        can_play_fn: |player_uuid: &PlayerUUID,
                      gambling_manager: &GamblingManager,
                      _interrupt_manager: &InterruptManager,
                      _turn_info: &TurnInfo|
         -> bool {
            gambling_manager.is_turn(player_uuid)
                && !gambling_manager.need_cheating_card_to_take_next_control()
        },
        pre_interrupt_play_fn_or: None,
        interrupt_play_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             targeted_player_uuid: &PlayerUUID,
             player_manager: &mut PlayerManager,
             gambling_manager: &mut GamblingManager| {
                gambling_manager.ante_up(targeted_player_uuid, player_manager)
            },
        ),
        interrupt_data_or: Some(RootPlayerCardInterruptData {
            interrupt_type_output: GameInterruptType::AboutToAnte,
            post_interrupt_play_fn_or: Some(Arc::from(
                |player_uuid: &PlayerUUID,
                 _player_manager: &mut PlayerManager,
                 gambling_manager: &mut GamblingManager,
                 _turn_info: &mut TurnInfo| {
                    gambling_manager.take_control_of_round(player_uuid.clone(), false);
                },
            )),
        }),
    }
}

pub fn winning_hand_card() -> RootPlayerCard {
    RootPlayerCard {
        display_name: String::from("Winning Hand!"),
        card_type: RootPlayerCardType::Cheating,
        target_style: TargetStyle::SelfPlayer,
        can_play_fn: |player_uuid: &PlayerUUID,
                      gambling_manager: &GamblingManager,
                      _interrupt_manager: &InterruptManager,
                      _turn_info: &TurnInfo|
         -> bool {
            gambling_manager.is_turn(player_uuid)
                && !gambling_manager.need_cheating_card_to_take_next_control()
        },
        pre_interrupt_play_fn_or: Some(Arc::from(
            move |player_uuid: &PlayerUUID,
                  _player_manager: &mut PlayerManager,
                  gambling_manager: &mut GamblingManager,
                  _turn_info: &mut TurnInfo| {
                gambling_manager.take_control_of_round(player_uuid.clone(), true);
                ShouldInterrupt::No
            },
        )),
        interrupt_play_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             _targeted_player_uuid: &PlayerUUID,
             _player_manager: &mut PlayerManager,
             _gambling_manager: &mut GamblingManager| {},
        ),
        interrupt_data_or: None,
    }
}

pub fn gambling_cheat_card(display_name: impl ToString) -> RootPlayerCard {
    RootPlayerCard {
        display_name: display_name.to_string(),
        card_type: RootPlayerCardType::Cheating,
        target_style: TargetStyle::SelfPlayer,
        can_play_fn: |player_uuid: &PlayerUUID,
                      gambling_manager: &GamblingManager,
                      _interrupt_manager: &InterruptManager,
                      _turn_info: &TurnInfo|
         -> bool { gambling_manager.is_turn(player_uuid) },
        pre_interrupt_play_fn_or: Some(Arc::from(
            move |player_uuid: &PlayerUUID,
                  _player_manager: &mut PlayerManager,
                  gambling_manager: &mut GamblingManager,
                  _turn_info: &mut TurnInfo| {
                gambling_manager.take_control_of_round(player_uuid.clone(), false);
                ShouldInterrupt::No
            },
        )),
        interrupt_play_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             _targeted_player_uuid: &PlayerUUID,
             _player_manager: &mut PlayerManager,
             _gambling_manager: &mut GamblingManager| {},
        ),
        interrupt_data_or: None,
    }
}

pub fn change_other_player_fortitude_card(
    display_name: impl ToString,
    amount: i32,
) -> RootPlayerCard {
    RootPlayerCard {
        display_name: display_name.to_string(),
        card_type: RootPlayerCardType::Action,
        target_style: TargetStyle::SingleOtherPlayer,
        can_play_fn: |player_uuid: &PlayerUUID,
                      gambling_manager: &GamblingManager,
                      _interrupt_manager: &InterruptManager,
                      turn_info: &TurnInfo|
         -> bool {
            turn_info.can_play_action_card(player_uuid, gambling_manager)
        },
        pre_interrupt_play_fn_or: None,
        interrupt_play_fn: Arc::from(
            move |_player_uuid: &PlayerUUID,
                  targeted_player_uuid: &PlayerUUID,
                  player_manager: &mut PlayerManager,
                  _gambling_manager: &mut GamblingManager| {
                if let Some(targeted_player) =
                    player_manager.get_player_by_uuid_mut(targeted_player_uuid)
                {
                    targeted_player.change_fortitude(amount);
                }
            },
        ),
        interrupt_data_or: Some(RootPlayerCardInterruptData {
            interrupt_type_output: GameInterruptType::DirectedActionCardPlayed(PlayerCardInfo {
                affects_fortitude: true,
                is_i_dont_think_so_card: false,
            }),
            post_interrupt_play_fn_or: None,
        }),
    }
}

pub fn ignore_root_card_affecting_fortitude(display_name: impl ToString) -> InterruptPlayerCard {
    InterruptPlayerCard {
        display_name: display_name.to_string(),
        can_interrupt_fn: |current_interrupt| {
            if let GameInterruptType::DirectedActionCardPlayed(player_card_info) = current_interrupt
            {
                player_card_info.affects_fortitude
            } else {
                false
            }
        },
        interrupt_type_output: GameInterruptType::SometimesCardPlayed(PlayerCardInfo {
            affects_fortitude: false,
            is_i_dont_think_so_card: false,
        }),
        interrupt_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             _interrupt_manager: &InterruptManager|
             -> ShouldCancelPreviousCard { ShouldCancelPreviousCard::Ignore },
        ),
        is_i_dont_think_so_card: false,
    }
}

pub fn gain_fortitude_anytime_card(display_name: impl ToString, amount: i32) -> RootPlayerCard {
    RootPlayerCard {
        display_name: display_name.to_string(),
        card_type: RootPlayerCardType::Anytime,
        target_style: TargetStyle::SelfPlayer,
        can_play_fn: |_player_uuid: &PlayerUUID,
                      _gambling_manager: &GamblingManager,
                      _interrupt_manager: &InterruptManager,
                      _turn_info: &TurnInfo|
         -> bool { true },
        pre_interrupt_play_fn_or: Some(Arc::from(
            move |player_uuid: &PlayerUUID,
                  player_manager: &mut PlayerManager,
                  _gambling_manager: &mut GamblingManager,
                  _turn_info: &mut TurnInfo| {
                if let Some(player) = player_manager.get_player_by_uuid_mut(player_uuid) {
                    player.change_fortitude(amount)
                }
                ShouldInterrupt::No
            },
        )),
        interrupt_play_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             _targeted_player_uuid: &PlayerUUID,
             _player_manager: &mut PlayerManager,
             _gambling_manager: &mut GamblingManager| {},
        ),
        interrupt_data_or: None,
    }
}

pub fn wench_bring_some_drinks_for_my_friends_card() -> RootPlayerCard {
    RootPlayerCard {
        display_name: String::from("Wench, bring some drinks for my friends!"),
        card_type: RootPlayerCardType::Sometimes,
        target_style: TargetStyle::SelfPlayer,
        can_play_fn: |player_uuid: &PlayerUUID,
                      _gambling_manager: &GamblingManager,
                      _interrupt_manager: &InterruptManager,
                      turn_info: &TurnInfo|
         -> bool {
            turn_info.get_current_player_turn() == player_uuid && turn_info.is_order_drink_phase()
        },
        pre_interrupt_play_fn_or: Some(Arc::from(
            move |_player_uuid: &PlayerUUID,
                  _player_manager: &mut PlayerManager,
                  _gambling_manager: &mut GamblingManager,
                  turn_info: &mut TurnInfo| {
                turn_info.add_drinks_to_order(2);
                ShouldInterrupt::No
            },
        )),
        interrupt_play_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             _targeted_player_uuid: &PlayerUUID,
             _player_manager: &mut PlayerManager,
             _gambling_manager: &mut GamblingManager| {},
        ),
        interrupt_data_or: Some(RootPlayerCardInterruptData {
            interrupt_type_output: GameInterruptType::SometimesCardPlayed(PlayerCardInfo {
                affects_fortitude: false,
                is_i_dont_think_so_card: false,
            }),
            post_interrupt_play_fn_or: None,
        }),
    }
}

pub fn oh_i_guess_the_wench_thought_that_was_her_tip_card() -> RootPlayerCard {
    RootPlayerCard {
        display_name: String::from("Oh, I guess the Wench thought that was her tip..."),
        card_type: RootPlayerCardType::Sometimes,
        target_style: TargetStyle::SelfPlayer,
        can_play_fn: |_player_uuid: &PlayerUUID,
                      gambling_manager: &GamblingManager,
                      interrupt_manager: &InterruptManager,
                      _turn_info: &TurnInfo|
         -> bool {
            gambling_manager.round_in_progress() && !interrupt_manager.interrupt_in_progress()
        },
        pre_interrupt_play_fn_or: Some(Arc::from(
            move |_player_uuid: &PlayerUUID,
                  _player_manager: &mut PlayerManager,
                  gambling_manager: &mut GamblingManager,
                  turn_info: &mut TurnInfo| {
                gambling_manager.end_round_and_discard_gold(turn_info);
                ShouldInterrupt::No
            },
        )),
        interrupt_play_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             _targeted_player_uuid: &PlayerUUID,
             _player_manager: &mut PlayerManager,
             _gambling_manager: &mut GamblingManager| {},
        ),
        interrupt_data_or: Some(RootPlayerCardInterruptData {
            interrupt_type_output: GameInterruptType::SometimesCardPlayed(PlayerCardInfo {
                affects_fortitude: false,
                is_i_dont_think_so_card: false,
            }),
            post_interrupt_play_fn_or: None,
        }),
    }
}

pub fn i_dont_think_so_card() -> InterruptPlayerCard {
    InterruptPlayerCard {
        display_name: String::from("I don't think so!"),
        can_interrupt_fn: |current_interrupt| {
            matches!(current_interrupt, GameInterruptType::SometimesCardPlayed(_))
        },
        interrupt_type_output: GameInterruptType::SometimesCardPlayed(PlayerCardInfo {
            affects_fortitude: false,
            is_i_dont_think_so_card: true,
        }),
        interrupt_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             _interrupt_manager: &InterruptManager|
             -> ShouldCancelPreviousCard { ShouldCancelPreviousCard::Negate },
        ),
        is_i_dont_think_so_card: true,
    }
}

// TODO - Add this card for all characters other than Zot. I only added the card to Zot's deck when I implemented this function.
pub fn ignore_drink_card(display_name: impl ToString) -> InterruptPlayerCard {
    InterruptPlayerCard {
        display_name: display_name.to_string(),
        can_interrupt_fn: |current_interrupt| {
            matches!(current_interrupt, GameInterruptType::AboutToDrink)
        },
        interrupt_type_output: GameInterruptType::SometimesCardPlayed(PlayerCardInfo {
            affects_fortitude: false,
            is_i_dont_think_so_card: false,
        }),
        interrupt_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             _interrupt_manager: &InterruptManager|
             -> ShouldCancelPreviousCard { ShouldCancelPreviousCard::Ignore },
        ),
        is_i_dont_think_so_card: false,
    }
}
