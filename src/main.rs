mod requests;
mod users;
mod game;
mod common;

#[macro_use] extern crate rocket;

use rocket_db_pools::{sqlx, Database};
use crate::game::{get_leaderboard, new_score, start_game};
use crate::users::{create_user, login};

#[derive(Database)]
#[database("spajgl")]
pub struct DB(pub sqlx::SqlitePool);

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(DB::init())
        .mount("/", routes![create_user, login, start_game, new_score, get_leaderboard])
}
