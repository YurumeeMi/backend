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

    let entry_status: Option<String> = sqlx::query_scalar(
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

    match entry_status.as_deref() {
        None => return Err(sqlx::Error::RowNotFound),
        Some("draft") => {}
        Some(_) => {
            return Err(sqlx::Error::Protocol(
                "Cannot add lines to a non-draft journal entry".into(),
            ));
        }
    }
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
        ORDER BY created_at ASC, id ASC",
    )
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

    // Lock the entry so concurrent posting attempts cannot both succeed.
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
        None => return Err(sqlx::Error::RowNotFound),
        Some("draft") => {}
        Some(_) => {
            return Err(sqlx::Error::Protocol("Journal entry is not a draft".into()));
        }
    }

    // Calculate totals and count the journal lines.
    let (debits, credits, line_count): (sqlx::types::BigDecimal, sqlx::types::BigDecimal, i64) =
        sqlx::query_as(
            "SELECT
             COALESCE(SUM(debit), 0),
             COALESCE(SUM(credit), 0),
             COUNT(*)
         FROM journal_lines
         WHERE organization_id = $1
           AND journal_entry_id = $2",
        )
        .bind(organization_id)
        .bind(journal_entry_id)
        .fetch_one(&mut *tx)
        .await?;

    let zero = sqlx::types::BigDecimal::from(0);

    if line_count < 2 || debits <= zero || credits <= zero || debits != credits {
        return Err(sqlx::Error::Protocol(
            "Journal entry must have at least two lines and equal positive debits and credits"
                .into(),
        ));
    }

    // Post only a draft entry belonging to this organization.
    let result = sqlx::query(
        "UPDATE journal_entries
         SET status = 'posted',
             posted_at = NOW()
         WHERE id = $1
           AND organization_id = $2
           AND status = 'draft'",
    )
    .bind(journal_entry_id)
    .bind(organization_id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() != 1 {
        return Err(sqlx::Error::Protocol(
            "Journal entry could not be posted".into(),
        ));
    }

    tx.commit().await?;

    Ok(())
}

pub async fn get_general_ledger(
    pool: &PgPool,
    organization_id: Uuid,
    account_id: Uuid,
    from: Option<chrono::NaiveDate>,
    until: Option<chrono::NaiveDate>,
    limit: i64,
    offset: i64,
) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query(
        "
        WITH ledger AS (
          SELECT
          je.id AS journal_entry_id,
          je.entry_date,
          je.created_at,
          je.description AS entry_description,
          je.reference,
          jl.id AS journal_line_id,
          jl.description AS line_description,
          jl.debit,
          jl.credit,
          SUM(jl.debit - jl.credit) OVER (
          ORDER BY je.entry_date, je.created_at, jl.id
          ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS running_balance
          FROM journal_lines jl
          JOIN journal_entries je
              ON je.id = jl.journal_entry_id
              AND je.organization_id = jl.organization_id
              WHERE jl.organization_id = $1
              AND jl.account_id = $2
              AND je.status = 'posted'
           )
            SELECT *
            FROM ledger
            WHERE ($3::date IS NULL OR entry_date >= $3)
             AND ($4::date IS NULL OR entry_date <= $4)
            ORDER BY entry_date, created_at, journal_line_id
            LIMIT $5 OFFSET $6",
    )
    .bind(organization_id)
    .bind(account_id)
    .bind(from)
    .bind(until)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}


pub async fn get_pf_and_loss(
    pool: &PgPool,
    organization_id: Uuid,
    from: NaiveDate,
    until: NaiveDate,
) -> Result<Vec<(String, String, String, String)>, sqlx::Error> {
    sqlx::query_as::<_, (String, String, String, String)>(
        r#"
              SELECT
                    a.account_type,
                    a.id::text,
                    a.name,
                    COALESCE(
                         SUM (
                              CASE
                                  WHEN a.account_type = 'revenue'
                                  THEN jl.credit - jl.debit
                                ELSE jl.debit - jl.credit
                                END
                           ) FILTER (WHERE je.id IS NOT NULL),
                           0
                           )::text AS AMOUNT
                           FROM accounts a
                           LEFT JOIN journal_lines jl
                           ON jl.account_id = a.id
                           AND jl.organization_id = a.organization_id
                           LEFT JOIN journal_entries je
                           ON je.id = jl.journal_entry_id
                           AND je.organization_id = jl.organization_id
                           AND je.status = 'posted'
                           AND je.entry_date >= $2
                           AND je.entry_date <= $3
                           WHERE a.organization_id = $1
                                 AND a.account_type IN ('revenue', 'expense')
                           GROUP BY a.account_type, a.id, a.name
                           ORDER BY a.account_type, a.name"#,

    )
    .bind(organization_id)
    .bind(from)
    .bind(until)
    .fetch_all(pool)
    .await
}

pub async fn get_bal_sheet(
    pool: &PgPool,
    organization_id: Uuid,
    as_of: NaiveDate,
) -> Result<Vec<(String, String, String, String)>, sqlx::Error> {
    sqlx::query_as(
        r#"
               SELECT
                   a.account_type,
                   a.id::TEXT,
                   a.name,
                   COALESCE(
                        SUM( 
                            CASE 
                                WHEN a.account_type = 'asset'
                                     THEN jl.debit - jl.credit
                                ELSE jl.credit - jl.debit
                            END 
                        ) FILTER (WHERE je.id IS NOT NULL),
                        0
                    )::TEXT AS amount
                FROM accounts a 
                LEFT JOIN journal_lines jl
                     ON jl.account_id = a.id
                     AND jl.organization_id = a.organization_id
                LEFT JOIN journal_entries je
                     ON je.id = jl.journal_entry_id
                     AND je.organization_id = a.organization_id
                     AND je.status = 'posted'
                     AND je.entry_date <= $2
                WHERE a.organization_id = $1
                      AND a.account_type IN ('asset', 'liability', 'equity')
                GROUP BY a.account_type, a.id, a.name
                ORDER BY a.account_type, a.name"#
    )
    .bind(organization_id)
    .bind(as_of)
    .fetch_all(pool)
    .await
}

pub async fn get_current_earnings(
    pool: &PgPool,
    organization_id: Uuid,
    as_of: NaiveDate,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar(
        r#" 
               SELECT COALESCE(
                           SUM(
                               CASE
                                   WHEN a.account_type = 'revenue'
                                       THEN jl.credit - jl.debit
                                    WHEN a.account_type = 'expense'
                                          THEN jl.debit - jl.credit
                                    ELSE 0
                                END
                            ) FILTER (WHERE je.id IS NOT NULL),
                            0
                        )::TEXT
                        FROM accounts a
                        LEFT JOIN journal_lines jl
                             ON jl.account_id = a.id
                             AND jl.organization_id = a.organization_id
                        LEFT JOIN journal_entries je
                             ON je.id = jl.journal_entry_id
                             AND je.organization_id = a.organization_id
                             AND je.status = 'posted'
                             AND je.entry_date <= $2
                        WHERE a.organization_id = $1
                            AND a.account_type IN ('revenue', 'expense')
                "#,               
    )
    .bind(organization_id)
    .bind(as_of)
    .fetch_one(pool)
    .await
}