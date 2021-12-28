#[macro_use]
extern crate rocket;

mod game;

use rocket::response::content;

#[get("/healthz")]
async fn healthz_handler() -> content::Html<String> {
    content::Html("<html><body><h1>200 OK</h1>Service ready.</body></html>".to_string())
}

#[rocket::launch]
async fn rocket() -> _ {
    rocket::build().mount("/", routes![healthz_handler])
}
