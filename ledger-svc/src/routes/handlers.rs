use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};
use axum_extra::extract::Multipart;
use futures::TryStreamExt;
use http::StatusCode;
use serde_json::{Value, json};
use tokio_util::{compat::TokioAsyncReadCompatExt, io::StreamReader};
use uuid::Uuid;

use crate::{
    domain::{error::Result, models::exchange::ExchangeId, models::utils::ParseContext},
    routes::AppState,
};

pub async fn mexc_csv_handler(
    State(state): State<AppState>, headers: HeaderMap, mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    let Some(service) = state.registry.get(ExchangeId::Mexc) else {
        return Ok((StatusCode::NOT_FOUND, Json(json!({ "error": "MEXC service not found" }))));
    };

    let mut file_part = None;
    while let Some(field) = multipart.next_field().await? {
        let is_named_file = field.name().map(|n| n == "file").unwrap_or(false);
        let has_filename = field.file_name().is_some();
        if is_named_file || has_filename {
            file_part = Some(field);
            break;
        }
    }

    let Some(field) = file_part else {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({ "error": "file part not found" }))));
    };

    let byte_stream = field.into_stream().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));

    let reader_tokio = StreamReader::new(byte_stream);
    let reader_futures = reader_tokio.compat();

    let tenant_id = headers
        .get("x-user-id")
        .or_else(|| headers.get("X-User-Id"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());

    let Some(tenant_id) = tenant_id else {
        return Ok((StatusCode::FORBIDDEN, Json(json!({ "error": "missing or invalid tenant id header" }))));
    };

    let ctx = ParseContext {
        tenant_id,
        import_id: Uuid::new_v4(),
        wallet: "MEXC".to_string(),
        file_name: None, // Нужно сделать подстановку имени файла
    };

    service.parse_csv(Box::new(reader_futures), ctx).await?;

    Ok((StatusCode::OK, Json(json!({ "status": "CSV parsed successfully" }))))
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
