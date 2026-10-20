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

pub async fn list_user_org(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<(Uuid, String, String)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String, String)>(
        "SELECT o.id, o.name, m.role
         FROM organizations o
         JOIN memberships m ON m.organization_id = o.id
         WHERE m.user_id = $1
         ORDER BY o.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn get_user_org(
    pool: &PgPool,
    user_id: Uuid,
    organization_id: Uuid,
) -> Result<Option<(Uuid, String, String)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String, String)>(
        "SELECT o.id, o.name, m.role
        FROM organizations o
        JOIN memberships m ON m.organization_id = o.id
        WHERE m.user_id = $1
        AND o.id = $2",
    )
    .bind(user_id)
    .bind(organization_id)
    .fetch_optional(pool)
    .await
}

pub async fn add_mem(
    pool: &PgPool,
    organization_id: Uuid,
    user_id: Uuid,
    role: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO memberships (organization_id, user_id, role)
        VALUES ($1, $2, $3)",
    )
    .bind(organization_id)
    .bind(user_id)
    .bind(role)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_mem_role(
    pool: &PgPool,
    organization_id: Uuid,
    user_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT role
     FROM memberships
     WHERE organization_id = $1
     AND user_id = $2",
    )
    .bind(organization_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}
