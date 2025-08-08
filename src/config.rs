use anyhow::{anyhow,Result};
use reqwest::Client;
use std::time::Duration;

use crate::{config_store::{self, CliArgs}};
use crate::{value_of, value_of_if};

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub client: Client,
    // Note: Keep this in sync with config_store::LocalJsonConfigV2 !
    pub wait_seconds: Duration,
    pub system_name: String,
    pub simply_plural_token: String,
    pub simply_plural_base_url: String,
    pub discord_token: String,
    pub vrchat_username: String,
    pub vrchat_password: String,
    pub vrchat_updater_prefix: String,
    pub vrchat_updater_no_fronts: String,
    pub vrchat_updater_truncate_names_to: usize,
    pub vrchat_cookie: String,
    pub cli_args: CliArgs,
}

pub fn setup_and_load_config(cli_args: &CliArgs) -> Result<Config> {
    let client = Client::builder().cookie_store(true).build()?;

    eprintln!("Loading config ...");

    // This will run the VRChat updater specific setup if not in webserver mode.
    initialize_environment_for_updater(cli_args)?;

    let local_config_with_defaults = config_store::read_local_config_file(cli_args)?
        .with_defaults(config_store::default_config());

    let platform_updater_mode = !cli_args.webserver;

    let config = Config {
        client,
        wait_seconds: value_of!(local_config_with_defaults,wait_seconds)?,
        system_name: value_of_if!(cli_args.webserver, local_config_with_defaults, system_name)?,
        simply_plural_token: value_of!(local_config_with_defaults, simply_plural_token)?,
        simply_plural_base_url: value_of!(local_config_with_defaults, simply_plural_base_url)?,
        discord_token: value_of_if!(platform_updater_mode, local_config_with_defaults, discord_token)?,
        vrchat_username: value_of_if!(platform_updater_mode, local_config_with_defaults, vrchat_username)?,
        vrchat_password: value_of_if!(platform_updater_mode, local_config_with_defaults, vrchat_password)?,
        vrchat_updater_prefix: value_of!(local_config_with_defaults, vrchat_updater_prefix)?,
        vrchat_updater_no_fronts: value_of!(local_config_with_defaults, vrchat_updater_no_fronts)?,
        vrchat_updater_truncate_names_to: value_of!(local_config_with_defaults, vrchat_updater_truncate_names_to)?,
        vrchat_cookie: value_of!(local_config_with_defaults, vrchat_cookie)
            .inspect(|_|eprintln!("A VRChat cookie was found and will be used."))
            .unwrap_or(String::new()),
        cli_args: cli_args.clone(),
    };

    eprintln!("Credentials loaded. VRChat Username is '{}'", config.vrchat_username);

    Ok(config)
}

/// Sets up environment variables based on remote and local files for `VRChat` updater mode.
pub fn initialize_environment_for_updater(cli_args: &CliArgs) -> Result<()> {
    if cli_args.webserver {
        eprintln!("In webserver mode: Skipping VRChat updater specific environment setup.");
        return Ok(());
    }
    eprintln!("Running VRChat updater specific environment setup...");

    let _is_fresh_config = config_store::initialise_if_not_exists(cli_args)?;

    // todo. if fresh config, then ensure, that updater doesn't automatically start
    // todo. we need a global state which indicates if the updater should be running or not.

    eprintln!("VRChat updater environment setup complete.");
    Ok(())
}
