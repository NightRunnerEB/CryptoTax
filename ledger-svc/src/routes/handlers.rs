use axum::{Json, extract::State, response::IntoResponse};
use http::StatusCode;
use serde_json::{Value, json};

use crate::{
    domain::{error::Result, models::exchange::ExchangeId},
    routes::AppState,
};

pub async fn mexc_csv_handler(State(state): State<AppState>) -> Result<impl IntoResponse> {
    match state.registry.get(ExchangeId::Mexc) {
        Some(service) => {
            service.parse_csv().await?;
            Ok((StatusCode::OK, Json(json!({ "status": "CSV parsed successfully" }))))
        }
        None => Ok((StatusCode::NOT_FOUND, Json(json!({ "error": "MEXC service not found" })))),
    }
}

// pub async fn okx_handler(
//     State(state): State<AppState>,
//     Json(req): Json<LoginReq>,
// ) -> Result<Json<LoginResult>, AuthError> {
//     let out = state.login(&req.email, &req.password, req.ip, req.ua).await?;
//     Ok(Json(out))
// }

pub async fn health_handler(State(_state): State<AppState>) -> Result<Json<Value>> {
    Ok(Json(json!({ "status": "im alive" })))
}
