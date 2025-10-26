use axum::{Json, extract::State, response::IntoResponse};
use axum_extra::extract::Multipart;
use futures::TryStreamExt;
use http::StatusCode;
use serde_json::{Value, json};
use tokio_util::compat::TokioAsyncReadCompatExt;
use tokio_util::io::StreamReader;

use crate::{
    domain::{error::Result, models::exchange::ExchangeId},
    routes::AppState,
};

// Этот метод просто какой то пиздец, надо детально разобрать, особенно reader_tokio.compat();
pub async fn mexc_csv_handler(
    State(state): State<AppState>, mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    let Some(service) = state.registry.get(ExchangeId::Mexc) else {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "MEXC service not found" })),
        ));
    };

    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("file") {
            // у axum_extra::Field — into_stream()
            let bytes_stream =
                field.into_stream().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

            // Stream<Bytes> -> tokio::AsyncRead
            let reader_tokio = StreamReader::new(bytes_stream);

            let reader_futures = reader_tokio.compat();

            service.parse_csv(Box::new(reader_futures)).await?;

            return Ok((
                StatusCode::OK,
                Json(serde_json::json!({ "status": "CSV parsed successfully" })),
            ));
        }
    }

    Ok((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "file part not found" }))))
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
