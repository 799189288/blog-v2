use std::net::IpAddr;

use serde_json::Value;
use sqlx::PgPool;

use crate::auth::AuthUser;

/// Record an audit event. Failures are logged but never propagated — auditing
/// should not be able to fail the underlying request.
pub async fn record(
    pool: &PgPool,
    user: &AuthUser,
    action: &str,
    target_type: Option<&str>,
    target_id: Option<i64>,
    detail: Option<Value>,
    ip: Option<IpAddr>,
) {
    let ip_str = ip.map(|a| a.to_string());
    let result = sqlx::query(
        r#"
        INSERT INTO audit_logs
            (user_id, username, action, target_type, target_id, detail, ip)
        VALUES ($1, $2, $3, $4, $5, $6, $7::inet)
        "#,
    )
    .bind(user.user_id)
    .bind(&user.username)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(detail)
    .bind(ip_str)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, action, "failed to record audit event");
    }
}
