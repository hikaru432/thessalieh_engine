use axum::{Extension, Json, http::HeaderMap};
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::admin::salary::format_date;
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;
use axum::http::StatusCode;

use super::guards::{require_employee, resolve_employee_id};

#[derive(Serialize)]
pub struct MySalaryReleaseEntryResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project_name: String,
    pub period_start: String,
    pub period_end: String,
    pub amount: f64,
    pub paid_at: String,
    pub note: Option<String>,
}

/// GET /me/employee/releases — every release recorded against this employee, across
/// every project (unlike the admin endpoint, which is scoped to one project at a
/// time), so the portal can show a complete "what's been paid" history.
pub async fn list_my_releases(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<MySalaryReleaseEntryResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_employee(&role)?;
    let employee_id = resolve_employee_id(&pool, user_id).await?;

    let rows = sqlx::query(
        "SELECT sre.id, sre.project_id, p.name AS project_name, sre.period_start,
                sre.period_end, sre.amount, sre.paid_at, sre.note
           FROM public.salary_release_entries sre
           JOIN public.projects p ON p.id = sre.project_id
          WHERE sre.employee_id = $1
       ORDER BY sre.period_start ASC, sre.paid_at ASC",
    )
    .bind(employee_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load salary releases",
        )
    })?;

    Ok(Json(
        rows.into_iter()
            .map(|row| {
                let period_start: NaiveDate = row.try_get("period_start").unwrap_or_default();
                let period_end: NaiveDate = row.try_get("period_end").unwrap_or_default();
                let paid_at: NaiveDate = row.try_get("paid_at").unwrap_or_default();
                MySalaryReleaseEntryResponse {
                    id: row.try_get("id").unwrap_or_default(),
                    project_id: row.try_get("project_id").unwrap_or_default(),
                    project_name: row.try_get("project_name").unwrap_or_default(),
                    period_start: format_date(period_start),
                    period_end: format_date(period_end),
                    amount: row.try_get("amount").unwrap_or(0.0),
                    paid_at: format_date(paid_at),
                    note: row.try_get("note").unwrap_or_default(),
                }
            })
            .collect(),
    ))
}
