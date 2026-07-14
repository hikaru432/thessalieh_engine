use axum::{
    Extension, Json,
    http::{HeaderMap, StatusCode},
};
use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::shared::E;
use crate::api::shared::require_admin;

#[derive(Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

/// All user accounts, for admin pickers (e.g. assigning a roster role).
pub async fn list_users(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<UserSummary>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query("SELECT id, username, email FROM public.users ORDER BY username ASC")
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load users")
        })?;

    let users = rows
        .into_iter()
        .map(|r| UserSummary {
            id: r.try_get("id").unwrap_or_default(),
            username: r.try_get("username").unwrap_or_default(),
            email: r.try_get("email").unwrap_or_default(),
        })
        .collect();

    Ok(Json(users))
}
