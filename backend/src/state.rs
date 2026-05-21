use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::models::dict::DictItemPublic;
use crate::handlers::posts::ArchiveGroup;
use crate::notify::Notifier;

const VIEW_DEDUPE_TTL: Duration = Duration::from_secs(30 * 60);
const VIEW_DEDUPE_CLEANUP_THRESHOLD: usize = 10_000;

const COMMENT_RATE_WINDOW: Duration = Duration::from_secs(30);
const COMMENT_DEDUPE_CLEANUP_THRESHOLD: usize = 10_000;

const LOGIN_FAIL_WINDOW: Duration = Duration::from_secs(60);
const LOGIN_MAX_FAILURES: u32 = 5;
const LOGIN_LOCKOUT_DURATION: Duration = Duration::from_secs(15 * 60);

const ARCHIVE_CACHE_TTL: Duration = Duration::from_secs(5 * 60);

#[derive(Debug)]
struct LoginRecord {
    failures: u32,
    window_start: Instant,
    locked_until: Option<Instant>,
}

#[derive(Clone, Debug)]
pub struct SiteInfo {
    pub url: String,
    pub title: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct UploadCfg {
    pub dir: String,
    pub max_bytes: usize,
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: Arc<String>,
    pub site: Arc<SiteInfo>,
    pub uploads: Arc<UploadCfg>,
    /// Lowercased substring tokens that flag a comment as spam on submit.
    pub comment_blocklist: Arc<Vec<String>>,
    /// `None` when SMTP isn't configured — submit handler treats sending
    /// as a no-op rather than special-casing.
    pub notifier: Option<Notifier>,
    /// Per-type cache of enabled dict items. Cleared on any admin write.
    pub dict_cache: Arc<RwLock<HashMap<String, Arc<Vec<DictItemPublic>>>>>,
    /// Cached archive response. Invalidated when a post is published or updated.
    pub archive_cache: Arc<RwLock<Option<(Instant, Arc<Vec<ArchiveGroup>>)>>>,
    /// In-memory dedupe of recent post views, keyed by "slug|ip".
    /// Prevents the views counter from inflating on refresh.
    view_dedupe: Arc<Mutex<HashMap<String, Instant>>>,
    /// In-memory per-IP last-comment timestamp for rate limiting.
    comment_dedupe: Arc<Mutex<HashMap<String, Instant>>>,
    /// Per-IP login failure tracking for brute-force lockout.
    login_failures: Arc<Mutex<HashMap<String, LoginRecord>>>,
}

impl AppState {
    pub fn new(
        db: PgPool,
        jwt_secret: String,
        site: SiteInfo,
        uploads: UploadCfg,
        comment_blocklist: Vec<String>,
        notifier: Option<Notifier>,
    ) -> Self {
        Self {
            db,
            jwt_secret: Arc::new(jwt_secret),
            site: Arc::new(site),
            uploads: Arc::new(uploads),
            comment_blocklist: Arc::new(comment_blocklist),
            notifier,
            dict_cache: Arc::new(RwLock::new(HashMap::new())),
            archive_cache: Arc::new(RwLock::new(None)),
            view_dedupe: Arc::new(Mutex::new(HashMap::new())),
            comment_dedupe: Arc::new(Mutex::new(HashMap::new())),
            login_failures: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn clear_dict_cache(&self) {
        self.dict_cache.write().await.clear();
    }

    pub async fn invalidate_dict_type(&self, type_code: &str) {
        self.dict_cache.write().await.remove(type_code);
    }

    pub async fn invalidate_archive_cache(&self) {
        *self.archive_cache.write().await = None;
    }

    pub async fn get_archive_cache(&self) -> Option<Arc<Vec<ArchiveGroup>>> {
        let guard = self.archive_cache.read().await;
        if let Some((cached_at, data)) = guard.as_ref() {
            if cached_at.elapsed() < ARCHIVE_CACHE_TTL {
                return Some(Arc::clone(data));
            }
        }
        None
    }

    pub async fn set_archive_cache(&self, data: Vec<ArchiveGroup>) {
        *self.archive_cache.write().await = Some((Instant::now(), Arc::new(data)));
    }

    /// Returns true if this view should be counted (not seen recently from same IP).
    /// Updates the dedupe cache as a side effect.
    pub fn should_count_view(&self, slug: &str, ip: &str) -> bool {
        let key = format!("{slug}|{ip}");
        let now = Instant::now();
        let mut map = self.view_dedupe.lock().expect("view_dedupe poisoned");
        if map.len() > VIEW_DEDUPE_CLEANUP_THRESHOLD {
            map.retain(|_, t| now.duration_since(*t) < VIEW_DEDUPE_TTL);
        }
        match map.get(&key) {
            Some(t) if now.duration_since(*t) < VIEW_DEDUPE_TTL => false,
            _ => {
                map.insert(key, now);
                true
            }
        }
    }

    /// Returns true if a comment from this IP is allowed right now. The
    /// side effect — recording 'now' as the last attempt — only fires
    /// when we *accept* the comment, so a spammer pinging us in a tight
    /// loop doesn't keep extending their own cooldown.
    pub fn comment_allowed(&self, ip: &str) -> bool {
        let now = Instant::now();
        let mut map = self.comment_dedupe.lock().expect("comment_dedupe poisoned");
        if map.len() > COMMENT_DEDUPE_CLEANUP_THRESHOLD {
            map.retain(|_, t| now.duration_since(*t) < COMMENT_RATE_WINDOW);
        }
        match map.get(ip) {
            Some(t) if now.duration_since(*t) < COMMENT_RATE_WINDOW => false,
            _ => true,
        }
    }

    pub fn record_comment(&self, ip: &str) {
        let mut map = self.comment_dedupe.lock().expect("comment_dedupe poisoned");
        map.insert(ip.to_string(), Instant::now());
    }

    /// Returns `true` if the IP is currently locked out from login attempts.
    pub fn login_locked(&self, ip: &str) -> bool {
        let now = Instant::now();
        let map = self.login_failures.lock().expect("login_failures poisoned");
        if let Some(rec) = map.get(ip) {
            if let Some(until) = rec.locked_until {
                return now < until;
            }
        }
        false
    }

    /// Record a failed login attempt. Returns `true` if this attempt
    /// triggered a new lockout (so the caller can log it).
    pub fn record_login_failure(&self, ip: &str) -> bool {
        let now = Instant::now();
        let mut map = self.login_failures.lock().expect("login_failures poisoned");
        let rec = map.entry(ip.to_string()).or_insert(LoginRecord {
            failures: 0,
            window_start: now,
            locked_until: None,
        });
        // Reset counter if the failure window has expired.
        if now.duration_since(rec.window_start) >= LOGIN_FAIL_WINDOW {
            rec.failures = 0;
            rec.window_start = now;
            rec.locked_until = None;
        }
        rec.failures += 1;
        if rec.failures >= LOGIN_MAX_FAILURES {
            rec.locked_until = Some(now + LOGIN_LOCKOUT_DURATION);
            return true;
        }
        false
    }

    /// Clear the failure record for an IP on successful login.
    pub fn clear_login_failures(&self, ip: &str) {
        let mut map = self.login_failures.lock().expect("login_failures poisoned");
        map.remove(ip);
    }
}
