use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CrJnulEntReq {
    entry_date: NaiveDate,
    description: String,
    reference: Option<String>,
}
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

#[derive(Serialize)]
pub struct CrJnulEntRes {
    id: Uuid,
    entry_date: NaiveDate,
    description: String,
    reference: Option<String>,
    status: String,
}
pub async fn create_journal_entry(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<CrJnulEntReq>,
) -> Result<(StatusCode, Json<CrJnulEntRes>), StatusCode> {
    let description = payload.description.trim();

    if description.is_empty() || description.len() > 500 {
        return Err(StatusCode::BAD_REQUEST);
    }

    if payload
        .reference
        .as_ref()
        .is_some_and(|r| r.trim().len() > 100)
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let token = bearer_token(&headers)?;

    let user_id = super::sessions::find_session_user(&pool, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let role = super::organizations::get_mem_role(&pool, organization_id, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if role.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

    let reference = payload
        .reference
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty());

    let id = super::journal_entries::create_journal_entry(
        &pool,
        organization_id,
        user_id,
        payload.entry_date,
        description,
        reference,
    )
    .await
    .map_err(|err| {
        eprintln!("Journal entry creation failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        StatusCode::CREATED,
        Json(CrJnulEntRes {
            id,
            entry_date: payload.entry_date,
            description: description.to_string(),
            reference: reference.map(str::to_string),
            status: "draft".to_string(),
        }),
    ))
}

#[derive(Deserialize)]
pub struct CreateJnulLineReq {
    account_id: Uuid,
    description: Option<String>,
    debit: String,
    credit: String,

}

#[derive(Serialize)]
pub struct CreateJnulLineRes {
    id: Uuid,
}

pub async fn create_journal_line(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Path((organization_id, entry_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreateJnulLineReq>,
) -> Result<(StatusCode, Json<CreateJnulLineRes>), StatusCode> {
    use std::str::FromStr;
    use sqlx::types::BigDecimal;

    let debit = BigDecimal::from_str(&payload.debit)
    .map_err(|_| StatusCode::BAD_REQUEST)?;
   
    let credit = BigDecimal::from_str(&payload.credit)
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    let zero = BigDecimal::from(0);
    if debit < zero || credit < zero {
        return Err(StatusCode::BAD_REQUEST);
    }

    if (debit > zero) == ( credit > zero) {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let description = payload.description.as_deref().map(str::trim).filter(|s| !s.is_empty());
    
    if description.is_some_and(|s| s.len() > 500) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let token = bearer_token(&headers)?;

    let user_id = super::sessions::find_session_user(&pool, token)
    .await 
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let role = super::organizations::get_mem_role(&pool, organization_id, user_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if role.is_none () {
        return Err(StatusCode::FORBIDDEN);
    }
    
    let entry_status: Option<String> = sqlx::query_scalar(
        "SELECT status
             FROM journal_entries
             WHERE id = $1 AND organization_id = $2",
    )
    .bind(entry_id)
    .bind(organization_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match entry_status.as_deref() {
        None => return Err(StatusCode::NOT_FOUND),
        Some("draft") => {}
        Some("posted") => return Err(StatusCode::CONFLICT),
        Some(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    let account_active: Option<bool> = sqlx::query_scalar(
        "SELECT is_active
             FROM accounts
             WHERE id = $1 AND organization_id = $2"
    )
    .bind(payload.account_id)
    .bind(organization_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

match account_active {
    None => return Err(StatusCode::BAD_REQUEST),
    Some(false) => return Err(StatusCode::BAD_REQUEST),
    Some(true) => {}
}
    

    let id = super::journal_entries::create_journal_line(&pool, organization_id, entry_id, payload.account_id, description, debit, credit)
    .await
    .map_err(|err| {
        eprintln!("journal line creation failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;



    Ok((
        StatusCode::CREATED,
        Json(CreateJnulLineRes { id }),
    ))
}

#[derive(Serialize)]
pub struct ListJnulLineRes {
    id: Uuid,
    account_id: Uuid,
    description: Option<String>,
    debit: String,
    credit: String,
}


pub async fn list_journal_lines(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Path((organization_id, entry_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<ListJnulLineRes>>, StatusCode> {
    use sqlx::Row;

    // Authenticate the user.
    let token = bearer_token(&headers)?;

    let user_id = super::sessions::find_session_user(&pool, token)
        .await
        .map_err(|err| {
            eprintln!("Session lookup failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify organization membership.
    let role = super::organizations::get_mem_role(
        &pool,
        organization_id,
        user_id,
    )
    .await
    .map_err(|err| {
        eprintln!("Membership lookup failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if role.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

    // Verify that the journal entry exists in this organization.
    let entry_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM journal_entries
            WHERE id = $1
              AND organization_id = $2
        )",
    )
    .bind(entry_id)
    .bind(organization_id)
    .fetch_one(&pool)
    .await
    .map_err(|err| {
        eprintln!("Journal entry lookup failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !entry_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    // Fetch journal lines.
    let rows = super::journal_entries::list_journal_lines(
        &pool,
        organization_id,
        entry_id,
    )
    .await
    .map_err(|err| {
        eprintln!("Journal line listing failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Convert database rows into JSON response objects.
    let mut lines = Vec::with_capacity(rows.len());

    for row in rows {
        let line = ListJnulLineRes {
            id: row.try_get("id").map_err(|err| {
                eprintln!("Failed to read journal line ID: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
            account_id: row.try_get("account_id").map_err(|err| {
                eprintln!("Failed to read account ID: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
            description: row.try_get("description").map_err(|err| {
                eprintln!("Failed to read description: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
            debit: row
                .try_get::<sqlx::types::BigDecimal, _>("debit")
                .map_err(|err| {
                    eprintln!("Failed to read debit: {err}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
                .to_string(),
            credit: row
                .try_get::<sqlx::types::BigDecimal, _>("credit")
                .map_err(|err| {
                    eprintln!("Failed to read credit: {err}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?
                .to_string(),
        };

        lines.push(line);
    }

    Ok(Json(lines))
}

pub async fn post_journal_entry(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Path((organization_id, entry_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let token = bearer_token(&headers)?;
    

    let user_id = super::sessions::find_session_user(&pool, token)
        .await
        .map_err(|err| {
            eprintln!("Session lookup failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;


    // Verify organization membership.
    let role = super::organizations::get_mem_role(
        &pool,
        organization_id,
        user_id,
    )
    .await
    .map_err(|err| {
        eprintln!("Membership lookup failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if role.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

    super::journal_entries::post_journal_entry(&pool, organization_id, entry_id).await.map_err(|err| { eprintln!("Journal entry posting failed :( :{err}");

    if matches!(err, sqlx::Error::RowNotFound) {
        StatusCode::NOT_FOUND
    } else if matches!(err, sqlx::Error::Protocol(_)) {
        StatusCode::CONFLICT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
    })?;

    Ok(StatusCode::OK)
}