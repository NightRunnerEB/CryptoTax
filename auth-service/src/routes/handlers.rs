use axum::{Json, extract::{Query, State}};
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
    State(state): State<AppState>,
    Json(reg): Json<RegisterReq>,
) -> Result<Json<Value>, AuthError> {
    state.register(&reg.email, &reg.password).await?;
    Ok(Json(json!({ "Ok": true })))
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> Result<Json<LoginResult>, AuthError> {
    let out = state.login(&req.email, &req.password, req.ip, req.ua).await?;
    Ok(Json(out))
}

pub async fn refresh_handler(
    State(state): State<AppState>,
    Json(req): Json<RefreshReq>,
) -> Result<Json<Tokens>, AuthError> {
    let out = state.refresh(&req.refresh_token).await?;
    Ok(Json(out))
}

pub async fn logout_handler(
    State(state): State<AppState>,
    BearerAuth(token): BearerAuth,
) -> Result<Json<serde_json::Value>, AuthError> {
    let claims: AccessClaims = state.access.validate(token.as_str())?;
    state.logout(&claims).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn verify_email_handler(
    State(state): State<AppState>,
    Query(req): Query<VerifyEmailReq>,
) -> Result<Json<serde_json::Value>, AuthError> {
    state.verify_email(&req.token).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn health_handler(State(_state): State<AppState>) -> Result<Json<Value>, AuthError> {
    Ok(Json(json!({ "status": "im alive" })))
}
