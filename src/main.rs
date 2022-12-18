mod entities;
mod routes;

use rocket::{launch, routes};

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![routes::game::game_css, routes::game::game,])
}
