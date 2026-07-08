use argon2::{
    Algorithm, Argon2, Params, PasswordHasher, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    Extension, Json,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::shared::{
    E, SESSION_MAX_AGE, UserResponse, csrf_cookie, is_strong_password, is_valid_username,
    new_csrf_token, session_cookie,
};
use crate::api::verified::Verified;

#[derive(Deserialize)]
struct RegisterInput {
    username: String,
    password: String,
    access_token: String,
}

pub async fn register(
    Extension(pool): Extension<PgPool>,
    Verified(msg, _): Verified,
) -> Result<(StatusCode, HeaderMap, Json<UserResponse>), E> {
    let p: RegisterInput = serde_json::from_slice(&msg)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid payload JSON"))?;

    let username = p.username.trim().to_string();
    if !is_valid_username(&username) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid username"));
    }
    if !is_strong_password(&p.password) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Password too weak"));
    }
    let access_token = p.access_token.trim().to_string();
    if access_token.len() > 100 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid access token"));
    }

    let now = Utc::now().timestamp();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(65536, 3, 4, None).expect("valid Argon2 params"),
    );
    let hash = argon2
        .hash_password(p.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Registration failed")
    })?;

    if sqlx::query("SELECT id FROM public.users WHERE username = $1")
        .bind(&username)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Registration failed")
        })?
        .is_some()
    {
        tracing::warn!(%username, "register failed: username already exists");
        return Err((StatusCode::CONFLICT, "Registration failed"));
    }

    let role = if access_token.is_empty() {
        "User"
    } else {
        let token_row = sqlx::query(
            "SELECT redeemed_at, revoked_at, expires_at
               FROM public.access_tokens
              WHERE token = $1
              FOR UPDATE",
        )
        .bind(&access_token)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Registration failed")
        })?
        .ok_or((StatusCode::UNPROCESSABLE_ENTITY, "Invalid access token"))?;

        use sqlx::Row;
        let revoked_at: Option<i64> = token_row.try_get("revoked_at").ok().flatten();
        let redeemed_at: Option<i64> = token_row.try_get("redeemed_at").ok().flatten();
        let expires_at: Option<i64> = token_row.try_get("expires_at").ok().flatten();

        if revoked_at.is_some() {
            return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid access token"));
        }
        if redeemed_at.is_some() {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Access token already used",
            ));
        }
        if expires_at.is_some_and(|exp| exp <= now) {
            return Err((StatusCode::UNPROCESSABLE_ENTITY, "Access token has expired"));
        }

        "Admin"
    };

    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO public.users (id, username, email, password_hash, role, created_at, updated_at)
         VALUES ($1, $2, '', $3, $4, $5, $5)",
    )
    .bind(user_id)
    .bind(&username)
    .bind(&hash)
    .bind(role)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Registration failed")
    })?;

    if !access_token.is_empty() {
        sqlx::query(
            "UPDATE public.access_tokens
                SET redeemed_by = $1, redeemed_at = $2
              WHERE token = $3",
        )
        .bind(user_id)
        .bind(now)
        .bind(&access_token)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Registration failed")
        })?;
    }

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
        (StatusCode::INTERNAL_SERVER_ERROR, "Registration failed")
    })?;

    tracing::info!(%username, %user_id, "user registered");

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
            email: String::new(),
            phone: None,
            role: role.into(),
            expires_at: now + SESSION_MAX_AGE,
        }),
    ))
}
