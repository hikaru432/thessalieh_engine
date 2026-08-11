use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::require_admin;
use crate::api::users::shared::E;

const STATUSES: [&str; 4] = ["not_yet", "partial", "pending", "paid"];

#[derive(Serialize)]
pub struct CommissionPeriodStatusResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub subject_agent_id: String,
    pub row_key: String,
    pub period_start: String,
    pub period_end: String,
    pub status: String,
    pub partial_amount: Option<f64>,
    pub partial_paid_at: Option<String>,
    /// The date this period actually transitioned to "paid" — set once, then
    /// preserved across later re-saves that keep it paid (unlike `updated_at`,
    /// which bumps on every save regardless of whether status changed). Null
    /// whenever status isn't "paid".
    pub paid_at: Option<String>,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct ListCommissionStatusQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Deserialize)]
pub struct UpsertCommissionStatusInput {
    pub subject_agent_id: String,
    pub row_key: String,
    pub period_start: String,
    pub period_end: String,
    pub status: String,
    pub partial_amount: Option<f64>,
    pub partial_paid_at: Option<String>,
}

fn parse_date(value: &str, field: &'static str) -> Result<NaiveDate, E> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            match field {
                "from" => "from must be YYYY-MM-DD",
                "to" => "to must be YYYY-MM-DD",
                "period_start" => "period_start must be YYYY-MM-DD",
                "period_end" => "period_end must be YYYY-MM-DD",
                "partial_paid_at" => "partial_paid_at must be YYYY-MM-DD",
                _ => "Date must be YYYY-MM-DD",
            },
        )
    })
}

fn format_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

pub(crate) fn row_to_status(row: sqlx::postgres::PgRow) -> CommissionPeriodStatusResponse {
    let period_start: NaiveDate = row.try_get("period_start").unwrap_or_default();
    let period_end: NaiveDate = row.try_get("period_end").unwrap_or_default();
    let partial_paid_at: Option<NaiveDate> = row.try_get("partial_paid_at").ok().flatten();
    let paid_at: Option<NaiveDate> = row.try_get("paid_at").ok().flatten();
    CommissionPeriodStatusResponse {
        id: row.try_get("id").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        subject_agent_id: row.try_get("subject_agent_id").unwrap_or_default(),
        row_key: row.try_get("row_key").unwrap_or_default(),
        period_start: format_date(period_start),
        period_end: format_date(period_end),
        status: row.try_get("status").unwrap_or_else(|_| "not_yet".into()),
        partial_amount: row.try_get("partial_amount").ok().flatten(),
        partial_paid_at: partial_paid_at.map(format_date),
        paid_at: paid_at.map(format_date),
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

pub async fn list_commission_status(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ListCommissionStatusQuery>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<CommissionPeriodStatusResponse>>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let from = query
        .from
        .as_deref()
        .map(|s| parse_date(s, "from"))
        .transpose()?;
    let to = query
        .to
        .as_deref()
        .map(|s| parse_date(s, "to"))
        .transpose()?;

    let rows = sqlx::query(
        "SELECT id, project_id, subject_agent_id, row_key, period_start, period_end,
                status, partial_amount, partial_paid_at, paid_at, updated_at,
                COUNT(*) OVER() AS total_count
           FROM public.commission_period_status
          WHERE project_id = $1
            AND ($2::date IS NULL OR period_start >= $2)
            AND ($3::date IS NULL OR period_start <= $3)
       ORDER BY period_start ASC, subject_agent_id ASC
          LIMIT $4 OFFSET $5",
    )
    .bind(project_id)
    .bind(from)
    .bind(to)
    .bind(page_query.per_page())
    .bind(page_query.offset())
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load commission status",
        )
    })?;

    let total = total_count(&rows);
    Ok(Json(Page::new(
        rows.into_iter().map(row_to_status).collect(),
        &page_query,
        total,
    )))
}

pub async fn upsert_commission_status(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(p): Json<UpsertCommissionStatusInput>,
) -> Result<Json<CommissionPeriodStatusResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let subject = p.subject_agent_id.trim();
    if subject.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "subject_agent_id is required",
        ));
    }
    let row_key = p.row_key.trim();
    if row_key.is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "row_key is required"));
    }
    if !STATUSES.contains(&p.status.as_str()) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "status must be not_yet, partial, pending, or paid",
        ));
    }

    let period_start = parse_date(&p.period_start, "period_start")?;
    let period_end = parse_date(&p.period_end, "period_end")?;
    if period_end < period_start {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "period_end must be on or after period_start",
        ));
    }

    let partial_amount = if p.status == "partial" {
        let amount = p.partial_amount.unwrap_or(0.0);
        if !amount.is_finite() || amount < 0.0 {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "partial_amount must be a non-negative number",
            ));
        }
        Some(amount)
    } else {
        None
    };

    let partial_paid_at = if p.status == "partial" {
        match p.partial_paid_at.as_deref() {
            Some(s) if !s.trim().is_empty() => Some(parse_date(s, "partial_paid_at")?),
            _ => Some(Utc::now().date_naive()),
        }
    } else {
        None
    };

    let now = Utc::now().timestamp();
    // Only used as the value for a brand-new row, or the first time an existing row
    // transitions to "paid" (see the COALESCE in the UPDATE branch below, which keeps
    // whatever paid_at was already recorded across any later re-save that leaves the
    // row paid).
    let paid_at_if_new = if p.status == "paid" {
        Some(Utc::now().date_naive())
    } else {
        None
    };

    let row = sqlx::query(
        "INSERT INTO public.commission_period_status (
            project_id, subject_agent_id, row_key, period_start, period_end,
            status, partial_amount, partial_paid_at, paid_at, updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (project_id, subject_agent_id, row_key, period_start) DO UPDATE
            SET period_end = EXCLUDED.period_end,
                status = EXCLUDED.status,
                partial_amount = EXCLUDED.partial_amount,
                partial_paid_at = EXCLUDED.partial_paid_at,
                paid_at = CASE
                    WHEN EXCLUDED.status = 'paid'
                        THEN COALESCE(public.commission_period_status.paid_at, EXCLUDED.paid_at)
                    ELSE NULL
                END,
                updated_at = EXCLUDED.updated_at
      RETURNING id, project_id, subject_agent_id, row_key, period_start, period_end,
                status, partial_amount, partial_paid_at, paid_at, updated_at",
    )
    .bind(project_id)
    .bind(subject)
    .bind(row_key)
    .bind(period_start)
    .bind(period_end)
    .bind(&p.status)
    .bind(partial_amount)
    .bind(partial_paid_at)
    .bind(paid_at_if_new)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save commission status",
        )
    })?;

    Ok(Json(row_to_status(row)))
}
