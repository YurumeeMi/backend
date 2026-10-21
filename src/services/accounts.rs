use sha2::digest::typenum::Or;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_account(
    pool: &PgPool,
    organization_id: Uuid,
    code: &str,
    name: &str,
    account_type: &str,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO accounts
         (id, organization_id, code, name, account_type)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(organization_id)
    .bind(code)
    .bind(name)
    .bind(account_type)
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn list_acconuts(
    pool: &PgPool,
    organization_id: Uuid,
) -> Result<Vec<(Uuid, String, String, String, bool)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String, String, String, bool)>(
        "SELECT id, code, name, account_type, is_active
    FROM accounts
    WHERE organization_id = $1
    ORDER BY code",
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await
}

pub async fn get_account(
    pool: &PgPool,
    organization_id: Uuid,
    account_id: Uuid,
) -> Result<Option<(Uuid, String, String, String, bool)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String, String, String, bool)>(
        "SELECT id, code, name, account_type, is_active
         FROM accounts
         WHERE organization_id = $1
           AND id = $2",
    )
    .bind(organization_id)
    .bind(account_id)
    .fetch_optional(pool)
    .await
}
