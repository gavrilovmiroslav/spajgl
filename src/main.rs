mod requests;
mod users;
mod game;
mod common;

#[macro_use] extern crate rocket;

use rocket_db_pools::{sqlx, Database};
use crate::game::{get_leaderboard, new_score, start_game, load_game, save_game, check_tokens, store_data, get_data};
use crate::users::{create_user, login};
use rocket_cors::{AllowedOrigins, CorsOptions, AllowedHeaders};
use rocket::http::Method;
use rocket::shield::Shield;
use rocket::shield::{Referrer, Prefetch, ExpectCt};
use time::Duration;

#[derive(Database)]
#[database("spajgl")]
pub struct DB(pub sqlx::SqlitePool);

#[launch]
fn rocket() -> _ {
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_headers(AllowedHeaders::all())
        .allowed_methods(vec![Method::Get, Method::Post, Method::Options]
            .into_iter()
            .map(From::from)
            .collect()
        ).allow_credentials(false)
        .send_wildcard(true);

    rocket::build()
        .attach(DB::init())
        .attach(Shield::default()
            .enable(Referrer::NoReferrer)
            .enable(Prefetch::Off)
            .enable(ExpectCt::Enforce(Duration::days(30)))
        )
        .attach(cors.to_cors().unwrap())
        .configure(rocket::Config::figment()
            .merge(("address", "0.0.0.0"))
            .merge(("port", 8001))
        )
        .mount("/", routes![
            create_user, login, 
            start_game, new_score, load_game, save_game, 
            get_leaderboard, check_tokens,
            store_data, get_data
        ])
}
