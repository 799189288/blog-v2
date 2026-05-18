use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

struct Bucket {
    count: u32,
    window_start: Instant,
}

#[derive(Clone)]
pub struct IpRateLimit {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
    max_requests: u32,
    window: Duration,
}

impl IpRateLimit {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    /// Returns `true` if the request is allowed, `false` if rate-limited.
    pub fn check(&self, ip: &str) -> bool {
        let now = Instant::now();
        let mut map = self.buckets.lock().expect("rate_limit poisoned");
        // Periodic cleanup to prevent unbounded growth.
        if map.len() > 10_000 {
            let w = self.window;
            map.retain(|_, b| now.duration_since(b.window_start) < w);
        }
        let bucket = map.entry(ip.to_string()).or_insert(Bucket {
            count: 0,
            window_start: now,
        });
        if now.duration_since(bucket.window_start) >= self.window {
            bucket.count = 1;
            bucket.window_start = now;
            true
        } else {
            bucket.count += 1;
            bucket.count <= self.max_requests
        }
    }
}

fn extract_ip(req: &Request) -> String {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            req.extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ci| ci.0.ip().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        })
}

/// Tower middleware: 120 requests per minute per IP on all API routes.
pub async fn api_rate_limit(
    req: Request,
    next: Next,
    rl: IpRateLimit,
) -> Response {
    let ip = extract_ip(&req);
    if !rl.check(&ip) {
        tracing::debug!(ip = %ip, "rate limit exceeded");
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    next.run(req).await
}
