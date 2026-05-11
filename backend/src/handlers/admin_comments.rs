use axum::{
    Json,
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use validator::Validate;

use crate::{
    audit,
    auth::AuthUser,
    error::{AppError, AppResult},
    models::comment::{Comment, UpdateCommentStatus},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<Comment>>> {
    let rows = if let Some(status) = q.status.as_deref() {
        if !["pending", "approved", "spam"].contains(&status) {
            return Err(AppError::BadRequest("invalid status filter".into()));
        }
        sqlx::query_as::<_, Comment>(
            r#"SELECT * FROM comments WHERE status = $1 ORDER BY created_at DESC"#,
        )
        .bind(status)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, Comment>(r#"SELECT * FROM comments ORDER BY created_at DESC"#)
            .fetch_all(&state.db)
            .await?
    };
    Ok(Json(rows))
}

pub async fn set_status(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateCommentStatus>,
) -> AppResult<Json<Comment>> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    if !["pending", "approved", "spam"].contains(&input.status.as_str()) {
        return Err(AppError::BadRequest("invalid status".into()));
    }

    let prev = sqlx::query_as::<_, Comment>(r#"SELECT * FROM comments WHERE id = $1"#)
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let row = sqlx::query_as::<_, Comment>(
        r#"UPDATE comments SET status = $2 WHERE id = $1 RETURNING *"#,
    )
    .bind(id)
    .bind(&input.status)
    .fetch_one(&state.db)
    .await?;

    audit::record(
        &state.db,
        &user,
        "comment.set_status",
        Some("comment"),
        Some(row.id),
        Some(json!({
            "post_id": row.post_id,
            "from": prev.status,
            "to": row.status,
        })),
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
    let existing = sqlx::query_as::<_, Comment>(r#"SELECT * FROM comments WHERE id = $1"#)
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    sqlx::query(r#"DELETE FROM comments WHERE id = $1"#)
        .bind(id)
        .execute(&state.db)
        .await?;

    audit::record(
        &state.db,
        &user,
        "comment.delete",
        Some("comment"),
        Some(existing.id),
        Some(json!({
            "post_id": existing.post_id,
            "author_name": existing.author_name,
            "status": existing.status,
        })),
        Some(addr.ip()),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
