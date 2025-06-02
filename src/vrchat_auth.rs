use crate::config::{self, Config};
use anyhow::{anyhow, Result};
use reqwest::{cookie::{self, CookieStore}, Url};
use std::{
    io::{self, Write}, str::FromStr, sync::Arc
};
use vrchatapi::{
    apis::{authentication_api, configuration::Configuration},
    models::{EitherUserOrTwoFactor, TwoFactorAuthCode, TwoFactorEmailCode},
};

const VRCHAT_UPDATER_USER_AGENT: &str = concat!(
    "SimplyPluralToVRChatUpdater/",
    env!("CARGO_PKG_VERSION"),
    " golly.ticker",
    "@",
    "gmail.com"
);

const VRCHAT_COOKIE_URL: &str = "https://api.vrchat.cloud";

pub(crate) async fn authenticate_vrchat(config: &Config) -> Result<(Configuration, String)> {
    let cookie_store = Arc::new(cookie::Jar::default());

    let vrchat_config = new_vrchat_config_with_basic_auth_and_optional_cookie(config, &cookie_store);
    
    let new_cookie_received = authenticate_vrchat_user(config, &vrchat_config).await?;

    if new_cookie_received {
        store_new_cookie(&cookie_store).await?;
    }

    let user_id = get_vrchat_user_id(config, &vrchat_config).await?;

    Ok((vrchat_config, user_id))
}

fn new_vrchat_config_with_basic_auth_and_optional_cookie(config: &Config, cookie_store: &Arc<cookie::Jar>) -> Configuration {
    let mut vrchat_config = Configuration::default();
    
    vrchat_config.user_agent = Some(VRCHAT_UPDATER_USER_AGENT.to_string());
    vrchat_config.basic_auth = Some((config.vrchat_username.clone(), Some(config.vrchat_password.clone())));

    let cookie_url = &Url::from_str(VRCHAT_COOKIE_URL).unwrap();
    
    cookie_store.add_cookie_str(&config.vrchat_cookie, cookie_url);

    vrchat_config.client = reqwest::Client::builder()
        .cookie_provider(cookie_store.clone())
        .build()
        .unwrap();

    vrchat_config
}

async fn authenticate_vrchat_user(config: &Config, vrchat_config: &Configuration) -> Result<bool, anyhow::Error> {
    let new_cookie_recieved_due_to_2fa = 
        match authentication_api::get_current_user(vrchat_config).await.unwrap() {
            EitherUserOrTwoFactor::CurrentUser(_me) => false,

            EitherUserOrTwoFactor::RequiresTwoFactorAuth(requires_auth) => {
                // either cookie was empty or invalid. we mark the cookie as such then

                if requires_auth.requires_two_factor_auth.contains(&String::from("emailOtp")) {
                    let code = read_user_input(&format!("Your account {} has received an Email with a 2FA code. Please enter it: ", config.vrchat_username));
                    authentication_api::verify2_fa_email_code(vrchat_config, TwoFactorEmailCode::new(code)).await?;
                } else {
                    let code = read_user_input(&format!("Please enter your Authenticator 2FA code for the account {}:", config.vrchat_username));
                    authentication_api::verify2_fa(vrchat_config, TwoFactorAuthCode::new(code)).await?;
                }

                true
            }
        };

    Ok(new_cookie_recieved_due_to_2fa)
}

async fn store_new_cookie(cookie_store: &Arc<cookie::Jar>) -> Result<()> {
    let cookie_url = &Url::from_str(VRCHAT_COOKIE_URL).unwrap();
    let cookie_value = cookie_store.cookies(cookie_url).unwrap();
    config::store_vrchat_cookie(cookie_value.to_str().unwrap()).await
}

async fn get_vrchat_user_id(config: &Config, vrchat_config: &Configuration) -> Result<String> {
    match authentication_api::get_current_user(&vrchat_config).await.unwrap() {
        EitherUserOrTwoFactor::CurrentUser(user) => Ok(user.id),
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
