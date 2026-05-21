use axum::{
    Json,
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use std::net::SocketAddr;
use time::OffsetDateTime;
use validator::Validate;

use crate::{
    audit,
    auth::AuthUser,
    error::{AppError, AppResult},
    handlers::posts::Paginated,
    models::comment::{Comment, UpdateCommentStatus},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AdminCommentRow {
    pub id: i64,
    pub post_id: i64,
    pub post_title: String,
    pub post_slug: String,
    pub parent_id: Option<i64>,
    pub parent_author_name: Option<String>,
    pub author_name: String,
    pub author_email: Option<String>,
    pub content: String,
    pub status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn list(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Paginated<AdminCommentRow>>> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let (rows, total) = if let Some(status) = q.status.as_deref() {
        if !["pending", "approved", "spam"].contains(&status) {
            return Err(AppError::BadRequest("invalid status filter".into()));
        }
        let rows = sqlx::query_as::<_, AdminCommentRow>(
            r#"
            SELECT c.id, c.post_id, po.title AS post_title, po.slug AS post_slug,
                   c.parent_id, p.author_name AS parent_author_name,
                   c.author_name, c.author_email, c.content, c.status, c.created_at
            FROM comments c
            LEFT JOIN comments p ON p.id = c.parent_id
            LEFT JOIN posts po ON po.id = c.post_id
            WHERE c.status = $1
            ORDER BY c.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

        let total: i64 =
            sqlx::query_scalar(r#"SELECT COUNT(*) FROM comments WHERE status = $1"#)
                .bind(status)
                .fetch_one(&state.db)
                .await?;
        (rows, total)
    } else {
        let rows = sqlx::query_as::<_, AdminCommentRow>(
            r#"
            SELECT c.id, c.post_id, po.title AS post_title, po.slug AS post_slug,
                   c.parent_id, p.author_name AS parent_author_name,
                   c.author_name, c.author_email, c.content, c.status, c.created_at
            FROM comments c
            LEFT JOIN comments p ON p.id = c.parent_id
            LEFT JOIN posts po ON po.id = c.post_id
            ORDER BY c.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

        let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM comments"#)
            .fetch_one(&state.db)
            .await?;
        (rows, total)
    };

    Ok(Json(Paginated { items: rows, page, per_page, total }))
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
