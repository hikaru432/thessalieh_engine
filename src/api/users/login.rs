use argon2::{Argon2, PasswordVerifier, password_hash::PasswordHash};
use axum::{
    Extension, Json,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{PgPool, Row};

use super::shared::{
    E, SESSION_MAX_AGE, UserResponse, csrf_cookie, new_csrf_token, session_cookie,
};
use crate::api::verified::Verified;

// Per-username lockout tuning: 5 bad attempts → 15-minute lockout.
const MAX_FAILED_ATTEMPTS: i32 = 5;
const LOCKOUT_SECONDS: i64 = 15 * 60;

#[derive(Deserialize)]
struct LoginInput {
    username: String,
    password: String,
}

pub async fn login(
    Extension(pool): Extension<PgPool>,
    Verified(msg, _): Verified,
) -> Result<(StatusCode, HeaderMap, Json<UserResponse>), E> {
    let p: LoginInput =
        serde_json::from_slice(&msg).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid payload"))?;

    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "SELECT id, username, email, phone, password_hash, role, failed_login_attempts, lockout_until
         FROM public.users WHERE username = $1",
    )
    .bind(&p.username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Login failed")
    })?
    .ok_or((StatusCode::UNAUTHORIZED, "Invalid username or password"))?;

    let user_id: uuid::Uuid = row.try_get("id").map_err(|e| {
        tracing::error!("DB login row id: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Login failed")
    })?;
    let username: String = row.try_get("username").unwrap_or_default();
    let email: String = row.try_get("email").unwrap_or_default();
    let phone: Option<String> = row.try_get("phone").ok().flatten();
    let password_hash: String = row.try_get("password_hash").unwrap_or_default();
    let role: String = row.try_get("role").unwrap_or_default();
    let failed_login_attempts: i32 = row.try_get("failed_login_attempts").unwrap_or(0);
    let lockout_until: i64 = row.try_get("lockout_until").unwrap_or(0);

    if lockout_until > now {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Too many failed attempts. Please try again later.",
        ));
    }

    let parsed = PasswordHash::new(&password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Login failed"))?;

    if Argon2::default()
        .verify_password(p.password.as_bytes(), &parsed)
        .is_err()
    {
        let new_count = failed_login_attempts + 1;
        if new_count >= MAX_FAILED_ATTEMPTS {
            let _ = sqlx::query(
                "UPDATE public.users
                   SET failed_login_attempts = 0, lockout_until = $2
                 WHERE id = $1",
            )
            .bind(user_id)
            .bind(now + LOCKOUT_SECONDS)
            .execute(&pool)
            .await;
            tracing::warn!(username = %p.username, "account locked after repeated failed logins");
        } else {
            let _ = sqlx::query(
                "UPDATE public.users SET failed_login_attempts = $2 WHERE id = $1",
            )
            .bind(user_id)
            .bind(new_count)
            .execute(&pool)
            .await;
        }
        return Err((StatusCode::UNAUTHORIZED, "Invalid username or password"));
    }

    if failed_login_attempts != 0 || lockout_until != 0 {
        let _ = sqlx::query(
            "UPDATE public.users
                SET failed_login_attempts = 0, lockout_until = 0
              WHERE id = $1",
        )
        .bind(user_id)
        .execute(&pool)
        .await;
    }

    let _ = sqlx::query("DELETE FROM public.sessions WHERE user_id = $1 OR expires_at <= $2")
        .bind(user_id)
        .bind(now)
        .execute(&pool)
        .await;

    let session_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO public.sessions (id, user_id, created_at, expires_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(now)
    .bind(now + SESSION_MAX_AGE)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Session creation failed")
    })?;

    tracing::info!(username = %p.username, user_id = %user_id, "user logged in");

    let mut headers = HeaderMap::new();
    let csrf_token = new_csrf_token();
    headers.append(SET_COOKIE, session_cookie(session_id));
    headers.append(SET_COOKIE, csrf_cookie(&csrf_token));
    headers.insert(
        axum::http::header::HeaderName::from_static("x-csrf-token"),
        HeaderValue::from_str(&csrf_token).expect("valid csrf token"),
    );
    Ok((
        StatusCode::OK,
        headers,
        Json(UserResponse {
            id: user_id,
            username,
            email,
            phone,
            role,
            expires_at: now + SESSION_MAX_AGE,
        }),
    ))
}
