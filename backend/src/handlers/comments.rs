use axum::{
    Json,
    extract::{Path, State},
};
use validator::Validate;

use crate::{
    error::{AppError, AppResult},
    models::comment::{Comment, NewCommentInput},
    state::AppState,
};

pub async fn list_approved(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<Vec<Comment>>> {
    let comments = sqlx::query_as::<_, Comment>(
        r#"
        SELECT c.*
        FROM comments c
        JOIN posts p ON p.id = c.post_id
        WHERE p.slug = $1 AND p.status = 'published' AND c.status = 'approved'
        ORDER BY c.created_at ASC
        "#,
    )
    .bind(&slug)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(comments))
}

pub async fn submit(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(input): Json<NewCommentInput>,
) -> AppResult<Json<Comment>> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let post_id: i64 = sqlx::query_scalar(
        r#"SELECT id FROM posts WHERE slug = $1 AND status = 'published'"#,
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let effective_parent_id = if let Some(parent_id) = input.parent_id {
        let parent: Option<(i64, Option<i64>)> = sqlx::query_as(
            r#"SELECT id, parent_id FROM comments WHERE id = $1 AND post_id = $2 AND status = 'approved'"#,
        )
        .bind(parent_id)
        .bind(post_id)
        .fetch_optional(&state.db)
        .await?;
        match parent {
            None => {
                return Err(AppError::BadRequest(
                    "parent comment not found or not approved".into(),
                ));
            }
            Some((_, Some(grandparent_id))) => Some(grandparent_id),
            Some((id, None)) => Some(id),
        }
    } else {
        None
    };

    let row = sqlx::query_as::<_, Comment>(
        r#"
        INSERT INTO comments (post_id, parent_id, author_name, author_email, content, status)
        VALUES ($1, $2, $3, $4, $5, 'pending')
        RETURNING *
        "#,
    )
    .bind(post_id)
    .bind(effective_parent_id)
    .bind(&input.author_name)
    .bind(input.author_email.as_deref())
    .bind(&input.content)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(row))
}
