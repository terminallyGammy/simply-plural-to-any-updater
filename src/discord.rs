/* WORK-IN-PROGRESS */

use crate::{config::Config, fronting_status, simply_plural};
use anyhow::Result;

pub async fn update_to_discord(config: &Config, fronts: &[simply_plural::Fronter]) -> Result<()> {
    let fronting_format = fronting_status::FrontingFormat {
        max_length: Some(fronting_status::DISCORD_STATUS_MAX_LENGTH),
        cleaning: fronting_status::CleanForPlatform::NoClean,
        prefix: config.vrchat_updater_prefix.clone(), // todo. rename to generic config
        status_if_no_fronters: config.vrchat_updater_no_fronts.clone(), // todo. rename to generic config
        truncate_names_to_length_if_status_too_long: config.vrchat_updater_truncate_names_to, // todo. rename to generic config
    };

    let status_string = fronting_status::format_fronting_status(&fronting_format, fronts);

    eprintln!("Setting Discord Status: {status_string}");

    // todo. set status

    Ok(())
}
