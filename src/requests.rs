use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RegisterData {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RegistrationResponse {
    pub success: bool,
    pub reason: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginResponse {
    pub success: bool,
    pub user_id: i32,
    pub token: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ScoreRequest {
    pub user_id: i32,
    pub timestamp: i64,
    pub token: String,
    pub session: i32,
    pub guesses: String,
    pub check: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ScoreResponse {
    pub success: bool
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameRequest {
    pub user_id: i32,
    pub timestamp: i64,
    pub token: String,
    pub session: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameResponse {
    pub success: bool,
    pub check: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LeaderboardEntry {
    pub username: String,
    pub started_at: i64,
    pub finished_at: i64,
    pub guess_count: usize,
    pub guesses: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LeaderboardState {
    pub entries: Vec<LeaderboardEntry>,
}
