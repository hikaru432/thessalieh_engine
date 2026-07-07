use axum::{
    Extension, Json,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{PgPool, Row};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use super::shared::{
    E, SESSION_MAX_AGE, UserResponse, csrf_cookie, new_csrf_token, session_cookie,
};
use crate::api::verified::Verified;

#[derive(Deserialize)]
struct VerifyInput {
    email: String,
    code: String,
}

pub async fn verify(
    Extension(pool): Extension<PgPool>,
    Verified(msg, _): Verified,
) -> Result<(StatusCode, HeaderMap, Json<UserResponse>), E> {
    let p: VerifyInput =
        serde_json::from_slice(&msg).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid payload"))?;

    if p.code.len() != 6 || !p.code.chars().all(|c| c.is_ascii_digit()) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid code format"));
    }

    let now = Utc::now().timestamp();
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Verification failed")
    })?;

    sqlx::query("DELETE FROM public.verification_codes WHERE expires_at <= $1")
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Verification failed")
        })?;

    let vc = sqlx::query(
        "SELECT username, password_hash, code, expires_at, access_token
           FROM public.verification_codes
          WHERE email = $1
          FOR UPDATE",
    )
    .bind(&p.email)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Verification failed")
    })?
    .ok_or((
        StatusCode::UNPROCESSABLE_ENTITY,
        "No pending verification for this email",
    ))?;

    let username: String = vc.try_get("username").unwrap_or_default();
    let password_hash: String = vc.try_get("password_hash").unwrap_or_default();
    let code: String = vc.try_get("code").unwrap_or_default();
    let expires_at: i64 = vc.try_get("expires_at").unwrap_or(0);
    let access_token: Option<String> = vc.try_get("access_token").ok().flatten();

    if expires_at <= now {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Verification code has expired",
        ));
    }

    let attempts: i32 = sqlx::query_scalar::<_, i32>(
        "SELECT failed_attempts
           FROM public.verification_codes
          WHERE email = $1",
    )
    .bind(&p.email)
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(0);

    if attempts >= 5 {
        sqlx::query("DELETE FROM public.verification_codes WHERE email = $1")
            .bind(&p.email)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Verification failed")
            })?;
        tx.commit().await.ok();
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Too many incorrect attempts. Please request a new code.",
        ));
    }

    if code.as_bytes().ct_eq(p.code.as_bytes()).unwrap_u8() == 0 {
        let _ = sqlx::query(
            "UPDATE public.verification_codes
                SET failed_attempts = failed_attempts + 1
              WHERE email = $1",
        )
        .bind(&p.email)
        .execute(&mut *tx)
        .await;
        tx.commit().await.ok();
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Incorrect verification code",
        ));
    }

    let role = if access_token.as_deref().is_some_and(|t| !t.is_empty()) {
        "Admin"
    } else {
        "User"
    };

    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO public.users
           (id, username, email, password_hash, role, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $6)",
    )
    .bind(user_id)
    .bind(&username)
    .bind(&p.email)
    .bind(&password_hash)
    .bind(role)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Account creation failed")
    })?;

    if let Some(ref token) = access_token {
        sqlx::query(
            "UPDATE public.access_tokens
                SET redeemed_by = $1, redeemed_at = $2
              WHERE token = $3",
        )
        .bind(user_id)
        .bind(now)
        .bind(token)
        .execute(&mut *tx)
        .await
        .ok();
    }

    sqlx::query(
        "DELETE FROM public.verification_codes
          WHERE email = $1 OR expires_at <= $2",
    )
    .bind(&p.email)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Verification failed")
    })?;

    sqlx::query(
        "DELETE FROM public.sessions
          WHERE user_id = $1 OR expires_at <= $2",
    )
    .bind(user_id)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Session cleanup failed")
    })?;

    let session_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO public.sessions (id, user_id, created_at, expires_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(now)
    .bind(now + SESSION_MAX_AGE)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Session creation failed")
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Account creation failed")
    })?;

    tracing::info!(email = %p.email, %user_id, "user registered");

    let mut headers = HeaderMap::new();
    let csrf_token = new_csrf_token();
    headers.append(SET_COOKIE, session_cookie(session_id));
    headers.append(SET_COOKIE, csrf_cookie(&csrf_token));
    headers.insert(
        axum::http::header::HeaderName::from_static("x-csrf-token"),
        HeaderValue::from_str(&csrf_token).expect("valid csrf token"),
    );
    Ok((
        StatusCode::CREATED,
        headers,
        Json(UserResponse {
            id: user_id,
            username,
            email: p.email,
            phone: None,
            role: role.into(),
            expires_at: now + SESSION_MAX_AGE,
        }),
    ))
}
