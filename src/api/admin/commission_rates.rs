use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::api::shared::require_admin;
use crate::api::users::shared::E;

const ROLES: [&str; 3] = ["Lead Broker", "Titling Officer", "Agent"];

#[derive(Serialize)]
pub struct CommissionRateResponse {
    pub role: String,
    pub commission_rate: f64,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct UpdateCommissionRateInput {
    pub commission_rate: f64,
}

fn row_to_rate(row: sqlx::postgres::PgRow) -> CommissionRateResponse {
    CommissionRateResponse {
        role: row.try_get("role").unwrap_or_default(),
        commission_rate: row.try_get("commission_rate").unwrap_or(0.0),
        updated_at: row.try_get("updated_at").unwrap_or(0),
    }
}

pub async fn list_commission_rates(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<CommissionRateResponse>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(
        "SELECT role, commission_rate, updated_at FROM public.commission_rates ORDER BY role ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load commission rates",
        )
    })?;

    Ok(Json(rows.into_iter().map(row_to_rate).collect()))
}

pub async fn update_commission_rate(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(role): Path<String>,
    Json(p): Json<UpdateCommissionRateInput>,
) -> Result<Json<CommissionRateResponse>, E> {
    require_admin(&pool, &headers).await?;

    if !ROLES.contains(&role.as_str()) {
        return Err((StatusCode::NOT_FOUND, "Unknown role"));
    }
    if !p.commission_rate.is_finite() || !(0.0..=100.0).contains(&p.commission_rate) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Commission rate must be between 0 and 100",
        ));
    }

    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "UPDATE public.commission_rates
            SET commission_rate = $1, updated_at = $2
          WHERE role = $3
      RETURNING role, commission_rate, updated_at",
    )
    .bind(p.commission_rate)
    .bind(now)
    .bind(&role)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update commission rate",
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Unknown role"))?;

    Ok(Json(row_to_rate(row)))
}
