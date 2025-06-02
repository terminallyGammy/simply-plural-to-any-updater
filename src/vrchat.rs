use crate::{config::Config, simply_plural, vrchat_auth, vrchat_status};
use std::{thread, time};
use anyhow::Result;
use time::Duration;
use vrchatapi::{
    apis::{configuration::Configuration, users_api},
    models::UpdateUserRequest,
};
use chrono::Utc;

pub async fn run_updater_loop(config: &Config) -> Result<()> {
    let (vrchat_config, user_id) = vrchat_auth::authenticate_vrchat(config).await?;

    loop {
        eprintln!("\n\n======================= UTC {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));

        let fronts = simply_plural::fetch_fronts(&config).await?;
        
        update_fronts_in_vrchat_status(&config, &vrchat_config, &user_id, fronts).await?;
        
        eprintln!("Waiting {}s for next update trigger...", config.wait_seconds);
        thread::sleep(Duration::from_secs(config.wait_seconds));
    }
}

async fn update_fronts_in_vrchat_status(
    config: &Config,
    vrchat_config: &Configuration,
    user_id: &String,
    fronts: Vec<simply_plural::MemberContent>,
) -> Result<()> {
    let status_string = vrchat_status::format_fronts_for_vrchat_status(config, fronts);

    let mut update_request = UpdateUserRequest::new();
    update_request.status_description = Some(status_string.clone());

    match users_api::update_user(vrchat_config, &user_id, Some(update_request)).await {
        Ok(_) => eprintln!("VRChat status updated successfully to: '{}'", status_string),
        Err(err) => eprintln!("VRChat status failed to be updated. Error: {}", err),
    }

    Ok(())
}

