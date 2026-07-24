use axum::{
    Extension, Json,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::insert::{InsertUserInput, insert_user};
use super::shared::{
    E, SESSION_MAX_AGE, UserResponse, csrf_cookie, new_csrf_token, session_cookie,
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

    let result = insert_user(
        &pool,
        InsertUserInput {
            username: p.username,
            password: p.password,
            access_token: p.access_token,
        },
    )
    .await?;

    let now = Utc::now().timestamp();
    let session_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO public.sessions (id, user_id, created_at, expires_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(session_id)
    .bind(result.user_id)
    .bind(now)
    .bind(now + SESSION_MAX_AGE)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Session creation failed")
    })?;

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
            id: result.user_id,
            username: result.username,
            email: String::new(),
            phone: None,
            role: result.role.into(),
            expires_at: now + SESSION_MAX_AGE,
        }),
    ))
}
