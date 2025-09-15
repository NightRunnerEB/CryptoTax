use std::sync::Arc;

use axum::{Extension, Json};
use serde_json::{Value, json};

use crate::{
    auth_core::{
        domain::ports::AccessTokenIssuer,
        errors::AuthError,
        models::{AccessClaims, LoginResult, Tokens},
    },
    routes::{extractors::BearerAuth, *},
};

pub async fn register_handler(
    Extension(uc): Extension<Arc<UC>>,
    Json(reg): Json<RegisterReq>,
) -> Result<Json<Value>, AuthError> {
    uc.register(&reg.email, &reg.password).await?;
    Ok(Json(json!({ "Ok": true })))
}

pub async fn login_handler(
    Extension(uc): Extension<Arc<UC>>,
    Json(req): Json<LoginReq>,
) -> Result<Json<LoginResult>, AuthError> {
    let out = uc.login(&req.email, &req.password, req.ip.clone(), req.ua.clone()).await?;
    Ok(Json(out))
}

pub async fn refresh_handler(
    Extension(uc): Extension<Arc<UC>>,
    Json(req): Json<RefreshReq>,
) -> Result<Json<Tokens>, AuthError> {
    let out = uc.refresh(&req.refresh_token).await?;
    Ok(Json(out))
}

pub async fn logout_handler(
    Extension(uc): Extension<Arc<UC>>,
    BearerAuth(token): BearerAuth,
) -> Result<Json<serde_json::Value>, AuthError> {
    let claims: AccessClaims = uc.access.validate(token.as_str())?;
    uc.logout(&claims).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn verify_email_handler(
    Extension(uc): Extension<Arc<UC>>,
    Json(req): Json<VerifyEmailReq>,
) -> Result<Json<serde_json::Value>, AuthError> {
    uc.verify_email(&req.token).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
