mod auth_core;
mod config;
mod db;
mod error;
mod infra;
mod routes;

use axum::{Router, routing::get};
use tokio::net::TcpListener;

use auth_core::AuthUseCases;

// pub type UC = AuthUseCases<
//     PgUserRepo,
//     // PgSessionRepo,
//     // PgRefreshRepo,
//     // Argon2Hasher,
//     // JwtIssuer,
//     // RefreshFactory,
//     CacheRevocation,
// >;

#[tokio::main]
    async fn main() {
    let server_address = "127.0.0.1:8085".to_string();
    let listener =
        TcpListener::bind(server_address).await.expect("unable to connect to the server");

    let routes = Router::new().route("/hello", get(|| async { "Hello, world!" }));

    axum::serve(listener, routes).await.expect("msg");
}
