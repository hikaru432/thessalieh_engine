use axum::{
    Extension, Json,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::shared::{E, UserResponse, extract_session_id};

#[derive(Deserialize)]
pub struct ProfileInput {
    username: String,
    phone: Option<String>,
}

/// PATCH /auth/profile — lets the signed-in user update their own display
/// name and phone number. Email is intentionally excluded since changing it
/// would require re-verification.
pub async fn update_profile(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(p): Json<ProfileInput>,
) -> Result<Json<UserResponse>, E> {
    let now = Utc::now().timestamp();
    let sid = extract_session_id(&headers).ok_or((StatusCode::UNAUTHORIZED, "Not authenticated"))?;

    let username = p.username.trim().to_string();
    if username.is_empty() || username.len() > 255 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid name"));
    }
    let phone = p
        .phone
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    if phone.as_ref().is_some_and(|v| v.len() > 32) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid phone number"));
    }

    let session = sqlx::query(
        "SELECT s.user_id, s.expires_at
           FROM public.sessions s
          WHERE s.id = $1 AND s.expires_at > $2",
    )
    .bind(sid)
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB session: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?
    .ok_or((StatusCode::UNAUTHORIZED, "Session expired or not found"))?;

    let user_id: Uuid = session.try_get("user_id").map_err(|e| {
        tracing::error!("DB session row user_id: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?;
    let expires_at: i64 = session.try_get("expires_at").unwrap_or(0);

    let row = sqlx::query(
        "UPDATE public.users
            SET username = $1, phone = $2, updated_at = $3
          WHERE id = $4
      RETURNING id, username, email, phone, role",
    )
    .bind(&username)
    .bind(&phone)
    .bind(now)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update profile")
    })?;

    Ok(Json(UserResponse {
        id: row.try_get("id").unwrap_or_default(),
        username: row.try_get("username").unwrap_or_default(),
        email: row.try_get("email").unwrap_or_default(),
        phone: row.try_get("phone").ok().flatten(),
        role: row.try_get("role").unwrap_or_default(),
        expires_at,
    }))
}
