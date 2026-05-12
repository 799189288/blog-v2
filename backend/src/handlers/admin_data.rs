use axum::{Json, extract::{Query, State}};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

use crate::{auth::AuthUser, error::{AppError, AppResult}, state::AppState};

const DEFAULT_PER_PAGE: i64 = 20;
const MAX_PER_PAGE: i64 = 200;

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct CommonQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub q: Option<String>,
    pub status: Option<String>,
    pub post_id: Option<i64>,
}

fn pagination(q: &CommonQuery) -> (i64, i64, i64) {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(DEFAULT_PER_PAGE).clamp(1, MAX_PER_PAGE);
    let offset = (page - 1) * per_page;
    (page, per_page, offset)
}

/// Resolve a user-supplied sort column against an allow-list. Returns the
/// final ORDER BY clause as a raw string. The allow-list ensures no SQL
/// injection is possible — we never interpolate untrusted input.
fn order_by(q: &CommonQuery, allowed: &[&str], default: &str) -> String {
    let col = q
        .sort
        .as_deref()
        .filter(|s| allowed.contains(s))
        .unwrap_or(default);
    let dir = match q.dir.as_deref() {
        Some("asc") | Some("ASC") => "ASC",
        _ => "DESC",
    };
    format!("{col} {dir}")
}

// ----- posts -----

#[derive(Debug, Serialize, FromRow)]
pub struct PostRow {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub status: String,
    pub author_id: i64,
    #[serde(with = "time::serde::rfc3339::option")]
    pub published_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

pub async fn posts(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(q): Query<CommonQuery>,
) -> AppResult<Json<Page<PostRow>>> {
    let (page, per_page, offset) = pagination(&q);
    let allowed = ["id", "title", "status", "published_at", "created_at", "updated_at"];
    let order = order_by(&q, &allowed, "created_at");

    // Validate status filter
    if let Some(s) = q.status.as_deref()
        && !["draft", "published"].contains(&s) {
        return Err(AppError::BadRequest("invalid status".into()));
    }

    let qpat = q.q.as_deref().map(|s| format!("%{}%", s.replace('%', "\\%")));

    let items_sql = format!(
        r#"
        SELECT id, slug, title, status, author_id, published_at, created_at, updated_at
        FROM posts
        WHERE ($1::text IS NULL OR status = $1)
          AND ($2::text IS NULL OR title ILIKE $2)
        ORDER BY {order}
        LIMIT $3 OFFSET $4
        "#,
    );

    let items: Vec<PostRow> = sqlx::query_as(&items_sql)
        .bind(q.status.as_deref())
        .bind(qpat.as_deref())
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint FROM posts
        WHERE ($1::text IS NULL OR status = $1)
          AND ($2::text IS NULL OR title ILIKE $2)
        "#,
    )
    .bind(q.status.as_deref())
    .bind(qpat.as_deref())
    .fetch_one(&state.db)
    .await?;

    Ok(Json(Page { items, page, per_page, total }))
}

// ----- comments -----

#[derive(Debug, Serialize, FromRow)]
pub struct CommentRow {
    pub id: i64,
    pub post_id: i64,
    pub parent_id: Option<i64>,
    pub parent_author_name: Option<String>,
    pub author_name: String,
    pub author_email: Option<String>,
    pub content: String,
    pub status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn comments(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(q): Query<CommonQuery>,
) -> AppResult<Json<Page<CommentRow>>> {
    let (page, per_page, offset) = pagination(&q);
    let allowed = ["id", "post_id", "author_name", "status", "created_at"];
    let order = order_by(&q, &allowed, "created_at");

    if let Some(s) = q.status.as_deref()
        && !["pending", "approved", "spam"].contains(&s) {
        return Err(AppError::BadRequest("invalid status".into()));
    }
    let qpat = q.q.as_deref().map(|s| format!("%{}%", s.replace('%', "\\%")));

    // Note: order by is on `c.<col>` after join — the allow-list keys are bare
    // column names so we qualify them here.
    let qualified_order = order.split_whitespace().collect::<Vec<_>>();
    let (sort_col, sort_dir) = (qualified_order[0], qualified_order[1]);
    let items_sql = format!(
        r#"
        SELECT c.id, c.post_id, c.parent_id, p.author_name AS parent_author_name,
               c.author_name, c.author_email, c.content, c.status, c.created_at
        FROM comments c
        LEFT JOIN comments p ON p.id = c.parent_id
        WHERE ($1::text IS NULL OR c.status = $1)
          AND ($2::bigint IS NULL OR c.post_id = $2)
          AND ($3::text IS NULL OR c.author_name ILIKE $3 OR c.content ILIKE $3)
        ORDER BY c.{sort_col} {sort_dir}
        LIMIT $4 OFFSET $5
        "#,
    );

    let items: Vec<CommentRow> = sqlx::query_as(&items_sql)
        .bind(q.status.as_deref())
        .bind(q.post_id)
        .bind(qpat.as_deref())
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint FROM comments
        WHERE ($1::text IS NULL OR status = $1)
          AND ($2::bigint IS NULL OR post_id = $2)
          AND ($3::text IS NULL OR author_name ILIKE $3 OR content ILIKE $3)
        "#,
    )
    .bind(q.status.as_deref())
    .bind(q.post_id)
    .bind(qpat.as_deref())
    .fetch_one(&state.db)
    .await?;

    Ok(Json(Page { items, page, per_page, total }))
}

// ----- tags -----

#[derive(Debug, Serialize, FromRow)]
pub struct TagRow {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub post_count: i64,
}

pub async fn tags(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(q): Query<CommonQuery>,
) -> AppResult<Json<Page<TagRow>>> {
    let (page, per_page, offset) = pagination(&q);
    let allowed = ["id", "name", "slug", "post_count"];
    let order = order_by(&q, &allowed, "name");
    let qpat = q.q.as_deref().map(|s| format!("%{}%", s.replace('%', "\\%")));

    let items_sql = format!(
        r#"
        SELECT t.id, t.name, t.slug,
               COUNT(pt.post_id)::bigint AS post_count
        FROM tags t
        LEFT JOIN post_tags pt ON pt.tag_id = t.id
        WHERE ($1::text IS NULL OR t.name ILIKE $1)
        GROUP BY t.id
        ORDER BY {order}
        LIMIT $2 OFFSET $3
        "#,
    );

    let items: Vec<TagRow> = sqlx::query_as(&items_sql)
        .bind(qpat.as_deref())
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*)::bigint FROM tags WHERE ($1::text IS NULL OR name ILIKE $1)"#,
    )
    .bind(qpat.as_deref())
    .fetch_one(&state.db)
    .await?;

    Ok(Json(Page { items, page, per_page, total }))
}

// ----- users (data browser) -----

#[derive(Debug, Serialize, FromRow)]
pub struct UserBrowseRow {
    pub id: i64,
    pub username: String,
    pub role: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

pub async fn users(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(q): Query<CommonQuery>,
) -> AppResult<Json<Page<UserBrowseRow>>> {
    let (page, per_page, offset) = pagination(&q);
    let allowed = ["id", "username", "role", "created_at"];
    let order = order_by(&q, &allowed, "id");

    let items_sql = format!(
        r#"
        SELECT id, username, role, created_at
        FROM users
        ORDER BY {order}
        LIMIT $1 OFFSET $2
        "#,
    );

    let items: Vec<UserBrowseRow> = sqlx::query_as(&items_sql)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*)::bigint FROM users"#)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(Page { items, page, per_page, total }))
}
