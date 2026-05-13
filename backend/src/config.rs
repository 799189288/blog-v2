use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub cors_allowed_origins: Vec<String>,
    pub static_dir: Option<String>,
    pub site_url: String,
    pub site_title: String,
    pub site_description: String,
    pub upload_dir: String,
    pub max_upload_bytes: usize,
    pub comment_blocklist: Vec<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| anyhow::anyhow!("JWT_SECRET must be set"))?;
        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 bytes");
        }
        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:5173".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let static_dir = env::var("STATIC_DIR").ok().filter(|s| !s.is_empty());
        let site_url = env::var("SITE_URL")
            .unwrap_or_else(|_| "http://localhost:5173".into())
            .trim_end_matches('/')
            .to_string();
        let site_title = env::var("SITE_TITLE").unwrap_or_else(|_| "Blog".into());
        let site_description =
            env::var("SITE_DESCRIPTION").unwrap_or_else(|_| "Personal blog".into());
        let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".into());
        let max_upload_bytes = env::var("MAX_UPLOAD_BYTES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5 * 1024 * 1024);
        let comment_blocklist = env::var("COMMENT_BLOCKLIST")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            bind_addr,
            database_url,
            jwt_secret,
            cors_allowed_origins,
            static_dir,
            site_url,
            site_title,
            site_description,
            upload_dir,
            max_upload_bytes,
            comment_blocklist,
        })
    }
}
