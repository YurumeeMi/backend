use std::env;

use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL has not been provided");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Connection Failed");

    let result: i32 = 
    sqlx::query_scalar("SELECT 1")
    .fetch_one(&pool)
    .await
  
    .expect("query fetcgh failed");

    println!("database test result: {result}");

    let app: Router<_> = Router::new()
         .route("/health", get(health))
         .with_state::<()>(pool);
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .expect("Failed to get the port;");

    axum::serve(listener, app)
          .await
          .expect("Servering Failed");
}

async fn health() -> &'static str {
    "I am Always Up Master!"
}