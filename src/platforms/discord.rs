use crate::{config::UserConfigForUpdater, plurality, record_if_error, updater};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    custom_status: Status,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Status {
    text: String,
}

pub struct DiscordUpdater {
    pub last_operation_error: Option<String>,
}
impl DiscordUpdater {
    pub const fn new(_platform: updater::Platform) -> Self {
        Self {
            last_operation_error: None,
        }
    }

    #[allow(clippy::unused_async)]
    pub async fn setup(&self, _config: &UserConfigForUpdater) -> Result<()> {
        Ok(())
    }

    pub async fn update_fronting_status(
        &mut self,
        config: &UserConfigForUpdater,
        fronts: &[plurality::Fronter],
    ) -> Result<()> {
        record_if_error!(self, update_to_discord(config, fronts).await)
    }
}

async fn update_to_discord(
    config: &UserConfigForUpdater,
    fronts: &[plurality::Fronter],
) -> Result<()> {
    let fronting_format = plurality::FrontingFormat {
        max_length: Some(plurality::DISCORD_STATUS_MAX_LENGTH),
        cleaning: plurality::CleanForPlatform::NoClean,
        prefix: config.status_prefix.clone(),
        status_if_no_fronters: config.status_no_fronts.clone(),
        truncate_names_to_length_if_status_too_long: config.status_truncate_names_to,
    };

    let status_string = plurality::format_fronting_status(&fronting_format, fronts);

    set_discord_status(config, status_string).await?;

    Ok(())
}

async fn set_discord_status(config: &UserConfigForUpdater, status_string: String) -> Result<()> {
    eprintln!("Setting Discord Status: {status_string}");

    let discord_status_url = format!(
        "{}{}",
        config.discord_base_url, "/api/v10/users/@me/settings"
    );

    let body = User {
        custom_status: Status {
            text: status_string,
        },
    };

    let result: User = config
        .client
        .patch(discord_status_url)
        .header("Authorization", &config.discord_token.secret)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body)?)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    eprintln!("Changed Discord User: {result:?}");

    Ok(())
}
