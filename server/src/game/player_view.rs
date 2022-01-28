use super::{game_logic::TurnPhase, GameUUID, PlayerUUID};
use serde::Serialize;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameViewPlayerCard {
    pub card_name: String,
    pub is_playable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameViewPlayerData {
    pub player_uuid: PlayerUUID,
    pub draw_pile_size: usize,
    pub discard_pile_size: usize,
    pub drink_me_pile_size: usize,
    pub alcohol_content: i32,
    pub fortitude: i32,
    pub gold: i32,
    pub is_dead: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    pub game_name: String,
    pub self_player_uuid: PlayerUUID,
    pub current_turn_player_uuid: Option<PlayerUUID>,
    pub current_turn_phase: Option<TurnPhase>,
    pub can_pass: bool,
    pub hand: Vec<GameViewPlayerCard>,
    pub player_data: Vec<GameViewPlayerData>,
    pub player_display_names: HashMap<PlayerUUID, String>,
}

#[derive(Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListedGameView {
    pub game_name: String,
    pub game_uuid: GameUUID,
    pub player_count: usize,
}

pub struct ListedGameViewCollection {
    pub listed_game_views: Vec<ListedGameView>,
}

impl PartialOrd for ListedGameView {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.game_name.partial_cmp(&other.game_name)
    }
}

impl Ord for ListedGameView {
    fn cmp(&self, other: &Self) -> Ordering {
        self.game_name.cmp(&other.game_name)
    }
}

// TODO - Abstract this into a procedural macro along with all other Responder impl blocks in other structs (if there are any).
impl<'r> rocket::response::Responder<'r, 'static> for ListedGameViewCollection {
    fn respond_to(
        self,
        _request: &'r rocket::request::Request,
    ) -> Result<rocket::response::Response<'static>, rocket::http::Status> {
        let json_string = serde_json::json!(self.listed_game_views).to_string();
        rocket::Response::build()
            .header(rocket::http::ContentType::JSON)
            .sized_body(json_string.len(), std::io::Cursor::new(json_string))
            .ok()
    }
}

// TODO - Abstract this into a procedural macro along with all other Responder impl blocks in other structs (if there are any).
impl<'r> rocket::response::Responder<'r, 'static> for GameView {
    fn respond_to(
        self,
        _request: &'r rocket::request::Request,
    ) -> Result<rocket::response::Response<'static>, rocket::http::Status> {
        let json_string = serde_json::json!(self).to_string();
        rocket::Response::build()
            .header(rocket::http::ContentType::JSON)
            .sized_body(json_string.len(), std::io::Cursor::new(json_string))
            .ok()
    }
}
