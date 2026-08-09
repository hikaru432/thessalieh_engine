use axum::{
    Extension, Json,
    http::{HeaderMap, StatusCode},
};
use sqlx::PgPool;

use crate::api::admin::projects::ProjectResponse;
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::guards::{require_agent, resolve_agent_roster_id};
use super::rows::{PROJECT_COLUMNS, row_to_project};

/// GET /me/agent/projects
pub async fn list_my_projects(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProjectResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_agent(&role)?;
    let roster_id = resolve_agent_roster_id(&pool, user_id).await?;

    let rows = sqlx::query(&format!(
        "SELECT DISTINCT {PROJECT_COLUMNS}
           FROM public.projects p
          WHERE EXISTS (
            SELECT 1
              FROM jsonb_array_elements(p.agents_json) AS a
             WHERE a->>'id' = $1
          )
       ORDER BY p.created_at ASC",
    ))
    .bind(roster_id.to_string())
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load projects")
    })?;

    Ok(Json(rows.into_iter().map(row_to_project).collect()))
}
