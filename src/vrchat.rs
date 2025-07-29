use crate::{config::Config, simply_plural, vrchat_auth, vrchat_status};
use anyhow::{Ok, Result};
use chrono::Utc;
use std::thread;
use vrchatapi::{
    apis::{configuration::Configuration, users_api},
    models::UpdateUserRequest,
};

pub async fn run_updater_loop(config: &Config) -> Result<()> {
    eprintln!("Running VRChat Updater ...");

    let (vrchat_config, user_id) = vrchat_auth::authenticate_vrchat(config).await?;

    loop {
        eprintln!(
            "\n\n======================= UTC {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        updater_loop_logic(&config, &vrchat_config, &user_id)
            .await
            .inspect_err(|err| {
                eprintln!(
                    "Error in VRChat Updater Loop. Skipping update. Error: {}",
                    err
                )
            })
            .or(Ok(()))?;

        eprintln!(
            "Waiting {}s for next update trigger...",
            config.wait_seconds.as_secs()
        );

        thread::sleep(config.wait_seconds);
    }
}

async fn updater_loop_logic(
    config: &Config,
    vrchat_config: &Configuration,
    user_id: &String,
) -> Result<()> {
    let fronts = simply_plural::fetch_fronts(&config).await?;

    let status_string = vrchat_status::format_fronts_for_vrchat_status(config, fronts);

    set_vrchat_status(&vrchat_config, &user_id, status_string).await
}

async fn set_vrchat_status(
    vrchat_config: &Configuration,
    user_id: &String,
    status_string: String,
) -> Result<()> {
    let mut update_request = UpdateUserRequest::new();
    update_request.status_description = Some(status_string.clone());

    users_api::update_user(vrchat_config, &user_id, Some(update_request))
        .await
        .inspect(|_| eprintln!("VRChat status updated successfully to: '{}'", status_string))?;

    Ok(())
}
