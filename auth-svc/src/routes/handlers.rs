use axum::{
    Json,
    extract::{Query, State},
};
use serde_json::{Value, json};

use crate::{
    auth_core::{
        errors::Result,
        models::{LoginResult, Tokens},
    },
    routes::{
        Auth,
        dto::{LoginReq, RefreshReq, RegisterReq, VerifyEmailReq},
        extractors::BearerAuth,
    },
};

pub async fn register_handler(State(auth): State<Auth>, Json(reg): Json<RegisterReq>) -> Result<Json<Value>> {
    auth.register(&reg.email, &reg.password, &reg.tax_profile).await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn login_handler(State(auth): State<Auth>, Json(req): Json<LoginReq>) -> Result<Json<LoginResult>> {
    let out = auth.login(&req.email, &req.password, req.ip, req.ua).await?;
    Ok(Json(out))
}

pub async fn refresh_handler(State(auth): State<Auth>, Json(req): Json<RefreshReq>) -> Result<Json<Tokens>> {
    let out = auth.refresh(&req.refresh_token).await?;
    Ok(Json(out))
}

pub async fn logout_handler(State(auth): State<Auth>, BearerAuth(token): BearerAuth) -> Result<Json<Value>> {
    auth.logout(&token).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn verify_email_handler(State(auth): State<Auth>, Query(req): Query<VerifyEmailReq>) -> Result<Json<Value>> {
    auth.verify_email(&req.token).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn health_handler(State(_state): State<Auth>) -> Result<Json<Value>> {
    Ok(Json(json!({ "status": "im alive" })))
}
