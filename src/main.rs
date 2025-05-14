#[macro_use] extern crate rocket;

mod base;
mod simply_plural;
mod vrchat;
mod webserver;

use anyhow::Result;
use tokio;


#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("Starting VRChat SPS status updater...");

    let config = base::load_config().await?;

    if config.serve_api {
        webserver::run_server(&config).await?;
    } else {
        vrchat::run_updater_loop(&config).await?;
    }

    Ok(())
}


