use rocket::serde::json::Json;
use rocket_db_pools::{sqlx, Connection};
use rocket_db_pools::sqlx::{Row};
use sha2::{Digest, Sha256};
use crate::DB;
use crate::requests::{LoginData, LoginResponse, RegisterData, RegistrationResponse};

#[post("/users/create", data="<register_data>", format="application/json")]
pub async fn create_user(mut db: Connection<DB>, register_data: Json<RegisterData>) -> Json<RegistrationResponse> {
    let RegisterData{ username, password } = register_data.into_inner();
    let mut sha256 = Sha256::new();
    sha256.update(password);
    let hash: String = format!("{:x}", sha256.finalize());
    let result = sqlx::query("insert or ignore into users('username', 'password') values (?, ?)")
        .bind(username)
        .bind(hash)
        .execute(&mut **db).await;

    if result.is_ok() {
        (RegistrationResponse { success: true, reason: "".into() }).into()
    } else {
        (RegistrationResponse { success: false, reason: format!("{:?}", result) }).into()
    }
}

#[post("/users/login", data="<login_data>", format="application/json")]
pub async fn login(mut db: Connection<DB>, login_data: Json<LoginData>) -> Json<LoginResponse> {
    let LoginData{ username, password } = login_data.into_inner();
    if let Ok(Some(result)) = sqlx::query("select id, password from users where username = ?")
        .bind(username)
        .fetch_optional(&mut **db).await {

        let user_id: i32 = result.get(0);
        let hash: String = result.get(1);
        if hash == password {
            let _ = sqlx::query("delete from tokens where user_id = ?")
                .bind(user_id)
                .execute(&mut **db).await;
            let token = uuid::Uuid::new_v4();
            let _ = sqlx::query("insert or ignore into tokens('user_id', 'token') values (?, ?)")
                .bind(user_id)
                .bind(token.to_string())
                .execute(&mut **db).await;

            (LoginResponse { success: true, user_id, token: token.to_string() }).into()
        } else {
            (LoginResponse { success: false, user_id, token: "".to_string() }).into()
        }
    } else {
        (LoginResponse { success: false, user_id: -1, token: "".to_string() }).into()
    }
}
