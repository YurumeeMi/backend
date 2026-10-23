use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_journal_entry(
    pool: &PgPool,
    organization_id: Uuid,
    created_by: Uuid,
    entry_date: NaiveDate,
    description: &str,
    reference: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO journal_entries
        (id, organization_id, entry_date, description, reference, created_by)
        VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(organization_id)
    .bind(entry_date)
    .bind(description)
    .bind(reference)
    .bind(created_by)
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn create_journal_line(
    pool: &PgPool,
    organization_id: Uuid,
    journal_entry_id: Uuid,
    account_id: Uuid,
    description: Option<&str>,
    debit: sqlx::types::BigDecimal,
    credit: sqlx::types::BigDecimal,
) -> Result<Uuid, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO journal_lines
             (id, organization_id, journal_entry_id, account_id, description, debit, credit)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(id)
            .bind(organization_id)
            .bind(journal_entry_id)
            .bind(account_id)
            .bind(description)
            .bind(debit)
            .bind(credit)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(id)
}

pub async fn list_journal_lines(
    pool: &PgPool,
    organization_id: Uuid,
    journal_entry_id: Uuid,
) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
    
    sqlx::query(
        "SELECT id, account_id, description, debit, credit
        FROM journal_lines
        WHERE organization_id = $1
        AND journal_entry_id = $2
        ORDER BY created_at ASC, id ASC",)
        .bind(organization_id)
        .bind(journal_entry_id)
        .fetch_all(pool)
        .await
}

pub async fn post_journal_entry(
     pool: &PgPool,
    organization_id: Uuid,
    journal_entry_id: Uuid,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let status: Option<String> = sqlx::query_scalar(
        "SELECT status
              FROM journal_entries
              WHERE id = $1
              AND organization_id = $2
              FOR UPDATE",
    )
    .bind(journal_entry_id)
    .bind(organization_id)
    .fetch_optional(&mut *tx)
    .await?;

    match status.as_deref() {
        None => {
            return Err(sqlx::Error::RowNotFound);
        }
        Some("draft") => {}
        Some(_) => {
         return Err(sqlx::Error::Protocol(
            "Journal ent is not a daft".into(),
         ));          
        }
    }
    // We are HECKERSSSS MUHWHWHWWHWH

    let (debits, credits): (sqlx::types::BigDecimal,
    sqlx::types::BigDecimal) =
    sqlx::query_as(" SELECT
        COALESCE(SUM(debit), 0),
        COALESCE(SUM(credit), 0)
        FROM  journal_lines
        WHERE organization_id = $1
        AND journal_entry_id = $2",
    )
    .bind(organization_id)
    .bind(journal_entry_id)
    .fetch_one(&mut *tx)
    .await?;

    let zero = sqlx::types::BigDecimal::from(0);

    if debits <= zero || credits <= zero || debits != credits {
        return Err(sqlx::Error::Protocol(
            "Journal ent is unbalanced".into(),
        ));
    }

    sqlx::query(
        "UPDATE journal_entries
        SET status = 'posted', posted_at = NOW()
        WHERE id = $1
        AND organization_id = $2
        AND status = 'draft'",
    )
    .bind(journal_entry_id)
    .bind(organization_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
