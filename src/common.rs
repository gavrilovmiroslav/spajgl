use rocket_db_pools::{sqlx, Connection};
use rocket_db_pools::sqlx::Row;
use crate::DB;

pub(crate) async fn check_token(db: &mut Connection<DB>, token: &str, user_id: i32) -> bool {
    if let Ok(Some(result)) = sqlx::query("select user_id from users where token = ?")
        .bind(token).fetch_optional(&mut ***db).await {
        let id: i32 = result.get(0);

        user_id == id
    } else {
        false
    }
}
