use axum::{Json, extract::{Query, State}};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use time::OffsetDateTime;

use crate::{auth::AuthUser, error::AppResult, handlers::admin_data::Page, state::AppState};

#[derive(Debug, Serialize, FromRow)]
pub struct AuditRow {
    pub id: i64,
    pub user_id: Option<i64>,
    pub username: String,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<i64>,
    pub detail: Option<Value>,
    pub ip: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub user_id: Option<i64>,
    pub action: Option<String>,
    #[serde(with = "time::serde::rfc3339::option", default)]
    pub from: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option", default)]
    pub to: Option<OffsetDateTime>,
}

pub async fn list(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(q): Query<AuditQuery>,
) -> AppResult<Json<Page<AuditRow>>> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let items: Vec<AuditRow> = sqlx::query_as(
        r#"
        SELECT id, user_id, username, action, target_type, target_id, detail,
               host(ip)::text AS ip, created_at
        FROM audit_logs
        WHERE ($1::bigint IS NULL OR user_id = $1)
          AND ($2::text   IS NULL OR action = $2)
          AND ($3::timestamptz IS NULL OR created_at >= $3)
          AND ($4::timestamptz IS NULL OR created_at <= $4)
        ORDER BY created_at DESC, id DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(q.user_id)
    .bind(q.action.as_deref())
    .bind(q.from)
    .bind(q.to)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint FROM audit_logs
        WHERE ($1::bigint IS NULL OR user_id = $1)
          AND ($2::text   IS NULL OR action = $2)
          AND ($3::timestamptz IS NULL OR created_at >= $3)
          AND ($4::timestamptz IS NULL OR created_at <= $4)
        "#,
    )
    .bind(q.user_id)
    .bind(q.action.as_deref())
    .bind(q.from)
    .bind(q.to)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(Page { items, page, per_page, total }))
}
