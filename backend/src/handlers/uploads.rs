use std::io::Cursor;
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::body::Bytes;
use axum::{
    Json,
    extract::{ConnectInfo, Multipart, State},
};
use image::{DynamicImage, ImageFormat, ImageReader, Limits, imageops::FilterType};
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

const MAX_DIMENSION: u32 = 2048;
const THUMB_MAX_DIMENSION: u32 = 400;
const WEBP_QUALITY: f32 = 85.0;
const DECODE_MAX_WH: u32 = 8192;
const DECODE_MAX_ALLOC: u64 = 256 * 1024 * 1024;

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    /// Public URL the editor can embed straight into markdown.
    pub url: String,
    /// Thumbnail URL when one was generated. `None` for pass-through
    /// formats (GIF, animated WebP) where we don't decode the source.
    pub thumb_url: Option<String>,
    pub filename: String,
    pub size: usize,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Accepts a single `multipart/form-data` field named `file`. The pipeline:
///   * sniff the actual format from bytes (do not trust client MIME / name)
///   * static JPEG/PNG/WebP → decode, strip EXIF, clamp to 2048px, encode
///     as lossy WebP + generate a 400px-wide WebP thumbnail
///   * GIF and animated WebP → store as-is to preserve animation
///   * anything else → 400
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
        // Coarse pre-filter only — the real check is byte-level sniffing
        // after we've buffered. Reject obvious non-images cheaply.
        if !content_type.is_empty() && !content_type.starts_with("image/") {
            return Err(AppError::BadRequest(format!(
                "only image/* uploads are accepted, got '{content_type}'"
            )));
        }

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

        let original_size = bytes.len();

        // All CPU-bound work (sniff, decode, resize, encode) runs on the
        // blocking pool so we don't stall the async runtime.
        let processed = tokio::task::spawn_blocking({
            let bytes = bytes.clone();
            move || process(bytes)
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("join: {e}")))?
        .map_err(map_process_error)?;

        let stem = Uuid::new_v4();
        let (filename, thumb_filename) = match &processed {
            Processed::Reencoded { .. } => (
                format!("{stem}.webp"),
                Some(format!("{stem}-thumb.webp")),
            ),
            Processed::Passthrough { ext, .. } => (format!("{stem}.{ext}"), None),
        };

        let primary_path = PathBuf::from(&state.uploads.dir).join(&filename);
        let thumb_path = thumb_filename
            .as_ref()
            .map(|n| PathBuf::from(&state.uploads.dir).join(n));

        // Write thumb first so a thumb failure leaves no orphan, and a
        // primary failure cleans up the (already written) thumb.
        if let (Some(thumb_path), Processed::Reencoded { thumb_bytes, .. }) =
            (thumb_path.as_ref(), &processed)
        {
            write_all(thumb_path, thumb_bytes).await?;
        }

        let primary_bytes = processed.primary_bytes();
        if let Err(e) = write_all(&primary_path, primary_bytes).await {
            if let Some(p) = &thumb_path {
                if let Err(rm) = fs::remove_file(p).await {
                    tracing::warn!(
                        path = %p.display(),
                        error = %rm,
                        "failed to clean up orphan thumbnail after primary write failure",
                    );
                }
            }
            return Err(e);
        }

        let (width, height, processed_flag) = match &processed {
            Processed::Reencoded { width, height, .. } => (Some(*width), Some(*height), true),
            Processed::Passthrough { .. } => (None, None, false),
        };

        let url = format!("/uploads/{filename}");
        let thumb_url = thumb_filename.as_ref().map(|n| format!("/uploads/{n}"));
        let size = primary_bytes.len();

        audit::record(
            &state.db,
            &user,
            "upload.create",
            Some("upload"),
            None,
            Some(json!({
                "filename": filename,
                "thumb_filename": thumb_filename,
                "size": size,
                "original_size": original_size,
                "original_filename": orig_name,
                "width": width,
                "height": height,
                "processed": processed_flag,
            })),
            Some(addr.ip()),
        )
        .await;

        return Ok(Json(UploadResponse {
            url,
            thumb_url,
            filename,
            size,
            width,
            height,
        }));
    }

    Err(AppError::BadRequest(
        "missing 'file' field in multipart".into(),
    ))
}

async fn write_all(path: &PathBuf, data: &[u8]) -> AppResult<()> {
    let mut f = fs::File::create(path)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("create file: {e}")))?;
    f.write_all(data)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("write: {e}")))?;
    f.flush()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("flush: {e}")))?;
    Ok(())
}

#[derive(Debug)]
enum ProcessKind {
    ReencodeStatic,
    PassthroughGif,
    PassthroughAnimatedWebp,
}

enum Processed {
    Reencoded {
        primary: Vec<u8>,
        thumb_bytes: Vec<u8>,
        width: u32,
        height: u32,
    },
    Passthrough {
        bytes: Vec<u8>,
        ext: &'static str,
    },
}

impl Processed {
    fn primary_bytes(&self) -> &[u8] {
        match self {
            Processed::Reencoded { primary, .. } => primary,
            Processed::Passthrough { bytes, .. } => bytes,
        }
    }
}

#[derive(Debug)]
enum ProcessError {
    UnsupportedFormat,
    Corrupt(String),
    Encode(String),
}

fn map_process_error(e: ProcessError) -> AppError {
    match e {
        ProcessError::UnsupportedFormat => AppError::BadRequest(
            "unsupported image format: only JPEG, PNG, GIF, WebP are accepted".into(),
        ),
        ProcessError::Corrupt(msg) => AppError::BadRequest(format!("corrupt image data: {msg}")),
        ProcessError::Encode(msg) => AppError::Internal(anyhow::anyhow!("image encode: {msg}")),
    }
}

fn process(bytes: Bytes) -> Result<Processed, ProcessError> {
    let kind = sniff_kind(&bytes)?;
    match kind {
        ProcessKind::PassthroughGif => Ok(Processed::Passthrough {
            bytes: bytes.to_vec(),
            ext: "gif",
        }),
        ProcessKind::PassthroughAnimatedWebp => Ok(Processed::Passthrough {
            bytes: bytes.to_vec(),
            ext: "webp",
        }),
        ProcessKind::ReencodeStatic => reencode_static(&bytes),
    }
}

fn reencode_static(bytes: &[u8]) -> Result<Processed, ProcessError> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| ProcessError::Corrupt(e.to_string()))?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(DECODE_MAX_WH);
    limits.max_image_height = Some(DECODE_MAX_WH);
    limits.max_alloc = Some(DECODE_MAX_ALLOC);
    reader.limits(limits);

    let img = reader
        .decode()
        .map_err(|e| ProcessError::Corrupt(e.to_string()))?;

    let primary_img = clamp_to(&img, MAX_DIMENSION);
    let (pw, ph) = (primary_img.width(), primary_img.height());
    let primary = encode_webp(&primary_img)?;

    let thumb_img = clamp_to(&primary_img, THUMB_MAX_DIMENSION);
    let thumb_bytes = encode_webp(&thumb_img)?;

    Ok(Processed::Reencoded {
        primary,
        thumb_bytes,
        width: pw,
        height: ph,
    })
}

/// Returns the original image (cheap clone via `DynamicImage`) when it's
/// already within bounds, otherwise a Lanczos3-resized copy that fits
/// inside `max × max` preserving aspect ratio.
fn clamp_to(img: &DynamicImage, max: u32) -> DynamicImage {
    if img.width() <= max && img.height() <= max {
        img.clone()
    } else {
        img.resize(max, max, FilterType::Lanczos3)
    }
}

fn encode_webp(img: &DynamicImage) -> Result<Vec<u8>, ProcessError> {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let encoder = webp::Encoder::from_rgba(rgba.as_raw(), w, h);
    let mem = encoder.encode(WEBP_QUALITY);
    if mem.is_empty() {
        return Err(ProcessError::Encode("libwebp returned empty buffer".into()));
    }
    Ok(mem.to_vec())
}

fn sniff_kind(bytes: &[u8]) -> Result<ProcessKind, ProcessError> {
    let format = image::guess_format(bytes).map_err(|_| ProcessError::UnsupportedFormat)?;
    Ok(match format {
        ImageFormat::Jpeg | ImageFormat::Png => ProcessKind::ReencodeStatic,
        ImageFormat::Gif => ProcessKind::PassthroughGif,
        ImageFormat::WebP => {
            if is_animated_webp(bytes) {
                ProcessKind::PassthroughAnimatedWebp
            } else {
                ProcessKind::ReencodeStatic
            }
        }
        _ => return Err(ProcessError::UnsupportedFormat),
    })
}

/// Detects animated WebP by inspecting the RIFF container: a VP8X chunk
/// at offset 12 carries a flags byte at offset 20 whose bit 1 (`0x02`) is
/// the animation flag. Static WebPs use VP8 (lossy) or VP8L (lossless)
/// chunks instead and never set this bit.
fn is_animated_webp(b: &[u8]) -> bool {
    b.len() >= 21
        && &b[0..4] == b"RIFF"
        && &b[8..12] == b"WEBP"
        && &b[12..16] == b"VP8X"
        && (b[20] & 0x02) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jpeg_header() -> Vec<u8> {
        vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00]
    }

    fn png_header() -> Vec<u8> {
        vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
    }

    fn gif_header() -> Vec<u8> {
        b"GIF89a".to_vec()
    }

    /// Build a minimal RIFF/WEBP container header with the given first
    /// chunk FOURCC and (for VP8X) a flags byte.
    fn webp_header(chunk: &[u8; 4], vp8x_flags: u8) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"RIFF");
        v.extend_from_slice(&[0, 0, 0, 0]); // size placeholder
        v.extend_from_slice(b"WEBP");
        v.extend_from_slice(chunk);
        v.extend_from_slice(&[0, 0, 0, 0]); // chunk size
        v.push(vp8x_flags); // flags byte (only meaningful for VP8X)
        // pad to >= 21 bytes
        while v.len() < 32 {
            v.push(0);
        }
        v
    }

    #[test]
    fn animated_webp_detection_matches_vp8x_flag() {
        assert!(is_animated_webp(&webp_header(b"VP8X", 0x02)));
        assert!(is_animated_webp(&webp_header(b"VP8X", 0xFF))); // any byte with bit 1
        assert!(!is_animated_webp(&webp_header(b"VP8X", 0x00)));
        assert!(!is_animated_webp(&webp_header(b"VP8X", 0x01))); // alpha bit only
        assert!(!is_animated_webp(&webp_header(b"VP8 ", 0x02))); // simple lossy
        assert!(!is_animated_webp(&webp_header(b"VP8L", 0x02))); // simple lossless
    }

    #[test]
    fn animated_webp_rejects_short_and_non_riff() {
        assert!(!is_animated_webp(&[]));
        assert!(!is_animated_webp(&[0u8; 20]));
        assert!(!is_animated_webp(b"NOTRIFFnotwebp"));
    }

    #[test]
    fn sniff_jpeg_png_gif() {
        assert!(matches!(
            sniff_kind(&jpeg_header()).unwrap(),
            ProcessKind::ReencodeStatic
        ));
        assert!(matches!(
            sniff_kind(&png_header()).unwrap(),
            ProcessKind::ReencodeStatic
        ));
        assert!(matches!(
            sniff_kind(&gif_header()).unwrap(),
            ProcessKind::PassthroughGif
        ));
    }

    #[test]
    fn sniff_static_vs_animated_webp() {
        assert!(matches!(
            sniff_kind(&webp_header(b"VP8 ", 0)).unwrap(),
            ProcessKind::ReencodeStatic
        ));
        assert!(matches!(
            sniff_kind(&webp_header(b"VP8L", 0)).unwrap(),
            ProcessKind::ReencodeStatic
        ));
        assert!(matches!(
            sniff_kind(&webp_header(b"VP8X", 0x02)).unwrap(),
            ProcessKind::PassthroughAnimatedWebp
        ));
        // Static VP8X (no animation bit) — still re-encode-able.
        assert!(matches!(
            sniff_kind(&webp_header(b"VP8X", 0x00)).unwrap(),
            ProcessKind::ReencodeStatic
        ));
    }

    #[test]
    fn sniff_rejects_unknown_and_svg() {
        assert!(matches!(
            sniff_kind(b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"),
            Err(ProcessError::UnsupportedFormat)
        ));
        assert!(matches!(
            sniff_kind(b"not an image at all"),
            Err(ProcessError::UnsupportedFormat)
        ));
        assert!(matches!(
            sniff_kind(&[]),
            Err(ProcessError::UnsupportedFormat)
        ));
    }
}
