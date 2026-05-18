use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Post {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub excerpt: Option<String>,
    pub content_md: String,
    pub content_html: String,
    pub status: String,
    pub author_id: i64,
    pub views: i64,
    pub word_count: i32,
    pub reading_time_min: i32,
    pub cover_image: Option<String>,
    pub preview_token: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub published_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostSummary {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub excerpt: Option<String>,
    pub status: String,
    pub views: i64,
    pub word_count: i32,
    pub reading_time_min: i32,
    pub cover_image: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub published_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub tags: Vec<super::tag::Tag>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostDetail {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub excerpt: Option<String>,
    pub content_md: String,
    pub content_html: String,
    pub status: String,
    pub views: i64,
    pub word_count: i32,
    pub reading_time_min: i32,
    pub cover_image: Option<String>,
    /// Present on drafts so the admin UI can render a private preview
    /// link. Always None for published posts.
    pub preview_token: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub published_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub tags: Vec<super::tag::Tag>,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreatePostInput {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1))]
    pub content_md: String,
    pub excerpt: Option<String>,
    pub slug: Option<String>,
    /// "draft" or "published"
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdatePostInput {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    pub content_md: Option<String>,
    pub excerpt: Option<String>,
    pub slug: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
}
