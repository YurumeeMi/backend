mod services;
use crate::services::auth::register;
use axum::{Router, routing::get, routing::post};
use services::auth::{login, logout, me};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::PeerIpKeyExtractor,
};

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL has not been provided");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Connection Failed");

    let governor_conf = GovernorConfigBuilder::default()
        .key_extractor(PeerIpKeyExtractor)
        .per_second(12)
        .burst_size(5)
        .finish()
        .expect("Ey! I need a valid config");

    let app: Router<_> = Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(register))
        .route(
            "/auth/login",
            axum::routing::post(login)
                .layer(GovernorLayer::new(std::sync::Arc::new(governor_conf))),
        )
        .route("/auth/me", get(me))
        .route("/auth/logout", post(logout))
        .with_state::<()>(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to get the port;");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Servering Failed");
}

async fn health() -> &'static str {
    "I am Always Up Master!"
}
