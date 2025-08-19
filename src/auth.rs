use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use crate::{
    database::UserInfo,
    jwt,
    model::{ApplicationJwtSecret, PasswordHashString, UserProvidedPassword},
};

pub fn create_password_hash(password: &UserProvidedPassword) -> Result<PasswordHashString> {
    let salt = SaltString::generate(&mut OsRng);

    let pwh = Argon2::default()
        .hash_password(password.inner.as_bytes(), &salt)
        .map_err(|e| anyhow!(e))?;

    Ok(PasswordHashString {
        inner: pwh.to_string(),
    })
}

pub fn verify_password_and_create_token(
    password: &UserProvidedPassword,
    user_info: &UserInfo,
    jwt_secret: &ApplicationJwtSecret,
) -> Result<jwt::JwtString> {
    // don't allow external user to infer what exactly failed

    let pwh = PasswordHash::new(&user_info.password_hash.inner)
        .map_err(|_| anyhow!("Invalid email/password"))?;

    Argon2::default()
        .verify_password(password.inner.as_bytes(), &pwh)
        .map_err(|_| anyhow!("Invalid email/password"))?;

    let token = jwt::create_token(&user_info.id, jwt_secret)
        .map_err(|_| anyhow!("Invalid email/password"))?;

    Ok(token)
}
