use argon2::{
    Algorithm, Argon2, Params, PasswordHasher, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{Extension, Json, http::StatusCode};
use chrono::Utc;
use rand::RngExt;
use serde::Deserialize;
use sqlx::{PgPool, Row};
use subtle::ConstantTimeEq;

use super::shared::{E, MessageResponse, is_strong_password, is_valid_email};
use crate::api::mailer;
use crate::api::verified::Verified;

#[derive(Deserialize)]
struct RequestInput {
    email: String,
}

#[derive(Deserialize)]
struct ConfirmInput {
    email: String,
    code: String,
    password: String,
}

/// POST /auth/password-reset/request
///
/// Always returns 200 with the same message regardless of whether the email
/// is registered, to avoid leaking which addresses have accounts. The OTP is
/// only generated and emailed when the account actually exists.
pub async fn request(
    Extension(pool): Extension<PgPool>,
    Verified(msg, _): Verified,
) -> Result<(StatusCode, Json<MessageResponse>), E> {
    let p: RequestInput = serde_json::from_slice(&msg)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid payload JSON"))?;

    if !is_valid_email(&p.email) || p.email.len() > 255 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid email"));
    }

    let now = Utc::now().timestamp();

    let user = sqlx::query("SELECT id FROM public.users WHERE email = $1")
        .bind(&p.email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
        })?;

    if user.is_none() {
        return Ok((
            StatusCode::OK,
            Json(MessageResponse {
                message: "If that email is registered, a code has been sent",
            }),
        ));
    }

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
    })?;

    if let Some(existing) = sqlx::query(
        "SELECT created_at
           FROM public.password_reset_codes
          WHERE email = $1
          FOR UPDATE",
    )
    .bind(&p.email)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
    })? {
        let existing_created_at: i64 = existing.try_get("created_at").unwrap_or(0);
        if existing_created_at > now - 60 {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                "Please wait 1 minute before requesting another code",
            ));
        }
        sqlx::query("DELETE FROM public.password_reset_codes WHERE email = $1")
            .bind(&p.email)
            .execute(&mut *tx)
            .await
            .ok();
    }

    let code = format!("{:06}", rand::rng().random_range(0..1_000_000u32));
    let expires_at = now + 600;

    sqlx::query(
        "INSERT INTO public.password_reset_codes (email, code, expires_at, created_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(&p.email)
    .bind(&code)
    .bind(expires_at)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
    })?;

    mailer::send_code(&p.email, &code).await.map_err(|e| {
        tracing::error!("mailer: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to send email")
    })?;

    tracing::info!(email = %p.email, "password reset code sent");
    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "If that email is registered, a code has been sent",
        }),
    ))
}

/// POST /auth/password-reset/confirm
pub async fn confirm(
    Extension(pool): Extension<PgPool>,
    Verified(msg, _): Verified,
) -> Result<(StatusCode, Json<MessageResponse>), E> {
    let p: ConfirmInput = serde_json::from_slice(&msg)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid payload JSON"))?;

    if p.code.len() != 6 || !p.code.chars().all(|c| c.is_ascii_digit()) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid code format"));
    }
    if !is_strong_password(&p.password) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Password too weak"));
    }

    let now = Utc::now().timestamp();
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
    })?;

    sqlx::query("DELETE FROM public.password_reset_codes WHERE expires_at <= $1")
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
        })?;

    let prc = sqlx::query(
        "SELECT code, expires_at, failed_attempts
           FROM public.password_reset_codes
          WHERE email = $1
          FOR UPDATE",
    )
    .bind(&p.email)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
    })?
    .ok_or((
        StatusCode::UNPROCESSABLE_ENTITY,
        "No pending reset for this email",
    ))?;

    let reset_code: String = prc.try_get("code").unwrap_or_default();
    let reset_expires_at: i64 = prc.try_get("expires_at").unwrap_or(0);
    let failed_attempts: i32 = prc.try_get("failed_attempts").unwrap_or(0);

    if reset_expires_at <= now {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Reset code has expired"));
    }

    if failed_attempts >= 5 {
        sqlx::query("DELETE FROM public.password_reset_codes WHERE email = $1")
            .bind(&p.email)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
            })?;
        tx.commit().await.ok();
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Too many incorrect attempts. Please request a new code.",
        ));
    }

    if reset_code.as_bytes().ct_eq(p.code.as_bytes()).unwrap_u8() == 0 {
        let _ = sqlx::query(
            "UPDATE public.password_reset_codes
                SET failed_attempts = failed_attempts + 1
              WHERE email = $1",
        )
        .bind(&p.email)
        .execute(&mut *tx)
        .await;
        tx.commit().await.ok();
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Incorrect reset code"));
    }

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

    let updated = sqlx::query(
        "UPDATE public.users
            SET password_hash         = $1,
                updated_at            = $2,
                failed_login_attempts = 0,
                lockout_until         = 0
          WHERE email = $3",
    )
    .bind(&hash)
    .bind(now)
    .bind(&p.email)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
    })?;

    if updated.rows_affected() == 0 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "No account for this email",
        ));
    }

    sqlx::query("DELETE FROM public.password_reset_codes WHERE email = $1")
        .bind(&p.email)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
        })?;

    sqlx::query(
        "DELETE FROM public.sessions
          WHERE user_id = (SELECT id FROM public.users WHERE email = $1)",
    )
    .bind(&p.email)
    .execute(&mut *tx)
    .await
    .ok();

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
    })?;

    tracing::info!(email = %p.email, "password reset successful");
    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Password updated",
        }),
    ))
}
