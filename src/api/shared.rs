use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::infra::session_cache::SessionCache;
use super::users::shared::{E, extract_session_id};

/// Requires an active session; returns `(user_id, role)`. Backed by an
/// in-memory cache (see [`SessionCache`]) since this runs on nearly every
/// authenticated request in the app.
pub async fn require_session_user(pool: &PgPool, headers: &HeaderMap) -> Result<(Uuid, String), E> {
    let now = Utc::now().timestamp();
    let sid = extract_session_id(headers).ok_or((StatusCode::UNAUTHORIZED, "Not authenticated"))?;

    let cache = SessionCache::global();
    if let Some(cached) = cache.get(sid, now) {
        return Ok(cached);
    }

    let row = sqlx::query(
        "SELECT u.id, u.role, s.expires_at FROM public.sessions s
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
    })?
    .ok_or((StatusCode::UNAUTHORIZED, "Session expired or not found"))?;

    let user_id: Uuid = row.try_get("id").map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?;
    let role: String = row.try_get("role").unwrap_or_default();
    let expires_at: i64 = row.try_get("expires_at").unwrap_or(0);

    cache.insert(sid, user_id, role.clone(), expires_at);
    Ok((user_id, role))
}

/// Requires an active session belonging to an Admin.
pub async fn require_admin(pool: &PgPool, headers: &HeaderMap) -> Result<(), E> {
    let (_id, role) = require_session_user(pool, headers).await?;
    if role == "Admin" {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Admin access required"))
    }
}
