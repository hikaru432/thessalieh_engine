use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::shared::require_admin;
use crate::api::users::shared::E;

const OTHER_FLAGS: [&str; 3] = ["none", "half", "full"];

#[derive(Serialize)]
pub struct CommissionRowMetaResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub subject_agent_id: String,
    pub row_key: String,
    pub period_start: String,
    pub other_flag: String,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct ListCommissionRowMetaQuery {
    pub subject_agent_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UpsertCommissionRowMetaInput {
    pub subject_agent_id: String,
    pub row_key: String,
    pub period_start: String,
    pub other_flag: String,
}

fn parse_date(value: &str) -> Result<NaiveDate, E> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            "period_start must be YYYY-MM-DD",
        )
    })
}

fn format_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn row_to_meta(row: sqlx::postgres::PgRow) -> CommissionRowMetaResponse {
    let period_start: NaiveDate = row.try_get("period_start").unwrap_or_default();
    CommissionRowMetaResponse {
        id: row.try_get("id").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        subject_agent_id: row.try_get("subject_agent_id").unwrap_or_default(),
        row_key: row.try_get("row_key").unwrap_or_default(),
        period_start: format_date(period_start),
        other_flag: row.try_get("other_flag").unwrap_or_else(|_| "none".into()),
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

pub async fn list_commission_row_meta(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ListCommissionRowMetaQuery>,
) -> Result<Json<Vec<CommissionRowMetaResponse>>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let subject = query
        .subject_agent_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let rows = sqlx::query(
        "SELECT id, project_id, subject_agent_id, row_key, period_start, other_flag, updated_at
           FROM public.commission_row_meta
          WHERE project_id = $1
            AND ($2::text IS NULL OR subject_agent_id = $2)
       ORDER BY period_start ASC, row_key ASC",
    )
    .bind(project_id)
    .bind(subject)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load commission row meta",
        )
    })?;

    Ok(Json(rows.into_iter().map(row_to_meta).collect()))
}

pub async fn upsert_commission_row_meta(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(p): Json<UpsertCommissionRowMetaInput>,
) -> Result<Json<CommissionRowMetaResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let subject = p.subject_agent_id.trim();
    let row_key = p.row_key.trim();
    if subject.is_empty() || row_key.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "subject_agent_id and row_key are required",
        ));
    }
    if !OTHER_FLAGS.contains(&p.other_flag.as_str()) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "other_flag must be none, half, or full",
        ));
    }

    let period_start = parse_date(&p.period_start)?;
    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "INSERT INTO public.commission_row_meta (
            project_id, subject_agent_id, row_key, period_start, other_flag, updated_at
         ) VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (project_id, subject_agent_id, row_key, period_start) DO UPDATE
            SET other_flag = EXCLUDED.other_flag,
                updated_at = EXCLUDED.updated_at
      RETURNING id, project_id, subject_agent_id, row_key, period_start, other_flag, updated_at",
    )
    .bind(project_id)
    .bind(subject)
    .bind(row_key)
    .bind(period_start)
    .bind(&p.other_flag)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save commission row meta",
        )
    })?;

    Ok(Json(row_to_meta(row)))
}
