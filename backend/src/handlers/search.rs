use axum::{Json, extract::{Query, State}};
use serde::Deserialize;

use crate::{
    error::AppResult,
    handlers::posts::{Paginated, load_tags_for_posts},
    models::post::{Post, PostSummary},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn search(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> AppResult<Json<Paginated<PostSummary>>> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(10).clamp(1, 50);
    let offset = (page - 1) * per_page;
    let query = q.q.trim();

    if query.is_empty() {
        return Ok(Json(Paginated {
            items: vec![],
            page,
            per_page,
            total: 0,
        }));
    }

    let rows = sqlx::query_as::<_, Post>(
        r#"
        SELECT p.*
        FROM posts p, websearch_to_tsquery('simple', $1) AS q
        WHERE p.status = 'published' AND p.search_vector @@ q
        ORDER BY ts_rank(p.search_vector, q) DESC, p.published_at DESC NULLS LAST
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(query)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM posts p, websearch_to_tsquery('simple', $1) AS q
        WHERE p.status = 'published' AND p.search_vector @@ q
        "#,
    )
    .bind(query)
    .fetch_one(&state.db)
    .await?;

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
