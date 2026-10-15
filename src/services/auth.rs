use crate::services::password::{hash_password, password_verify};
use crate::services::users::{
    create_user, find_password_hash_by_id, find_usr_in_email, update_password_hash,
};
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

fn bearer_token(headers: &axum::http::HeaderMap) -> Result<&str, StatusCode> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = value
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(token)
}

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

    let id = create_user(
        &pool,
        &payload.email.trim().to_lowercase(),
        &payload.password,
    )
    .await
    .map_err(|err| {
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.is_unique_violation() {
                return StatusCode::CONFLICT;
            }
        }
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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

    let users = find_usr_in_email(&pool, &payload.email.trim().to_lowercase())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some((id, stored_hash)) = users else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    password_verify(&payload.password, &stored_hash).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let token = super::sessions::create_session(&pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, Json(LoginResponse { id, token })))
}

pub async fn me(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let token = bearer_token(&headers)?;

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
    let token = bearer_token(&headers)?;

    super::sessions::delete_session(&pool, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn change_pass(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<StatusCode, StatusCode> {
    if payload.new_password.len() < 12 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let token = bearer_token(&headers)?;

    let user_id = super::sessions::find_session_user(&pool, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let stored_hash = find_password_hash_by_id(&pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    password_verify(&payload.current_password, &stored_hash)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let new_hash =
        hash_password(&payload.new_password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    update_password_hash(&pool, user_id, &new_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    super::sessions::delete_user_session(&pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
