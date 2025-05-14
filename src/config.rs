
use std::env::{self, var}; // Added `env` for `set_var`
use std::fs;               // Added for file operations
use std::path::Path;        // Added for path manipulation
use std::process;         // Added for `exit`


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

const DEFAULTS_ENV_URL: &str = "https://raw.githubusercontent.com/GollyTicker/simply-plural-to-any-updater/main/defaults.env";
const VRCUPDATER_SAMPLE_ENV_URL: &str = "https://raw.githubusercontent.com/GollyTicker/simply-plural-to-any-updater/main/vrcupdater.sample.env";


pub(crate) async fn setup_and_load_config() -> Result<Config> {

    let client = Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to build HTTP client");

    // This will run the VRChat updater specific setup if SERVE_API is not already 'true'.
    // It might exit if it creates a sample .env file for the user to edit.
    initialize_environment_for_updater(&client).await?;

    eprintln!("Loading environment variables...");
    let serve_api = str::eq(&var("SERVE_API").expect("SERVE_API not set."), "true");
    eprintln!("SERVE_API is {}", serve_api);

    let sps_token = var("SPS_API_TOKEN").expect("SPS_API_TOKEN not set");

    let optional_vrchat_config = if serve_api { Ok("".to_string()) } else { Err("VRChat variables needs configuration.") };
    
    let vrchat_username = optional_vrchat_config.clone().or(var("VRCHAT_USERNAME")).expect("VRCHAT_USERNAME not set");
    let vrchat_password = optional_vrchat_config.clone().or(var("VRCHAT_PASSWORD")).expect("VRCHAT_PASSWORD not set");
    eprintln!("Credentials loaded. VRCHAT_USERNAME is {}", vrchat_username);

    let system_name = if serve_api { var("SYSTEM_PUBLIC_NAME").expect("SYSTEM_PUBLIC_NAME not set.") } else { "".to_string() }; 

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


/// Sets up environment variables based on remote and local files for VRChat updater mode.
pub async fn initialize_environment_for_updater(client: &Client) -> Result<()> {
    // Only run this setup if SERVE_API is not explicitly true from the environment already
    let serve_api = str::eq(&var("SERVE_API").unwrap_or("false".to_owned()), "true");
    if serve_api {
        eprintln!("SERVE_API is true, skipping VRChat updater specific environment setup.");
        return Ok(());
    }
    eprintln!("Running VRChat updater specific environment setup...");

    load_remote_defaults_env(client).await?;

    load_vrcupdater_env_or_create_for_user_and_exit(client).await?;
    
    eprintln!("VRChat updater environment setup complete.");
    Ok(())
}

/// Loads default environment variables from the remote defaults.env file.
/// This is part of the VRChat updater specific environment setup.
async fn load_remote_defaults_env(client: &Client) -> Result<()> {
    let defaults_env_content = download_file_content_for_setup(DEFAULTS_ENV_URL, client).await?;
    load_env_vars_from_string(&defaults_env_content, "defaults.env (remote)");
    Ok(())
}

/// Handles the local vrcupdater.env file for VRChat updater mode.
/// If it exists, it's loaded. If not, it's created from a remote sample,
/// and the program exits, prompting the user to configure it.
async fn load_vrcupdater_env_or_create_for_user_and_exit(client: &Client) -> Result<()> {
    let vrc_env_path_str = "vrcupdater.env";
    let vrc_env_path = Path::new(vrc_env_path_str);

    if vrc_env_path.exists() {
        let content = fs::read_to_string(vrc_env_path)?;
        load_env_vars_from_string(&content, vrc_env_path_str);
        eprintln!("Using local {}...", vrc_env_path_str);
    } else {
        eprintln!("{} not found. Creating sample environment file...", vrc_env_path_str);
        let sample_content = download_file_content_for_setup(VRCUPDATER_SAMPLE_ENV_URL, client).await?;
        fs::write(vrc_env_path, sample_content)?;
        eprintln!(
            "\n\n\n######### IMPORTANT #########\n\
            Configuration file '{}' has been created.\n\
            Please edit it with a simple text editor and\n\
            enter the SimplyPlural and VRChat credentials.\n\
            The file explains how to get these.\n\
            Please, run the application again then.",
            vrc_env_path_str
        );
        process::exit(0); // Exit successfully for user to configure
    }
    Ok(())
}


// Helper function to parse a string containing KEY=VALUE pairs and set them as environment variables.
fn load_env_vars_from_string(content: &str, source_name: &str) {
    eprintln!("Loading environment variables from {}...", source_name);
    for item in dotenvy::Iter::new(content.as_bytes()).filter_map(Result::ok) {
        env::set_var(item.0.clone(), item.1.clone());
    }
}


// Helper function to download file content specifically for setup
async fn download_file_content_for_setup(url: &str, client: &reqwest::Client) -> Result<String> {
    let response = client.get(url).send().await?.error_for_status()?;
    let content = response.text().await?;
    eprintln!("Downloaded for setup: {}", url);
    Ok(content)
}
