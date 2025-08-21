use crate::database;
use crate::model::Email;
use crate::model::HttpResult;
use crate::model::UserId;
use crate::model::UserLoginCredentials;
use crate::users::auth;
use crate::users::jwt;
use rocket::response;
use rocket::{serde::json::Json, State};
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;

#[post("/user/register", data = "<credentials>")]
pub async fn post_api_user_register(
    db_pool: &State<PgPool>,
    credentials: Json<UserLoginCredentials>,
) -> HttpResult<()> {
    let pwh = auth::create_password_hash(&credentials.password)?;

    database::create_user(db_pool, credentials.email.clone(), pwh)
        .await
        .map_err(response::Debug)
}

#[post("/user/login", data = "<credentials>")]
pub async fn post_api_user_login(
    db_pool: &State<PgPool>,
    jwt_app_secret: &State<jwt::ApplicationJwtSecret>,
    credentials: Json<UserLoginCredentials>,
) -> HttpResult<Json<jwt::JwtString>> {
    let user_id = database::get_user_id(db_pool, credentials.email.clone()).await?;

    let user_info = database::get_user_info(db_pool, user_id)
        .await
        .map_err(response::Debug)?;

    let jwt_string =
        auth::verify_password_and_create_token(&credentials.password, &user_info, jwt_app_secret)?;

    Ok(Json(jwt_string))
}
// todo. how can we enable users to reset their password? Do I really have to do this all manually here???

#[get("/user/info")]
pub async fn get_api_user_info(
    db_pool: &State<PgPool>,
    jwt: HttpResult<jwt::Jwt>,
) -> HttpResult<Json<UserInfoUI>> {
    let user_id = jwt?.user_id()?;
    let user_info = database::get_user_info(db_pool, user_id)
        .await
        .map_err(response::Debug)?;
    Ok(Json(user_info.into()))
}

#[derive(Serialize, Deserialize)]
pub struct UserInfoUI {
    pub id: UserId,
    pub email: Email,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<database::UserInfo> for UserInfoUI {
    fn from(user: database::UserInfo) -> Self {
        let database::UserInfo {
            id,
            email,
            password_hash: _,
            created_at,
        } = user;
        Self {
            id,
            email,
            created_at,
        }
    }
}
