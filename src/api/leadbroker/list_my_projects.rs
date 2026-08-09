use axum::{
    Extension, Json,
    http::{HeaderMap, StatusCode},
};
use sqlx::PgPool;

use crate::api::admin::projects::ProjectResponse;
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::guards::require_lead_broker;
use super::rows::{PROJECT_COLUMNS, row_to_project};

/// GET /me/lb/projects
pub async fn list_my_projects(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProjectResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_lead_broker(&role)?;

    let rows = sqlx::query(&format!(
        "SELECT DISTINCT {PROJECT_COLUMNS}
           FROM public.projects p
           JOIN public.roster r ON r.user_id = $1
          WHERE p.lead_broker_roster_id = r.id
             OR EXISTS (
               SELECT 1
                 FROM jsonb_array_elements(p.agents_json) AS a
                WHERE a->>'role' = 'lead-broker'
                  AND a->>'id' = r.id::text
             )
       ORDER BY p.created_at ASC",
    ))
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load projects")
    })?;

    Ok(Json(rows.into_iter().map(row_to_project).collect()))
}
