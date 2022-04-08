mod deck;
mod drink;
mod error;
mod gambling_manager;
mod game_logic;
mod interrupt_manager;
mod player;
mod player_card;
mod player_manager;
pub mod player_view;
mod uuid;

pub use self::uuid::GameUUID;
pub use self::uuid::PlayerUUID;
pub use error::Error;

use game_logic::GameLogic;
use player_card::{
    change_all_other_player_fortitude_card, change_other_player_fortitude_card,
    combined_interrupt_player_card, gain_fortitude_anytime_card, gambling_cheat_card,
    gambling_im_in_card, i_dont_think_so_card, i_raise_card, ignore_drink_card,
    ignore_root_card_affecting_fortitude, leave_gambling_round_instead_of_anteing_card,
    oh_i_guess_the_wench_thought_that_was_her_tip_card,
    wench_bring_some_drinks_for_my_friends_card, winning_hand_card, PlayerCard,
};
use player_view::{GameView, ListedGameView};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone)]
pub struct Game {
    display_name: String,
    players: Vec<(PlayerUUID, Option<Character>)>,
    // Is `Some` if game is running, otherwise is `None`.
    game_logic_or: Option<GameLogic>,
}

impl Game {
    pub fn new(display_name: String) -> Self {
        Self {
            display_name,
            players: Vec::new(),
            game_logic_or: None,
        }
    }

    pub fn join(&mut self, player_uuid: PlayerUUID) -> Result<(), Error> {
        // TODO - Can't join game when it is already running. Perhaps allow for joining as spectator?
        if self.player_is_in_game(&player_uuid) {
            Err(Error::new("Player is already in this game"))
        } else {
            self.players.push((player_uuid, None));
            Ok(())
        }
    }

    pub fn leave(&mut self, player_uuid: &PlayerUUID) -> Result<(), Error> {
        // TODO - Stop the game if a player leaves while it is running.
        if !self.player_is_in_game(player_uuid) {
            Err(Error::new("Player is not in this game"))
        } else {
            self.players.retain(|(uuid, _)| uuid != player_uuid);
            Ok(())
        }
    }

    pub fn start(&mut self, player_uuid: &PlayerUUID) -> Result<(), Error> {
        if !self.is_owner(player_uuid) {
            return Err(Error::new("Must be game owner to start game"));
        }

        if self.is_running() {
            return Err(Error::new("Game is already running"));
        }

        let players: Vec<(PlayerUUID, Character)> = self
            .players
            .iter()
            .filter_map(|(player_uuid, character_or)| {
                character_or
                    .as_ref()
                    .map(|character| (player_uuid.clone(), *character))
            })
            .collect();
        if players.len() < self.players.len() {
            return Err(Error::new("Not all players have selected a character"));
        }
        let game_logic = match GameLogic::new(players) {
            Ok(game_logic) => game_logic,
            Err(err) => return Err(err),
        };
        self.game_logic_or = Some(game_logic);
        Ok(())
    }

    pub fn select_character(
        &mut self,
        player_uuid: &PlayerUUID,
        character: Character,
    ) -> Result<(), Error> {
        if !self.player_is_in_game(player_uuid) {
            return Err(Error::new("Player is not in this game"));
        }
        if self.is_running() {
            return Err(Error::new("Cannot change characters while game is running"));
        }
        self.players.iter_mut().for_each(|(uuid, character_or)| {
            if uuid == player_uuid {
                *character_or = Some(character);
            }
        });
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    /// Plays a card from the given player's hand.
    ///
    /// Accepts a zero-based card index which refers to a card in the player's hand.
    /// Returns an error if the card cannot currently be played or does not exist with given index or if the player does not exist.
    pub fn play_card(
        &mut self,
        player_uuid: &PlayerUUID,
        other_player_uuid_or: &Option<PlayerUUID>,
        card_index: usize,
    ) -> Result<(), Error> {
        self.get_game_logic_mut()?
            .play_card(player_uuid, other_player_uuid_or, card_index)
    }

    /// Discards any number of cards from the given player's hand.
    ///
    /// The values in `card_indices` represent cards in the player's hand.
    /// This must be called at the beginning of every player's turn.
    /// If the player doesn't want to discard anything, an empty vector
    /// should be passed in for `card_indices`.
    pub fn discard_cards_and_draw_to_full(
        &mut self,
        player_uuid: &PlayerUUID,
        card_indices: Vec<usize>,
    ) -> Result<(), Error> {
        self.get_game_logic_mut()?
            .discard_cards_and_draw_to_full(player_uuid, card_indices)
    }

    /// Order a drink for another player.
    ///
    /// This must be called after the player's action phase is over.
    /// If the player has more than one drink to order, this must
    /// be called repeatedly until all drinks are handed out.
    pub fn order_drink(
        &mut self,
        player_uuid: &PlayerUUID,
        other_player_uuid: &PlayerUUID,
    ) -> Result<(), Error> {
        self.get_game_logic_mut()?
            .order_drink(player_uuid, other_player_uuid)
    }

    fn player_can_pass(&self, player_uuid: &PlayerUUID) -> bool {
        if let Some(game_logic) = &self.game_logic_or {
            game_logic.player_can_pass(player_uuid)
        } else {
            false
        }
    }

    pub fn pass(&mut self, player_uuid: &PlayerUUID) -> Result<(), Error> {
        self.get_game_logic_mut()?.pass(player_uuid)
    }

    pub fn get_game_view(
        &self,
        player_uuid: PlayerUUID,
        player_uuids_to_display_names: &HashMap<PlayerUUID, String>,
    ) -> Result<GameView, Error> {
        Ok(GameView {
            game_name: self.display_name.clone(),
            current_turn_player_uuid: self
                .game_logic_or
                .as_ref()
                .map(|game_logic| game_logic.get_turn_info().get_current_player_turn().clone()),
            current_turn_phase: self
                .game_logic_or
                .as_ref()
                .map(|game_logic| game_logic.get_turn_phase()),
            can_pass: self.player_can_pass(&player_uuid),
            hand: match &self.game_logic_or {
                Some(game_logic) => game_logic.get_game_view_player_hand(&player_uuid),
                None => Vec::new(),
            },
            self_player_uuid: player_uuid,
            player_data: match &self.game_logic_or {
                Some(game_logic) => game_logic.get_game_view_player_data_of_all_players(),
                None => Vec::new(),
            },
            // TODO - Handle this `unwrap`.
            player_display_names: self
                .players
                .iter()
                .map(|(player_uuid, _)| {
                    (
                        player_uuid.clone(),
                        player_uuids_to_display_names
                            .get(player_uuid)
                            .unwrap()
                            .to_string(),
                    )
                })
                .collect(),
            interrupts: match &self.game_logic_or {
                Some(game_logic) => game_logic.get_game_view_interrupt_data_or(),
                None => None,
            },
            drink_event: match &self.game_logic_or {
                Some(game_logic) => game_logic.get_game_view_drink_event_or(),
                None => None,
            },
            is_running: self.is_running(),
            winner_uuid: match &self.game_logic_or {
                Some(game_logic) => game_logic.get_winner_or(),
                None => None,
            },
        })
    }

    pub fn get_listed_game_view(&self, game_uuid: GameUUID) -> ListedGameView {
        ListedGameView {
            game_name: self.display_name.clone(),
            game_uuid,
            player_count: self.players.len(),
        }
    }

    #[cfg(test)]
    fn get_game_logic(&self) -> Option<&GameLogic> {
        self.game_logic_or.as_ref()
    }

    fn get_game_logic_mut(&mut self) -> Result<&mut GameLogic, Error> {
        match &mut self.game_logic_or {
            Some(game_logic) => Ok(game_logic),
            None => Err(Error::new("Game is not currently running")),
        }
    }

    pub fn player_is_in_game(&self, player_uuid: &PlayerUUID) -> bool {
        self.players.iter().any(|(uuid, _)| uuid == player_uuid)
    }

    fn get_owner(&self) -> Option<&PlayerUUID> {
        Some(&self.players.first()?.0)
    }

    fn is_owner(&self, player_uuid: &PlayerUUID) -> bool {
        match self.get_owner() {
            Some(owner_uuid) => owner_uuid == player_uuid,
            None => false,
        }
    }

    fn is_running(&self) -> bool {
        match &self.game_logic_or {
            Some(game_logic) => game_logic.is_running(),
            None => false,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Character {
    Fiona,
    Zot,
    Deirdre,
    Gerki,
}

impl FromStr for Character {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "fiona" => Ok(Self::Fiona),
            "zot" => Ok(Self::Zot),
            "deirdre" => Ok(Self::Deirdre),
            "gerki" => Ok(Self::Gerki),
            _ => Err(String::from("Character does not exist with specified name")),
        }
    }
}

impl<'a> rocket::request::FromParam<'a> for Character {
    type Error = String;
    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Self::from_str(param)
    }
}

impl Character {
    // TODO - Finish implementing entire decks for each character.
    pub fn create_deck(&self) -> Vec<PlayerCard> {
        match self {
            Self::Fiona => vec![
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                i_raise_card().into(),
                i_raise_card().into(),
                change_other_player_fortitude_card(
                    "So then I got the ogre in a headlock like this!",
                    -3,
                )
                .into(),
                change_other_player_fortitude_card("Hey! No more chain mail bikini jokes!", -2)
                    .into(),
                change_other_player_fortitude_card("Hey! No more chain mail bikini jokes!", -2)
                    .into(),
                change_other_player_fortitude_card("Who says I'm not a lady?", -2).into(),
                change_other_player_fortitude_card("It'll hurt more if you do it like this!", -1)
                    .into(),
                change_other_player_fortitude_card("It'll hurt more if you do it like this!", -1)
                    .into(),
                change_other_player_fortitude_card("You wanna arm wrestle?", -1).into(),
                ignore_root_card_affecting_fortitude("Luckily for me, I was wearing my armor!")
                    .into(),
                ignore_root_card_affecting_fortitude("Luckily for me, I was wearing my armor!")
                    .into(),
                gain_fortitude_anytime_card("I'm a quick healer.", 2).into(),
                wench_bring_some_drinks_for_my_friends_card().into(),
                wench_bring_some_drinks_for_my_friends_card().into(),
                oh_i_guess_the_wench_thought_that_was_her_tip_card().into(),
                winning_hand_card().into(),
                winning_hand_card().into(),
                i_dont_think_so_card().into(),
            ],
            Self::Zot => vec![
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                i_raise_card().into(),
                i_raise_card().into(),
                change_other_player_fortitude_card(
                    "How many times have I told you? Keep your hands off my wand!",
                    -2,
                )
                .into(),
                change_other_player_fortitude_card(
                    "How many times have I told you? Keep your hands off my wand!",
                    -2,
                )
                .into(),
                change_other_player_fortitude_card("I told you not to distract me!", -2).into(),
                change_other_player_fortitude_card("Watch out! Don't step on Pooky!", -2).into(),
                change_other_player_fortitude_card("Down Pooky!", -1).into(),
                change_all_other_player_fortitude_card(
                    "Oh no! Not again! Pooky's on a drunken rampage!",
                    -1,
                )
                .into(),
                change_all_other_player_fortitude_card(
                    "Oh no! Not again! Pooky's on a drunken rampage!",
                    -1,
                )
                .into(),
                ignore_root_card_affecting_fortitude("Now you see me... Now you don't!").into(),
                wench_bring_some_drinks_for_my_friends_card().into(),
                wench_bring_some_drinks_for_my_friends_card().into(),
                oh_i_guess_the_wench_thought_that_was_her_tip_card().into(),
                gambling_cheat_card("Pooky! Stop looking at everyone's cards!").into(),
                gambling_cheat_card("Look over there! It's the Lich King!").into(),
                gambling_cheat_card("This time, we'll use my dice.").into(),
                winning_hand_card().into(),
                winning_hand_card().into(),
                i_dont_think_so_card().into(),
                ignore_drink_card("Bad Pooky! Don't drink that!").into(),
                combined_interrupt_player_card(
                    "Not now, I'm meditating.",
                    leave_gambling_round_instead_of_anteing_card(""),
                    ignore_drink_card(""),
                )
                .into(),
            ],
            Self::Deirdre => vec![
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                i_raise_card().into(),
                i_raise_card().into(),
                change_other_player_fortitude_card("My Goddess made me do it!", -2).into(),
                change_other_player_fortitude_card("My Goddess made me do it!", -2).into(),
                change_other_player_fortitude_card("I'm not that kind of priestess!", -2).into(),
                change_other_player_fortitude_card(
                    "Oh no! I think that growth on your arm might be Mummy Rot!",
                    -2,
                )
                .into(),
                change_other_player_fortitude_card(
                    "Sorry, sometimes my healing spells just wear off.",
                    -1,
                )
                .into(),
                ignore_root_card_affecting_fortitude("My Goddess protects me!").into(),
                ignore_root_card_affecting_fortitude("My Goddess protects me!").into(),
                gain_fortitude_anytime_card("My Goddess heals me.", 2).into(),
                gain_fortitude_anytime_card("My Goddess heals me.", 2).into(),
                wench_bring_some_drinks_for_my_friends_card().into(),
                wench_bring_some_drinks_for_my_friends_card().into(),
                oh_i_guess_the_wench_thought_that_was_her_tip_card().into(),
                winning_hand_card().into(),
                winning_hand_card().into(),
                i_dont_think_so_card().into(),
            ],
            Self::Gerki => vec![
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                gambling_im_in_card().into(),
                i_raise_card().into(),
                i_raise_card().into(),
                change_other_player_fortitude_card(
                    "Uh oh! I forgot to disarm one of the traps!",
                    -3,
                )
                .into(),
                change_other_player_fortitude_card(
                    "Have you seen my poison? I left it in a mug right here...",
                    -3,
                )
                .into(),
                change_other_player_fortitude_card(
                    "That's not healing salve! It's contact poison!",
                    -2,
                )
                .into(),
                change_other_player_fortitude_card(
                    "That's not healing salve! It's contact poison!",
                    -2,
                )
                .into(),
                change_other_player_fortitude_card("How did this get stuck in your back?", -2)
                    .into(),
                change_other_player_fortitude_card("How did this get stuck in your back?", -2)
                    .into(),
                ignore_root_card_affecting_fortitude("Hide in shadows").into(),
                wench_bring_some_drinks_for_my_friends_card().into(),
                wench_bring_some_drinks_for_my_friends_card().into(),
                oh_i_guess_the_wench_thought_that_was_her_tip_card().into(),
                gambling_cheat_card("I'm winning... Honestly!").into(),
                gambling_cheat_card("Oops... I dropped my cards...").into(),
                gambling_cheat_card("Five of a kind! Does this mean I win?").into(),
                winning_hand_card().into(),
                winning_hand_card().into(),
                i_dont_think_so_card().into(),
            ],
        }
    }

    pub fn is_orc(&self) -> bool {
        // Currently none of the implemented characters are orcs. This may change later.
        false
    }

    pub fn is_troll(&self) -> bool {
        // Currently none of the implemented characters are trolls. This may change later.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_perform_full_round() {
        // We're running this loop many times to make sure that the test isn't flaky.
        for _ in 1..100 {
            // Setup game with 2 players.
            let mut game = Game::new("Test Game".to_string());
            let player1_uuid = PlayerUUID::new();
            let player2_uuid = PlayerUUID::new();
            assert_eq!(game.join(player1_uuid.clone()), Ok(()));
            assert_eq!(game.join(player2_uuid.clone()), Ok(()));
            assert_eq!(
                game.select_character(&player1_uuid, Character::Deirdre),
                Ok(())
            );
            assert_eq!(
                game.select_character(&player2_uuid, Character::Gerki),
                Ok(())
            );
            assert_eq!(game.start(&player1_uuid), Ok(()));

            pass_until_game_ends_2_player_game(&mut game, &player1_uuid, &player2_uuid);

            // Players can change characters after the game ends.
            assert_eq!(
                game.select_character(&player1_uuid, Character::Deirdre),
                Ok(())
            );
            assert_eq!(game.start(&player1_uuid), Ok(()));

            pass_until_game_ends_2_player_game(&mut game, &player1_uuid, &player2_uuid);
        }
    }

    fn pass_until_game_ends_2_player_game(
        game: &mut Game,
        player1_uuid: &PlayerUUID,
        player2_uuid: &PlayerUUID,
    ) {
        loop {
            if !game.get_game_logic().unwrap().is_running() {
                break;
            }

            assert_eq!(
                game.discard_cards_and_draw_to_full(player1_uuid, Vec::new()),
                Ok(())
            );
            assert_eq!(game.pass(player1_uuid), Ok(()));
            assert_eq!(game.order_drink(player1_uuid, player2_uuid), Ok(()));

            while game.get_game_logic().unwrap().is_running()
                && game
                    .get_game_logic()
                    .unwrap()
                    .get_turn_info()
                    .is_drink_phase()
            {
                if game.player_can_pass(player1_uuid) {
                    game.pass(player1_uuid).unwrap();
                } else if game.player_can_pass(player2_uuid) {
                    game.pass(player2_uuid).unwrap();
                } else {
                    panic!("Neither player can pass");
                }
            }

            if !game.get_game_logic().unwrap().is_running() {
                break;
            }

            assert_eq!(
                game.discard_cards_and_draw_to_full(player2_uuid, Vec::new()),
                Ok(())
            );
            assert_eq!(game.pass(player2_uuid), Ok(()));
            assert_eq!(game.order_drink(player2_uuid, player1_uuid), Ok(()));

            while game.get_game_logic().unwrap().is_running()
                && game
                    .get_game_logic()
                    .unwrap()
                    .get_turn_info()
                    .is_drink_phase()
            {
                if game.player_can_pass(player1_uuid) {
                    game.pass(player1_uuid).unwrap();
                } else if game.player_can_pass(player2_uuid) {
                    game.pass(player2_uuid).unwrap();
                } else {
                    panic!("Neither player can pass");
                }
            }
        }
    }
}
