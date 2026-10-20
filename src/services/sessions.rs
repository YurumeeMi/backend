use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_session(pool: &PgPool, user_id: Uuid) -> Result<String, sqlx::Error> {
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());

    let digest = Sha256::digest(token.as_bytes());
    let token_hash = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, expires_at)
         VALUES ($1, $2, $3, NOW() + INTERVAL '7 days')",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(token_hash)
    .execute(pool)
    .await?;

    Ok(token)
}
pub async fn find_session_user(pool: &PgPool, token: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let digest = Sha256::digest(token.as_bytes());
    let token_hash = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id
             FROM sessions
             WHERE token_hash = $1
             AND expires_at > NOW()",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
}
pub async fn delete_session(pool: &PgPool, token: &str) -> Result<(), sqlx::Error> {
    let digest = Sha256::digest(token.as_bytes());
    let token_hash = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    sqlx::query("DELETE FROM sessions WHERE  token_hash = $1")
        .bind(token_hash)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_expired_session(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM sessions WHERE expires_at <= NOW()")
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

pub async fn delete_user_session(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE user_id =$1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}
