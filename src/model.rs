use std::{fmt::Display, str::FromStr, time::Duration};

use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, FromRow};

use crate::users;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, sqlx::Type)]
pub struct Email {
    pub inner: String,
}

impl From<String> for Email {
    fn from(val: String) -> Self {
        Self { inner: val }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, sqlx::Type, Eq, Hash, PartialEq)]
pub struct UserId {
    pub inner: Uuid,
}

impl From<Uuid> for UserId {
    fn from(val: Uuid) -> Self {
        Self { inner: val }
    }
}

impl TryFrom<&str> for UserId {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let uuid = Uuid::from_str(value)?;
        Ok(Self { inner: uuid })
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserId({})", self.inner)
    }
}

#[derive(Deserialize, Clone)]
pub struct UserLoginCredentials {
    pub email: Email,
    pub password: users::UserProvidedPassword,
}

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct WaitSeconds {
    pub inner: Duration,
}

impl From<Duration> for WaitSeconds {
    fn from(value: Duration) -> Self {
        Self { inner: value }
    }
}

impl From<i32> for WaitSeconds {
    #[allow(clippy::cast_sign_loss)]
    fn from(secs: i32) -> Self {
        Duration::from_secs(secs as u64).into()
    }
}

pub type HttpResult<T> = Result<T, rocket::response::Debug<anyhow::Error>>;
