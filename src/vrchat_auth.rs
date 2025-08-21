use crate::config::UserConfigForUpdater;
use crate::database;

use anyhow::{anyhow, Result};
use either::Either;
use reqwest::cookie::{self, CookieStore};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use strum_macros::Display;
use vrchatapi::models::current_user::RequiresTwoFactorAuth;
use vrchatapi::{
    apis::{authentication_api, configuration::Configuration as VrcConfig},
    models as vrc,
};

const VRCHAT_UPDATER_USER_AGENT: &str = concat!(
    "SimplyPlural2Any/",
    env!("CARGO_PKG_VERSION"),
    " golly.ticker",
    "@",
    "gmail.com"
);

const VRCHAT_COOKIE_URL: &str = "https://api.vrchat.cloud";

/* Called in updater. Cookie is only validated, no new cookie is created. */
pub async fn authenticate_vrchat_with_cookie(
    config: &UserConfigForUpdater,
) -> Result<(VrcConfig, String)> {
    let creds: VRChatCredentialsWithCookie = (
        &config.vrchat_username,
        &config.vrchat_password,
        &config.vrchat_cookie,
    )
        .into();

    let (vrchat_config, _) =
        new_vrchat_config_with_basic_auth_and_optional_cookie(Either::Right(&creds))?;

    let () = match authentication_api::get_current_user(&vrchat_config).await? {
        vrc::EitherUserOrTwoFactor::CurrentUser(_me) => {
            eprintln!("VRChat Cookie valid for {}", config.user_id);
            Ok(())
        }
        vrc::EitherUserOrTwoFactor::RequiresTwoFactorAuth(_) => Err(anyhow!("Login failed")),
    }?;

    let user_id = get_vrchat_user_id(config, &vrchat_config).await?;

    Ok((vrchat_config, user_id))
}

pub async fn authenticate_vrchat_for_new_cookie(
    creds: VRChatCredentials,
) -> Result<Either<VRChatCredentialsWithCookie, TwoFactorAuthMethod>> {
    let (vrchat_config, cookie_store) =
        new_vrchat_config_with_basic_auth_and_optional_cookie(Either::Left(&creds))?;

    match authentication_api::get_current_user(&vrchat_config).await? {
        // User doesn't need 2fa
        vrc::EitherUserOrTwoFactor::CurrentUser(_me) => {
            let cookie = extract_new_cookie(&cookie_store)?;
            let creds_with_cookie = (&creds, cookie.as_str()).into();
            Ok(Either::Left(creds_with_cookie))
        }

        vrc::EitherUserOrTwoFactor::RequiresTwoFactorAuth(requires_auth) => {
            let auth_method = TwoFactorAuthMethod::from(&requires_auth);
            Ok(Either::Right(auth_method))
        }
    }
}

pub async fn authenticate_vrchat_for_new_cookie_with_2fa(
    creds_with_tfa: VRChatCredentialsWithTwoFactorAuth,
) -> Result<VRChatCredentialsWithCookie> {
    let (vrchat_config, cookie_store) =
        new_vrchat_config_with_basic_auth_and_optional_cookie(Either::Left(&creds_with_tfa.creds))?;

    let () = creds_with_tfa
        .method
        .vrchat_verify_2fa(creds_with_tfa.code, &vrchat_config)
        .await?;

    let cookie = extract_new_cookie(&cookie_store)?;

    Ok((&creds_with_tfa.creds, cookie.as_str()).into())
}

fn new_vrchat_config_with_basic_auth_and_optional_cookie(
    creds: Either<&VRChatCredentials, &VRChatCredentialsWithCookie>,
) -> Result<(VrcConfig, Arc<reqwest::cookie::Jar>)> {
    let cookie_store = Arc::new(cookie::Jar::default());
    let cookie_url = &Url::from_str(VRCHAT_COOKIE_URL)?;

    let username = creds.either(|c| &c.username, |c| &c.creds.username);
    let password = creds.either(|c| &c.password, |c| &c.creds.password);
    let cookie = creds.right().map(|c| &c.cookie);

    if let Some(cookie) = cookie {
        cookie_store.add_cookie_str(cookie, cookie_url);
    }

    let vrchat_config = VrcConfig {
        user_agent: Some(VRCHAT_UPDATER_USER_AGENT.to_string()),
        basic_auth: Some((username.clone(), Some(password.clone()))),
        client: reqwest::Client::builder()
            .cookie_provider(cookie_store.clone())
            .build()?,
        ..Default::default()
    };

    Ok((vrchat_config, cookie_store))
}

fn extract_new_cookie(cookie_store: &Arc<cookie::Jar>) -> Result<String> {
    let cookie_url = &Url::from_str(VRCHAT_COOKIE_URL)?;
    let cookie_value = cookie_store
        .cookies(cookie_url)
        .ok_or_else(|| anyhow!("no cookies"))?;
    Ok(cookie_value.to_str()?.to_owned())
}

async fn get_vrchat_user_id(
    config: &UserConfigForUpdater,
    vrchat_config: &VrcConfig,
) -> Result<String> {
    match authentication_api::get_current_user(vrchat_config).await? {
        vrc::EitherUserOrTwoFactor::CurrentUser(user) => Ok(user.id),
        vrc::EitherUserOrTwoFactor::RequiresTwoFactorAuth(_) => Err(anyhow!(
            "Cookie invalid for user {}",
            config.vrchat_username.secret
        )),
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct VRChatCredentials {
    pub username: String,
    pub password: String,
}

impl From<(&str, &str)> for VRChatCredentials {
    fn from((username, password): (&str, &str)) -> Self {
        Self {
            username: username.to_owned(),
            password: password.to_owned(),
        }
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct VRChatCredentialsWithCookie {
    pub creds: VRChatCredentials,
    pub cookie: String,
}

impl From<(&str, &str, &str)> for VRChatCredentialsWithCookie {
    fn from((username, password, cookie): (&str, &str, &str)) -> Self {
        Self {
            creds: (username, password).into(),
            cookie: cookie.to_owned(),
        }
    }
}

impl From<(&VRChatCredentials, &str)> for VRChatCredentialsWithCookie {
    fn from((creds, cookie): (&VRChatCredentials, &str)) -> Self {
        (creds.username.as_str(), creds.password.as_str(), cookie).into()
    }
}

impl
    From<(
        &database::Decrypted,
        &database::Decrypted,
        &database::Decrypted,
    )> for VRChatCredentialsWithCookie
{
    fn from(
        (username, password, cookie): (
            &database::Decrypted,
            &database::Decrypted,
            &database::Decrypted,
        ),
    ) -> Self {
        (
            username.secret.as_str(),
            password.secret.as_str(),
            cookie.secret.as_str(),
        )
            .into()
    }
}

#[derive(Clone, Serialize, Deserialize, Display, Debug)]
pub enum TwoFactorAuthMethod {
    TwoFactorAuthMethodEmail,
    TwoFactorAuthMethodApp,
}

impl TwoFactorAuthMethod {
    async fn vrchat_verify_2fa(
        &self,
        auth_code: TwoFactorAuthCode,
        vrchat_config: &VrcConfig,
    ) -> Result<()> {
        match self {
            Self::TwoFactorAuthMethodEmail => {
                authentication_api::verify2_fa_email_code(
                    vrchat_config,
                    vrc::TwoFactorEmailCode::new(auth_code.into()),
                )
                .await?;
            }
            Self::TwoFactorAuthMethodApp => {
                authentication_api::verify2_fa(
                    vrchat_config,
                    vrc::TwoFactorAuthCode::new(auth_code.into()),
                )
                .await?;
            }
        }

        Ok(())
    }

    fn from(requires_2fa_auth: &RequiresTwoFactorAuth) -> Self {
        let is_email_2fa = requires_2fa_auth
            .requires_two_factor_auth
            .contains(&String::from("emailOtp"));

        if is_email_2fa {
            Self::TwoFactorAuthMethodEmail
        } else {
            Self::TwoFactorAuthMethodApp
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TwoFactorAuthCode(String);

impl From<TwoFactorAuthCode> for String {
    fn from(val: TwoFactorAuthCode) -> Self {
        val.0
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VRChatCredentialsWithTwoFactorAuth {
    creds: VRChatCredentials,
    method: TwoFactorAuthMethod,
    code: TwoFactorAuthCode,
}
