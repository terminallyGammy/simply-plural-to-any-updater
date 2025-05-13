use crate::{base::Config, simply_plural::MemberContent};
use anyhow::{anyhow, Result};
use std::{
    io::{self, Write},
    thread, time,
};
use time::Duration;
use vrchatapi::{
    apis::{authentication_api, configuration::Configuration, users_api},
    models::{EitherUserOrTwoFactor, TwoFactorAuthCode, TwoFactorEmailCode, UpdateUserRequest},
};
use chrono;

const VRCHAT_UPDATER_USER_AGENT: &str = concat!(
    "SimplyPluralToVRChatUpdater/",
    env!("CARGO_PKG_VERSION"),
    " golly.ticker",
    "@",
    "gmail.com"
);
const VRCHAT_MAX_ALLOWED_STATUS_LENGTH: usize = 23;

pub async fn run_updater_loop(config: &Config, initial_fronts: Vec<MemberContent>) -> Result<()> {
    let (vrchat_config, user_id) = authenticate_vrchat(config).await?;

    // Initial update with already fetched fronts
    eprintln!("\n\n======================= UTC {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
    update_fronts_in_vrchat_status(&config, &vrchat_config, &user_id, initial_fronts).await?;
    eprintln!("Waiting {}s for next update trigger...", config.wait_seconds);
    thread::sleep(Duration::from_secs(config.wait_seconds));

    loop {
        eprintln!("\n\n======================= UTC {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));

        // Fetch new fronts for subsequent updates
        // Assuming fetch_fronts is now in simply_plural module and public
        let fronts = crate::simply_plural::fetch_fronts(&config).await?;
        
        update_fronts_in_vrchat_status(&config, &vrchat_config, &user_id, fronts).await?;
        
        eprintln!("Waiting {}s for next update trigger...", config.wait_seconds);
        thread::sleep(Duration::from_secs(config.wait_seconds));
    }
}

async fn update_fronts_in_vrchat_status(
    config: &Config,
    vrchat_config: &Configuration,
    user_id: &String,
    fronts: Vec<MemberContent>,
) -> Result<()> {
    let status_string = format_vrchat_status(config, fronts);

    let mut update_request = UpdateUserRequest::new();
    update_request.status_description = Some(status_string.clone());

    match users_api::update_user(vrchat_config, &user_id, Some(update_request)).await {
        Ok(_) => eprintln!("VRChat status updated successfully to: {}", status_string),
        Err(err) => eprintln!("VRChat status failed to be updated. Error: {}", err),
    }

    Ok(())
}

fn format_vrchat_status(config: &Config, fronts: Vec<MemberContent>) -> String {
    let fronter_strings: Vec<&str> = if fronts.is_empty() {
        vec![&config.vrchat_updater_no_fronts]
    } else {
        fronts.iter().map(|m| m.name.as_str()).collect()
    };
    eprintln!("Status string elements: {:?}", fronter_strings);

    let long_string = format!("{} {}", config.vrchat_updater_prefix.as_str(), fronter_strings.join(", "));
    let short_string = format!("{}{}", config.vrchat_updater_prefix.as_str(), fronter_strings.join(","));
    let truncated_string = {
        let prefix_slice = 0..config.vrchat_updater_truncate_names_to;
        let truncated_names: Vec<&str> = fronter_strings
            .iter()
            .map(|name| name.get(prefix_slice.clone()).unwrap_or_default())
            .collect();
        format!("{}{}", config.vrchat_updater_prefix.as_str(), truncated_names.join(","))
    };

    eprintln!("Long      string: '{}' ({})", long_string, long_string.len());
    eprintln!("Short     string: '{}' ({})", short_string, short_string.len());
    eprintln!("Truncated string: '{}' ({})", truncated_string, truncated_string.len());

    if long_string.len() <= VRCHAT_MAX_ALLOWED_STATUS_LENGTH { long_string }
    else if short_string.len() <= VRCHAT_MAX_ALLOWED_STATUS_LENGTH { short_string }
    else { truncated_string }
}

async fn authenticate_vrchat(config: &Config) -> Result<(Configuration, String)> {
    let mut vrchat_config = Configuration::default();
    vrchat_config.user_agent = Some(VRCHAT_UPDATER_USER_AGENT.to_string());
    vrchat_config.basic_auth = Some((config.vrchat_username.clone(), Some(config.vrchat_password.clone())));

    match authentication_api::get_current_user(&vrchat_config).await.unwrap() {
        EitherUserOrTwoFactor::CurrentUser(_me) => true,
        EitherUserOrTwoFactor::RequiresTwoFactorAuth(requires_auth) => {
            if requires_auth.requires_two_factor_auth.contains(&String::from("emailOtp")) {
                let code = read_user_input(&format!("Your account {} has received an Email with a 2FA code. Please enter it: ", config.vrchat_username));
                authentication_api::verify2_fa_email_code(&vrchat_config, TwoFactorEmailCode::new(code)).await?.verified
            } else {
                let code = read_user_input(&format!("Please enter your Authenticator 2FA code for the account {}:", config.vrchat_username));
                authentication_api::verify2_fa(&vrchat_config, TwoFactorAuthCode::new(code)).await?.verified
            }
        }
    };

    match authentication_api::get_current_user(&vrchat_config).await.unwrap() {
        EitherUserOrTwoFactor::CurrentUser(user) => Ok((vrchat_config, user.id)),
        EitherUserOrTwoFactor::RequiresTwoFactorAuth(_) => Err(anyhow!("Cookie invalid for user {}", config.vrchat_username)),
    }
}

fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}
