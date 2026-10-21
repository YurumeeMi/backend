mod services;
use crate::services::auth::{logout_all, register};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use services::auth::{
    add_mem_org, change_pass, create_account, create_org, get_acc, get_org, list_acc, list_org,
    login, logout, me,
};
use services::password_reset::{forgot_password, rst_pass_hand};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::PeerIpKeyExtractor,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL has not been provided");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Connection Failed");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("DB MIGRATIONS FAILED");

    let governor_conf = GovernorConfigBuilder::default()
        .key_extractor(PeerIpKeyExtractor)
        .per_second(12)
        .burst_size(5)
        .finish()
        .expect("Ey! I need a valid config");

    let reg_governor_conf = GovernorConfigBuilder::default()
        .key_extractor(PeerIpKeyExtractor)
        .per_second(60)
        .burst_size(3)
        .finish()
        .expect("Ey! I need a valid config");

    let for_governor_conf = GovernorConfigBuilder::default()
        .key_extractor(PeerIpKeyExtractor)
        .per_second(60)
        .burst_size(3)
        .finish()
        .expect("Ey! I need a valid config");

    let res_governor_conf = GovernorConfigBuilder::default()
        .key_extractor(PeerIpKeyExtractor)
        .per_second(60)
        .burst_size(3)
        .finish()
        .expect("Ey! I need a valid config");

    let app: Router<_> = Router::new()
        .route("/health", get(health))
        .route(
            "/auth/register",
            post(register).layer(GovernorLayer::new(std::sync::Arc::new(reg_governor_conf))),
        )
        .route(
            "/auth/login",
            axum::routing::post(login)
                .layer(GovernorLayer::new(std::sync::Arc::new(governor_conf))),
        )
        .route("/organizations", get(list_org).post(create_org))
        .route(
            "/organizations/{id}/accounts",
            get(list_acc).post(create_account),
        )
        .route("/organizations/{id}/accounts/{account_id}", get(get_acc),)
        .route("/organizations/{id}", get(get_org))
        .route("/organizations/{id}/members", post(add_mem_org))
        .route("/auth/me", get(me))
        .route("/auth/change-pass", post(change_pass))
        .route("/auth/logout-all", post(logout_all))
        .route("/auth/logout", post(logout))
        .route(
            "/auth/forgot-pass",
            post(forgot_password).layer(GovernorLayer::new(std::sync::Arc::new(for_governor_conf))),
        )
        .route(
            "/auth/reset-password",
            post(rst_pass_hand).layer(GovernorLayer::new(std::sync::Arc::new(res_governor_conf))),
        )
        .layer(DefaultBodyLimit::max(64 * 1024))
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
