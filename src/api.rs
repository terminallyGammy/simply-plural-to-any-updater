use crate::auth;
use crate::config;
use crate::config::UserConfigDbEntries;
use crate::database;
use crate::jwt;
use crate::model::Email;
use crate::model::UserId;
use crate::model::UserLoginCredentials;
use crate::setup;
use crate::setup::ApplicationSetup;
use crate::simply_plural;
use crate::updater;
use crate::vrchat_auth;
use crate::webview;
use anyhow::anyhow;
use anyhow::Result;
use either::Either;
use rocket::{
    response::{self, content::RawHtml},
    serde::json::Json,
    State,
};
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;

type ResponseResult<T> = Result<T, response::Debug<anyhow::Error>>;

pub async fn start_application(setup: setup::ApplicationSetup) -> Result<()> {
    let () = restart_all_user_updaters_for_app_startups(setup.clone()).await?;

    let _ = rocket::build()
        .manage(setup.db_pool)
        .manage(setup.jwt_secret)
        .manage(setup.application_user_secrets)
        .manage(setup.client)
        .manage(setup.shared_updaters)
        .mount(
            "/api",
            routes![
                rest_get_fronting,
                get_updaters_status,
                restart_updaters,
                register,
                login,
                get_user_info,
                get_config,
                set_config,
                vrchat_user_authentication_request,
                vrchat_user_authentication_resolve
            ],
        )
        .launch()
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

async fn restart_all_user_updaters_for_app_startups(setup: ApplicationSetup) -> Result<()> {
    eprintln!("Starting all user updaters ...");

    let all_users = database::get_all_users(&setup.db_pool).await?;

    eprintln!("Users: {all_users:?}");

    for user in all_users {
        restart_updater_for_user(
            &user,
            &setup.db_pool,
            &setup.application_user_secrets,
            &setup.client,
            &setup.shared_updaters,
        )
        .await?;
    }

    eprintln!("Starting all user updaters. DONE.");

    Ok(())
}

#[get("/updaters/status")]
fn get_updaters_status(
    shared_updaters: &State<updater::UpdaterManager>,
    jwt: ResponseResult<jwt::Jwt>,
) -> ResponseResult<Json<updater::work_loop::UserUpdatersStatuses>> {
    let user_id = jwt?.user_id()?;

    let updaters_state: updater::work_loop::UserUpdatersStatuses =
        shared_updaters.get_updaters_state(&user_id)?;

    Ok(Json(updaters_state))
}

#[post("/updaters/restart")]
async fn restart_updaters(
    jwt: ResponseResult<jwt::Jwt>,
    db_pool: &State<PgPool>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
    shared_updater_state: &State<updater::UpdaterManager>,
) -> ResponseResult<()> {
    let user_id = jwt?.user_id()?;

    let () = restart_updater_for_user(
        &user_id,
        db_pool,
        application_user_secrets,
        client,
        shared_updater_state,
    )
    .await?;

    Ok(())
}

async fn restart_updater_for_user(
    user_id: &UserId,
    db_pool: &PgPool,
    application_user_secrets: &database::ApplicationUserSecrets,
    client: &reqwest::Client,
    shared_updaters: &updater::UpdaterManager,
) -> Result<()> {
    eprintln!("Restarting user updaters {user_id} ...");

    let db_config = database::get_user_secrets(db_pool, user_id, application_user_secrets).await?;

    let (config, _) = config::create_config_with_strong_constraints(user_id, client, &db_config)?;

    let () = shared_updaters.restart_updater(user_id, config)?;

    eprintln!("Restarting user updaters {user_id}. DONE.");

    Ok(())
}

#[get("/fronting/<user_id>")]
async fn rest_get_fronting(
    user_id: &str, // todo. actually use system name here instead of user-id
    db_pool: &State<PgPool>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
) -> ResponseResult<RawHtml<String>> {
    eprintln!("GET /fronting/{user_id}.");

    let user_id: UserId = user_id.try_into()?;

    eprintln!("GET /fronting/{user_id}. Getting user secrets");

    let user_config =
        database::get_user_secrets(db_pool, &user_id, application_user_secrets).await?;

    eprintln!("GET /fronting/{user_id}. Creating config");

    let (updater_config, _) =
        config::create_config_with_strong_constraints(&user_id, client, &user_config)?;

    eprintln!("GET /fronting/{user_id}. Fetching fronts");

    let fronts = simply_plural::fetch_fronts(&updater_config)
        .await
        .map_err(response::Debug)?;

    eprintln!("GET /fronting/{user_id}. Rendering HTML");

    let html = webview::generate_html(&updater_config.system_name, fronts);

    eprintln!("GET /fronting/{user_id}. OK");
    Ok(RawHtml(html))
}

#[post("/user/register", data = "<credentials>")]
async fn register(
    db_pool: &State<PgPool>,
    credentials: Json<UserLoginCredentials>,
) -> ResponseResult<()> {
    let pwh = auth::create_password_hash(&credentials.password)?;

    database::create_user(db_pool, credentials.email.clone(), pwh)
        .await
        .map_err(response::Debug)
}

#[post("/user/login", data = "<credentials>")]
async fn login(
    db_pool: &State<PgPool>,
    jwt_app_secret: &State<jwt::ApplicationJwtSecret>,
    credentials: Json<UserLoginCredentials>,
) -> Result<Json<jwt::JwtString>, response::Debug<anyhow::Error>> {
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
async fn get_user_info(
    db_pool: &State<PgPool>,
    jwt: ResponseResult<jwt::Jwt>,
) -> ResponseResult<Json<UserInfoUI>> {
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

#[get("/user/config")]
async fn get_config(
    db_pool: &State<PgPool>,
    jwt: ResponseResult<jwt::Jwt>,
) -> ResponseResult<Json<UserConfigDbEntries<database::Encrypted>>> {
    let user_id = jwt?.user_id()?;

    let user_config = database::get_user(db_pool, &user_id).await?;

    Ok(Json(user_config))
}

#[post("/user/config", data = "<config>")]
async fn set_config(
    config: Json<UserConfigDbEntries<database::Decrypted>>,
    jwt: ResponseResult<jwt::Jwt>,
    db_pool: &State<PgPool>,
    app_user_secrets: &State<database::ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
) -> ResponseResult<()> {
    let user_id = jwt?.user_id()?;

    // check that config satisfies contraints
    let (_, valid_db_config) =
        config::create_config_with_strong_constraints(&user_id, client, &config)?;

    let () = database::set_user_config_secrets(db_pool, user_id, valid_db_config, app_user_secrets)
        .await?;

    Ok(())
}

#[post("/user/platform/vrchat/auth_2fa/request", data = "<creds>")]
async fn vrchat_user_authentication_request(
    creds: Json<vrchat_auth::VRChatCredentials>,
    _jwt: ResponseResult<jwt::Jwt>, // request should be authenticated, but we don't need user id
) -> ResponseResult<
    Json<Either<vrchat_auth::VRChatCredentialsWithCookie, vrchat_auth::TwoFactorAuthMethod>>,
> {
    let creds = creds.into_inner();

    let creds_or_tfa_method = vrchat_auth::authenticate_vrchat_for_new_cookie(creds).await?;

    Ok(Json(creds_or_tfa_method))
}

#[post("/user/platform/vrchat/auth_2fa/resolve", data = "<creds_with_tfa>")]
async fn vrchat_user_authentication_resolve(
    _jwt: ResponseResult<jwt::Jwt>, // request should be authenticated, but we don't need user id
    creds_with_tfa: Json<vrchat_auth::VRChatCredentialsWithTwoFactorAuth>,
) -> ResponseResult<Json<vrchat_auth::VRChatCredentialsWithCookie>> {
    let creds_with_tfa = creds_with_tfa.into_inner();

    let valid_creds =
        vrchat_auth::authenticate_vrchat_for_new_cookie_with_2fa(creds_with_tfa).await?;

    Ok(Json(valid_creds))
}
