use super::player_card::PlayerCard;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameViewPlayerCard {
    card_name: String,
    is_playable: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    hand: Vec<GameViewPlayerCard>,
    alcohol_content: i32,
    fortitude: i32,
    gold: i32,
}

impl GameView {
    // TODO - Remove this dummy function.
    pub fn new() -> Self {
        Self {
            hand: Vec::new(),
            alcohol_content: 0,
            fortitude: 0,
            gold: 0,
        }
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
