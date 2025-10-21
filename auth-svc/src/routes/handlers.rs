use axum::{
    Json,
    extract::{Query, State},
};
use serde_json::{Value, json};

use crate::{
    auth_core::{
        errors::AuthError,
        models::{LoginResult, Tokens},
    },
    routes::{extractors::BearerAuth, *},
};

pub async fn register_handler(State(auth): State<Auth>, Json(reg): Json<RegisterReq>) -> Result<Json<Value>, AuthError> {
    auth.register(&reg.email, &reg.password).await?;
    Ok(Json(json!({ "Ok": true })))
}

pub async fn login_handler(State(auth): State<Auth>, Json(req): Json<LoginReq>) -> Result<Json<LoginResult>, AuthError> {
    let out = auth.login(&req.email, &req.password, req.ip, req.ua).await?;
    Ok(Json(out))
}

pub async fn refresh_handler(State(auth): State<Auth>, Json(req): Json<RefreshReq>) -> Result<Json<Tokens>, AuthError> {
    let out = auth.refresh(&req.refresh_token).await?;
    Ok(Json(out))
}

pub async fn logout_handler(State(auth): State<Auth>, BearerAuth(token): BearerAuth) -> Result<Json<serde_json::Value>, AuthError> {
    auth.logout(&token).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn verify_email_handler(State(auth): State<Auth>, Query(req): Query<VerifyEmailReq>) -> Result<Json<serde_json::Value>, AuthError> {
    auth.verify_email(&req.token).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn health_handler(State(_state): State<Auth>) -> Result<Json<Value>, AuthError> {
    Ok(Json(json!({ "status": "im alive" })))
}
