use axum::{Json, extract::{Query, State}};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
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

// ---------------- Dashboard aggregate ----------------

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub overview: Overview,
    pub top_posts: Vec<TopPost>,
    pub recent_comments: Vec<RecentComment>,
    pub tag_cloud: Vec<TagCloudItem>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TopPost {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub views: i64,
    pub comment_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RecentComment {
    pub id: i64,
    pub post_id: i64,
    pub post_title: String,
    pub post_slug: String,
    pub author_name: String,
    pub content: String,
    pub status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TagCloudItem {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub post_count: i64,
}

pub async fn dashboard(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<DashboardResponse>> {
    // Reuse the existing aggregator for the headline stats.
    let overview = overview(State(state.clone()), user).await?.0;

    let top_posts: Vec<TopPost> = sqlx::query_as(
        r#"
        SELECT
            p.id,
            p.slug,
            p.title,
            p.views,
            COALESCE(c.cnt, 0)::bigint AS comment_count
        FROM posts p
        LEFT JOIN (
            SELECT post_id, COUNT(*)::bigint AS cnt
            FROM comments
            WHERE status = 'approved'
            GROUP BY post_id
        ) c ON c.post_id = p.id
        WHERE p.status = 'published'
        ORDER BY p.views DESC, p.id DESC
        LIMIT 5
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let recent_comments: Vec<RecentComment> = sqlx::query_as(
        r#"
        SELECT
            c.id,
            c.post_id,
            p.title AS post_title,
            p.slug  AS post_slug,
            c.author_name,
            c.content,
            c.status,
            c.created_at
        FROM comments c
        JOIN posts p ON p.id = c.post_id
        ORDER BY c.created_at DESC
        LIMIT 8
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let tag_cloud: Vec<TagCloudItem> = sqlx::query_as(
        r#"
        SELECT
            t.id,
            t.name,
            t.slug,
            COUNT(pt.post_id)::bigint AS post_count
        FROM tags t
        LEFT JOIN post_tags pt ON pt.tag_id = t.id
        LEFT JOIN posts p ON p.id = pt.post_id AND p.status = 'published'
        GROUP BY t.id
        HAVING COUNT(pt.post_id) > 0
        ORDER BY post_count DESC, t.name
        LIMIT 40
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(DashboardResponse {
        overview,
        top_posts,
        recent_comments,
        tag_cloud,
    }))
}
