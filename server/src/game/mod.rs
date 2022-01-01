mod drink;
mod error;
mod game_logic;
mod player;
mod player_card;
pub mod player_view;

pub use error::Error;
pub use player::PlayerUUID;

use game_logic::GameLogic;
use player_view::GameView;
use std::str::FromStr;

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
                    .filter_map(|(player_uuid, character_or)| match character_or {
                        Some(character) => Some((player_uuid.clone(), *character)),
                        None => None,
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
    pub fn play_card(&mut self, player_uuid: &PlayerUUID, card_index: usize) -> Option<Error> {
        let game_logic = match self.get_mut_game_logic() {
            Ok(game_logic) => game_logic,
            Err(err) => return Some(err),
        };
        game_logic.play_card(player_uuid, card_index)
    }

    /// Discards any number of cards from the given player's hand.
    ///
    /// The values in `card_indices` represent cards in the player's hand.
    /// This must be called at the beginning of every player's turn.
    /// If the player doesn't want to discard anything, an empty vector
    /// should be passed in for `card_indices`.
    pub fn discard_cards(&self, player_uuid: &PlayerUUID, card_indices: Vec<i32>) -> Option<Error> {
        // TODO - Implement.
        None
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

    pub fn pass(&self, player_uuid: &PlayerUUID) -> Option<Error> {
        // TODO - Implement.
        None
    }

    pub fn get_game_view(&self, player_uuid: &PlayerUUID) -> Result<GameView, Error> {
        self.get_game_logic()?.get_game_view(player_uuid)
    }

    fn get_game_logic(&self) -> Result<&GameLogic, Error> {
        match &self.game_logic_or {
            Some(game_logic) => Ok(game_logic),
            None => Err(Error::new("Game is not currently running")),
        }
    }

    fn get_mut_game_logic(&mut self) -> Result<&mut GameLogic, Error> {
        match &mut self.game_logic_or {
            Some(game_logic) => Ok(game_logic),
            None => Err(Error::new("Game is not currently running")),
        }
    }

    fn player_is_in_game(&self, player_uuid: &PlayerUUID) -> bool {
        self.players
            .iter()
            .find(|(uuid, _)| uuid == player_uuid)
            .is_some()
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
    fn from_param(param: &'a str) -> Result<Self, String> {
        Ok(Self::from_str(param)?)
    }
}
