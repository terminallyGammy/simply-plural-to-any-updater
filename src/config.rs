use anyhow::{anyhow, Error, Result};
use reqwest::Client;
use std::time;
use time::Duration;

use crate::config_store::{self, CliArgs, LocalConfigV2Field, LocalJsonConfigV2};

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub client: Client,
    // Note: Keep this in sync with config_store::LocalJsonConfigV2 !
    pub wait_seconds: Duration,
    pub system_name: String,
    pub simply_plural_token: String,
    pub simply_plural_base_url: String,
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
        .with_defaults_from(config_store::default_config());

    let system_name = if cli_args.webserver {
        local_config_with_defaults.get_string_field(&LocalConfigV2Field::SystemName)?
    } else {
        String::new()
    };
    #[allow(clippy::or_fun_call)]
    let wait_seconds = local_config_with_defaults
        .wait_seconds
        .ok_or(field_err(&LocalConfigV2Field::WaitSeconds))?;

    let simply_plural_token =
        local_config_with_defaults.get_string_field(&LocalConfigV2Field::SimplyPluralToken)?;
    let simply_plural_base_url =
        local_config_with_defaults.get_string_field(&LocalConfigV2Field::SimplyPluralBaseUrl)?;
    eprintln!("Using Simply Plural Base URL: {simply_plural_base_url}");

    let needs_vrchat_config = !cli_args.webserver;

    let vrchat_username = if needs_vrchat_config {
        local_config_with_defaults.get_string_field(&LocalConfigV2Field::VrchatUsername)?
    } else {
        String::new()
    };
    let vrchat_password = if needs_vrchat_config {
        local_config_with_defaults.get_string_field(&LocalConfigV2Field::VrchatPassword)?
    } else {
        String::new()
    };
    eprintln!("Credentials loaded. VRChat Username is '{vrchat_username}'");

    let vrchat_cookie = local_config_with_defaults
        .get_string_field(&LocalConfigV2Field::VrchatCookie)
        .unwrap_or_default();
    if !vrchat_cookie.is_empty() {
        eprintln!("A VRChat cookie was found and will be used.");
    }

    let vrchat_updater_prefix =
        local_config_with_defaults.get_string_field(&LocalConfigV2Field::VrchatUpdaterPrefix)?;
    let vrchat_updater_no_fronts =
        local_config_with_defaults.get_string_field(&LocalConfigV2Field::VrchatUpdaterNoFronts)?;
    #[allow(clippy::or_fun_call)]
    let vrchat_updater_truncate_names_to = local_config_with_defaults
        .vrchat_updater_truncate_names_to
        .ok_or(field_err(&LocalConfigV2Field::VrchatUpdaterTruncateNamesTo))?;

    Ok(Config {
        client,
        wait_seconds,
        system_name,
        simply_plural_token,
        simply_plural_base_url,
        vrchat_username,
        vrchat_password,
        vrchat_updater_prefix,
        vrchat_updater_no_fronts,
        vrchat_updater_truncate_names_to,
        vrchat_cookie,
        cli_args: cli_args.clone(),
    })
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

// todo... we should find a different way to do this. marcos would be better here.
impl LocalConfigV2Field {
    fn display(&self) -> String {
        match self {
            Self::WaitSeconds => "Wait X seconds between each update (advanced)".to_string(),
            Self::SystemName => "Name of your System".to_string(),
            Self::SimplyPluralToken => "SimplyPlural Access Token".to_string(),
            Self::SimplyPluralBaseUrl => "SimplyPlural Remote Base URL (advanced)".to_string(),
            Self::VrchatUsername => "VRChat Username".to_string(),
            Self::VrchatPassword => "VRChat Password".to_string(),
            Self::VrchatCookie => "VRChat Cookie (advanced)".to_string(),
            Self::VrchatUpdaterPrefix => "How the fronts status begins".to_string(),
            Self::VrchatUpdaterNoFronts => "What to show when there are no fronts".to_string(),
            Self::VrchatUpdaterTruncateNamesTo => {
                "The number of characters to truncate each fronter name if the status were to be too long".to_string()
            }
        }
    }
}

impl LocalJsonConfigV2 {
    // todo. could we perhaps make this into a macro?
    #[allow(clippy::or_fun_call)]
    fn get_string_field(&self, field: &LocalConfigV2Field) -> Result<String> {
        match field {
            LocalConfigV2Field::SystemName => self.system_name.clone().ok_or(field_err(field)),
            LocalConfigV2Field::SimplyPluralToken => {
                self.simply_plural_token.clone().ok_or(field_err(field))
            }
            LocalConfigV2Field::SimplyPluralBaseUrl => {
                self.simply_plural_base_url.clone().ok_or(field_err(field))
            }
            LocalConfigV2Field::VrchatUsername => {
                self.vrchat_username.clone().ok_or(field_err(field))
            }
            LocalConfigV2Field::VrchatPassword => {
                self.vrchat_password.clone().ok_or(field_err(field))
            }
            LocalConfigV2Field::VrchatCookie => self.vrchat_cookie.clone().ok_or(field_err(field)),
            LocalConfigV2Field::VrchatUpdaterPrefix => {
                self.vrchat_updater_prefix.clone().ok_or(field_err(field))
            }
            LocalConfigV2Field::VrchatUpdaterNoFronts => self
                .vrchat_updater_no_fronts
                .clone()
                .ok_or(field_err(field)),
            LocalConfigV2Field::WaitSeconds => unimplemented!(),
            LocalConfigV2Field::VrchatUpdaterTruncateNamesTo => unimplemented!(),
        }
    }

    fn with_defaults_from(&self, defaults: Self) -> Self {
        Self {
            wait_seconds: self.wait_seconds.or(defaults.wait_seconds),
            system_name: self.system_name.clone().or(defaults.system_name),
            simply_plural_token: self
                .simply_plural_token
                .clone()
                .or(defaults.simply_plural_token),
            simply_plural_base_url: self
                .simply_plural_base_url
                .clone()
                .or(defaults.simply_plural_base_url),
            vrchat_username: self.vrchat_username.clone().or(defaults.vrchat_username),
            vrchat_password: self.vrchat_password.clone().or(defaults.vrchat_password),
            vrchat_updater_prefix: self
                .vrchat_updater_prefix
                .clone()
                .or(defaults.vrchat_updater_prefix),
            vrchat_updater_no_fronts: self
                .vrchat_updater_no_fronts
                .clone()
                .or(defaults.vrchat_updater_no_fronts),
            vrchat_updater_truncate_names_to: self
                .vrchat_updater_truncate_names_to
                .or(defaults.vrchat_updater_truncate_names_to),
            vrchat_cookie: self.vrchat_cookie.clone().or(defaults.vrchat_cookie),
        }
    }
}

fn field_err(config_field: &LocalConfigV2Field) -> Error {
    anyhow!(format!(
        "Mandatory field undefined or invalid: '{}'",
        config_field.display()
    ))
}
