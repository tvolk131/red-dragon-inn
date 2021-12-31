#[macro_use]
extern crate rocket;

mod auth;
mod game;
mod game_manager;

use auth::SESSION_COOKIE_NAME;
use game::PlayerUUID;
use game::{player_view::GameView, Error};
use game_manager::{GameUUID, GameManager};
use std::sync::RwLock;

use rocket::{
    http::{Cookie, CookieJar},
    response::content,
    State,
};

// TODO - Use JWT to sign cookies. Currently they are completely unsecure.

#[get("/healthz")]
async fn healthz_handler() -> content::Html<String> {
    content::Html("<html><body><h1>200 OK</h1>Service ready.</body></html>".to_string())
}

#[get("/api/signin?<display_name>")]
async fn signin_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
    display_name: String,
) -> Option<Error> {
    if let Ok(_) = PlayerUUID::from_cookie_jar(cookie_jar) {
        return Some(Error::new("User is already signed in"));
    };
    let player_uuid = PlayerUUID::new();
    if let Some(err) = game_manager
        .write()
        .unwrap()
        .add_player(player_uuid.clone(), display_name)
    {
        return Some(err);
    }
    player_uuid.to_cookie_jar(cookie_jar);
    None
}

#[get("/api/signout")]
async fn signout_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
) -> Option<Error> {
    let player_uuid = match PlayerUUID::from_cookie_jar(cookie_jar) {
        Ok(player_uuid) => player_uuid,
        Err(err) => return Some(err),
    };

    if let Some(err) = game_manager.write().unwrap().remove_player(&player_uuid) {
        return Some(err);
    }
    match PlayerUUID::from_cookie_jar(cookie_jar) {
        Ok(_) => {}
        Err(err) => return Some(err),
    };
    cookie_jar.remove(Cookie::named(SESSION_COOKIE_NAME));
    None
}

#[get("/api/createGame/<game_name>")]
async fn create_game_handler(game_manager: &State<RwLock<GameManager>>, cookie_jar: &CookieJar<'_>, game_name: String) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let mut unlocked_game_manager = game_manager.write().unwrap();
    unlocked_game_manager.create_game(player_uuid.clone(), game_name)?;
    unlocked_game_manager.get_game_view(&player_uuid)
}

#[get("/api/joinGame/<game_uuid>")]
async fn join_game_handler(game_manager: &State<RwLock<GameManager>>, cookie_jar: &CookieJar<'_>, game_uuid: GameUUID) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let mut unlocked_game_manager = game_manager.write().unwrap();
    if let Some(err) = unlocked_game_manager.join_game(player_uuid.clone(), game_uuid) {
        return Err(err);
    };
    unlocked_game_manager.get_game_view(&player_uuid)
}

#[get("/api/leaveGame")]
async fn leave_game_handler(game_manager: &State<RwLock<GameManager>>, cookie_jar: &CookieJar<'_>) -> Option<Error> {
    let player_uuid = match PlayerUUID::from_cookie_jar(cookie_jar) {
        Ok(player_uuid) => player_uuid,
        Err(err) => return Some(err)
    };
    let mut unlocked_game_manager = game_manager.write().unwrap();
    unlocked_game_manager.leave_game(&player_uuid)?;
    None
}

#[get("/api/playCard/<card_index>")]
async fn play_card_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
    card_index: usize,
) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let unlocked_game_manager = game_manager.read().unwrap();
    if let Some(err) = unlocked_game_manager.play_card(&player_uuid, card_index) {
        return Err(err);
    }
    unlocked_game_manager.get_game_view(&player_uuid)
}

#[get("/api/discardCards?<card_indices_string>")]
async fn discard_cards_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
    card_indices_string: Option<String>,
) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let unlocked_game_manager = game_manager.read().unwrap();
    if let Some(err) =
        unlocked_game_manager.discard_cards(&player_uuid, parse_i32_vec(card_indices_string)?)
    {
        return Err(err);
    }
    unlocked_game_manager.get_game_view(&player_uuid)
}

#[get("/api/orderDrink/<other_player_uuid>")]
async fn order_drink_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
    other_player_uuid: PlayerUUID,
) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let unlocked_game_manager = game_manager.read().unwrap();
    if let Some(err) = unlocked_game_manager.order_drink(&player_uuid, &other_player_uuid) {
        return Err(err);
    }
    unlocked_game_manager.get_game_view(&player_uuid)
}

#[get("/api/getGameView")]
async fn get_game_view_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    game_manager.read().unwrap().get_game_view(&player_uuid)
}

fn parse_i32_vec(items_string_or: Option<String>) -> Result<Vec<i32>, Error> {
    match items_string_or {
        Some(items_string) => {
            let mut items: Vec<i32> = Vec::new();
            for item_string in items_string.split(',') {
                match item_string.parse::<i32>() {
                    Ok(item) => items.push(item),
                    Err(_) => return Err(Error::new("Unable to parse items")),
                };
            }
            Ok(items)
        }
        None => Ok(Vec::new()),
    }
}

#[rocket::launch]
async fn rocket() -> _ {
    rocket::build()
        .manage(RwLock::from(GameManager::new()))
        .mount(
            "/",
            routes![
                healthz_handler,
                signin_handler,
                signout_handler,
                create_game_handler,
                join_game_handler,
                leave_game_handler,
                play_card_handler,
                discard_cards_handler,
                order_drink_handler,
                get_game_view_handler
            ],
        )
}
