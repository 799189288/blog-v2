use axum::{
    Json,
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
};
use serde_json::json;
use std::net::SocketAddr;
use validator::Validate;

use crate::{
    audit,
    auth::AuthUser,
    error::{AppError, AppResult},
    models::dict::{DictItem, DictType, UpsertDictItem, UpsertDictType},
    state::AppState,
};

// ---------- types ----------

pub async fn list_types(
    State(state): State<AppState>,
    _user: AuthUser,
) -> AppResult<Json<Vec<DictType>>> {
    let rows = sqlx::query_as::<_, DictType>(
        r#"
        SELECT id, code, name_zh, name_en, is_system, created_at
        FROM dict_types
        ORDER BY code
        "#,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn create_type(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<UpsertDictType>,
) -> AppResult<Json<DictType>> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let row = sqlx::query_as::<_, DictType>(
        r#"
        INSERT INTO dict_types (code, name_zh, name_en, is_system)
        VALUES ($1, $2, $3, FALSE)
        RETURNING id, code, name_zh, name_en, is_system, created_at
        "#,
    )
    .bind(input.code.trim())
    .bind(input.name_zh.trim())
    .bind(input.name_en.trim())
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            AppError::BadRequest("dict type code already exists".into())
        }
        _ => AppError::from(e),
    })?;

    audit::record(
        &state.db,
        &user,
        "dict.type.create",
        Some("dict_type"),
        Some(row.id),
        Some(json!({ "code": row.code })),
        Some(addr.ip()),
    )
    .await;

    Ok(Json(row))
}

pub async fn update_type(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<i64>,
    Json(input): Json<UpsertDictType>,
) -> AppResult<Json<DictType>> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let prev = sqlx::query_as::<_, DictType>(
        r#"SELECT id, code, name_zh, name_en, is_system, created_at FROM dict_types WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    // System types: code is immutable (other modules query by it). Only the
    // bilingual display names may be edited.
    let new_code = if prev.is_system {
        prev.code.clone()
    } else {
        input.code.trim().to_string()
    };

    let row = sqlx::query_as::<_, DictType>(
        r#"
        UPDATE dict_types
        SET code = $2, name_zh = $3, name_en = $4
        WHERE id = $1
        RETURNING id, code, name_zh, name_en, is_system, created_at
        "#,
    )
    .bind(id)
    .bind(&new_code)
    .bind(input.name_zh.trim())
    .bind(input.name_en.trim())
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            AppError::BadRequest("dict type code already exists".into())
        }
        _ => AppError::from(e),
    })?;

    state.invalidate_dict_type(&prev.code).await;
    if new_code != prev.code {
        state.invalidate_dict_type(&new_code).await;
    }

    audit::record(
        &state.db,
        &user,
        "dict.type.update",
        Some("dict_type"),
        Some(row.id),
        Some(json!({ "code": row.code })),
        Some(addr.ip()),
    )
    .await;

    Ok(Json(row))
}

pub async fn delete_type(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let existing = sqlx::query_as::<_, DictType>(
        r#"SELECT id, code, name_zh, name_en, is_system, created_at FROM dict_types WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    if existing.is_system {
        return Err(AppError::BadRequest("cannot delete system dict type".into()));
    }

    sqlx::query(r#"DELETE FROM dict_types WHERE id = $1"#)
        .bind(id)
        .execute(&state.db)
        .await?;

    state.invalidate_dict_type(&existing.code).await;

    audit::record(
        &state.db,
        &user,
        "dict.type.delete",
        Some("dict_type"),
        Some(existing.id),
        Some(json!({ "code": existing.code })),
        Some(addr.ip()),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

// ---------- items ----------

pub async fn list_items(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(type_id): Path<i64>,
) -> AppResult<Json<Vec<DictItem>>> {
    // Verify type exists for nicer error than empty list.
    let exists: Option<i64> = sqlx::query_scalar(r#"SELECT id FROM dict_types WHERE id = $1"#)
        .bind(type_id)
        .fetch_optional(&state.db)
        .await?;
    if exists.is_none() {
        return Err(AppError::NotFound);
    }

    let rows = sqlx::query_as::<_, DictItem>(
        r#"
        SELECT id, type_id, code, label_zh, label_en, sort, enabled, created_at
        FROM dict_items
        WHERE type_id = $1
        ORDER BY sort, id
        "#,
    )
    .bind(type_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn create_item(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(type_id): Path<i64>,
    Json(input): Json<UpsertDictItem>,
) -> AppResult<Json<DictItem>> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let type_code: String = sqlx::query_scalar(r#"SELECT code FROM dict_types WHERE id = $1"#)
        .bind(type_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let row = sqlx::query_as::<_, DictItem>(
        r#"
        INSERT INTO dict_items (type_id, code, label_zh, label_en, sort, enabled)
        VALUES ($1, $2, $3, $4, COALESCE($5, 0), COALESCE($6, TRUE))
        RETURNING id, type_id, code, label_zh, label_en, sort, enabled, created_at
        "#,
    )
    .bind(type_id)
    .bind(input.code.trim())
    .bind(input.label_zh.trim())
    .bind(input.label_en.trim())
    .bind(input.sort)
    .bind(input.enabled)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            AppError::BadRequest("item code already exists in this type".into())
        }
        _ => AppError::from(e),
    })?;

    state.invalidate_dict_type(&type_code).await;

    audit::record(
        &state.db,
        &user,
        "dict.item.create",
        Some("dict_item"),
        Some(row.id),
        Some(json!({ "type_code": type_code, "code": row.code })),
        Some(addr.ip()),
    )
    .await;

    Ok(Json(row))
}

pub async fn update_item(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<i64>,
    Json(input): Json<UpsertDictItem>,
) -> AppResult<Json<DictItem>> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let item = sqlx::query_as::<_, DictItem>(
        r#"
        SELECT id, type_id, code, label_zh, label_en, sort, enabled, created_at
        FROM dict_items WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let (type_code, type_is_system): (String, bool) = sqlx::query_as(
        r#"SELECT code, is_system FROM dict_types WHERE id = $1"#,
    )
    .bind(item.type_id)
    .fetch_one(&state.db)
    .await?;

    // For system dict items, code is immutable (backend joins/filters on it).
    // For business items, the user may rename freely.
    let new_code = if type_is_system {
        item.code.clone()
    } else {
        input.code.trim().to_string()
    };

    let row = sqlx::query_as::<_, DictItem>(
        r#"
        UPDATE dict_items
        SET code = $2,
            label_zh = $3,
            label_en = $4,
            sort = COALESCE($5, sort),
            enabled = COALESCE($6, enabled)
        WHERE id = $1
        RETURNING id, type_id, code, label_zh, label_en, sort, enabled, created_at
        "#,
    )
    .bind(id)
    .bind(&new_code)
    .bind(input.label_zh.trim())
    .bind(input.label_en.trim())
    .bind(input.sort)
    .bind(input.enabled)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            AppError::BadRequest("item code already exists in this type".into())
        }
        _ => AppError::from(e),
    })?;

    state.invalidate_dict_type(&type_code).await;

    audit::record(
        &state.db,
        &user,
        "dict.item.update",
        Some("dict_item"),
        Some(row.id),
        Some(json!({ "type_code": type_code, "code": row.code })),
        Some(addr.ip()),
    )
    .await;

    Ok(Json(row))
}

pub async fn delete_item(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    let item = sqlx::query_as::<_, DictItem>(
        r#"
        SELECT id, type_id, code, label_zh, label_en, sort, enabled, created_at
        FROM dict_items WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let (type_code, type_is_system): (String, bool) = sqlx::query_as(
        r#"SELECT code, is_system FROM dict_types WHERE id = $1"#,
    )
    .bind(item.type_id)
    .fetch_one(&state.db)
    .await?;

    if type_is_system {
        return Err(AppError::BadRequest(
            "cannot delete items from a system dict type; disable instead".into(),
        ));
    }

    sqlx::query(r#"DELETE FROM dict_items WHERE id = $1"#)
        .bind(id)
        .execute(&state.db)
        .await?;

    state.invalidate_dict_type(&type_code).await;

    audit::record(
        &state.db,
        &user,
        "dict.item.delete",
        Some("dict_item"),
        Some(item.id),
        Some(json!({ "type_code": type_code, "code": item.code })),
        Some(addr.ip()),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
