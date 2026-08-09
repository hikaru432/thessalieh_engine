use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::admin::commission_status::{
    CommissionPeriodStatusResponse, ListCommissionStatusQuery, row_to_status,
};
use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::agents::{load_project_agents, resolve_subject_id};
use super::guards::{assert_owns_project, require_upline_role};

fn parse_optional_ymd(value: Option<&str>, field: &'static str) -> Result<Option<NaiveDate>, E> {
    match value {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
            .map(Some)
            .map_err(|_| {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    match field {
                        "from" => "from must be YYYY-MM-DD",
                        "to" => "to must be YYYY-MM-DD",
                        _ => "Date must be YYYY-MM-DD",
                    },
                )
            }),
    }
}

/// GET /me/upline/{role_slug}/projects/{project_id}/commission-status — subject only, read-only.
pub async fn list_commission_status(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path((role_slug, project_id)): Path<(String, Uuid)>,
    Query(query): Query<ListCommissionStatusQuery>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<CommissionPeriodStatusResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    let label = require_upline_role(&pool, &role_slug, &role).await?;
    assert_owns_project(&pool, user_id, project_id, &role_slug).await?;

    let (agents_json, legacy_root_id) = load_project_agents(&pool, project_id, &role_slug).await?;
    let subject_id = resolve_subject_id(&agents_json, &role_slug, legacy_root_id).ok_or((
        StatusCode::UNPROCESSABLE_ENTITY,
        "Subject not found on project",
    ))?;
    let _ = &label; // role validated above; kept for symmetry with sibling handlers

    let from = parse_optional_ymd(query.from.as_deref(), "from")?;
    let to = parse_optional_ymd(query.to.as_deref(), "to")?;

    let rows = sqlx::query(
        "SELECT id, project_id, subject_agent_id, period_start, period_end,
                status, partial_amount, partial_paid_at, updated_at,
                COUNT(*) OVER() AS total_count
           FROM public.commission_period_status
          WHERE project_id = $1
            AND subject_agent_id = $2
            AND ($3::date IS NULL OR period_start >= $3)
            AND ($4::date IS NULL OR period_start <= $4)
       ORDER BY period_start ASC
          LIMIT $5 OFFSET $6",
    )
    .bind(project_id)
    .bind(&subject_id)
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
