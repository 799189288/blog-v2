use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use std::collections::HashMap;

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

pub async fn get_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<PostDetail>> {
    let post = sqlx::query_as::<_, Post>(
        r#"SELECT * FROM posts WHERE slug = $1 AND status = 'published'"#,
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await?
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
        published_at: post.published_at,
        created_at: post.created_at,
        updated_at: post.updated_at,
        tags,
    }))
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
