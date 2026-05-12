use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use validator::Validate;

use crate::{
    audit,
    auth::AuthUser,
    error::{AppError, AppResult},
    models::tag::{Tag, TagWithCount},
    state::AppState,
};

pub async fn list(State(state): State<AppState>, _user: AuthUser) -> AppResult<Json<Vec<TagWithCount>>> {
    // Admin view: count all posts (drafts included) attached to each tag.
    let rows = sqlx::query_as::<_, TagWithCount>(
        r#"
        SELECT t.id, t.name, t.slug, COUNT(pt.post_id) AS post_count
        FROM tags t
        LEFT JOIN post_tags pt ON pt.tag_id = t.id
        GROUP BY t.id
        ORDER BY t.name
        "#,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpsertTagInput {
    #[validate(length(min = 1, max = 64))]
    pub name: String,
    /// Optional override; if omitted/blank, derived from `name`.
    pub slug: Option<String>,
}

fn resolve_slug(input: &UpsertTagInput) -> String {
    let candidate = input
        .slug
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| slug::slugify(s))
        .unwrap_or_else(|| slug::slugify(&input.name));
    candidate
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<UpsertTagInput>,
) -> AppResult<Json<Tag>> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let slug_value = resolve_slug(&input);
    if slug_value.is_empty() {
        return Err(AppError::BadRequest("slug cannot be empty".into()));
    }

    let row = sqlx::query_as::<_, Tag>(
        r#"INSERT INTO tags (name, slug) VALUES ($1, $2) RETURNING id, name, slug"#,
    )
    .bind(input.name.trim())
    .bind(&slug_value)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            AppError::BadRequest("tag name or slug already exists".into())
        }
        _ => AppError::from(e),
    })?;

    audit::record(
        &state.db,
        &user,
        "tag.create",
        Some("tag"),
        Some(row.id),
        Some(json!({ "name": row.name, "slug": row.slug })),
        Some(addr.ip()),
    )
    .await;

    Ok(Json(row))
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<i64>,
    Json(input): Json<UpsertTagInput>,
) -> AppResult<Json<Tag>> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let prev = sqlx::query_as::<_, Tag>(r#"SELECT id, name, slug FROM tags WHERE id = $1"#)
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let slug_value = resolve_slug(&input);
    if slug_value.is_empty() {
        return Err(AppError::BadRequest("slug cannot be empty".into()));
    }

    let row = sqlx::query_as::<_, Tag>(
        r#"
        UPDATE tags SET name = $2, slug = $3
        WHERE id = $1
        RETURNING id, name, slug
        "#,
    )
    .bind(id)
    .bind(input.name.trim())
    .bind(&slug_value)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            AppError::BadRequest("tag name or slug already exists".into())
        }
        _ => AppError::from(e),
    })?;

    audit::record(
        &state.db,
        &user,
        "tag.update",
        Some("tag"),
        Some(row.id),
        Some(json!({
            "from": { "name": prev.name, "slug": prev.slug },
            "to":   { "name": row.name,  "slug": row.slug },
        })),
        Some(addr.ip()),
    )
    .await;

    Ok(Json(row))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let existing = sqlx::query_as::<_, Tag>(r#"SELECT id, name, slug FROM tags WHERE id = $1"#)
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    // post_tags.tag_id has ON DELETE CASCADE — relations get cleaned up
    // automatically when we delete the tag itself.
    sqlx::query(r#"DELETE FROM tags WHERE id = $1"#)
        .bind(id)
        .execute(&state.db)
        .await?;

    audit::record(
        &state.db,
        &user,
        "tag.delete",
        Some("tag"),
        Some(existing.id),
        Some(json!({ "name": existing.name, "slug": existing.slug })),
        Some(addr.ip()),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
