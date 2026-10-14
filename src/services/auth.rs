
use crate::services::password::password_verify;
use crate::services::users::{create_user, find_usr_in_email};
use axum::{extract::State, http::StatusCode, Json};
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
    if payload.email.trim().is_empty()
        || payload.password.is_empty()
        || payload.password.len() < 12
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let id = create_user(&pool, &payload.email, &payload.password)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(RegisterResponse { id })))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub id: Uuid,
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), StatusCode> {
    if payload.email.trim().is_empty() || payload.password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let users = find_usr_in_email(&pool, payload.email.trim())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some((id, stored_hash)) = users else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    password_verify(&payload.password, &stored_hash)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let token = super::sessions::create_session(&pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, Json(LoginResponse { id, token })))
}

pub async fn me(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) ->  Result<Json<RegisterResponse>, StatusCode> {
    let token = headers
    .get(axum::http::header::AUTHORIZATION)
    .and_then(|value| value.to_str().ok())
    .and_then(|value| value.strip_prefix("Bearer "))
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let id = super::sessions::find_session_user(&pool, token)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(Json(RegisterResponse { id }))
}
pub async fn logout(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let token = headers
    .get(axum::http::header::AUTHORIZATION)
    .and_then(|value| value.to_str().ok())
    .and_then(|value| value.strip_prefix("Bearer "))
    .ok_or(StatusCode::UNAUTHORIZED)?;

    super::sessions::delete_session(&pool, token)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
     
Ok(StatusCode::NO_CONTENT)
}