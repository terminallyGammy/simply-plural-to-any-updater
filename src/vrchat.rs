use crate::config::Config;
use crate::simply_plural;
use crate::vrchat_status;
use anyhow::{Ok, Result};
use vrchatapi::{
    apis::{configuration::Configuration, users_api},
    models::UpdateUserRequest,
};

pub async fn updater_loop_logic(
    config: &Config,
    vrchat_config: &Configuration,
    user_id: &str,
) -> Result<()> {
    let fronts = simply_plural::fetch_fronts(config).await?;

    let status_string = vrchat_status::format_fronts_for_vrchat_status(config, &fronts);

    set_vrchat_status(vrchat_config, user_id, status_string.as_str()).await
}

async fn set_vrchat_status(
    vrchat_config: &Configuration,
    user_id: &str,
    status_string: &str,
) -> Result<()> {
    let mut update_request = UpdateUserRequest::new();
    update_request.status_description = Some(status_string.to_string());

    users_api::update_user(vrchat_config, user_id, Some(update_request))
        .await
        .inspect(|_| eprintln!("VRChat status updated successfully to: '{status_string}'"))?;

    Ok(())
}
