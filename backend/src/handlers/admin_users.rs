use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use time::OffsetDateTime;

use crate::{
    audit,
    auth::{AuthUser, password},
    error::{AppError, AppResult},
    state::AppState,
};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub role: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn list(
    State(state): State<AppState>,
    _user: AuthUser,
) -> AppResult<Json<Vec<UserRow>>> {
    let rows = sqlx::query_as::<_, UserRow>(
        r#"SELECT id, username, role, created_at FROM users ORDER BY id ASC"#,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub username: String,
    pub password: String,
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<CreateUserInput>,
) -> AppResult<Json<UserRow>> {
    let username = input.username.trim();
    if username.is_empty() {
        return Err(AppError::BadRequest("username required".into()));
    }
    if input.password.len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }

    let hash = password::hash(&input.password).map_err(AppError::Internal)?;

    let row = sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (username, password_hash, role)
        VALUES ($1, $2, 'admin')
        RETURNING id, username, role, created_at
        "#,
    )
    .bind(username)
    .bind(&hash)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db) = e {
            if db.constraint() == Some("users_username_key") {
                return AppError::Conflict("username already taken".into());
            }
        }
        AppError::Database(e)
    })?;

    audit::record(
        &state.db,
        &user,
        "user.create",
        Some("user"),
        Some(row.id),
        Some(json!({ "username": row.username })),
        Some(addr.ip()),
    )
    .await;

    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordInput {
    pub password: String,
}

pub async fn reset_password(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<i64>,
    Json(input): Json<ResetPasswordInput>,
) -> AppResult<Json<UserRow>> {
    if input.password.len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    let hash = password::hash(&input.password).map_err(AppError::Internal)?;

    let row = sqlx::query_as::<_, UserRow>(
        r#"
        UPDATE users SET password_hash = $2 WHERE id = $1
        RETURNING id, username, role, created_at
        "#,
    )
    .bind(id)
    .bind(&hash)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    audit::record(
        &state.db,
        &user,
        "user.reset_password",
        Some("user"),
        Some(row.id),
        Some(json!({ "username": row.username })),
        Some(addr.ip()),
    )
    .await;

    Ok(Json(row))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    if id == user.user_id {
        return Err(AppError::BadRequest(
            "you cannot delete your own account".into(),
        ));
    }

    let admin_count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*)::bigint FROM users WHERE role = 'admin'"#,
    )
    .fetch_one(&state.db)
    .await?;

    let target = sqlx::query_as::<_, UserRow>(
        r#"SELECT id, username, role, created_at FROM users WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    if target.role == "admin" && admin_count <= 1 {
        return Err(AppError::BadRequest(
            "cannot delete the last admin user".into(),
        ));
    }

    sqlx::query(r#"DELETE FROM users WHERE id = $1"#)
        .bind(id)
        .execute(&state.db)
        .await?;

    audit::record(
        &state.db,
        &user,
        "user.delete",
        Some("user"),
        Some(target.id),
        Some(json!({ "username": target.username })),
        Some(addr.ip()),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
