use axum::{
    Extension, Json,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::api::shared::require_admin;
use crate::api::users::shared::E;

#[derive(Serialize)]
pub struct CompanySettings {
    pub company_name: String,
    pub office_address: String,
    pub currency: String,
    pub timezone: String,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct CompanySettingsInput {
    pub company_name: String,
    pub office_address: String,
    pub currency: String,
    pub timezone: String,
}

fn row_to_settings(row: sqlx::postgres::PgRow) -> CompanySettings {
    CompanySettings {
        company_name: row.try_get("company_name").unwrap_or_default(),
        office_address: row.try_get("office_address").unwrap_or_default(),
        currency: row.try_get("currency").unwrap_or_default(),
        timezone: row.try_get("timezone").unwrap_or_default(),
        updated_at: row.try_get("updated_at").unwrap_or(0),
    }
}

pub async fn get_settings(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<CompanySettings>, E> {
    require_admin(&pool, &headers).await?;

    let row = sqlx::query(
        "SELECT company_name, office_address, currency, timezone, updated_at
           FROM public.company_settings WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load company settings",
        )
    })?;

    Ok(Json(row_to_settings(row)))
}

pub async fn update_settings(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(p): Json<CompanySettingsInput>,
) -> Result<Json<CompanySettings>, E> {
    require_admin(&pool, &headers).await?;

    let company_name = p.company_name.trim();
    if company_name.is_empty() || company_name.len() > 255 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid company name"));
    }
    if p.office_address.len() > 2000 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Office address too long"));
    }
    if p.currency.trim().is_empty() || p.currency.len() > 10 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid currency"));
    }
    if p.timezone.trim().is_empty() || p.timezone.len() > 64 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid timezone"));
    }

    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "UPDATE public.company_settings
            SET company_name = $1, office_address = $2, currency = $3, timezone = $4, updated_at = $5
          WHERE id = 1
      RETURNING company_name, office_address, currency, timezone, updated_at",
    )
    .bind(company_name)
    .bind(&p.office_address)
    .bind(&p.currency)
    .bind(&p.timezone)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update company settings",
        )
    })?;

    Ok(Json(row_to_settings(row)))
}
