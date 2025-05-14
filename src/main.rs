#[macro_use] extern crate rocket;

mod config;
mod simply_plural;
mod vrchat;
mod webserver;

use anyhow::Result;
use tokio;


#[tokio::main]
async fn main() -> Result<()> {

    eprintln!("Starting Simply Plural to Any Updater...");

    let config = config::setup_and_load_config().await?;

    if config.serve_api {
        eprintln!("Running in Webserver mode.");
        webserver::run_server(&config).await?;
    } else {
        eprintln!("Running in VRChat Updater mode.");
        vrchat::run_updater_loop(&config).await?;
    }

    Ok(())
}
