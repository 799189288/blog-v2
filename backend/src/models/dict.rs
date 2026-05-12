use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct DictType {
    pub id: i64,
    pub code: String,
    pub name_zh: String,
    pub name_en: String,
    pub is_system: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct DictItem {
    pub id: i64,
    pub type_id: i64,
    pub code: String,
    pub label_zh: String,
    pub label_en: String,
    pub sort: i32,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Public-facing item — drops bookkeeping fields the frontend doesn't need.
#[derive(Debug, Clone, Serialize)]
pub struct DictItemPublic {
    pub code: String,
    pub label_zh: String,
    pub label_en: String,
    pub sort: i32,
}

impl From<&DictItem> for DictItemPublic {
    fn from(it: &DictItem) -> Self {
        Self {
            code: it.code.clone(),
            label_zh: it.label_zh.clone(),
            label_en: it.label_en.clone(),
            sort: it.sort,
        }
    }
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpsertDictItem {
    #[validate(length(min = 1, max = 128))]
    pub code: String,
    #[validate(length(min = 1, max = 128))]
    pub label_zh: String,
    #[validate(length(min = 1, max = 128))]
    pub label_en: String,
    pub sort: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpsertDictType {
    #[validate(length(min = 1, max = 64))]
    pub code: String,
    #[validate(length(min = 1, max = 64))]
    pub name_zh: String,
    #[validate(length(min = 1, max = 64))]
    pub name_en: String,
}
