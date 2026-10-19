use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_organization(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
) -> Result<Uuid, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let organization_id = Uuid::new_v4();

    sqlx::query("INSERT INTO organizations (id, name, created_by) VALUES ($1, $2, $3)")
        .bind(organization_id)
        .bind(name.trim())
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO memberships (organization_id, user_id, role) VALUES ($1, $2, 'owner')",
    )
    .bind(organization_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(organization_id)
}
