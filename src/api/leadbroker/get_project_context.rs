use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::admin::commission_rates::CommissionRateResponse;
use crate::api::admin::projects::ProjectResponse;
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::guards::{assert_lb_owns_project, require_lead_broker};
use super::rows::{PROJECT_COLUMNS, row_to_project, row_to_rate};

#[derive(Serialize)]
pub struct LbContextResponse {
    pub project: ProjectResponse,
    pub rates: Vec<CommissionRateResponse>,
}

/// GET /me/lb/projects/{id}/context
pub async fn get_project_context(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<LbContextResponse>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_lead_broker(&role)?;
    assert_lb_owns_project(&pool, user_id, project_id).await?;

    let row = sqlx::query(&format!(
        "SELECT {PROJECT_COLUMNS} FROM public.projects p WHERE p.id = $1",
    ))
    .bind(project_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load project")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Project not found"))?;

    let rate_rows = sqlx::query(
        "SELECT role, commission_rate, updated_at FROM public.commission_rates ORDER BY role ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load rates")
    })?;

    Ok(Json(LbContextResponse {
        project: row_to_project(row),
        rates: rate_rows.into_iter().map(row_to_rate).collect(),
    }))
}
