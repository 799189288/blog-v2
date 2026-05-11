use axum::{Json, extract::{Query, State}};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    auth::AuthUser,
    error::AppResult,
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct Overview {
    pub posts: PostStats,
    pub comments: CommentStats,
    pub tags: TagStats,
    pub users: UserStats,
}

#[derive(Debug, Serialize)]
pub struct PostStats {
    pub total: i64,
    pub published: i64,
    pub draft: i64,
}

#[derive(Debug, Serialize)]
pub struct CommentStats {
    pub total: i64,
    pub pending: i64,
    pub approved: i64,
    pub spam: i64,
}

#[derive(Debug, Serialize)]
pub struct TagStats {
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct UserStats {
    pub total: i64,
}

pub async fn overview(
    State(state): State<AppState>,
    _user: AuthUser,
) -> AppResult<Json<Overview>> {
    #[derive(sqlx::FromRow)]
    struct PostsRow {
        total: i64,
        published: i64,
        draft: i64,
    }
    let posts: PostsRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint AS total,
            COUNT(*) FILTER (WHERE status = 'published')::bigint AS published,
            COUNT(*) FILTER (WHERE status = 'draft')::bigint AS draft
        FROM posts
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    #[derive(sqlx::FromRow)]
    struct CommentsRow {
        total: i64,
        pending: i64,
        approved: i64,
        spam: i64,
    }
    let comments: CommentsRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)::bigint AS total,
            COUNT(*) FILTER (WHERE status = 'pending')::bigint AS pending,
            COUNT(*) FILTER (WHERE status = 'approved')::bigint AS approved,
            COUNT(*) FILTER (WHERE status = 'spam')::bigint AS spam
        FROM comments
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    let tags_total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*)::bigint FROM tags"#)
        .fetch_one(&state.db)
        .await?;
    let users_total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*)::bigint FROM users"#)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(Overview {
        posts: PostStats {
            total: posts.total,
            published: posts.published,
            draft: posts.draft,
        },
        comments: CommentStats {
            total: comments.total,
            pending: comments.pending,
            approved: comments.approved,
            spam: comments.spam,
        },
        tags: TagStats { total: tags_total },
        users: UserStats { total: users_total },
    }))
}

#[derive(Debug, Deserialize)]
pub struct TrendQuery {
    pub days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TrendResponse {
    pub posts: Vec<TrendPoint>,
    pub comments: Vec<TrendPoint>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TrendPoint {
    #[serde(with = "time::serde::rfc3339")]
    pub date: OffsetDateTime,
    pub count: i64,
}

pub async fn trend(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(q): Query<TrendQuery>,
) -> AppResult<Json<TrendResponse>> {
    let days = q.days.unwrap_or(30).clamp(1, 365);

    let posts: Vec<TrendPoint> = sqlx::query_as(
        r#"
        SELECT d::timestamptz AS date,
               COUNT(p.id)::bigint AS count
        FROM generate_series(
            (now()::date - ($1::int - 1) * interval '1 day'),
            now()::date,
            interval '1 day'
        ) AS d
        LEFT JOIN posts p
            ON p.status = 'published'
           AND p.published_at::date = d::date
        GROUP BY d
        ORDER BY d
        "#,
    )
    .bind(days as i32)
    .fetch_all(&state.db)
    .await?;

    let comments: Vec<TrendPoint> = sqlx::query_as(
        r#"
        SELECT d::timestamptz AS date,
               COUNT(c.id)::bigint AS count
        FROM generate_series(
            (now()::date - ($1::int - 1) * interval '1 day'),
            now()::date,
            interval '1 day'
        ) AS d
        LEFT JOIN comments c
            ON c.created_at::date = d::date
        GROUP BY d
        ORDER BY d
        "#,
    )
    .bind(days as i32)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(TrendResponse { posts, comments }))
}
