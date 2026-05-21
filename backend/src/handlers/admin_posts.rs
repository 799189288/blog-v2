use axum::{
    Json,
    extract::{ConnectInfo, Path, Query, State},
};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use time::OffsetDateTime;
use validator::Validate;

use crate::{
    audit,
    auth::AuthUser,
    error::{AppError, AppResult},
    handlers::posts::Paginated,
    markdown,
    models::{
        post::{CreatePostInput, Post, PostDetail, PostSummary, UpdatePostInput},
        tag::Tag,
    },
    state::AppState,
};

use super::posts::{load_tags_for_post, load_tags_for_posts};

fn gen_preview_token() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

#[derive(Debug, Deserialize)]
pub struct AdminListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list_all(
    State(state): State<AppState>,
    _user: AuthUser,
    Query(q): Query<AdminListQuery>,
) -> AppResult<Json<Paginated<PostSummary>>> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let rows = sqlx::query_as::<_, Post>(
        r#"
        SELECT * FROM posts
        ORDER BY
            CASE WHEN status = 'draft' THEN 0 ELSE 1 END,
            COALESCE(published_at, updated_at) DESC,
            id DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM posts"#)
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
                cover_image: p.cover_image,
                published_at: p.published_at,
                created_at: p.created_at,
                tags,
            }
        })
        .collect();
    Ok(Json(Paginated { items, page, per_page, total }))
}

pub async fn get_by_id(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<i64>,
) -> AppResult<Json<PostDetail>> {
    let post = sqlx::query_as::<_, Post>(r#"SELECT * FROM posts WHERE id = $1"#)
        .bind(id)
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
        views: post.views,
        word_count: post.word_count,
        reading_time_min: post.reading_time_min,
        cover_image: post.cover_image,
        preview_token: post.preview_token,
        published_at: post.published_at,
        created_at: post.created_at,
        updated_at: post.updated_at,
        tags,
    }))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<CreatePostInput>,
) -> AppResult<Json<PostDetail>> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let status = input.status.unwrap_or_else(|| "draft".into());
    if status != "draft" && status != "published" {
        return Err(AppError::BadRequest(
            "status must be draft or published".into(),
        ));
    }
    let slug_value = match input.slug {
        Some(s) if !s.trim().is_empty() => slug::slugify(&s),
        _ => slug::slugify(&input.title),
    };
    if slug_value.is_empty() {
        return Err(AppError::BadRequest("slug could not be generated".into()));
    }

    let html = markdown::render(&input.content_md);
    let excerpt = match input.excerpt {
        Some(s) if !s.trim().is_empty() => Some(s),
        _ => Some(markdown::excerpt(&input.content_md, 200)),
    };
    let word_count = markdown::word_count(&input.content_md) as i32;
    let reading_time = markdown::reading_time_min(&input.content_md);
    let published_at: Option<OffsetDateTime> = if status == "published" {
        Some(OffsetDateTime::now_utc())
    } else {
        None
    };
    // Drafts get a private preview token so the author can share an
    // unlisted URL. Published posts don't need one (they're public).
    let preview_token: Option<String> = if status == "draft" {
        Some(gen_preview_token())
    } else {
        None
    };

    let mut tx = state.db.begin().await?;

    let post = sqlx::query_as::<_, Post>(
        r#"
        INSERT INTO posts (slug, title, excerpt, content_md, content_html, status, author_id, published_at, word_count, reading_time_min, preview_token)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
    )
    .bind(&slug_value)
    .bind(&input.title)
    .bind(excerpt.as_deref())
    .bind(&input.content_md)
    .bind(&html)
    .bind(&status)
    .bind(user.user_id)
    .bind(published_at)
    .bind(word_count)
    .bind(reading_time)
    .bind(preview_token.as_deref())
    .fetch_one(&mut *tx)
    .await
    .map_err(map_slug_conflict)?;

    let tags = upsert_tags(&mut tx, post.id, input.tags.as_deref().unwrap_or(&[])).await?;

    tx.commit().await?;

    audit::record(
        &state.db,
        &user,
        "post.create",
        Some("post"),
        Some(post.id),
        Some(json!({ "title": post.title, "slug": post.slug, "status": post.status })),
        Some(addr.ip()),
    )
    .await;

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
        preview_token: post.preview_token,
        published_at: post.published_at,
        created_at: post.created_at,
        updated_at: post.updated_at,
        tags,
    }))
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<i64>,
    Json(input): Json<UpdatePostInput>,
) -> AppResult<Json<PostDetail>> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let mut tx = state.db.begin().await?;
    let existing = sqlx::query_as::<_, Post>(r#"SELECT * FROM posts WHERE id = $1"#)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;

    let title = input.title.unwrap_or(existing.title.clone());
    let content_md = input.content_md.unwrap_or(existing.content_md.clone());
    let html = markdown::render(&content_md);
    let excerpt = match input.excerpt {
        Some(s) if s.trim().is_empty() => Some(markdown::excerpt(&content_md, 200)),
        Some(s) => Some(s),
        None => existing.excerpt.clone(),
    };
    let slug_value = match input.slug {
        Some(s) if !s.trim().is_empty() => slug::slugify(&s),
        Some(_) => slug::slugify(&title),
        None => existing.slug.clone(),
    };
    let new_status = input.status.unwrap_or_else(|| existing.status.clone());
    if new_status != "draft" && new_status != "published" {
        return Err(AppError::BadRequest(
            "status must be draft or published".into(),
        ));
    }
    let published_at = if new_status == "published" {
        existing
            .published_at
            .or(Some(OffsetDateTime::now_utc()))
    } else {
        None
    };
    // Preview token lifecycle:
    //   published → published : carry forward (always None anyway)
    //   draft     → draft     : keep the existing token so a shared link
    //                           still works after edits
    //   published → draft     : mint a fresh token
    //   draft     → published : drop the token (post is now public)
    let preview_token: Option<String> = if new_status == "draft" {
        existing
            .preview_token
            .clone()
            .or_else(|| Some(gen_preview_token()))
    } else {
        None
    };

    let post = sqlx::query_as::<_, Post>(
        r#"
        UPDATE posts
        SET slug = $2, title = $3, excerpt = $4, content_md = $5, content_html = $6,
            status = $7, published_at = $8, word_count = $9, reading_time_min = $10,
            preview_token = $11,
            updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&slug_value)
    .bind(&title)
    .bind(excerpt.as_deref())
    .bind(&content_md)
    .bind(&html)
    .bind(&new_status)
    .bind(published_at)
    .bind(markdown::word_count(&content_md) as i32)
    .bind(markdown::reading_time_min(&content_md))
    .bind(preview_token.as_deref())
    .fetch_one(&mut *tx)
    .await
    .map_err(map_slug_conflict)?;

    let tags = if let Some(tags) = input.tags {
        sqlx::query(r#"DELETE FROM post_tags WHERE post_id = $1"#)
            .bind(post.id)
            .execute(&mut *tx)
            .await?;
        upsert_tags(&mut tx, post.id, &tags).await?
    } else {
        load_tags_for_post_tx(&mut tx, post.id).await?
    };

    tx.commit().await?;

    audit::record(
        &state.db,
        &user,
        "post.update",
        Some("post"),
        Some(post.id),
        Some(json!({
            "title": post.title,
            "slug": post.slug,
            "status": post.status,
            "prev_status": existing.status,
            "prev_title": existing.title,
        })),
        Some(addr.ip()),
    )
    .await;

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
        preview_token: post.preview_token,
        published_at: post.published_at,
        created_at: post.created_at,
        updated_at: post.updated_at,
        tags,
    }))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<i64>,
) -> AppResult<axum::http::StatusCode> {
    let existing = sqlx::query_as::<_, Post>(r#"SELECT * FROM posts WHERE id = $1"#)
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    sqlx::query(r#"DELETE FROM posts WHERE id = $1"#)
        .bind(id)
        .execute(&state.db)
        .await?;

    audit::record(
        &state.db,
        &user,
        "post.delete",
        Some("post"),
        Some(existing.id),
        Some(json!({ "title": existing.title, "slug": existing.slug })),
        Some(addr.ip()),
    )
    .await;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

fn map_slug_conflict(e: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(ref db) = e {
        if db.constraint() == Some("posts_slug_key") {
            return AppError::Conflict("slug already in use".into());
        }
    }
    AppError::Database(e)
}

async fn upsert_tags(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    post_id: i64,
    names: &[String],
) -> AppResult<Vec<Tag>> {
    let mut attached = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for raw in names {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        let slug_value = slug::slugify(name);
        if slug_value.is_empty() || !seen.insert(slug_value.clone()) {
            continue;
        }
        let tag = sqlx::query_as::<_, Tag>(
            r#"
            INSERT INTO tags (name, slug) VALUES ($1, $2)
            ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
            RETURNING id, name, slug
            "#,
        )
        .bind(name)
        .bind(&slug_value)
        .fetch_one(&mut **tx)
        .await?;

        sqlx::query(
            r#"INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
        )
        .bind(post_id)
        .bind(tag.id)
        .execute(&mut **tx)
        .await?;
        attached.push(tag);
    }
    attached.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(attached)
}

async fn load_tags_for_post_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    post_id: i64,
) -> AppResult<Vec<Tag>> {
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
    .fetch_all(&mut **tx)
    .await?;
    Ok(tags)
}
