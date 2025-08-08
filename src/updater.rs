/* WORK-IN-PROGRESS */

use std::thread;

use crate::{config::Config, discord, simply_plural, vrchat, vrchat_auth};
use anyhow::{Ok, Result};
use chrono::Utc;
use serde::Serialize;

pub async fn run_loop(config: &Config) -> Result<()> {
    eprintln!("Running VRChat Updater ...");

    let (vrchat_config, user_id) = vrchat_auth::authenticate_vrchat(config).await?;

    loop {
        eprintln!(
            "\n\n======================= UTC {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        log_error_and_continue(
            "Updater Logic",
            loop_logic(config, &vrchat_config, &user_id).await,
        );

        eprintln!(
            "Waiting {}s for next update trigger...",
            config.wait_seconds.as_secs()
        );

        thread::sleep(config.wait_seconds);
    }
}

async fn loop_logic(
    config: &Config,
    vrchat_config: &vrchatapi::apis::configuration::Configuration,
    user_id: &str,
) -> Result<()> {
    let fronts = simply_plural::fetch_fronts(config).await?;

    log_error_and_continue(
        "VRChat",
        vrchat::update_to_vrchat(config, vrchat_config, &fronts, user_id).await,
    );

    log_error_and_continue("Discord", discord::update_to_discord(config, &fronts).await);

    Ok(())
}

fn log_error_and_continue(loop_part_name: &str, res: Result<()>) {
    match res {
        core::result::Result::Ok(()) => {}
        Err(err) => {
            eprintln!("Error in {loop_part_name}. Skipping update. Error: {err}");
        }
    }
}

#[derive(Clone, Serialize)]
pub enum Platform {
    VRChat,
    Discord,
}

#[derive(Clone, Serialize)]
pub enum UpdaterStatus {
    Inactive,
    Paused,
    Running,
    Error,
    Unknown,
}
