use crate::services::password::hash_password;
use crate::services::users::find_usr_in_email;
use axum::{Json, extract::State, http::StatusCode};
use rand_core::{OsRng, TryRngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Error, PgPool};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

pub fn gen_rst_token() -> Result<String, std::io::Error> {
    let mut bytes = [0u8; 32];

    OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(std::io::Error::other)?;

    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

pub fn hash_reset_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());

    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub async fn store_reset_token(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO password_reset_tokens
        (id, user_id, token_hash, expires_at)
        VALUES ($1, $2, $3, NOW() + INTERVAL '15 minutes')",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(token_hash)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn invalid_user_rst_tokens(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM password_reset_tokens
        WHERE user_id = $1 AND used_at IS NULL",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn replace_rst_token(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "DELETE FROM password_reset_tokens
        WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO password_reset_tokens 
        (id, user_id, token_hash, expires_at)
        VALUES ($1, $2, $3, NOW + INTERVAL '15 minutes')",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(token_hash)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn find_valid_reset_token(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id
              FROM password_reset_tokens
              WHERE token_hash = $1
              AND expires_at > NOW()
              AND used_at IS NULL",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
}

pub async fn reset_password(
    pool: &PgPool,
    token_hash: &str,
    new_password: &str,
) -> Result<bool, Error> {
    let password_hash = hash_password(new_password)
        .map_err(|_| Error::Protocol("Password hashing failed".into()))?;

    let mut tx = pool.begin().await?;

    let user_id = sqlx::query_scalar::<_, Uuid>(
        "UPDATE password_reset_tokens
              SET used_at NOW()
              WHERE token_hash = $1
              AND expires_at > NOW()
              RETURNING user_id",
    )
    .bind(token_hash)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(user_id) = user_id else {
        return Ok(false);
    };

    sqlx::query("UPDATE user SET password_hash = $1 WHERE id = $2")
        .bind(password_hash)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    Ok(true)
}

pub async fn forgot_password(
    State(pool): State<PgPool>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<(StatusCode, Json<ForgotPasswordResponse>), StatusCode> {
    let email = payload.email.trim().to_lowercase();

    if email.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let user = find_usr_in_email(&pool, &email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some((user_id, _)) = user {
        let token = gen_rst_token().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let token_hash = hash_reset_token(&token);

        replace_rst_token(&pool, user_id, &token_hash)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // TODO: Send the reset link by email when SMTP is configured.
        // Never return the raw token in the API response.
    }

    Ok((
        StatusCode::ACCEPTED,
        Json(ForgotPasswordResponse {
            message: "If an account exists for that email, reset instructions will be sent.".into(),
        }),
    ))
}

pub async fn rst_pass_hand(
    State(pool): State<PgPool>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<StatusCode, StatusCode> {
    if payload.token.trim().is_empty() || payload.new_password.len() < 12 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let token_hash = hash_reset_token(&payload.token);

    let reset = reset_password(&pool, &token_hash, &payload.new_password)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !reset {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(StatusCode::NO_CONTENT)
}
