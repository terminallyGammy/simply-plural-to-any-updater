use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use crate::auth;
use crate::auth::Jwt;
use crate::auth::JwtString;
use crate::config;
use crate::config::UserConfigDbEntries;
use crate::database;
use crate::database::get_all_users;
use crate::model::ApplicationJwtSecret;
use crate::model::ApplicationUserSecrets;
use crate::model::DecryptedDbSecret;
use crate::model::EncryptedDbSecret;
use crate::model::UserId;
use crate::model::UserLoginCredentials;
use crate::setup;
use crate::setup::ApplicationSetup;
use crate::simply_plural;
use crate::updater_loop;
use crate::updater_manager;
use crate::updater_manager::ThreadSafePerUserNew;
use crate::vrchat_auth;
use crate::vrchat_auth::TwoFactorAuthCode;
use crate::vrchat_auth::TwoFactorAuthMethod;
use crate::vrchat_auth::VRChatCredentials;
use crate::webview;
use anyhow::anyhow;
use anyhow::Result;
use futures::channel::oneshot;
use rocket::{
    response::{self, content::RawHtml},
    serde::json::Json,
    State,
};
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
        .manage(setup.tmp_vrchat_auth_code_state)
        .mount(
            "/api",
            routes![
                rest_get_fronting,
                get_updaters_state,
                restart_updaters,
                register,
                login,
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

    let all_users = get_all_users(&setup.db_pool).await?;

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

#[get("/updaters/state")]
fn get_updaters_state(
    shared_updaters: &State<updater_manager::SharedUpdaters>,
    jwt: ResponseResult<Jwt>,
) -> ResponseResult<Json<updater_loop::UserUpdatersStatuses>> {
    let user_id = jwt?.user_id()?;

    let updaters_state: updater_loop::UserUpdatersStatuses =
        shared_updaters.get_updaters_state(&user_id)?;

    Ok(Json(updaters_state))
}

#[post("/updaters/restart")]
async fn restart_updaters(
    jwt: ResponseResult<Jwt>,
    db_pool: &State<PgPool>,
    application_user_secrets: &State<ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
    shared_updater_state: &State<updater_manager::SharedUpdaters>,
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
    application_user_secrets: &ApplicationUserSecrets,
    client: &reqwest::Client,
    shared_updaters: &updater_manager::SharedUpdaters,
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
    application_user_secrets: &State<ApplicationUserSecrets>,
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
    jwt_app_secret: &State<ApplicationJwtSecret>,
    credentials: Json<UserLoginCredentials>,
) -> Result<Json<JwtString>, response::Debug<anyhow::Error>> {
    let user_id = database::get_user_id(db_pool, credentials.email.clone()).await?;

    let user_info = database::get_user_info(db_pool, user_id)
        .await
        .map_err(response::Debug)?;

    let jwt_string =
        auth::verify_password_and_create_token(&credentials.password, &user_info, jwt_app_secret)?;

    Ok(Json(jwt_string))
}
// todo. how can we enable users to reset their password? Do I really have to do this all manually here???

#[get("/user/config")]
async fn get_config(
    db_pool: &State<PgPool>,
    jwt: ResponseResult<Jwt>,
) -> ResponseResult<Json<UserConfigDbEntries<EncryptedDbSecret>>> {
    let user_id = jwt?.user_id()?;

    let user_config = database::get_user(db_pool, &user_id).await?;

    Ok(Json(user_config))
}

#[post("/user/config", data = "<config>")]
async fn set_config(
    config: Json<UserConfigDbEntries<DecryptedDbSecret>>,
    jwt: ResponseResult<Jwt>,
    db_pool: &State<PgPool>,
    app_user_secrets: &State<ApplicationUserSecrets>,
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
    jwt: ResponseResult<Jwt>,
    tmp_vrchat_auth_code_state: &State<VRChatAuthCodeState>,
) -> ResponseResult<()> {
    let user_id = jwt?.user_id()?;
    let creds = creds.into_inner();

    let (tfa_promise, tfa_future) = oneshot::channel();
    let (method_promise, method_future) = oneshot::channel();

    let () = insert_channels(
        &user_id,
        &creds,
        tmp_vrchat_auth_code_state,
        method_future,
        tfa_promise,
    )
    .await?;

    let request_two_factor_auth_code = |_c, request_method| async move {
        let () = method_promise
            .send(request_method)
            .map_err(|e| anyhow!("Promoie send failed: {e}"))?;
        let code = tfa_future
            .await
            .map_err(|e| anyhow!("TwoFactorAuth Future Error: {e}"))?;
        Ok(code)
    };

    let _ = vrchat_auth::authenticate_vrchat_for_new_cookie(creds, request_two_factor_auth_code)
        .await?;

    todo!();
}

pub type VRChatAuthCodeState = ThreadSafePerUserNew<(
    VRChatCredentials,
    oneshot::Receiver<TwoFactorAuthMethod>,
    oneshot::Sender<TwoFactorAuthCode>,
)>;
async fn insert_channels(
    user_id: &UserId,
    creds: &vrchat_auth::VRChatCredentials,
    tmp_vrchat_auth_code_state: &VRChatAuthCodeState,
    method_future: oneshot::Receiver<TwoFactorAuthMethod>,
    tfa_promise: oneshot::Sender<TwoFactorAuthCode>,
) -> ResponseResult<()> {
    let arc_mut_tuple = Arc::new(Mutex::new((creds.clone(), method_future, tfa_promise)));

    tmp_vrchat_auth_code_state
        .lock()
        .map_err(|e| response::Debug(anyhow!("Lock error: {e}")))?
        .deref_mut()
        .insert(user_id.clone(), arc_mut_tuple);

    Ok(())
}

#[post("/user/platform/vrchat/auth_2fa/resolve", data = "<tfa_code>")]
async fn vrchat_user_authentication_resolve(
    jwt: ResponseResult<Jwt>,
    tfa_code: Json<vrchat_auth::TwoFactorAuthCode>,
    tmp_vrchat_auth_code_state: &State<VRChatAuthCodeState>,
) -> ResponseResult<Json<vrchat_auth::VRChatCredentialsWithCookie>> {
    let user_id = jwt?.user_id()?;
    let tfa_code = tfa_code.into_inner();

    // let t = {
    //     let hm = tmp_vrchat_auth_code_state
    //         .lock()
    //         .map_err(|e| response::Debug(anyhow!("Lock error: {e}")))?;

    //     let tuple = hm
    //         .get(&user_id)
    //         .ok_or_else(|| anyhow!("No channels for userid found. {}", &user_id))?;

    //     let mut t = tuple.lock();

    //     let mut t2 = t.as_deref_mut().map_err(|e| anyhow!("Error: {e}"))?;

    //     let creds = &t2.0;
    //     let code_prom = &t2.2;

    //     todo!();
    //     let cp = *code_prom;

    //     cp.send(tfa_code);

    //     creds.clone()
    // };

    todo!();
}
