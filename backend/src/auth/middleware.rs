use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};

use crate::{auth::jwt, error::AppError, state::AppState};

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: i64,
    pub username: String,
    pub role: String,
}

pub async fn require_admin(
    State(state): State<AppState>,
    mut req: axum::extract::Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims = jwt::verify(&state.jwt_secret, token).map_err(|_| AppError::Unauthorized)?;
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    req.extensions_mut().insert(AuthUser {
        user_id: claims.sub,
        username: claims.username,
        role: claims.role,
    });

    Ok(next.run(req).await)
}

// Allow `AuthUser` to be extracted from request extensions inside admin handlers.
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(AppError::Unauthorized)
    }
}
