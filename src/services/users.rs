use sqlx::PgPool;
use uuid::Uuid;

use crate::services::password::hash_password;

pub async fn create_user(pool: &PgPool, email: &str, password: &str) -> Result<Uuid, sqlx::Error> {
    let password_hash = hash_password(password)
        .map_err(|_| sqlx::Error::Protocol("Password Failed to Hash".into()))?;

    let user_id = Uuid::new_v4();

    let created_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO users (id, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id",
    )
    .bind(user_id)
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(created_id)
}

pub async fn find_usr_in_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(Uuid, String)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, password_hash
         FROM users
         WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}
pub async fn update_password_hash(
    pool: &PgPool,
    user_id: Uuid,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(password_hash)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn find_password_hash_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>("SELECT password_hash FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}
