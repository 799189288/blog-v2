use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TagWithCount {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub post_count: i64,
}
