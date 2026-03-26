use axum::{
    extract::FromRequestParts,
    http::{HeaderValue, header, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

use crate::auth_core::errors::AuthError;

pub struct BearerAuth(pub String);

fn unauthorized_invalid_token() -> Response {
    let mut resp = AuthError::TokenInvalid.into_response();
    let hdr = HeaderValue::from_static(r#"Bearer, error="invalid_token""#);
    resp.headers_mut().insert(header::WWW_AUTHENTICATE, hdr);
    resp
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for BearerAuth
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError::TokenInvalid.into_response())?;

        // предохранитель от совсем мусорных значений
        if bearer.token().len() > 16 * 1024 {
            return Err(unauthorized_invalid_token());
        }

        Ok(BearerAuth(bearer.token().to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        extract::FromRequestParts,
        http::{Request, StatusCode, header},
    };

    use super::BearerAuth;

    #[tokio::test]
    async fn extracts_bearer_token() {
        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, "Bearer token-123")
            .body(Body::empty())
            .expect("request must be built");
        let (mut parts, _) = req.into_parts();

        let extracted = BearerAuth::from_request_parts(&mut parts, &()).await.expect("token should be extracted");

        assert_eq!(extracted.0, "token-123");
    }

    #[tokio::test]
    async fn rejects_missing_authorization_header() {
        let req = Request::builder().uri("/").body(Body::empty()).expect("request must be built");
        let (mut parts, _) = req.into_parts();

        let rejection = match BearerAuth::from_request_parts(&mut parts, &()).await {
            Ok(_) => panic!("missing header must fail"),
            Err(rejection) => rejection,
        };

        assert_eq!(rejection.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_oversized_bearer_token() {
        let long_token = "a".repeat(16 * 1024 + 1);
        let auth_header = format!("Bearer {long_token}");

        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, auth_header)
            .body(Body::empty())
            .expect("request must be built");
        let (mut parts, _) = req.into_parts();

        let rejection = match BearerAuth::from_request_parts(&mut parts, &()).await {
            Ok(_) => panic!("oversized token must fail"),
            Err(rejection) => rejection,
        };

        assert_eq!(rejection.status(), StatusCode::UNAUTHORIZED);
        let www_auth = rejection.headers().get(header::WWW_AUTHENTICATE).and_then(|v| v.to_str().ok()).unwrap_or_default();
        assert!(www_auth.contains("invalid_token"), "expected invalid_token in WWW-Authenticate header");
    }
}
