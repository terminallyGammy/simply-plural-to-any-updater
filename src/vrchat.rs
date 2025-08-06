use crate::config::Config;
use crate::fronting_status;
use crate::simply_plural;
use anyhow::{Ok, Result};
use vrchatapi::{
    apis::{configuration::Configuration, users_api},
    models::UpdateUserRequest,
};

pub async fn update_to_vrchat(
    config: &Config,
    vrchat_config: &Configuration,
    fronts: &[simply_plural::Fronter],
    user_id: &str,
) -> Result<()> {
    let fronting_format = fronting_status::FrontingFormat {
        max_length: Some(fronting_status::VRCHAT_MAX_ALLOWED_STATUS_LENGTH),
        cleaning: fronting_status::CleanForPlatform::VRChat,
        prefix: config.vrchat_updater_prefix.clone(),
        status_if_no_fronters: config.vrchat_updater_no_fronts.clone(),
        truncate_names_to_length_if_status_too_long: config.vrchat_updater_truncate_names_to,
    };

    let status_string = fronting_status::format_fronting_status(&fronting_format, fronts);

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
