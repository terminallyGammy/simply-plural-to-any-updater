use crate::{config::Config, simply_plural, vrchat_auth, vrchat_status};
use std::thread;
use anyhow::Result;
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

        let status_string = vrchat_status::format_fronts_for_vrchat_status(config, fronts);
        
        set_vrchat_status(&vrchat_config, &user_id, status_string).await?;
        
        eprintln!("Waiting {}s for next update trigger...", config.wait_seconds.as_secs());

        thread::sleep(config.wait_seconds);
    }
}

async fn set_vrchat_status(vrchat_config: &Configuration, user_id: &String, status_string: String) -> Result<()> {
    let mut update_request = UpdateUserRequest::new();
    update_request.status_description = Some(status_string.clone());

    let update_result = users_api::update_user(vrchat_config, &user_id, Some(update_request)).await;

    match update_result {
        Ok(_) => eprintln!("VRChat status updated successfully to: '{}'", status_string),
        Err(err) => eprintln!("❌❌❌ VRChat status failed to be updated. Error: {}", err),
    }

    Ok(())
}

