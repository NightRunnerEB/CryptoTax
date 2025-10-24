// use axum::{
//     extract::FromRequestParts,
//     http::{HeaderValue, header, request::Parts},
//     response::{IntoResponse, Response},
// };
// use axum_extra::{
//     TypedHeader,
//     headers::{Authorization, authorization::Bearer},
// };

// pub struct BearerAuth(pub String);

// fn unauthorized_invalid_token() -> Response {
//     let mut resp = AuthError::TokenInvalid.into_response();
//     let hdr = HeaderValue::from_static(r#"Bearer, error="invalid_token""#);
//     resp.headers_mut().insert(header::WWW_AUTHENTICATE, hdr);
//     resp
// }

// #[axum::async_trait]
// impl<S> FromRequestParts<S> for BearerAuth
// where
//     S: Send + Sync,
// {
//     type Rejection = Response;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         let TypedHeader(Authorization(bearer)) =
//             TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await.map_err(|_| AuthError::TokenInvalid.into_response())?;

//         // предохранитель от совсем мусорных значений
//         if bearer.token().len() > 16 * 1024 {
//             return Err(unauthorized_invalid_token());
//         }

//         Ok(BearerAuth(bearer.token().to_owned()))
//     }
// }
