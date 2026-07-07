use axum::{
    Extension, Json,
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::shared::{E, extract_session_id};

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Serialize)]
pub struct UserHit {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

/// Lookup users by partial username OR email match for messaging compose.
/// Requires a session. Excludes the caller.
pub async fn search(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Query(p): Query<SearchQuery>,
) -> Result<Json<Vec<UserHit>>, E> {
    let now = Utc::now().timestamp();
    let sid =
        extract_session_id(&headers).ok_or((StatusCode::UNAUTHORIZED, "Not authenticated"))?;

    let me: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT u.id, u.role FROM public.sessions s
         JOIN public.users u ON u.id = s.user_id
         WHERE s.id = $1
           AND s.expires_at > $2",
    )
    .bind(sid)
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB session: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?;

    let (my_id, my_role) = me.ok_or((StatusCode::UNAUTHORIZED, "Session expired or not found"))?;
    let i_am_admin = my_role == "Admin";

    let q = p.q.trim();
    if q.chars().count() < 2 {
        return Ok(Json(Vec::new()));
    }
    if q.len() > 100 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Query too long"));
    }

    let pattern = format!("%{q}%");

    // Messaging is user<->admin only. Non-admins can only find admins;
    // admins can find anyone (excluding themselves).
    let sql = if i_am_admin {
        "SELECT id, username, email
           FROM public.users
          WHERE id <> $1
            AND (username ILIKE $2 OR email ILIKE $2)
       ORDER BY username ASC
          LIMIT 20"
    } else {
        "SELECT id, username, email
           FROM public.users
          WHERE id <> $1
            AND role = 'Admin'
            AND (username ILIKE $2 OR email ILIKE $2)
       ORDER BY username ASC
          LIMIT 20"
    };

    let rows = sqlx::query(sql)
        .bind(my_id)
        .bind(&pattern)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB user search: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
        })?;

    let hits: Vec<UserHit> = rows
        .into_iter()
        .map(|r| UserHit {
            id: r.try_get("id").unwrap_or_default(),
            username: r.try_get("username").unwrap_or_default(),
            email: r.try_get("email").unwrap_or_default(),
        })
        .collect();

    Ok(Json(hits))
}
