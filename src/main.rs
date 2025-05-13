#[macro_use] extern crate rocket;

// Modules
mod base;
mod simply_plural;
mod vrchat;
mod webserver;

use anyhow::Result;
use tokio;

// Imports from local modules
use base::load_config;
use simply_plural::fetch_fronts;

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("Starting VRChat SPS status updater...");

    let config = load_config().await?; // From base module

    if config.serve_api {
        webserver::run_server(&config).await?;
    } else {
        // Perform an initial fetch of fronts before starting the loop
        // This avoids an extra fetch inside the loop on the first iteration if not needed
        // or allows passing initial data if the loop expects it.
        // For simplicity here, we'll let the loop do its first fetch.
        // If `run_updater_loop` needs initial fronts, fetch them here:
        let initial_fronts = fetch_fronts(&config).await?;
        vrchat::run_updater_loop(&config, initial_fronts).await?;
    }

    Ok(())
}


