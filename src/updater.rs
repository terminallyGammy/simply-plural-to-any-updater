use std::thread;

use crate::{config::Config, discord, vrchat, vrchat_auth};
use anyhow::{Ok, Result};
use chrono::Utc;


pub async fn run_loop(config: &Config) -> Result<()> {
    eprintln!("Running VRChat Updater ...");

    let (vrchat_config, user_id) = vrchat_auth::authenticate_vrchat(config).await?;

    loop {
        eprintln!(
            "\n\n======================= UTC {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        #[allow(clippy::or_fun_call)]
        vrchat::updater_loop_logic(config, &vrchat_config, &user_id)
            .await
            .inspect_err(|err| {
                eprintln!("Error in VRChat Updater Loop. Skipping update. Error: {err}");
            })
            .or(Ok(()))?;

        discord::updater_loop_login(config)
            .await
            .inspect_err(|err| {
                eprintln!("Error in Discord Updater Loop. Skipping update. Error: {err}");
            })
            .or(Ok(()))?;

        eprintln!(
            "Waiting {}s for next update trigger...",
            config.wait_seconds.as_secs()
        );

        thread::sleep(config.wait_seconds);
    }
}
