use serde::Serialize;
use super::PlayerUUID;
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameViewPlayerCard {
    card_name: String,
    is_playable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameViewPlayerData {
    pub player_uuid: PlayerUUID,
    pub draw_pile_size: usize,
    pub discard_pile_size: usize,
    pub drink_deck_size: usize,
    pub alcohol_content: i32,
    pub fortitude: i32,
    pub gold: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    pub self_player_uuid: PlayerUUID,
    pub hand: Vec<GameViewPlayerCard>,
    pub player_data: Vec<GameViewPlayerData>,
    pub player_display_names: HashMap<PlayerUUID, String>
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
