use crate::config::UserConfigForUpdater;
use crate::model::DecryptedDbSecret;

use anyhow::{anyhow, Result};
use either::Either;
use reqwest::cookie::{self, CookieStore};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::future::Future;
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
    let cookie_store = Arc::new(cookie::Jar::default());

    let creds: VRChatCredentialsWithCookie = (
        &config.vrchat_username,
        &config.vrchat_password,
        &config.vrchat_cookie,
    )
        .into();

    let vrchat_config = new_vrchat_config_with_basic_auth_and_optional_cookie(
        Either::Right(&creds),
        &cookie_store,
    )?;

    let () = match authentication_api::get_current_user(&vrchat_config).await? {
        vrc::EitherUserOrTwoFactor::CurrentUser(_me) => {
            eprintln!("VRChat Cookie valid for {}", config.user_id);
            Ok(())
        }
        _ => Err(anyhow!("Login failed")),
    }?;

    let user_id = get_vrchat_user_id(config, &vrchat_config).await?;

    Ok((vrchat_config, user_id))
}

pub async fn authenticate_vrchat_for_new_cookie<AuthCodeFuture>(
    creds: VRChatCredentials,
    request_two_factor_auth_code: impl FnOnce(VRChatCredentials, TwoFactorAuthMethod) -> AuthCodeFuture,
) -> Result<VRChatCredentialsWithCookie>
where
    AuthCodeFuture: Future<Output = Result<TwoFactorAuthCode>>,
{
    let cookie_store = Arc::new(cookie::Jar::default());

    let vrchat_config =
        new_vrchat_config_with_basic_auth_and_optional_cookie(Either::Left(&creds), &cookie_store)?;

    authenticate_vrchat_user(&creds, &vrchat_config, request_two_factor_auth_code).await?;

    let cookie = extract_new_cookie(cookie_store)?;

    Ok((&creds, cookie.as_str()).into())
}

async fn authenticate_vrchat_user<AuthCodeFuture>(
    creds: &VRChatCredentials,
    vrchat_config: &VrcConfig,
    request_two_factor_auth_code: impl FnOnce(VRChatCredentials, TwoFactorAuthMethod) -> AuthCodeFuture,
) -> Result<()>
where
    AuthCodeFuture: Future<Output = Result<TwoFactorAuthCode>>,
{
    let authentication = authentication_api::get_current_user(vrchat_config).await?;

    match authentication {
        // User doesn't need 2fa
        vrc::EitherUserOrTwoFactor::CurrentUser(_me) => Ok(()),

        vrc::EitherUserOrTwoFactor::RequiresTwoFactorAuth(requires_auth) => {
            let auth_method = TwoFactorAuthMethod::from(requires_auth);

            let auth_code =
                request_two_factor_auth_code(creds.clone(), auth_method.clone()).await?;

            let () = auth_method
                .vrchat_verify_2fa(auth_code, vrchat_config)
                .await?;

            Ok(())
        }
    }
}

fn new_vrchat_config_with_basic_auth_and_optional_cookie(
    creds: Either<&VRChatCredentials, &VRChatCredentialsWithCookie>,
    cookie_store: &Arc<cookie::Jar>,
) -> Result<VrcConfig> {
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

    Ok(vrchat_config)
}

fn extract_new_cookie(cookie_store: Arc<cookie::Jar>) -> Result<String> {
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

impl From<(&DecryptedDbSecret, &DecryptedDbSecret, &DecryptedDbSecret)>
    for VRChatCredentialsWithCookie
{
    fn from(
        (username, password, cookie): (&DecryptedDbSecret, &DecryptedDbSecret, &DecryptedDbSecret),
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

    fn from(requires_2fa_auth: RequiresTwoFactorAuth) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compilation_with_async_move_future() -> Result<()> {
        let creds = ("user", "pwd").into();

        let request_two_factor_auth_code =
            |creds: VRChatCredentials, method: TwoFactorAuthMethod| async move {
                Ok(TwoFactorAuthCode(format!("{creds:?}: {method}(123456)")))
            };

        if false {
            authenticate_vrchat_for_new_cookie(creds, request_two_factor_auth_code).await?;
        }

        Ok(())
    }
}
