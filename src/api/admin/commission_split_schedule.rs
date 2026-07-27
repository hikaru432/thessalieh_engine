use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::shared::require_admin;
use crate::api::users::shared::E;

#[derive(Serialize)]
pub struct CommissionSplitScheduleResponse {
    pub id: Uuid,
    pub effective_date: NaiveDate,
    pub split_months: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

const COLUMNS: &str = "id, effective_date, split_months, created_at, updated_at";

fn row_to_entry(row: sqlx::postgres::PgRow) -> CommissionSplitScheduleResponse {
    CommissionSplitScheduleResponse {
        id: row.try_get("id").unwrap_or_default(),
        effective_date: row
            .try_get("effective_date")
            .unwrap_or_else(|_| Utc::now().date_naive()),
        split_months: row.try_get("split_months").unwrap_or(36),
        created_at: row.try_get("created_at").unwrap_or(0),
        updated_at: row.try_get("updated_at").unwrap_or(0),
    }
}

/// GET /commission-split-schedule — admin only.
pub async fn list_commission_split_schedule(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<CommissionSplitScheduleResponse>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(&format!(
        "SELECT {COLUMNS} FROM public.commission_split_schedule ORDER BY effective_date ASC",
    ))
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load commission split schedule",
        )
    })?;

    Ok(Json(rows.into_iter().map(row_to_entry).collect()))
}

#[derive(Deserialize)]
pub struct CreateCommissionSplitScheduleInput {
    pub effective_date: NaiveDate,
    pub split_months: i32,
}

/// POST /commission-split-schedule — admin only.
pub async fn create_commission_split_schedule(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(p): Json<CreateCommissionSplitScheduleInput>,
) -> Result<Json<CommissionSplitScheduleResponse>, E> {
    require_admin(&pool, &headers).await?;

    if !(1..=120).contains(&p.split_months) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Split months must be between 1 and 120",
        ));
    }

    let now = Utc::now().timestamp();

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO public.commission_split_schedule (effective_date, split_months, created_at, updated_at)
         VALUES ($1, $2, $3, $3)
      RETURNING id",
    )
    .bind(p.effective_date)
    .bind(p.split_months)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if let Some(d) = e.as_database_error()
            && d.code().as_deref() == Some("23505")
        {
            return (
                StatusCode::CONFLICT,
                "An entry already exists for that effective date",
            );
        }
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save schedule entry",
        )
    })?;

    let row = sqlx::query(&format!(
        "SELECT {COLUMNS} FROM public.commission_split_schedule WHERE id = $1",
    ))
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load schedule entry",
        )
    })?;

    Ok(Json(row_to_entry(row)))
}

/// DELETE /commission-split-schedule/{id} — admin only. Rejected if it's the only
/// remaining entry, so a lookup always has a baseline to resolve to.
pub async fn delete_commission_split_schedule(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.commission_split_schedule")
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
        })?;

    if total <= 1 {
        return Err((
            StatusCode::CONFLICT,
            "Cannot delete the only remaining schedule entry",
        ));
    }

    let result = sqlx::query("DELETE FROM public.commission_split_schedule WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete schedule entry",
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Schedule entry not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}
