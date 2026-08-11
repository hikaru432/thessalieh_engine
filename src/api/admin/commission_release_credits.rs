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

/// A credit grant recorded when a release amount exceeded everything owed at the
/// time — see the migration comment for how this is applied. Never mutated after
/// creation; `buildCommissionWaterfall` (frontend) recomputes how much of it is
/// still unapplied on every read, the same way it recomputes carry-forward from
/// commission_release_entries.
#[derive(Serialize)]
pub struct CommissionReleaseCreditResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub subject_agent_id: String,
    pub share_kind: Option<String>,
    pub amount: f64,
    pub paid_at: String,
    pub note: Option<String>,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct CreateCommissionReleaseCreditInput {
    pub subject_agent_id: String,
    #[serde(default)]
    pub share_kind: Option<String>,
    pub amount: f64,
    pub paid_at: String,
    #[serde(default)]
    pub note: Option<String>,
}

fn parse_date(value: &str, field: &'static str) -> Result<NaiveDate, E> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            match field {
                "paid_at" => "paid_at must be YYYY-MM-DD",
                _ => "Date must be YYYY-MM-DD",
            },
        )
    })
}

fn format_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

pub(crate) fn row_to_credit(row: sqlx::postgres::PgRow) -> CommissionReleaseCreditResponse {
    let paid_at: NaiveDate = row.try_get("paid_at").unwrap_or_default();
    CommissionReleaseCreditResponse {
        id: row.try_get("id").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        subject_agent_id: row.try_get("subject_agent_id").unwrap_or_default(),
        share_kind: row.try_get("share_kind").unwrap_or_default(),
        amount: row.try_get("amount").unwrap_or(0.0),
        paid_at: format_date(paid_at),
        note: row.try_get("note").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(0),
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

pub async fn list_commission_release_credits(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<CommissionReleaseCreditResponse>>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let rows = sqlx::query(
        "SELECT id, project_id, subject_agent_id, share_kind, amount, paid_at, note, created_at,
                COUNT(*) OVER() AS total_count
           FROM public.commission_release_credits
          WHERE project_id = $1
       ORDER BY paid_at ASC, subject_agent_id ASC
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
            "Failed to load commission release credits",
        )
    })?;

    let total = total_count(&rows);
    Ok(Json(Page::new(
        rows.into_iter().map(row_to_credit).collect(),
        &page_query,
        total,
    )))
}

pub async fn create_commission_release_credit(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(p): Json<CreateCommissionReleaseCreditInput>,
) -> Result<Json<CommissionReleaseCreditResponse>, E> {
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

    let paid_at = parse_date(&p.paid_at, "paid_at")?;
    let note = p.note.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "INSERT INTO public.commission_release_credits (
            project_id, subject_agent_id, share_kind, amount, paid_at, note, created_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7)
      RETURNING id, project_id, subject_agent_id, share_kind, amount, paid_at, note, created_at",
    )
    .bind(project_id)
    .bind(subject)
    .bind(&p.share_kind)
    .bind(p.amount)
    .bind(paid_at)
    .bind(note)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save commission release credit",
        )
    })?;

    Ok(Json(row_to_credit(row)))
}

pub async fn delete_commission_release_credit(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(credit_id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let result = sqlx::query("DELETE FROM public.commission_release_credits WHERE id = $1")
        .bind(credit_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete commission release credit",
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Commission release credit not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}
