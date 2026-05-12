use axum::{Json, extract::{Path, State}};
use std::sync::Arc;

use crate::{
    error::{AppError, AppResult},
    models::dict::{DictItem, DictItemPublic},
    state::AppState,
};

/// Public endpoint: returns enabled items for a dict type, sorted.
///
/// Uses a per-type in-memory cache keyed by `type_code`. Admin writes
/// invalidate the relevant key, so subsequent reads repopulate from DB.
pub async fn get_by_type(
    State(state): State<AppState>,
    Path(type_code): Path<String>,
) -> AppResult<Json<Vec<DictItemPublic>>> {
    if let Some(cached) = state.dict_cache.read().await.get(&type_code).cloned() {
        return Ok(Json((*cached).clone()));
    }

    // Look up the type id first so we can return 404 for unknown codes
    // instead of an empty array (which would conceal typos).
    let type_id: i64 = sqlx::query_scalar(
        r#"SELECT id FROM dict_types WHERE code = $1"#,
    )
    .bind(&type_code)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let items: Vec<DictItem> = sqlx::query_as::<_, DictItem>(
        r#"
        SELECT id, type_id, code, label_zh, label_en, sort, enabled, created_at
        FROM dict_items
        WHERE type_id = $1 AND enabled = TRUE
        ORDER BY sort, id
        "#,
    )
    .bind(type_id)
    .fetch_all(&state.db)
    .await?;

    let public: Vec<DictItemPublic> = items.iter().map(DictItemPublic::from).collect();
    state
        .dict_cache
        .write()
        .await
        .insert(type_code, Arc::new(public.clone()));
    Ok(Json(public))
}
