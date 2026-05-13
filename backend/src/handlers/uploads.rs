use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{
    Json,
    extract::{ConnectInfo, Multipart, State},
};
use serde::Serialize;
use serde_json::json;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    audit,
    auth::AuthUser,
    error::{AppError, AppResult},
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    /// Public URL the editor can embed straight into markdown.
    pub url: String,
    pub filename: String,
    pub size: usize,
}

/// Accepts a single `multipart/form-data` field named `file`. Saves to
/// `<upload_dir>/<uuid>.<ext>` and returns a `/uploads/...` URL. The
/// admin editor (`md-editor-v3`) calls this from its `onUploadImg` hook.
///
/// Constraints:
///   * mime-type must start with `image/`
///   * extension restricted to a small whitelist so we never write things
///     we can't safely serve back
///   * total bytes <= `state.uploads.max_bytes` (default 5 MB)
pub async fn upload(
    State(state): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut multipart: Multipart,
) -> AppResult<Json<UploadResponse>> {
    let max_bytes = state.uploads.max_bytes;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart: {e}")))?
    {
        if field.name() != Some("file") {
            continue;
        }

        let orig_name = field.file_name().unwrap_or("upload").to_string();
        let content_type = field.content_type().unwrap_or("").to_string();
        if !content_type.starts_with("image/") {
            return Err(AppError::BadRequest(format!(
                "only image/* uploads are accepted, got '{content_type}'"
            )));
        }

        let ext = pick_extension(&orig_name, &content_type)
            .ok_or_else(|| AppError::BadRequest("unsupported image type".into()))?;

        let bytes = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("read body: {e}")))?;
        if bytes.len() > max_bytes {
            return Err(AppError::BadRequest(format!(
                "file too large: {} bytes (max {max_bytes})",
                bytes.len()
            )));
        }
        if bytes.is_empty() {
            return Err(AppError::BadRequest("empty file".into()));
        }

        let filename = format!("{}.{ext}", Uuid::new_v4());
        let path = PathBuf::from(&state.uploads.dir).join(&filename);
        let mut f = fs::File::create(&path)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("create file: {e}")))?;
        f.write_all(&bytes)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("write: {e}")))?;
        f.flush()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("flush: {e}")))?;

        // URL the editor will embed. Relative so it survives moving
        // between origins — in dev the SPA's Vite proxy forwards
        // /uploads to the backend; in production the reverse proxy does
        // the same. No env var to babysit.
        let url = format!("/uploads/{filename}");
        audit::record(
            &state.db,
            &user,
            "upload.create",
            Some("upload"),
            None,
            Some(json!({ "filename": filename, "size": bytes.len(), "orig": orig_name })),
            Some(addr.ip()),
        )
        .await;

        return Ok(Json(UploadResponse {
            url,
            filename,
            size: bytes.len(),
        }));
    }

    Err(AppError::BadRequest(
        "missing 'file' field in multipart".into(),
    ))
}

/// Pick a safe extension. Prefer the original filename's extension when
/// it's in the whitelist; otherwise fall back to mapping content-type.
/// Anything not in the whitelist is rejected — we never invent a name.
fn pick_extension(orig_name: &str, content_type: &str) -> Option<&'static str> {
    let from_name = orig_name
        .rsplit('.')
        .next()
        .map(|s| s.to_ascii_lowercase())
        .filter(|s| !s.is_empty() && s.len() <= 5);
    let from_type = content_type
        .strip_prefix("image/")
        .map(|s| s.to_ascii_lowercase());

    for candidate in [from_name, from_type].into_iter().flatten() {
        match candidate.as_str() {
            "jpg" | "jpeg" => return Some("jpg"),
            "png" => return Some("png"),
            "gif" => return Some("gif"),
            "webp" => return Some("webp"),
            "svg" | "svg+xml" => return Some("svg"),
            _ => continue,
        }
    }
    None
}
