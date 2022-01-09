#[macro_use]
extern crate rocket;

mod auth;
mod game;
mod game_manager;

use auth::SESSION_COOKIE_NAME;
use game::{
    player_view::{GameView, ListedGameViewCollection},
    Character, Error, GameUUID, PlayerUUID,
};
use game_manager::GameManager;
use std::sync::RwLock;

use rocket::{
    http::{Cookie, CookieJar},
    response::{content, status},
    Request, State,
};

const FAVICON_BYTES: &[u8] = include_bytes!("../../client/out/favicon.ico");
const HTML_BYTES: &[u8] = include_bytes!("../../client/out/index.html");
const JS_BUNDLE_BYTES: &[u8] = include_bytes!("../../client/out/bundle.js");

// TODO - Use JWT to sign cookies. Currently they are completely unsecure.

enum NotFoundResponse {
    Html(status::Custom<content::Html<&'static [u8]>>),
    JavaScript(status::Custom<content::JavaScript<&'static [u8]>>),
    Favicon(Box<status::Custom<content::Custom<&'static [u8]>>>),
    NotFound(status::NotFound<String>),
}

impl<'r> rocket::response::Responder<'r, 'static> for NotFoundResponse {
    fn respond_to(
        self,
        request: &'r Request<'_>,
    ) -> Result<rocket::response::Response<'static>, rocket::http::Status> {
        match self {
            NotFoundResponse::Html(html) => html.respond_to(request),
            NotFoundResponse::JavaScript(javascript) => javascript.respond_to(request),
            NotFoundResponse::Favicon(favicon) => favicon.respond_to(request),
            NotFoundResponse::NotFound(not_found) => not_found.respond_to(request),
        }
    }
}

#[catch(404)]
fn not_found_handler(req: &Request) -> NotFoundResponse {
    let last_chunk = match req.uri().path().split('/').last() {
        Some(raw_str) => raw_str.as_str().to_string(),
        None => "".to_string(),
    };

    if req
        .uri()
        .path()
        .split('/')
        .find(|chunk| !chunk.is_empty())
        .unwrap_or_else(|| "".into())
        == "api"
    {
        NotFoundResponse::NotFound(status::NotFound(format!(
            "404 - API path '{}' does not exist!",
            req.uri().path()
        )))
    } else if last_chunk == "bundle.js" {
        NotFoundResponse::JavaScript(status::Custom(
            rocket::http::Status::Ok,
            content::JavaScript(JS_BUNDLE_BYTES),
        ))
    } else if last_chunk == "favicon.ico" {
        NotFoundResponse::Favicon(Box::from(status::Custom(
            rocket::http::Status::Ok,
            content::Custom(rocket::http::ContentType::Icon, FAVICON_BYTES),
        )))
    } else {
        NotFoundResponse::Html(status::Custom(
            rocket::http::Status::Ok,
            content::Html(HTML_BYTES),
        ))
    }
}

#[get("/healthz")]
async fn healthz_handler() -> content::Html<String> {
    content::Html("<html><body><h1>200 OK</h1>Service ready.</body></html>".to_string())
}

#[get("/api/signin?<display_name>")]
async fn signin_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
    display_name: String,
) -> Result<(), Error> {
    let mut unlocked_game_manager = game_manager.write().unwrap();
    if let Ok(player_uuid) = PlayerUUID::from_cookie_jar(cookie_jar) {
        if unlocked_game_manager
            .get_player_display_name(&player_uuid)
            .is_some()
        {
            return Err(Error::new("User is already signed in"));
        }
    };
    let player_uuid = PlayerUUID::new();
    if let Some(err) = unlocked_game_manager.add_player(player_uuid.clone(), display_name) {
        return Err(err);
    }
    player_uuid.to_cookie_jar(cookie_jar);
    Ok(())
}

#[get("/api/signout")]
async fn signout_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
) -> Result<(), Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;

    if let Some(err) = game_manager.write().unwrap().remove_player(&player_uuid) {
        return Err(err);
    }
    match PlayerUUID::from_cookie_jar(cookie_jar) {
        Ok(_) => {}
        Err(err) => return Err(err),
    };
    cookie_jar.remove(Cookie::named(SESSION_COOKIE_NAME));

    Ok(())
}

#[get("/api/me")]
async fn me_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
) -> Result<String, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let unlocked_game_manager = game_manager.read().unwrap();
    match unlocked_game_manager.get_player_display_name(&player_uuid) {
        Some(display_name) => Ok(display_name.clone()),
        None => Err(Error::new("Player does not exist")),
    }
}

#[get("/api/listGames")]
async fn list_games_handler(game_manager: &State<RwLock<GameManager>>) -> ListedGameViewCollection {
    game_manager.read().unwrap().list_games()
}

#[get("/api/createGame/<game_name>")]
async fn create_game_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
    game_name: String,
) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let mut unlocked_game_manager = game_manager.write().unwrap();
    unlocked_game_manager.create_game(player_uuid.clone(), game_name)?;
    unlocked_game_manager.get_game_view(player_uuid)
}

#[get("/api/joinGame/<game_uuid>")]
async fn join_game_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
    game_uuid: GameUUID,
) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let mut unlocked_game_manager = game_manager.write().unwrap();
    if let Some(err) = unlocked_game_manager.join_game(player_uuid.clone(), game_uuid) {
        return Err(err);
    };
    unlocked_game_manager.get_game_view(player_uuid)
}

#[get("/api/leaveGame")]
async fn leave_game_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
) -> Result<(), Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let mut unlocked_game_manager = game_manager.write().unwrap();
    if let Some(err) = unlocked_game_manager.leave_game(&player_uuid) {
        return Err(err);
    }
    Ok(())
}

#[get("/api/startGame")]
async fn start_game_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let unlocked_game_manager = game_manager.read().unwrap();
    if let Some(err) = unlocked_game_manager.start_game(&player_uuid) {
        return Err(err);
    };
    unlocked_game_manager.get_game_view(player_uuid)
}

#[get("/api/selectCharacter/<character>")]
async fn select_character_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
    character: Character,
) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let unlocked_game_manager = game_manager.read().unwrap();
    if let Some(err) = unlocked_game_manager.select_character(&player_uuid, character) {
        return Err(err);
    };
    unlocked_game_manager.get_game_view(player_uuid)
}

#[get("/api/playCard?<other_player_uuid>&<card_index>")]
async fn play_card_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
    other_player_uuid: Option<PlayerUUID>,
    card_index: usize,
) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let unlocked_game_manager = game_manager.read().unwrap();
    if let Some(err) = unlocked_game_manager.play_card(&player_uuid, &other_player_uuid, card_index)
    {
        return Err(err);
    }
    unlocked_game_manager.get_game_view(player_uuid)
}

#[get("/api/discardCards?<card_indices_string>")]
async fn discard_cards_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
    card_indices_string: Option<String>,
) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let unlocked_game_manager = game_manager.read().unwrap();
    if let Some(err) = unlocked_game_manager
        .discard_cards_and_draw_to_full(&player_uuid, parse_usize_vec(card_indices_string)?)
    {
        return Err(err);
    }
    unlocked_game_manager.get_game_view(player_uuid)
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
    unlocked_game_manager.get_game_view(player_uuid)
}

#[get("/api/pass")]
async fn pass_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    let unlocked_game_manager = game_manager.read().unwrap();
    if let Some(err) = unlocked_game_manager.pass(&player_uuid) {
        return Err(err);
    }
    unlocked_game_manager.get_game_view(player_uuid)
}

#[get("/api/getGameView")]
async fn get_game_view_handler(
    game_manager: &State<RwLock<GameManager>>,
    cookie_jar: &CookieJar<'_>,
) -> Result<GameView, Error> {
    let player_uuid = PlayerUUID::from_cookie_jar(cookie_jar)?;
    game_manager.read().unwrap().get_game_view(player_uuid)
}

fn parse_usize_vec(items_string_or: Option<String>) -> Result<Vec<usize>, Error> {
    match items_string_or {
        Some(items_string) => {
            let mut items: Vec<usize> = Vec::new();
            for item_string in items_string.split(',') {
                match item_string.parse::<usize>() {
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
        .register("/", catchers![not_found_handler])
        .mount(
            "/",
            routes![
                healthz_handler,
                signin_handler,
                signout_handler,
                me_handler,
                list_games_handler,
                create_game_handler,
                join_game_handler,
                leave_game_handler,
                start_game_handler,
                select_character_handler,
                play_card_handler,
                discard_cards_handler,
                order_drink_handler,
                pass_handler,
                get_game_view_handler
            ],
        )
}
