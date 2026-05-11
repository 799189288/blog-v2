use std::path::PathBuf;

use axum::{
    Router,
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
        .route("/posts/:slug", get(handlers::posts::get_by_slug))
        .route("/posts/:slug/comments", get(handlers::comments::list_approved))
        .route("/posts/:slug/comments", post(handlers::comments::submit))
        .route("/tags", get(handlers::tags::list_with_counts))
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
        // stats
        .route("/stats/overview", get(handlers::admin_stats::overview))
        .route("/stats/trend", get(handlers::admin_stats::trend))
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
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_admin,
        ));

    let api = Router::new()
        .merge(public)
        .nest("/admin", admin)
        .with_state(state);

    let cors = build_cors(config);

    let mut router = Router::new()
        .nest("/api", api)
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
