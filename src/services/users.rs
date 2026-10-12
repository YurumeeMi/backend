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
