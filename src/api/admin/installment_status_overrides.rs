use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::require_admin;
use crate::api::users::shared::E;

const STATUSES: [&str; 4] = ["", "Paid", "Half", "Hold"];

#[derive(Serialize)]
pub struct InstallmentStatusOverrideResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub contract_id: Uuid,
    pub year: i32,
    pub month: i32,
    pub status: String,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct UpsertInstallmentStatusOverrideInput {
    pub contract_id: Uuid,
    pub year: i32,
    pub month: i32,
    pub status: String,
}

fn row_to_override(row: sqlx::postgres::PgRow) -> InstallmentStatusOverrideResponse {
    InstallmentStatusOverrideResponse {
        id: row.try_get("id").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        contract_id: row.try_get("contract_id").unwrap_or_default(),
        year: row.try_get("year").unwrap_or_default(),
        month: row.try_get("month").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        updated_at: row.try_get("updated_at").unwrap_or(0),
    }
}

async fn ensure_project(pool: &PgPool, project_id: Uuid) -> Result<(), E> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM public.projects WHERE id = $1 AND company_id = 1)",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to verify project")
    })?;
    if !exists {
        return Err((StatusCode::NOT_FOUND, "Project not found"));
    }
    Ok(())
}

/// Every manual status override an admin has set on the Payment Tracker grid for
/// this project — a persisted override always wins over the auto-computed status;
/// see MonthStatusCell in tracking/Main.tsx.
pub async fn list_installment_status_overrides(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<InstallmentStatusOverrideResponse>>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let rows = sqlx::query(
        "SELECT id, project_id, contract_id, year, month, status, updated_at,
                COUNT(*) OVER() AS total_count
           FROM public.installment_status_overrides
          WHERE project_id = $1
       ORDER BY year ASC, month ASC, contract_id ASC
          LIMIT $2 OFFSET $3",
    )
    .bind(project_id)
    .bind(page_query.per_page())
    .bind(page_query.offset())
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load installment status overrides",
        )
    })?;

    let total = total_count(&rows);
    Ok(Json(Page::new(
        rows.into_iter().map(row_to_override).collect(),
        &page_query,
        total,
    )))
}

pub async fn upsert_installment_status_override(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(p): Json<UpsertInstallmentStatusOverrideInput>,
) -> Result<Json<InstallmentStatusOverrideResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    if !STATUSES.contains(&p.status.as_str()) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "status must be one of: (blank), Paid, Half, Hold",
        ));
    }
    if !(0..=11).contains(&p.month) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "month must be 0-11"));
    }

    let contract_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM public.contracts WHERE id = $1 AND project_id = $2)",
    )
    .bind(p.contract_id)
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to verify contract")
    })?;
    if !contract_exists {
        return Err((StatusCode::NOT_FOUND, "Contract not found in this project"));
    }

    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "INSERT INTO public.installment_status_overrides (
            project_id, contract_id, year, month, status, updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (contract_id, year, month) DO UPDATE
            SET status = EXCLUDED.status,
                updated_at = EXCLUDED.updated_at
      RETURNING id, project_id, contract_id, year, month, status, updated_at",
    )
    .bind(project_id)
    .bind(p.contract_id)
    .bind(p.year)
    .bind(p.month)
    .bind(&p.status)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save installment status override",
        )
    })?;

    Ok(Json(row_to_override(row)))
}

pub async fn delete_installment_status_override(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    sqlx::query("DELETE FROM public.installment_status_overrides WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete installment status override",
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}
