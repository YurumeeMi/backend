use crate::services::organizations;
use crate::services::password::{hash_password, password_verify};
use crate::services::users::{
    create_user, find_password_hash_by_id, find_usr_in_email, update_password_hash,
};
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

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

#[derive(Deserialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    id: Uuid,
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), StatusCode> {
    let email = payload.email.trim().to_lowercase();

    if email.is_empty()
        || !email.contains('@')
        || email.contains(char::is_whitespace)
        || payload.password.is_empty()
        || payload.password.len() < 12
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let id = create_user(&pool, &email, &payload.password)
        .await
        .map_err(|err| {
            if let sqlx::Error::Database(db_err) = &err {
                if db_err.is_unique_violation() {
                    return StatusCode::CONFLICT;
                }
            }

            eprintln!("User registration failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(RegisterResponse { id })))
}
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub id: Uuid,
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), StatusCode> {
    if payload.email.trim().is_empty() || payload.password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let users = find_usr_in_email(&pool, &payload.email.trim().to_lowercase())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some((id, stored_hash)) = users else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    password_verify(&payload.password, &stored_hash).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let token = super::sessions::create_session(&pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, Json(LoginResponse { id, token })))
}

pub async fn me(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let token = bearer_token(&headers)?;

    let id = super::sessions::find_session_user(&pool, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(Json(RegisterResponse { id }))
}
pub async fn logout(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let token = bearer_token(&headers)?;

    super::sessions::delete_session(&pool, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn change_pass(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<StatusCode, StatusCode> {
    if payload.new_password.len() < 12 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let token = bearer_token(&headers)?;

    let user_id = super::sessions::find_session_user(&pool, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let stored_hash = find_password_hash_by_id(&pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    password_verify(&payload.current_password, &stored_hash)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let new_hash =
        hash_password(&payload.new_password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    update_password_hash(&pool, user_id, &new_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    super::sessions::delete_user_session(&pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn logout_all(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let token = bearer_token(&headers)?;

    let user_id = super::sessions::find_session_user(&pool, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    super::sessions::delete_user_session(&pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct CreateOrgReq {
    name: String,
}
#[derive(Serialize)]
pub struct CreateOrgRes {
    id: Uuid,
}
pub async fn create_org(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateOrgReq>,
) -> Result<(StatusCode, Json<CreateOrgRes>), StatusCode> {
    let name = payload.name.trim();

    if name.is_empty() || name.len() > 120 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let token = bearer_token(&headers)?;

    let user_id = super::sessions::find_session_user(&pool, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let id = super::organizations::create_organization(&pool, user_id, name)
        .await
        .map_err(|err| {
            eprint!("Org creation failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(CreateOrgRes { id })))
}

#[derive(Serialize)]
pub struct ListOrgRep {
    organizations: Vec<OrgItems>,
}

#[derive(Serialize)]
pub struct OrgItems {
    id: Uuid,
    name: String,
    role: String,
}

pub async fn list_org(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ListOrgRep>, StatusCode> {
    let token = bearer_token(&headers)?;

    let user_id = super::sessions::find_session_user(&pool, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let rows = super::organizations::list_user_org(&pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let organizations = rows
        .into_iter()
        .map(|(id, name, role)| OrgItems { id, name, role })
        .collect();

    Ok(Json(ListOrgRep { organizations }))
}

#[derive(Serialize)]
pub struct GetOrgRep {
    id: Uuid,
    name: String,
    role: String,
}

pub async fn get_org(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(organization_id): axum::extract::Path<Uuid>,
) -> Result<Json<GetOrgRep>, StatusCode> {
    let token = bearer_token(&headers)?;

    let user_id = super::sessions::find_session_user(&pool, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let (id, name, role) = super::organizations::get_user_org(&pool, user_id, organization_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(GetOrgRep { id, name, role }))
}

#[derive(Deserialize)]
pub struct AddMemReq {
    user_id: Uuid,
    role: String,
}

#[derive(Serialize)]
pub struct AddMemRes {
    message: String,
}

pub async fn add_mem_org(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(organization_id): axum::extract::Path<Uuid>,
    Json(payload): Json<AddMemReq>,
) -> Result<(StatusCode, Json<AddMemRes>), StatusCode> {
    if !["admin", "member"].contains(&payload.role.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let token = bearer_token(&headers)?;

    let caller_id = super::sessions::find_session_user(&pool, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let caller_role = organizations::get_mem_role(&pool, organization_id, caller_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !matches!(caller_role.as_deref(), Some("owner" | "admin")) {
        return Err(StatusCode::FORBIDDEN);
    }
    organizations::add_mem(&pool, organization_id, payload.user_id, &payload.role)
        .await
        .map_err(|err| {
            eprintln!("Adding Memer fail: {err}");
            StatusCode::CONFLICT
        })?;

    Ok((
        StatusCode::CREATED,
        Json(AddMemRes {
            message: ("Member added".to_string()),
        }),
    ))
}
