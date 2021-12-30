#[macro_use]
extern crate rocket;

mod game;
mod game_manager;

use game::{player_view::GameView, Error};
use game_manager::GameManager;

use rocket::{response::content, State};

#[get("/healthz")]
async fn healthz_handler() -> content::Html<String> {
    content::Html("<html><body><h1>200 OK</h1>Service ready.</body></html>".to_string())
}

#[get("/api/playCard")]
async fn play_card_handler(game_manager: &State<GameManager>) -> Result<GameView, Error> {
    if let Some(err) = game_manager.play_card() {
        return Err(err);
    }
    game_manager.get_game_view()
}

#[get("/api/discardCards")]
async fn discard_cards_handler(game_manager: &State<GameManager>) -> Result<GameView, Error> {
    if let Some(err) = game_manager.discard_cards() {
        return Err(err);
    }
    game_manager.get_game_view()
}

#[get("/api/orderDrink")]
async fn order_drink_handler(game_manager: &State<GameManager>) -> Result<GameView, Error> {
    if let Some(err) = game_manager.order_drink() {
        return Err(err);
    }
    game_manager.get_game_view()
}

#[get("/api/getGameView")]
async fn get_game_view_handler(game_manager: &State<GameManager>) -> Result<GameView, Error> {
    game_manager.get_game_view()
}

#[rocket::launch]
async fn rocket() -> _ {
    rocket::build().manage(GameManager::new()).mount(
        "/",
        routes![
            healthz_handler,
            play_card_handler,
            discard_cards_handler,
            order_drink_handler,
            get_game_view_handler
        ],
    )
}
