use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::HeaderMap,
};
use std::net::SocketAddr;
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Json(input): Json<NewCommentInput>,
) -> AppResult<Json<Comment>> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Honeypot: a real form never carries a non-empty `website`. Pretend
    // we accepted the comment (200 with the original payload echoed back
    // as 'pending') so bots don't learn that the field is the trap.
    if let Some(w) = input.website.as_deref() {
        if !w.trim().is_empty() {
            tracing::info!(slug = %slug, "honeypot triggered, dropping comment");
            return Ok(Json(fake_pending(&input)));
        }
    }

    let ip = client_ip(&headers, addr);
    if !state.comment_allowed(&ip) {
        return Err(AppError::BadRequest(
            "you're commenting too fast, please wait a bit".into(),
        ));
    }

    let post: Option<(i64, String)> = sqlx::query_as(
        r#"SELECT id, title FROM posts WHERE slug = $1 AND status = 'published'"#,
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await?;
    let (post_id, post_title) = post.ok_or(AppError::NotFound)?;

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

    // Auto-route obvious spam to the spam bucket. Keyword check is a
    // simple lowercased substring match against name + email + body.
    let initial_status = if matches_blocklist(&state.comment_blocklist, &input) {
        "spam"
    } else {
        "pending"
    };

    let row = sqlx::query_as::<_, Comment>(
        r#"
        INSERT INTO comments (post_id, parent_id, author_name, author_email, content, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(post_id)
    .bind(effective_parent_id)
    .bind(&input.author_name)
    .bind(input.author_email.as_deref())
    .bind(&input.content)
    .bind(initial_status)
    .fetch_one(&state.db)
    .await?;

    state.record_comment(&ip);

    // Best-effort email notification. Spawn so the HTTP response isn't
    // tied to SMTP latency, and so a flaky SMTP server never produces a
    // user-facing 500 for a comment that *was* persisted.
    if let Some(notifier) = state.notifier.clone() {
        let slug = slug.clone();
        let post_title = post_title.clone();
        let author_name = row.author_name.clone();
        let author_email = row.author_email.clone();
        let content = row.content.clone();
        let status = row.status.clone();
        tokio::spawn(async move {
            notifier
                .send_new_comment(
                    &slug,
                    &post_title,
                    &author_name,
                    author_email.as_deref(),
                    &content,
                    &status,
                )
                .await;
        });
    }

    Ok(Json(row))
}

fn matches_blocklist(blocklist: &[String], input: &NewCommentInput) -> bool {
    if blocklist.is_empty() {
        return false;
    }
    let haystack = format!(
        "{} {} {}",
        input.author_name,
        input.author_email.as_deref().unwrap_or(""),
        input.content
    )
    .to_lowercase();
    blocklist.iter().any(|needle| haystack.contains(needle))
}

fn client_ip(headers: &HeaderMap, addr: SocketAddr) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| addr.ip().to_string())
}

/// Synthetic "pending" response for honeypot-tripped submissions. We
/// don't touch the DB but the frontend gets the same shape it expects,
/// so the user/bot sees nothing unusual happen.
fn fake_pending(input: &NewCommentInput) -> Comment {
    use time::OffsetDateTime;
    Comment {
        id: 0,
        post_id: 0,
        parent_id: input.parent_id,
        author_name: input.author_name.clone(),
        author_email: input.author_email.clone(),
        content: input.content.clone(),
        status: "pending".into(),
        created_at: OffsetDateTime::now_utc(),
    }
}
