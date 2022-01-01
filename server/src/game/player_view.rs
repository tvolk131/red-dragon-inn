use serde::Serialize;
use super::PlayerUUID;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameViewPlayerCard {
    card_name: String,
    is_playable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameViewPlayerData {
    player_uuid: PlayerUUID,
    player_display_name: String,
    draw_pile_size: i32,
    discard_pile_size: i32,
    drink_deck_size: i32,
    alcohol_content: i32,
    fortitude: i32,
    gold: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    self_player_uuid: PlayerUUID,
    hand: Vec<GameViewPlayerCard>,
    player_data: Vec<GameViewPlayerData>,
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
