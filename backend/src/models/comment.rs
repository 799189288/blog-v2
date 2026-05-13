use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Comment {
    pub id: i64,
    pub post_id: i64,
    pub parent_id: Option<i64>,
    pub author_name: String,
    pub author_email: Option<String>,
    pub content: String,
    pub status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct NewCommentInput {
    #[validate(length(min = 1, max = 64))]
    pub author_name: String,
    #[validate(email)]
    pub author_email: Option<String>,
    #[validate(length(min = 1, max = 4000))]
    pub content: String,
    pub parent_id: Option<i64>,
    /// Honeypot. Real browsers leave this empty (the field is hidden via
    /// CSS); naive scrapers happily fill in anything that looks like a
    /// URL field. Non-empty values get silently discarded server-side.
    #[serde(default)]
    pub website: Option<String>,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdateCommentStatus {
    /// "approved" or "spam" or "pending"
    pub status: String,
}
