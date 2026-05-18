mod audit;
mod auth;
mod config;
mod error;
mod handlers;
mod markdown;
mod models;
mod notify;
mod rate_limit;
mod routes;
mod state;

use std::net::SocketAddr;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::{config::Config, state::{AppState, SiteInfo, UploadCfg}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer())
        .init();

    let config = Config::from_env()?;
    tracing::info!("starting blog backend on {}", config.bind_addr);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .context("connecting to database")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("running migrations")?;

    // Make sure the upload directory exists so the first POST doesn't 500.
    std::fs::create_dir_all(&config.upload_dir)
        .with_context(|| format!("creating upload dir {}", config.upload_dir))?;

    // Build the optional comment-notification mailer. We refuse to start
    // if SMTP is configured but invalid — better than discovering it on
    // the first comment.
    let notifier = match &config.smtp {
        Some(smtp_cfg) => {
            let n = notify::Notifier::from_cfg(smtp_cfg, &config.site_url, &config.site_title)
                .context("initializing SMTP notifier")?;
            tracing::info!(host = %smtp_cfg.host, to = %smtp_cfg.to, "comment notifications enabled");
            Some(n)
        }
        None => {
            tracing::info!("SMTP not configured — new-comment notifications disabled");
            None
        }
    };

    let state = AppState::new(
        pool,
        config.jwt_secret.clone(),
        SiteInfo {
            url: config.site_url.clone(),
            title: config.site_title.clone(),
            description: config.site_description.clone(),
        },
        UploadCfg {
            dir: config.upload_dir.clone(),
            max_bytes: config.max_upload_bytes,
        },
        config.comment_blocklist.clone(),
        notifier,
    );

    let app = routes::build(state, &config);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .with_context(|| format!("binding {}", config.bind_addr))?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
