use crate::services::users::create_user;
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    id: Uuid,
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), StatusCode> {
    if payload.email.trim().is_empty() || payload.password.is_empty() || payload.password.len() < 12
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let id = create_user(&pool, &payload.email, &payload.password)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(RegisterResponse { id })))
}
