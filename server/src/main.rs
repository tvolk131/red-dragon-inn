#[macro_use]
extern crate rocket;

mod game;
mod game_manager;

use game::player_view::GameView;
use game_manager::GameManager;

use rocket::{response::content, State};

#[get("/healthz")]
async fn healthz_handler() -> content::Html<String> {
    content::Html("<html><body><h1>200 OK</h1>Service ready.</body></html>".to_string())
}

#[get("/api/playCard")]
async fn play_card_handler(game_manager: &State<GameManager>) -> GameView {
    game_manager.play_card()
}

#[rocket::launch]
async fn rocket() -> _ {
    rocket::build()
        .manage(GameManager::new())
        .mount("/", routes![healthz_handler, play_card_handler])
}
