use axum::{Json, extract::State};

use crate::{error::AppResult, models::tag::TagWithCount, state::AppState};

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
