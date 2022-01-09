mod deck;
mod drink;
mod error;
mod game_logic;
mod player;
mod player_card;
pub mod player_view;
mod uuid;

pub use self::uuid::GameUUID;
pub use self::uuid::PlayerUUID;
pub use error::Error;

use game_logic::GameLogic;
use player_card::{change_other_player_fortitude, gambling_im_in_card, i_raise_card, PlayerCard};
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

    pub fn join(&mut self, player_uuid: PlayerUUID) -> Option<Error> {
        if self.player_is_in_game(&player_uuid) {
            Some(Error::new("Player is already in this game"))
        } else {
            self.players.push((player_uuid, None));
            None
        }
    }

    pub fn leave(&mut self, player_uuid: &PlayerUUID) -> Option<Error> {
        if !self.player_is_in_game(player_uuid) {
            Some(Error::new("Player is not in this game"))
        } else {
            // TODO - Find out why the clone on this line is necessary.
            self.players = self
                .players
                .clone()
                .into_iter()
                .filter(|(uuid, _)| uuid != player_uuid)
                .collect();
            None
        }
    }

    pub fn start(&mut self, player_uuid: &PlayerUUID) -> Option<Error> {
        if !self.is_owner(player_uuid) {
            return Some(Error::new("Must be game owner to start game"));
        }
        match self.game_logic_or {
            Some(_) => return Some(Error::new("Game is already running")),
            None => {
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
                    return Some(Error::new("Not all players have selected a character"));
                }
                let game_logic = match GameLogic::new(players) {
                    Ok(game_logic) => game_logic,
                    Err(err) => return Some(err),
                };
                self.game_logic_or = Some(game_logic);
            }
        };
        None
    }

    pub fn select_character(
        &mut self,
        player_uuid: &PlayerUUID,
        character: Character,
    ) -> Option<Error> {
        if !self.player_is_in_game(player_uuid) {
            return Some(Error::new("Player is not in this game"));
        }
        if self.game_logic_or.is_some() {
            return Some(Error::new("Cannot change characters while game is running"));
        }
        // TODO - Find out why the clone on this line is necessary.
        self.players = self
            .players
            .clone()
            .into_iter()
            .map(|(uuid, character_or)| {
                if &uuid == player_uuid {
                    (uuid, Some(character))
                } else {
                    (uuid, character_or)
                }
            })
            .collect();
        None
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
    ) -> Option<Error> {
        let game_logic = match self.get_mut_game_logic() {
            Ok(game_logic) => game_logic,
            Err(err) => return Some(err),
        };
        game_logic.play_card(player_uuid, other_player_uuid_or, card_index)
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
    ) -> Option<Error> {
        let game_logic = match self.get_mut_game_logic() {
            Ok(game_logic) => game_logic,
            Err(err) => return Some(err),
        };
        game_logic.discard_cards_and_draw_to_full(player_uuid, card_indices)
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
    ) -> Option<Error> {
        let game_logic = match self.get_mut_game_logic() {
            Ok(game_logic) => game_logic,
            Err(err) => return Some(err),
        };
        game_logic.order_drink(player_uuid, other_player_uuid)
    }

    pub fn pass(&mut self, player_uuid: &PlayerUUID) -> Option<Error> {
        match &mut self.get_mut_game_logic() {
            Ok(game_logic) => {
                if game_logic.can_play_action_card(player_uuid) {
                    game_logic.skip_action_phase();
                    return None;
                }

                if game_logic.is_gambling_turn(player_uuid) {
                    game_logic.gambling_pass();
                    return None;
                }
            }
            Err(_) => {}
        };
        Some(Error::new("Unable to pass at this time"))
    }

    fn player_can_pass(&self, player_uuid: &PlayerUUID) -> bool {
        self.clone().pass(player_uuid).is_none()
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
                .map(|game_logic| game_logic.get_current_player_turn().clone()),
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
                Some(game_logic) => game_logic.get_game_view_player_data(),
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
        })
    }

    pub fn get_listed_game_view(&self, game_uuid: GameUUID) -> ListedGameView {
        ListedGameView {
            game_name: self.display_name.clone(),
            game_uuid,
            player_count: self.players.len(),
        }
    }

    fn get_mut_game_logic(&mut self) -> Result<&mut GameLogic, Error> {
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
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(i_raise_card()),
                PlayerCard::SimplePlayerCard(i_raise_card()),
            ],
            Self::Zot => vec![
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(i_raise_card()),
                PlayerCard::SimplePlayerCard(i_raise_card()),
                PlayerCard::DirectedPlayerCard(change_other_player_fortitude(
                    "How many times have I told you? Keep your hands off my wand!",
                    2,
                )),
                PlayerCard::DirectedPlayerCard(change_other_player_fortitude(
                    "How many times have I told you? Keep your hands off my wand!",
                    2,
                )),
                PlayerCard::DirectedPlayerCard(change_other_player_fortitude(
                    "I told you not to distract me!",
                    2,
                )),
                PlayerCard::DirectedPlayerCard(change_other_player_fortitude(
                    "Watch out! Don't step on Pooky!",
                    2,
                )),
                PlayerCard::DirectedPlayerCard(change_other_player_fortitude("Down Pooky!", 1)),
            ],
            Self::Deirdre => vec![
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(i_raise_card()),
                PlayerCard::SimplePlayerCard(i_raise_card()),
                PlayerCard::DirectedPlayerCard(change_other_player_fortitude(
                    "My Goddess made me do it!",
                    2,
                )),
                PlayerCard::DirectedPlayerCard(change_other_player_fortitude(
                    "My Goddess made me do it!",
                    2,
                )),
                PlayerCard::DirectedPlayerCard(change_other_player_fortitude(
                    "I'm not that kind of priestess!",
                    2,
                )),
                PlayerCard::DirectedPlayerCard(change_other_player_fortitude(
                    "Oh no! I think that growth on your arm might be Mummy Rot!",
                    2,
                )),
                PlayerCard::DirectedPlayerCard(change_other_player_fortitude(
                    "Sorry, sometimes my healing spells just wear off.",
                    1,
                )),
            ],
            Self::Gerki => vec![
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(gambling_im_in_card()),
                PlayerCard::SimplePlayerCard(i_raise_card()),
                PlayerCard::SimplePlayerCard(i_raise_card()),
            ],
        }
    }

    pub fn is_orc(&self) -> bool {
        // Currently none of the implemented characters are orcs. This may change later.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_perform_full_round() {
        // Setup game with 2 players.
        let mut game = Game::new("Test Game".to_string());
        let player1_uuid = PlayerUUID::new();
        let player2_uuid = PlayerUUID::new();
        assert_eq!(game.join(player1_uuid.clone()), None);
        assert_eq!(game.join(player2_uuid.clone()), None);
        assert_eq!(
            game.select_character(&player1_uuid, Character::Deirdre),
            None
        );
        assert_eq!(game.select_character(&player2_uuid, Character::Gerki), None);
        assert_eq!(game.start(&player1_uuid), None);

        for _ in 1..10 {
            assert_eq!(
                game.discard_cards_and_draw_to_full(&player1_uuid, Vec::new()),
                None
            );
            assert_eq!(game.pass(&player1_uuid), None);
            assert_eq!(game.order_drink(&player1_uuid, &player2_uuid), None);

            assert_eq!(
                game.discard_cards_and_draw_to_full(&player2_uuid, Vec::new()),
                None
            );
            assert_eq!(game.pass(&player2_uuid), None);
            assert_eq!(game.order_drink(&player2_uuid, &player2_uuid), None);
        }
    }
}
