use axum::{Json, extract::{ConnectInfo, State}};
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
    Json(input): Json<LoginInput>,
) -> AppResult<Json<LoginResponse>> {
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
    .await?
    .ok_or(AppError::Unauthorized)?;

    if !password::verify(&input.password, &user.password_hash) {
        return Err(AppError::Unauthorized);
    }

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
