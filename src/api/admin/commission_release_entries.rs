use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::{normalize_share_kind, require_admin};
use crate::api::users::shared::E;

#[derive(Serialize)]
pub struct CommissionReleaseEntryResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub subject_agent_id: String,
    pub period_start: String,
    pub period_end: String,
    pub amount: f64,
    pub paid_at: String,
    pub created_at: i64,
    /// Scopes this entry to a specific commission component ("base"/"pool" for a
    /// root's Baseline vs Direct buyer sections, "promo" for the Promo tab) so they
    /// can be released independently. Null for plain-agent release entries, which
    /// have no split.
    pub share_kind: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateCommissionReleaseEntryInput {
    pub subject_agent_id: String,
    pub period_start: String,
    pub period_end: String,
    pub amount: f64,
    pub paid_at: String,
    #[serde(default)]
    pub share_kind: Option<String>,
}

fn parse_date(value: &str, field: &'static str) -> Result<NaiveDate, E> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            match field {
                "period_start" => "period_start must be YYYY-MM-DD",
                "period_end" => "period_end must be YYYY-MM-DD",
                "paid_at" => "paid_at must be YYYY-MM-DD",
                _ => "Date must be YYYY-MM-DD",
            },
        )
    })
}

fn format_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let (ny, nm) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    NaiveDate::from_ymd_opt(ny, nm, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day()
}

/// This system's release cadence has exactly two periods per calendar month: 1-15
/// and 16-(28/29/30/31) — same convention `change_contract_split` already enforces
/// on `effective_period_start` (see contract_split_history.rs). A release entry
/// whose period doesn't land on that boundary (e.g. a typo'd start date) would
/// silently fail to match the frontend's period-keyed lookup, recording money as
/// released while leaving it invisible in the waterfall — this rejects that at
/// write time instead of letting it become an orphaned row.
fn validate_period_boundary(period_start: NaiveDate, period_end: NaiveDate) -> Result<(), E> {
    let expected_end_day = match period_start.day() {
        1 => 15,
        16 => last_day_of_month(period_start.year(), period_start.month()),
        _ => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "period_start must be the 1st or the 16th of a month",
            ));
        }
    };
    let expected_end =
        NaiveDate::from_ymd_opt(period_start.year(), period_start.month(), expected_end_day)
            .unwrap_or(period_start);
    if period_end != expected_end {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "period_end must match the biweekly period implied by period_start (15th, or the last day of the month)",
        ));
    }
    Ok(())
}

pub(crate) fn row_to_entry(row: sqlx::postgres::PgRow) -> CommissionReleaseEntryResponse {
    let period_start: NaiveDate = row.try_get("period_start").unwrap_or_default();
    let period_end: NaiveDate = row.try_get("period_end").unwrap_or_default();
    let paid_at: NaiveDate = row.try_get("paid_at").unwrap_or_default();
    CommissionReleaseEntryResponse {
        id: row.try_get("id").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        subject_agent_id: row.try_get("subject_agent_id").unwrap_or_default(),
        period_start: format_date(period_start),
        period_end: format_date(period_end),
        amount: row.try_get("amount").unwrap_or(0.0),
        paid_at: format_date(paid_at),
        created_at: row.try_get("created_at").unwrap_or(0),
        share_kind: row.try_get("share_kind").unwrap_or_default(),
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

/// Every dated commission release amount an admin has recorded against a subject +
/// biweekly period — the ledger can hold multiple entries per period (e.g. a partial
/// release now, another later), which the frontend sums to compute Remaining.
pub async fn list_commission_release_entries(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<CommissionReleaseEntryResponse>>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let rows = sqlx::query(
        "SELECT id, project_id, subject_agent_id, period_start, period_end,
                amount, paid_at, created_at, share_kind, COUNT(*) OVER() AS total_count
           FROM public.commission_release_entries
          WHERE project_id = $1
       ORDER BY period_start ASC, subject_agent_id ASC, paid_at ASC
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
            "Failed to load commission release entries",
        )
    })?;

    let total = total_count(&rows);
    Ok(Json(Page::new(
        rows.into_iter().map(row_to_entry).collect(),
        &page_query,
        total,
    )))
}

pub async fn create_commission_release_entry(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(p): Json<CreateCommissionReleaseEntryInput>,
) -> Result<Json<CommissionReleaseEntryResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let subject = p.subject_agent_id.trim();
    if subject.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "subject_agent_id is required",
        ));
    }
    if !p.amount.is_finite() || p.amount <= 0.0 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "amount must be a positive number",
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
    validate_period_boundary(period_start, period_end)?;
    let paid_at = parse_date(&p.paid_at, "paid_at")?;
    let share_kind = normalize_share_kind(p.share_kind)?;

    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "INSERT INTO public.commission_release_entries (
            project_id, subject_agent_id, period_start, period_end,
            amount, paid_at, created_at, share_kind
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
      RETURNING id, project_id, subject_agent_id, period_start, period_end,
                amount, paid_at, created_at, share_kind",
    )
    .bind(project_id)
    .bind(subject)
    .bind(period_start)
    .bind(period_end)
    .bind(p.amount)
    .bind(paid_at)
    .bind(now)
    .bind(&share_kind)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save commission release entry",
        )
    })?;

    Ok(Json(row_to_entry(row)))
}

#[derive(Serialize)]
pub struct ProjectCommissionReleaseTotal {
    pub project_id: Uuid,
    pub project_name: String,
    pub total: f64,
}

#[derive(Serialize)]
pub struct CommissionReleaseSummaryResponse {
    pub total: f64,
    pub projects: Vec<ProjectCommissionReleaseTotal>,
}

/// GET /commission-release-summary — admin only. Company-wide commission-release
/// total plus a per-project breakdown, for the Cash Out card on the Projects
/// overview page (a `LEFT JOIN` keeps projects with zero releases in the list, at
/// total = 0, instead of silently dropping them).
pub async fn commission_release_summary(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<CommissionReleaseSummaryResponse>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(
        "SELECT p.id AS project_id, p.name AS project_name, COALESCE(SUM(e.amount), 0) AS total
           FROM public.projects p
           LEFT JOIN public.commission_release_entries e ON e.project_id = p.id
          WHERE p.company_id = 1
       GROUP BY p.id, p.name
       ORDER BY p.created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load commission release summary",
        )
    })?;

    let projects: Vec<ProjectCommissionReleaseTotal> = rows
        .into_iter()
        .map(|row| ProjectCommissionReleaseTotal {
            project_id: row.try_get("project_id").unwrap_or_default(),
            project_name: row.try_get("project_name").unwrap_or_default(),
            total: row.try_get("total").unwrap_or(0.0),
        })
        .collect();
    let total = projects.iter().map(|p| p.total).sum();

    Ok(Json(CommissionReleaseSummaryResponse { total, projects }))
}

pub async fn delete_commission_release_entry(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(entry_id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let result = sqlx::query("DELETE FROM public.commission_release_entries WHERE id = $1")
        .bind(entry_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete commission release entry",
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Commission release entry not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}
