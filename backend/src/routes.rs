use std::path::PathBuf;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method, header},
    middleware,
    routing::{delete, get, patch, post, put},
};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{auth::require_admin, config::Config, handlers, state::AppState};

pub fn build(state: AppState, config: &Config) -> Router {
    let public = Router::new()
        .route("/posts", get(handlers::posts::list_published))
        .route("/posts/archive", get(handlers::posts::archive))
        .route("/posts/:slug", get(handlers::posts::get_by_slug))
        .route("/posts/:slug/related", get(handlers::posts::related))
        .route("/posts/:slug/comments", get(handlers::comments::list_approved))
        .route("/posts/:slug/comments", post(handlers::comments::submit))
        .route("/tags", get(handlers::tags::list_with_counts))
        .route("/tags/:slug", get(handlers::tags::get_by_slug))
        .route("/dict/:type_code", get(handlers::dict::get_by_type))
        .route("/search", get(handlers::search::search))
        .route("/auth/login", post(handlers::auth::login))
        .route("/health", get(health));

    let admin = Router::new()
        .route("/posts", get(handlers::admin_posts::list_all))
        .route("/posts", post(handlers::admin_posts::create))
        .route("/posts/:id", get(handlers::admin_posts::get_by_id))
        .route("/posts/:id", put(handlers::admin_posts::update))
        .route("/posts/:id", delete(handlers::admin_posts::delete))
        .route("/comments", get(handlers::admin_comments::list))
        .route("/comments/:id", patch(handlers::admin_comments::set_status))
        .route("/comments/:id", delete(handlers::admin_comments::delete))
        // tag management
        .route("/tags", get(handlers::admin_tags::list))
        .route("/tags", post(handlers::admin_tags::create))
        .route("/tags/:id", put(handlers::admin_tags::update))
        .route("/tags/:id", delete(handlers::admin_tags::delete))
        // dictionary management
        .route("/dict/types", get(handlers::admin_dict::list_types))
        .route("/dict/types", post(handlers::admin_dict::create_type))
        .route("/dict/types/:id", put(handlers::admin_dict::update_type))
        .route("/dict/types/:id", delete(handlers::admin_dict::delete_type))
        .route("/dict/types/:id/items", get(handlers::admin_dict::list_items))
        .route("/dict/types/:id/items", post(handlers::admin_dict::create_item))
        .route("/dict/items/:id", put(handlers::admin_dict::update_item))
        .route("/dict/items/:id", delete(handlers::admin_dict::delete_item))
        // stats
        .route("/stats/overview", get(handlers::admin_stats::overview))
        .route("/stats/trend", get(handlers::admin_stats::trend))
        .route("/stats/dashboard", get(handlers::admin_stats::dashboard))
        // user management
        .route("/users", get(handlers::admin_users::list))
        .route("/users", post(handlers::admin_users::create))
        .route("/users/:id/password", patch(handlers::admin_users::reset_password))
        .route("/users/:id", delete(handlers::admin_users::delete))
        // raw data tables
        .route("/data/posts", get(handlers::admin_data::posts))
        .route("/data/comments", get(handlers::admin_data::comments))
        .route("/data/tags", get(handlers::admin_data::tags))
        .route("/data/users", get(handlers::admin_data::users))
        // audit log
        .route("/audit", get(handlers::admin_audit::list))
        // image uploads — multipart, capped by AppState.uploads.max_bytes
        // at the handler level; axum's default 2 MB body limit is raised
        // here so the multipart parser can see the whole body first.
        .route(
            "/uploads",
            post(handlers::uploads::upload).layer(DefaultBodyLimit::max(
                state.uploads.max_bytes.saturating_add(64 * 1024),
            )),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_admin,
        ));

    let api = Router::new()
        .merge(public)
        .nest("/admin", admin)
        .with_state(state.clone());

    // Feeds live at the site root (RSS readers expect /rss.xml).
    let feeds = Router::new()
        .route("/rss.xml", get(handlers::feed::rss))
        .route("/sitemap.xml", get(handlers::feed::sitemap))
        .with_state(state);

    // Serve uploaded images straight from disk. ServeDir handles range
    // requests, ETags, content-type sniffing — no need for a handler.
    let uploads_dir = PathBuf::from(&config.upload_dir);
    let upload_files = Router::<()>::new().nest_service(
        "/uploads",
        ServeDir::new(&uploads_dir).precompressed_gzip(),
    );

    let cors = build_cors(config);

    let mut router = Router::new()
        .nest("/api", api)
        .merge(feeds)
        .merge(upload_files)
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http());

    if let Some(dir) = &config.static_dir {
        let dir = PathBuf::from(dir);
        let index = dir.join("index.html");
        let service = ServeDir::new(&dir).fallback(ServeFile::new(&index));
        router = router.fallback_service(service);
    }

    router
}

async fn health() -> &'static str {
    "ok"
}

fn build_cors(config: &Config) -> CorsLayer {
    let origins: Vec<HeaderValue> = config
        .cors_allowed_origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}
