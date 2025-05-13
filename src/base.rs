use std::env::var;

use reqwest::Client;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub sps_token: String,
    pub vrchat_username: String,
    pub vrchat_password: String,
    pub sps_base_url: String,
    pub client: Client,
    pub serve_api: bool,
    pub wait_seconds: u64,
    pub system_name: String,
    pub vrchat_updater_prefix: String,
    pub vrchat_updater_no_fronts: String,
    pub vrchat_updater_truncate_names_to: usize,
}




pub(crate) async fn load_config() -> Result<Config> {
    eprintln!("Loading environment variables...");
    let serve_api = str::eq(&var("SERVE_API").expect("SERVE_API not set."), "true");
    eprintln!("SERVE_API is {}", serve_api);

    let sps_token = var("SPS_API_TOKEN").expect("SPS_API_TOKEN not set");

    let optional_vrchat_config = if serve_api { Ok("".to_string()) } else { Err("VRChat variables needs configuration.") };
    
    let vrchat_username = optional_vrchat_config.clone().or(var("VRCHAT_USERNAME")).expect("VRCHAT_USERNAME not set");
    let vrchat_password = optional_vrchat_config.clone().or(var("VRCHAT_PASSWORD")).expect("VRCHAT_PASSWORD not set");
    eprintln!("Credentials loaded. VRCHAT_USERNAME is {}", vrchat_username);

    let system_name = var("SYSTEM_PUBLIC_NAME").expect("SYSTEM_PUBLIC_NAME not set.");

    let vrchat_updater_prefix = var("VRCHAT_UPDATER_PREFIX").expect("VRCHAT_UPDATER_PREFIX not set.");
    let vrchat_updater_no_fronts = var("VRCHAT_UPDATER_NO_FRONTS").expect("VRCHAT_UPDATER_NO_FRONTS not set.");
    let vrchat_updater_truncate_names_to = var("VRCHAT_UPDATER_TRUNCATE_NAMES_TO")
        .expect("VRCHAT_UPDATER_TRUNCATE_NAMES_TO not set.")
        .parse::<usize>()
        .unwrap();

    let wait_seconds = var("SECONDS_BETWEEN_UPDATES")
        .expect("SECONDS_BETWEEN_UPDATES not set.")
        .parse::<u64>()
        .unwrap();
    
    let sps_base_url = var("SPS_API_BASE_URL").expect("SPS_API_BASE_URL not set.");
    eprintln!("Using SPS base URL: {}", sps_base_url);

    // Build HTTP client
    let client = Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to build HTTP client");

    return Ok(Config{
        sps_token,
        vrchat_username,
        vrchat_password,
        vrchat_updater_prefix,
        vrchat_updater_no_fronts,
        vrchat_updater_truncate_names_to,
        sps_base_url,
        serve_api,
        system_name,
        wait_seconds,
        client,
    })
}

