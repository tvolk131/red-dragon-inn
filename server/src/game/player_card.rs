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

    pub fn get_display_description(&self) -> &str {
        match &self {
            Self::RootPlayerCard(root_player_card) => root_player_card.get_display_description(),
            Self::InterruptPlayerCard(interrupt_player_card) => {
                interrupt_player_card.get_display_description()
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
    display_description: String,
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

    pub fn get_display_description(&self) -> &str {
        &self.display_description
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
    display_description: String,
    can_interrupt_fn: Arc<dyn Fn(GameInterruptType) -> bool + Send + Sync>,
    interrupt_type_output: GameInterruptType,
    interrupt_fn: Arc<
        dyn Fn(&PlayerUUID, &InterruptManager, &mut GamblingManager) -> ShouldCancelPreviousCard
            + Send
            + Sync,
    >,
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

    pub fn get_display_description(&self) -> &str {
        &self.display_description
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
        interrupt_manager: &InterruptManager,
        gambling_manager: &mut GamblingManager,
    ) -> ShouldCancelPreviousCard {
        (self.interrupt_fn)(player_uuid, interrupt_manager, gambling_manager)
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
        display_description: String::from("Start a Round of Gambling. (Each player, including you, must ante.)\n- OR -\nTake control of a Round of Gambling."),
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
                |player_uuid: &PlayerUUID,
                 player_manager: &mut PlayerManager,
                 gambling_manager: &mut GamblingManager,
                 turn_info: &mut TurnInfo| {
                    if gambling_manager.is_turn(player_uuid) {
                        gambling_manager.pass(player_manager, turn_info);
                    }
                },
            )),
        }),
    }
}

pub fn i_raise_card() -> RootPlayerCard {
    RootPlayerCard {
        display_name: String::from("I raise!"),
        display_description: String::from(
            "Take control of a Round of Gambling.\nEach player (including you) must ante again.",
        ),
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
        display_description: String::from("Take control of a Round of Gambling.\nThe next card to take control must be a Cheating Card."),
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
        display_description: String::from("Take control of a Round of Gambling."),
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

fn get_change_other_player_fortitude_card_description(amount: i32) -> String {
    let modifier = if amount > 0 {
        format!("gain {}", amount)
    } else {
        format!("lose {}", -amount)
    };

    format!("Pick another player. They {} Fortitude.", modifier)
}

pub fn change_other_player_fortitude_card(
    display_name: impl ToString,
    amount: i32,
) -> RootPlayerCard {
    RootPlayerCard {
        display_name: display_name.to_string(),
        display_description: get_change_other_player_fortitude_card_description(amount),
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

fn get_change_all_other_player_fortitude_card_description(amount: i32) -> String {
    let modifier = if amount > 0 {
        format!("gains {}", amount)
    } else {
        format!("loses {}", -amount)
    };

    format!("Each other player {} Fortitude.", modifier)
}

// TODO - Add this card for all characters other than Zot. I only added the card to Zot's deck when I implemented this function.
pub fn change_all_other_player_fortitude_card(
    display_name: impl ToString,
    amount: i32,
) -> RootPlayerCard {
    RootPlayerCard {
        display_name: display_name.to_string(),
        display_description: get_change_all_other_player_fortitude_card_description(amount),
        card_type: RootPlayerCardType::Action,
        target_style: TargetStyle::AllOtherPlayers,
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
        display_description: String::from(
            "Ignore an Action or Sometimes Card that affects your Fortitude.",
        ),
        can_interrupt_fn: Arc::from(|current_interrupt| {
            if let GameInterruptType::DirectedActionCardPlayed(player_card_info) = current_interrupt
            {
                player_card_info.affects_fortitude
            } else {
                false
            }
        }),
        interrupt_type_output: GameInterruptType::SometimesCardPlayed(PlayerCardInfo {
            affects_fortitude: false,
            is_i_dont_think_so_card: false,
        }),
        interrupt_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             _interrupt_manager: &InterruptManager,
             _gambling_manager: &mut GamblingManager|
             -> ShouldCancelPreviousCard { ShouldCancelPreviousCard::Ignore },
        ),
        is_i_dont_think_so_card: false,
    }
}

pub fn gain_fortitude_anytime_card(display_name: impl ToString, amount: i32) -> RootPlayerCard {
    RootPlayerCard {
        display_name: display_name.to_string(),
        display_description: format!("Gain {} Fortitude.", amount),
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
        display_description: String::from("You may play this card during the Order a Drink Phase of your turn.\nPay 1 Gold to the Inn. Order 2 additional Drinks. (Drinks you order may be placed on any other players' Drink Me! Piles.)"),
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
        display_description: String::from("You may play this card at any time during a Round of Gambling, even if you have left the Round. You may not play this card if the Round has already ended. You may not play it in response to a card that would make players ante or would end the Round when it resolves.\nThe Round of Gambling ends immediately. All anted Gold goes to the Inn."),
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
        display_description: String::from("Negate a Sometimes Card.\nThis card can only be affected by another I don't think so !"),
        can_interrupt_fn: Arc::from(|current_interrupt| {
            matches!(current_interrupt, GameInterruptType::SometimesCardPlayed(_))
        }),
        interrupt_type_output: GameInterruptType::SometimesCardPlayed(PlayerCardInfo {
            affects_fortitude: false,
            is_i_dont_think_so_card: true,
        }),
        interrupt_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             _interrupt_manager: &InterruptManager,
             _gambling_manager: &mut GamblingManager|
             -> ShouldCancelPreviousCard { ShouldCancelPreviousCard::Negate },
        ),
        is_i_dont_think_so_card: true,
    }
}

// TODO - Add this card for all characters other than Zot. I only added the card to Zot's deck when I implemented this function.
pub fn ignore_drink_card(display_name: impl ToString) -> InterruptPlayerCard {
    InterruptPlayerCard {
        display_name: display_name.to_string(),
        display_description: String::from("Ignore a Drink.\n(Reveal the Drink first!)"),
        can_interrupt_fn: Arc::from(|current_interrupt| {
            matches!(current_interrupt, GameInterruptType::AboutToDrink)
        }),
        interrupt_type_output: GameInterruptType::SometimesCardPlayed(PlayerCardInfo {
            affects_fortitude: false,
            is_i_dont_think_so_card: false,
        }),
        interrupt_fn: Arc::from(
            |_player_uuid: &PlayerUUID,
             _interrupt_manager: &InterruptManager,
             _gambling_manager: &mut GamblingManager|
             -> ShouldCancelPreviousCard { ShouldCancelPreviousCard::Ignore },
        ),
        is_i_dont_think_so_card: false,
    }
}

pub fn leave_gambling_round_instead_of_anteing_card(
    display_name: impl ToString,
) -> InterruptPlayerCard {
    InterruptPlayerCard {
        display_name: display_name.to_string(),
        display_description: String::from("You may play this card when you must ante. Instead of anteing, you leave the Round of Gambling."),
        can_interrupt_fn: Arc::from(|current_interrupt| {
            matches!(current_interrupt, GameInterruptType::AboutToAnte)
        }),
        interrupt_type_output: GameInterruptType::SometimesCardPlayed(PlayerCardInfo {
            affects_fortitude: false,
            is_i_dont_think_so_card: false,
        }),
        interrupt_fn: Arc::from(
            |player_uuid: &PlayerUUID,
             _interrupt_manager: &InterruptManager,
             gambling_manager: &mut GamblingManager|
             -> ShouldCancelPreviousCard {
                // TODO - Handle this unwrap.
                gambling_manager.leave_gambling_round(player_uuid).unwrap();
                ShouldCancelPreviousCard::No
            },
        ),
        is_i_dont_think_so_card: false,
    }
}

// TODO - Come up with a better solution for combining/composing card functionality. This was quick and easy, but it has a few downsides...
// 1. If the two cards being combined have different values set for `interrupt_type_output`, this will lead to weird behavior. Right now the first card's `interrupt_type_output` will be used and the second card's will be ignored.
// 2. Overall this is a bit messy and hard to test & maintain.
//
// When this refactor is done, we can convert the type of `can_interrupt_fn` from `Arc<dyn Fn(GameInterruptType) -> bool + Send + Sync>` back to `fn(GameInterruptType) -> bool`.
pub fn combined_interrupt_player_card(
    display_name: impl ToString,
    first_interrupt_player_card: InterruptPlayerCard,
    second_interrupt_player_card: InterruptPlayerCard,
) -> InterruptPlayerCard {
    let interrupt_type_output = first_interrupt_player_card.interrupt_type_output;
    let first_interrupt_player_card_clone = first_interrupt_player_card.clone();
    let second_interrupt_player_card_clone = second_interrupt_player_card.clone();

    InterruptPlayerCard {
        display_name: display_name.to_string(),
        display_description: format!(
            "{}\n- OR -\n{}",
            first_interrupt_player_card.display_description,
            second_interrupt_player_card.display_description
        ),
        can_interrupt_fn: Arc::from(move |current_interrupt| {
            first_interrupt_player_card.can_interrupt(current_interrupt)
                || second_interrupt_player_card.can_interrupt(current_interrupt)
        }),
        interrupt_type_output,
        interrupt_fn: Arc::from(
            move |player_uuid: &PlayerUUID,
                  interrupt_manager: &InterruptManager,
                  gambling_manager: &mut GamblingManager|
                  -> ShouldCancelPreviousCard {
                if let Some(current_interrupt) = interrupt_manager.get_current_interrupt() {
                    if first_interrupt_player_card_clone.can_interrupt(current_interrupt) {
                        first_interrupt_player_card_clone.interrupt(
                            player_uuid,
                            interrupt_manager,
                            gambling_manager,
                        )
                    } else if second_interrupt_player_card_clone.can_interrupt(current_interrupt) {
                        second_interrupt_player_card_clone.interrupt(
                            player_uuid,
                            interrupt_manager,
                            gambling_manager,
                        )
                    } else {
                        ShouldCancelPreviousCard::No
                    }
                } else {
                    ShouldCancelPreviousCard::No
                }
            },
        ),
        is_i_dont_think_so_card: false,
    }
}
