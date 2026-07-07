use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use sqlx::PgPool;

use super::users::shared::{E, extract_session_id};

/// Requires an active session belonging to an Admin.
pub async fn require_admin(pool: &PgPool, headers: &HeaderMap) -> Result<(), E> {
    let now = Utc::now().timestamp();
    let sid = extract_session_id(headers).ok_or((StatusCode::UNAUTHORIZED, "Not authenticated"))?;

    let role: Option<String> = sqlx::query_scalar(
        "SELECT u.role FROM public.sessions s
         JOIN public.users u ON u.id = s.user_id
         WHERE s.id = $1 AND s.expires_at > $2",
    )
    .bind(sid)
    .bind(now)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB session: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?;

    match role.as_deref() {
        Some("Admin") => Ok(()),
        Some(_) => Err((StatusCode::FORBIDDEN, "Admin access required")),
        None => Err((StatusCode::UNAUTHORIZED, "Session expired or not found")),
    }
}
