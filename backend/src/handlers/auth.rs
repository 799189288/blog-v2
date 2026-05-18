use axum::{Json, extract::{ConnectInfo, State}, http::HeaderMap};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::{
    audit,
    auth::{AuthUser, jwt, password},
    error::{AppError, AppResult},
    models::user::User,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserOut,
}

#[derive(Debug, Serialize)]
pub struct UserOut {
    pub id: i64,
    pub username: String,
    pub role: String,
}

pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(input): Json<LoginInput>,
) -> AppResult<Json<LoginResponse>> {
    let ip = client_ip(&headers, addr);

    if state.login_locked(&ip) {
        return Err(AppError::BadRequest(
            "too many failed login attempts, please try again later".into(),
        ));
    }

    if input.username.is_empty() || input.password.is_empty() {
        return Err(AppError::BadRequest(
            "username and password are required".into(),
        ));
    }

    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, username, password_hash, role, created_at FROM users WHERE username = $1"#,
    )
    .bind(&input.username)
    .fetch_optional(&state.db)
    .await?;

    // Verify password. On any failure (user not found or wrong password)
    // record the attempt and return a generic Unauthorized — never reveal
    // which of the two failed.
    let user = match user {
        Some(u) if password::verify(&input.password, &u.password_hash) => u,
        _ => {
            let locked = state.record_login_failure(&ip);
            if locked {
                tracing::warn!(ip = %ip, "login lockout triggered");
            }
            return Err(AppError::Unauthorized);
        }
    };

    state.clear_login_failures(&ip);

    let token = jwt::issue(&state.jwt_secret, user.id, &user.username, &user.role)
        .map_err(AppError::Internal)?;

    let auth_user = AuthUser {
        user_id: user.id,
        username: user.username.clone(),
        role: user.role.clone(),
    };
    audit::record(
        &state.db,
        &auth_user,
        "login",
        None,
        None,
        None,
        Some(addr.ip()),
    )
    .await;

    Ok(Json(LoginResponse {
        token,
        user: UserOut {
            id: user.id,
            username: user.username,
            role: user.role,
        },
    }))
}

fn client_ip(headers: &HeaderMap, addr: SocketAddr) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| addr.ip().to_string())
}
