use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use sqlx::PgPool;

use crate::api::admin::projects::ProjectResponse;
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::guards::require_upline_role;
use super::rows::{PROJECT_COLUMNS, row_to_project};

/// GET /me/upline/{role_slug}/projects
pub async fn list_my_projects(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(role_slug): Path<String>,
) -> Result<Json<Vec<ProjectResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_upline_role(&pool, &role_slug, &role).await?;

    let rows = sqlx::query(&format!(
        "SELECT DISTINCT {PROJECT_COLUMNS}
           FROM public.projects p
           JOIN public.roster r ON r.user_id = $1
          WHERE ($2 = 'lead-broker' AND p.lead_broker_roster_id = r.id)
             OR ($2 = 'titling-officer' AND p.titling_officer_roster_id = r.id)
             OR EXISTS (
               SELECT 1
                 FROM jsonb_array_elements(p.agents_json) AS a
                WHERE a->>'role' = $2
                  AND a->>'id' = r.id::text
             )
       ORDER BY p.created_at ASC",
    ))
    .bind(user_id)
    .bind(&role_slug)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load projects")
    })?;

    Ok(Json(rows.into_iter().map(row_to_project).collect()))
}
