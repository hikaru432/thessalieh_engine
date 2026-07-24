use argon2::{
    Algorithm, Argon2, Params, PasswordHasher, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::shared::{E, is_strong_password, is_valid_username};

pub struct InsertUserInput {
    pub username: String,
    pub password: String,
    pub access_token: String,
}

pub struct InsertUserResult {
    pub user_id: Uuid,
    pub username: String,
    pub role: &'static str,
}

pub fn validate_user_credentials(username: &str, password: &str, access_token: &str) -> Result<(), E> {
    if !is_valid_username(username) {
        return Err((
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid username",
        ));
    }
    if !is_strong_password(password) {
        return Err((
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Password too weak",
        ));
    }
    if access_token.len() > 100 {
        return Err((
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid access token",
        ));
    }
    Ok(())
}

async fn validate_and_redeem_access_token(
    tx: &mut Transaction<'_, Postgres>,
    access_token: &str,
    now: i64,
) -> Result<(), E> {
    if access_token.is_empty() {
        return Ok(());
    }

    let token_row = sqlx::query(
        "SELECT redeemed_at, revoked_at, expires_at
           FROM public.access_tokens
          WHERE token = $1
          FOR UPDATE",
    )
    .bind(access_token)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Registration failed",
        )
    })?
    .ok_or((
        axum::http::StatusCode::UNPROCESSABLE_ENTITY,
        "Invalid access token",
    ))?;

    use sqlx::Row;
    let revoked_at: Option<i64> = token_row.try_get("revoked_at").ok().flatten();
    let redeemed_at: Option<i64> = token_row.try_get("redeemed_at").ok().flatten();
    let expires_at: Option<i64> = token_row.try_get("expires_at").ok().flatten();

    if revoked_at.is_some() {
        return Err((
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid access token",
        ));
    }
    if redeemed_at.is_some() {
        return Err((
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Access token already used",
        ));
    }
    if expires_at.is_some_and(|exp| exp <= now) {
        return Err((
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            "Access token has expired",
        ));
    }

    Ok(())
}

pub async fn insert_user(
    pool: &PgPool,
    input: InsertUserInput,
) -> Result<InsertUserResult, E> {
    let username = input.username.trim().to_string();
    let access_token = input.access_token.trim().to_string();
    validate_user_credentials(&username, &input.password, &access_token)?;

    let now = Utc::now().timestamp();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(65536, 3, 4, None).expect("valid Argon2 params"),
    );
    let hash = argon2
        .hash_password(input.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Registration failed",
        )
    })?;

    if sqlx::query("SELECT id FROM public.users WHERE username = $1")
        .bind(&username)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Registration failed",
            )
        })?
        .is_some()
    {
        tracing::warn!(%username, "register failed: username already exists");
        return Err((
            axum::http::StatusCode::CONFLICT,
            "Registration failed",
        ));
    }

    validate_and_redeem_access_token(&mut tx, &access_token, now).await?;

    let role = "User";
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
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Registration failed",
        )
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
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Registration failed",
            )
        })?;
    }

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Registration failed",
        )
    })?;

    tracing::info!(%username, %user_id, "user registered");

    Ok(InsertUserResult {
        user_id,
        username,
        role,
    })
}
