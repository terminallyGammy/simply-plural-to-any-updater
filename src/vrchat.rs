use crate::config::Config;
use crate::fronting_status;
use crate::simply_plural;
use crate::updater::Platform;
use crate::vrchat_auth;
use anyhow::anyhow;
use anyhow::{Ok, Result};
use vrchatapi::{
    apis::{configuration::Configuration, users_api},
    models::UpdateUserRequest,
};

type InitializedUpdater = (Configuration, String);
pub struct VRChatUpdater {
    initialized: Option<InitializedUpdater>,
}
impl VRChatUpdater {
    pub const fn new(_platform: Platform) -> Self {
        Self { initialized: None }
    }

    pub async fn setup(&mut self, config: &Config) -> Result<()> {
        self.initialized = Some(vrchat_auth::authenticate_vrchat(config).await?);
        Ok(())
    }

    pub async fn update_fronting_status(
        &self,
        config: &Config,
        fronts: &[simply_plural::Fronter],
    ) -> Result<()> {
        let initialized_updater = self
            .initialized
            .as_ref()
            .ok_or_else(|| anyhow!("Updater not initalized!"))?;
        update_to_vrchat(config, initialized_updater, fronts).await
    }
}

async fn update_to_vrchat(
    config: &Config,
    initialized_updater: &InitializedUpdater,
    fronts: &[simply_plural::Fronter],
) -> Result<()> {
    let fronting_format = fronting_status::FrontingFormat {
        max_length: Some(fronting_status::VRCHAT_MAX_ALLOWED_STATUS_LENGTH),
        cleaning: fronting_status::CleanForPlatform::VRChat,
        prefix: config.vrchat_updater_prefix.clone(),
        status_if_no_fronters: config.vrchat_updater_no_fronts.clone(),
        truncate_names_to_length_if_status_too_long: config.vrchat_updater_truncate_names_to,
    };

    let status_string = fronting_status::format_fronting_status(&fronting_format, fronts);

    set_vrchat_status(initialized_updater, status_string.as_str()).await
}

async fn set_vrchat_status(
    initialized_updater: &InitializedUpdater,
    status_string: &str,
) -> Result<()> {
    let mut update_request = UpdateUserRequest::new();
    update_request.status_description = Some(status_string.to_string());

    let (vrchat_config, user_id) = initialized_updater;
    users_api::update_user(vrchat_config, user_id, Some(update_request))
        .await
        .inspect(|_| eprintln!("VRChat status updated successfully to: '{status_string}'"))?;

    Ok(())
}
