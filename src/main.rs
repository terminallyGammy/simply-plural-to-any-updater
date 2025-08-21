#[macro_use]
extern crate rocket;

use anyhow::anyhow;
use anyhow::Result;

use clap::Parser;

mod database;
mod macros;
mod model;
mod platforms;
mod plurality;
mod setup;
mod updater;
mod users;

#[tokio::main]
async fn main() -> Result<()> {
    let cli_args = setup::CliArgs::parse();

    let app_setup = setup::application_setup(&cli_args).await?;

    let () = updater::api::restart_all_user_updaters_for_app_startups(app_setup.clone()).await?;

    run_webserver(app_setup).await
}

async fn run_webserver(setup: setup::ApplicationSetup) -> Result<()> {
    let _ = rocket::build()
        .manage(setup.db_pool)
        .manage(setup.jwt_secret)
        .manage(setup.application_user_secrets)
        .manage(setup.client)
        .manage(setup.shared_updaters)
        .mount(
            "/api",
            routes![
                platforms::webview_api::get_api_fronting_by_user_id,
                updater::api::get_api_updaters_status,
                updater::api::post_api_updaters_restart,
                users::user_api::post_api_user_register,
                users::user_api::post_api_user_login,
                users::user_api::get_api_user_info,
                users::config_api::get_api_user_config,
                users::config_api::post_api_user_config,
                platforms::vrchat_api::post_api_user_platform_vrchat_auth_2fa_request,
                platforms::vrchat_api::post_api_user_platform_vrchat_auth_2fa_resolve
            ],
        )
        .launch()
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}
