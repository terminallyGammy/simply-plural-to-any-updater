use std::{fmt::Display, str::FromStr, time::Duration};

use serde::{Deserialize, Serialize};
use sqlx::{error::BoxDynError, postgres, types::Uuid, Decode, FromRow, Postgres};

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
pub struct UserProvidedPassword {
    pub inner: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, sqlx::Type)]
pub struct PasswordHashString {
    pub inner: String,
}
impl From<String> for PasswordHashString {
    fn from(val: String) -> Self {
        Self { inner: val }
    }
}

#[derive(Clone)]
pub struct UserSecretsKey {
    pub inner: String,
}

#[derive(Clone)]
pub struct ApplicationJwtSecret {
    pub inner: String,
}

#[derive(Clone)]
pub struct ApplicationUserSecrets {
    pub inner: String,
}

#[derive(Deserialize, Clone)]
pub struct UserLoginCredentials {
    pub email: Email,
    pub password: UserProvidedPassword,
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

#[derive(Default, Clone, Serialize, FromRow, PartialEq, Eq)]
pub struct EncryptedDbSecret {}

impl From<String> for EncryptedDbSecret {
    fn from(_: String) -> Self {
        Self {}
    }
}

// manual implementation of `Type<Postgres>` because derive doesn't work for non-newtype structs
impl sqlx::Type<Postgres> for EncryptedDbSecret {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as sqlx::Type<Postgres>>::compatible(ty)
    }
}

impl<'r> Decode<'r, Postgres> for EncryptedDbSecret {
    fn decode(value: postgres::PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let _ = <String as Decode<Postgres>>::decode(value)?;
        Ok(Self {})
    }
}

#[derive(Default, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct DecryptedDbSecret {
    pub secret: String,
}

// manual implementation of `Type<Postgres>` because derive doesn't work for non-newtype structs
impl sqlx::Type<Postgres> for DecryptedDbSecret {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as sqlx::Type<Postgres>>::compatible(ty)
    }
}

impl<'r> Decode<'r, Postgres> for DecryptedDbSecret {
    fn decode(value: postgres::PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let secret = <String as Decode<Postgres>>::decode(value)?;
        Ok(Self { secret })
    }
}

impl From<&str> for DecryptedDbSecret {
    fn from(value: &str) -> Self {
        Self {
            secret: value.to_string(),
        }
    }
}

impl From<String> for DecryptedDbSecret {
    fn from(secret: String) -> Self {
        Self { secret }
    }
}

pub trait SecretType: Default + Clone {}
impl SecretType for EncryptedDbSecret {}
impl SecretType for DecryptedDbSecret {}
