use axum::{
    Json,
    extract::{ConnectInfo, Path, Query, State},
    http::HeaderMap,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use time::OffsetDateTime;

use crate::{
    error::{AppError, AppResult},
    models::{
        post::{Post, PostDetail, PostSummary},
        tag::Tag,
    },
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub tag: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

pub async fn list_published(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Paginated<PostSummary>>> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(10).clamp(1, 50);
    let offset = (page - 1) * per_page;

    let (rows, total): (Vec<Post>, i64) = if let Some(tag_slug) = q.tag.as_deref() {
        let rows = sqlx::query_as::<_, Post>(
            r#"
            SELECT p.*
            FROM posts p
            JOIN post_tags pt ON pt.post_id = p.id
            JOIN tags t ON t.id = pt.tag_id
            WHERE p.status = 'published' AND t.slug = $1
            ORDER BY p.published_at DESC NULLS LAST, p.id DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tag_slug)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT p.id)
            FROM posts p
            JOIN post_tags pt ON pt.post_id = p.id
            JOIN tags t ON t.id = pt.tag_id
            WHERE p.status = 'published' AND t.slug = $1
            "#,
        )
        .bind(tag_slug)
        .fetch_one(&state.db)
        .await?;
        (rows, total)
    } else {
        let rows = sqlx::query_as::<_, Post>(
            r#"
            SELECT *
            FROM posts
            WHERE status = 'published'
            ORDER BY published_at DESC NULLS LAST, id DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

        let total: i64 =
            sqlx::query_scalar(r#"SELECT COUNT(*) FROM posts WHERE status = 'published'"#)
                .fetch_one(&state.db)
                .await?;
        (rows, total)
    };

    let tags_by_post = load_tags_for_posts(&state, &rows).await?;
    let items = rows
        .into_iter()
        .map(|p| {
            let tags = tags_by_post.get(&p.id).cloned().unwrap_or_default();
            PostSummary {
                id: p.id,
                slug: p.slug,
                title: p.title,
                excerpt: p.excerpt,
                status: p.status,
                views: p.views,
                word_count: p.word_count,
                reading_time_min: p.reading_time_min,
                cover_image: p.cover_image,
                published_at: p.published_at,
                created_at: p.created_at,
                tags,
            }
        })
        .collect();

    Ok(Json(Paginated {
        items,
        page,
        per_page,
        total,
    }))
}

#[derive(Debug, Deserialize)]
pub struct GetBySlugQuery {
    /// Preview token. When present and matching a draft post's stored
    /// token, the draft is returned; otherwise drafts 404.
    pub token: Option<String>,
}

pub async fn get_by_slug(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Query(q): Query<GetBySlugQuery>,
) -> AppResult<Json<PostDetail>> {
    // Two paths share most of the code:
    //   * Normal: only published posts; bump views with per-IP dedupe.
    //   * Preview: draft with a matching ?token=. Never bump views —
    //     the author is checking their own draft.
    let ip = client_ip(&headers, addr);

    let post = if let Some(token) = q.token.as_deref().filter(|s| !s.is_empty()) {
        sqlx::query_as::<_, Post>(
            r#"
            SELECT * FROM posts
            WHERE slug = $1
              AND status = 'draft'
              AND preview_token = $2
            "#,
        )
        .bind(&slug)
        .bind(token)
        .fetch_optional(&state.db)
        .await?
    } else {
        let should_count = state.should_count_view(&slug, &ip);
        if should_count {
            sqlx::query_as::<_, Post>(
                r#"
                UPDATE posts
                SET views = views + 1
                WHERE slug = $1 AND status = 'published'
                RETURNING *
                "#,
            )
            .bind(&slug)
            .fetch_optional(&state.db)
            .await?
        } else {
            sqlx::query_as::<_, Post>(
                r#"SELECT * FROM posts WHERE slug = $1 AND status = 'published'"#,
            )
            .bind(&slug)
            .fetch_optional(&state.db)
            .await?
        }
    }
    .ok_or(AppError::NotFound)?;

    let tags = load_tags_for_post(&state, post.id).await?;
    Ok(Json(PostDetail {
        id: post.id,
        slug: post.slug,
        title: post.title,
        excerpt: post.excerpt,
        content_md: post.content_md,
        content_html: post.content_html,
        status: post.status,
        views: post.views,
        word_count: post.word_count,
        reading_time_min: post.reading_time_min,
        cover_image: post.cover_image,
        // Don't leak the token back through the public response — readers
        // get URL-only access; the admin path returns it separately.
        preview_token: None,
        published_at: post.published_at,
        created_at: post.created_at,
        updated_at: post.updated_at,
        tags,
    }))
}

/// Best-effort client IP, honoring `X-Forwarded-For` when present.
/// Falls back to the immediate peer address.
fn client_ip(headers: &HeaderMap, addr: SocketAddr) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| addr.ip().to_string())
}

#[derive(Debug, serde::Serialize)]
pub struct NavPost {
    pub slug: String,
    pub title: String,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct RelatedPost {
    pub slug: String,
    pub title: String,
    pub excerpt: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub published_at: Option<OffsetDateTime>,
}

#[derive(Debug, serde::Serialize)]
pub struct PostNav {
    pub prev: Option<NavPost>,
    pub next: Option<NavPost>,
    pub related: Vec<RelatedPost>,
}

/// Prev/next neighbours in the published timeline + posts that share tags.
/// Mirrors `get_by_slug`'s 404 on missing/draft so the frontend only fetches
/// this after the article itself was found.
pub async fn related(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<PostNav>> {
    // One pass over the published list with window functions: in DESC order,
    // LAG = newer post (next), LEAD = older post (prev).
    #[derive(sqlx::FromRow)]
    struct NavRow {
        newer_slug: Option<String>,
        newer_title: Option<String>,
        older_slug: Option<String>,
        older_title: Option<String>,
    }
    let nav = sqlx::query_as::<_, NavRow>(
        r#"
        WITH ordered AS (
            SELECT
                slug,
                LAG(slug)  OVER w AS newer_slug,
                LAG(title) OVER w AS newer_title,
                LEAD(slug) OVER w AS older_slug,
                LEAD(title) OVER w AS older_title
            FROM posts
            WHERE status = 'published'
            WINDOW w AS (ORDER BY published_at DESC NULLS LAST, id DESC)
        )
        SELECT newer_slug, newer_title, older_slug, older_title
        FROM ordered
        WHERE slug = $1
        "#,
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let related = sqlx::query_as::<_, RelatedPost>(
        r#"
        SELECT p.slug, p.title, p.excerpt, p.published_at
        FROM posts p
        JOIN post_tags pt   ON pt.post_id   = p.id
        JOIN post_tags pt2  ON pt2.tag_id   = pt.tag_id
        JOIN posts curr     ON curr.id      = pt2.post_id
        WHERE curr.slug = $1
          AND curr.status = 'published'
          AND p.id != curr.id
          AND p.status = 'published'
        GROUP BY p.id, p.slug, p.title, p.excerpt, p.published_at
        ORDER BY COUNT(*) DESC, p.published_at DESC NULLS LAST, p.id DESC
        LIMIT 5
        "#,
    )
    .bind(&slug)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(PostNav {
        prev: nav.older_slug.zip(nav.older_title).map(|(slug, title)| NavPost { slug, title }),
        next: nav.newer_slug.zip(nav.newer_title).map(|(slug, title)| NavPost { slug, title }),
        related,
    }))
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveEntry {
    pub slug: String,
    pub title: String,
    #[serde(with = "time::serde::rfc3339")]
    pub published_at: OffsetDateTime,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveGroup {
    /// "2026-05" — string keyed because clients sort + dedupe in render.
    pub year_month: String,
    pub year: i32,
    pub month: u8,
    pub posts: Vec<ArchiveEntry>,
}

/// All published posts, newest first, grouped by year-month in the
/// response. Cheap enough to compute in one pass over the whole posts
/// table (this is a personal blog; even a few thousand posts is trivial)
/// so we don't bother paginating.
pub async fn archive(State(state): State<AppState>) -> AppResult<Json<Vec<ArchiveGroup>>> {
    if let Some(cached) = state.get_archive_cache().await {
        return Ok(Json((*cached).clone()));
    }

    let rows = sqlx::query_as::<_, (String, String, OffsetDateTime)>(
        r#"
        SELECT slug, title, published_at
        FROM posts
        WHERE status = 'published' AND published_at IS NOT NULL
        ORDER BY published_at DESC, id DESC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut groups: Vec<ArchiveGroup> = Vec::new();
    for (slug, title, published_at) in rows {
        let year = published_at.year();
        let month: u8 = published_at.month().into();
        let key = format!("{year:04}-{month:02}");
        let entry = ArchiveEntry {
            slug,
            title,
            published_at,
        };
        match groups.last_mut() {
            Some(g) if g.year_month == key => g.posts.push(entry),
            _ => groups.push(ArchiveGroup {
                year_month: key,
                year,
                month,
                posts: vec![entry],
            }),
        }
    }

    state.set_archive_cache(groups.clone()).await;
    Ok(Json(groups))
}

pub(crate) async fn load_tags_for_post(state: &AppState, post_id: i64) -> AppResult<Vec<Tag>> {
    let tags = sqlx::query_as::<_, Tag>(
        r#"
        SELECT t.id, t.name, t.slug
        FROM tags t
        JOIN post_tags pt ON pt.tag_id = t.id
        WHERE pt.post_id = $1
        ORDER BY t.name
        "#,
    )
    .bind(post_id)
    .fetch_all(&state.db)
    .await?;
    Ok(tags)
}

pub(crate) async fn load_tags_for_posts(
    state: &AppState,
    posts: &[Post],
) -> AppResult<HashMap<i64, Vec<Tag>>> {
    if posts.is_empty() {
        return Ok(HashMap::new());
    }
    let ids: Vec<i64> = posts.iter().map(|p| p.id).collect();
    #[derive(sqlx::FromRow)]
    struct Row {
        post_id: i64,
        id: i64,
        name: String,
        slug: String,
    }
    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT pt.post_id, t.id, t.name, t.slug
        FROM post_tags pt
        JOIN tags t ON t.id = pt.tag_id
        WHERE pt.post_id = ANY($1)
        ORDER BY t.name
        "#,
    )
    .bind(&ids)
    .fetch_all(&state.db)
    .await?;

    let mut map: HashMap<i64, Vec<Tag>> = HashMap::new();
    for r in rows {
        map.entry(r.post_id).or_default().push(Tag {
            id: r.id,
            name: r.name,
            slug: r.slug,
        });
    }
    Ok(map)
}
