use axum::{Json, extract::{Path, State}};

use crate::{error::{AppError, AppResult}, models::tag::{Tag, TagWithCount}, state::AppState};

pub async fn list_with_counts(State(state): State<AppState>) -> AppResult<Json<Vec<TagWithCount>>> {
    let rows = sqlx::query_as::<_, TagWithCount>(
        r#"
        SELECT t.id, t.name, t.slug, COUNT(p.id) AS post_count
        FROM tags t
        LEFT JOIN post_tags pt ON pt.tag_id = t.id
        LEFT JOIN posts p ON p.id = pt.post_id AND p.status = 'published'
        GROUP BY t.id
        ORDER BY t.name
        "#,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn get_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Tag>> {
    let row = sqlx::query_as::<_, Tag>(
        r#"SELECT id, name, slug FROM tags WHERE slug = $1"#,
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(row))
}
