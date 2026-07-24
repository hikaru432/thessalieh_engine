use axum::{
    Extension, Json,
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::shared::E;
use crate::api::shared::require_admin;

#[derive(Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub phone: Option<String>,
}

#[derive(Deserialize)]
pub struct ListUsersQuery {
    pub role: Option<String>,
}

/// All user accounts, for admin pickers (e.g. assigning a roster role or buyer).
pub async fn list_users(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<Vec<UserSummary>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = if let Some(role) = query.role.as_deref().map(str::trim).filter(|r| !r.is_empty()) {
        sqlx::query(
            "SELECT id, username, email, role, phone
               FROM public.users
              WHERE role = $1
           ORDER BY username ASC",
        )
        .bind(role)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query(
            "SELECT id, username, email, role, phone
               FROM public.users
           ORDER BY username ASC",
        )
        .fetch_all(&pool)
        .await
    }
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
            role: r.try_get("role").unwrap_or_default(),
            phone: r.try_get("phone").ok().flatten(),
        })
        .collect();

    Ok(Json(users))
}
