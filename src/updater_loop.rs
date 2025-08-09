use std::thread;

use crate::{
    config::Config,
    simply_plural::{self},
    updater::{Platform, Updater},
};
use anyhow::{Ok, Result};
use chrono::Utc;

pub async fn run_loop(config: &Config) {
    eprintln!("Running Updater ...");

    let mut updaters = vec![
        Updater::new(Platform::VRChat),
        Updater::new(Platform::Discord),
    ];

    for u in updaters.as_mut_slice() {
        if u.enabled(config) {
            log_error_and_continue(&u.platform().to_string(), u.setup(config).await);
        }
    }

    loop {
        eprintln!(
            "\n\n======================= UTC {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        log_error_and_continue(
            "Updater Logic",
            loop_logic(config, updaters.as_mut_slice()).await,
        );

        eprintln!(
            "Waiting {}s for next update trigger...",
            config.wait_seconds.as_secs()
        );

        thread::sleep(config.wait_seconds);
    }
}

async fn loop_logic(config: &Config, updaters: &mut [Updater]) -> Result<()> {
    let fronts = simply_plural::fetch_fronts(config).await?;

    for updater in updaters {
        if updater.enabled(config) {
            log_error_and_continue(
                &updater.platform().to_string(),
                updater.update_fronting_status(config, &fronts).await,
            );
        }
    }

    Ok(())
}

fn log_error_and_continue(loop_part_name: &str, res: Result<()>) {
    match res {
        core::result::Result::Ok(()) => {}
        Err(err) => {
            eprintln!("Error in {loop_part_name}. Skipping. Error: {err}");
        }
    }
}
