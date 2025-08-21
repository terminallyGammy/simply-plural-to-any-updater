use crate::model::HttpResult;
use crate::platforms::vrchat_auth;
use crate::users;
use either::Either;
use rocket::serde::json::Json;

#[post("/user/platform/vrchat/auth_2fa/request", data = "<creds>")]
pub async fn post_api_user_platform_vrchat_auth_2fa_request(
    creds: Json<vrchat_auth::VRChatCredentials>,
    _jwt: HttpResult<users::Jwt>, // request should be authenticated, but we don't need user id
) -> HttpResult<
    Json<Either<vrchat_auth::VRChatCredentialsWithCookie, vrchat_auth::TwoFactorAuthMethod>>,
> {
    let creds = creds.into_inner();

    let creds_or_tfa_method = vrchat_auth::authenticate_vrchat_for_new_cookie(creds).await?;

    Ok(Json(creds_or_tfa_method))
}

#[post("/user/platform/vrchat/auth_2fa/resolve", data = "<creds_with_tfa>")]
pub async fn post_api_user_platform_vrchat_auth_2fa_resolve(
    _jwt: HttpResult<users::Jwt>, // request should be authenticated, but we don't need user id
    creds_with_tfa: Json<vrchat_auth::VRChatCredentialsWithTwoFactorAuth>,
) -> HttpResult<Json<vrchat_auth::VRChatCredentialsWithCookie>> {
    let creds_with_tfa = creds_with_tfa.into_inner();

    let valid_creds =
        vrchat_auth::authenticate_vrchat_for_new_cookie_with_2fa(creds_with_tfa).await?;

    Ok(Json(valid_creds))
}
