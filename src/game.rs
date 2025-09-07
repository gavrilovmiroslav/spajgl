use rocket::serde::json::Json;
use rocket_db_pools::{sqlx, Connection};
use rocket_db_pools::sqlx::Row;
use rocket_db_pools::sqlx::sqlite::SqliteRow;
use sha2::{Digest, Sha256};
use crate::common::check_token;
use crate::DB;
use crate::requests::{GameRequest, GameResponse, LeaderboardEntry, LeaderboardState, ScoreRequest, ScoreResponse, LoadRequest, LoadResponse, CheckResponse, DataStoreRequest, DataLoadRequest, DataLoadResponse};

#[post("/game/start", data="<game_data>", format="application/json")]
pub async fn start_game(mut db: Connection<DB>, game_data: Json<GameRequest>) -> Json<GameResponse> {
    let GameRequest{ token, user_id, session, timestamp } = game_data.into_inner();

    if check_token(&mut db, &token, user_id).await {
        let mut sha256 = Sha256::new();
        sha256.update(&format!("{}-{}-{}-{}", token, user_id, session, timestamp));
        let salt: String = format!("{:x}", sha256.finalize());
        let _ = sqlx::query("insert into games('user_id', 'timestamp', 'session', 'salt') values (?, ?, ?, ?)")
            .bind(user_id)
            .bind(timestamp)
            .bind(session)
            .bind(salt.clone())
            .execute(&mut **db).await;

        return (GameResponse { success: true, check: salt.to_string() }).into()
    }

    (GameResponse { success: false, check: "".to_string() }).into()
}

#[post("/game/save", data="<score_data>", format="application/json")]
pub async fn save_game(mut db: Connection<DB>, score_data: Json<ScoreRequest>) {
    let ScoreRequest{ token, user_id, session, guesses, .. } = score_data.into_inner();

    if check_token(&mut db, &token, user_id).await {
        if let Ok(Some(row)) = sqlx::query("select id from games where user_id = ? and session = ?")
            .bind(user_id)
            .bind(session)
            .fetch_optional(&mut **db).await {

            let id: i32 = row.get(0);
            let _ = sqlx::query("update games set guesses = ? where id = ?")
                .bind(guesses)
                .bind(id)
                .execute(&mut **db).await;
        }
    }
}

#[post("/game/check_tokens", data="<load_data>", format="application/json")]
pub async fn check_tokens(mut db: Connection<DB>, load_data: Json<LoadRequest>) -> Json<CheckResponse> {
    let LoadRequest{ token, user_id, .. } = load_data.into_inner();
    (CheckResponse { success: check_token(&mut db, &token, user_id).await }).into()
}

#[post("/game/load", data="<load_data>", format="application/json")]
pub async fn load_game(mut db: Connection<DB>, load_data: Json<LoadRequest>) -> Json<LoadResponse> {
    let LoadRequest{ token, user_id, session } = load_data.into_inner();
    if check_token(&mut db, &token, user_id).await {
        if let Ok(Some(row)) = sqlx::query("select id from games where user_id = ? and session = ?")
            .bind(user_id)
            .bind(session)
            .fetch_optional(&mut **db).await {

            let id: i32 = row.get(0);

            if let Ok(Some(row)) = sqlx::query("select guesses from games where id = ?")
                .bind(id)
                .fetch_optional(&mut **db).await {

                let guesses = row.get(0);
                return (LoadResponse { success: true, guesses }).into();
            }
        }
    }

    (LoadResponse { success: false, guesses: "".into() }).into()
}

#[post("/game/submit", data="<score_data>", format="application/json")]
pub async fn new_score(mut db: Connection<DB>, score_data: Json<ScoreRequest>) -> Json<ScoreResponse> {
    let ScoreRequest{ token, user_id, session, check, guesses, timestamp } = score_data.into_inner();

    if check_token(&mut db, &token, user_id).await {
        if let Ok(Some(row)) = sqlx::query("select id, salt from games where user_id = ? and session = ? and timestamp < ?")
            .bind(user_id)
            .bind(session)
            .bind(timestamp)
            .fetch_optional(&mut **db).await {

            let id: i32 = row.get(0);
            let salt: String = row.get(1);

            let mut sha256 = Sha256::new();
            sha256.update(&format!("{}+{}+{}", salt, guesses, timestamp));
            let sanity_check: String = format!("{:x}", sha256.finalize());

            if sanity_check == check {
                let _ = sqlx::query("update games set finished = 1, finished_at = ?, guesses = ? where id = ?")
                    .bind(timestamp)
                    .bind(guesses)
                    .bind(id)
                    .execute(&mut **db).await;

                return (ScoreResponse { success: true }).into()
            }
        }
    }

    (ScoreResponse { success: false }).into()
}

#[get("/game/leaderboard/<session>")]
pub async fn get_leaderboard(mut db: Connection<DB>, session: i32) -> Json<LeaderboardState> {
    fn construct_entry(row: SqliteRow) -> LeaderboardEntry {
        let username: String = row.get(0);
        let started_at: i64 = row.get(1);
        let finished_at: i64 = row.get(2);
        let guess_string: String = row.get(3);
        let guesses = guess_string.split("+").map(|s| s.to_string()).collect::<Vec<_>>();

        LeaderboardEntry {
            username,
            started_at,
            finished_at,
            guess_count: guesses.len(),
            guesses,
        }
    }

    if let Ok(results) = sqlx::query("select u.username as username, u.timestamp as started_time, u.finished_at as finished_time, g.guesses as guesses from games g full outer join users u on u.id = g.user_id where g.session = ? and g.finished = 1")
        .bind(session)
        .fetch_all(&mut **db).await {

        (LeaderboardState { entries: results.into_iter().map(construct_entry).collect() }).into()
    } else {
        (LeaderboardState { entries: vec![] }).into()
    }
}

#[post("/game/store_data", data="<data_store>", format="application/json")]
pub async fn store_data(mut db: Connection<DB>, data_store: Json<DataStoreRequest>) {
    let DataStoreRequest{ key, value } = data_store.into_inner();
    let _ = sqlx::query("insert or replace into kvdata('key', 'value') values (?, ?)")
        .bind(key)
        .bind(value)
        .execute(&mut **db).await;
}

#[post("/game/get_data", data="<data_load>", format="application/json")]
pub async fn get_data(mut db: Connection<DB>, data_load: Json<DataLoadRequest>) -> Json<DataLoadResponse> {
    let DataLoadRequest{ key } = data_load.into_inner();
    if let Ok(Some(row)) = sqlx::query("select value from kvdata where key = ?")
        .bind(key)
        .fetch_optional(&mut **db).await {
        
        let val: String = row.get(0);
        (DataLoadResponse { success: true, value: val }).into()
    } else {
        (DataLoadResponse { success: false, value: "".to_string() }).into()
    }
}

