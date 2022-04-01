use super::{game_logic::TurnPhase, GameUUID, PlayerUUID};
use serde::Serialize;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameViewPlayerCard {
    pub card_name: String,
    pub is_playable: bool,
    pub is_directed: bool,
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
pub struct GameViewInterruptData {
    pub interrupts: Vec<GameViewInterruptStack>,
    pub current_interrupt_turn: PlayerUUID,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameViewInterruptStack {
    pub root_item: GameViewInterruptStackRootItem,
    pub interrupt_card_names: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameViewInterruptStackRootItem {
    pub name: String,
    pub item_type: String,
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
    pub interrupts: Option<GameViewInterruptData>,
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

macro_rules! impl_to_json_string_responder {
    ($struct_name:ident, $get_serialized_var:expr) => {
        impl<'r> rocket::response::Responder<'r, 'static> for $struct_name {
            fn respond_to(
                self,
                _request: &'r rocket::request::Request,
            ) -> Result<rocket::response::Response<'static>, rocket::http::Status> {
                let json_string = serde_json::json!($get_serialized_var(self)).to_string();
                rocket::Response::build()
                    .header(rocket::http::ContentType::JSON)
                    .sized_body(json_string.len(), std::io::Cursor::new(json_string))
                    .ok()
            }
        }
    };
}

impl_to_json_string_responder!(
    ListedGameViewCollection,
    |collection: ListedGameViewCollection| collection.listed_game_views
);
impl_to_json_string_responder!(GameView, |game_view: GameView| game_view);
