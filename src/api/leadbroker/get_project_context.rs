use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::admin::project_rate_config::{ProjectRateConfigResponse, fetch_project_rate_config};
use crate::api::admin::projects::ProjectResponse;
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::guards::{assert_lb_owns_project, require_lead_broker};
use super::rows::{PROJECT_COLUMNS, row_to_project};

#[derive(Serialize)]
pub struct LbContextResponse {
    pub project: ProjectResponse,
    pub rate_config: ProjectRateConfigResponse,
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

    let rate_config = fetch_project_rate_config(&pool, project_id).await?;

    Ok(Json(LbContextResponse {
        project: row_to_project(row),
        rate_config,
    }))
}
