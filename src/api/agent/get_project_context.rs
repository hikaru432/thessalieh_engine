use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::admin::commission_rates::CommissionRateResponse;
use crate::api::admin::projects::ProjectResponse;
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::guards::{require_agent, resolve_agent_roster_id, assert_agent_on_project};
use super::rows::{PROJECT_COLUMNS, row_to_project, row_to_rate};

fn agent_name_from_json(agents: &Value, agent_id: &str) -> Option<String> {
    agents.as_array()?.iter().find_map(|a| {
        if a.get("id").and_then(|v| v.as_str()) == Some(agent_id) {
            a.get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    })
}

#[derive(Serialize)]
pub struct AgentContextResponse {
    pub project: ProjectResponse,
    pub rates: Vec<CommissionRateResponse>,
    pub agent_id: String,
    pub agent_name: String,
}

/// GET /me/agent/projects/{id}/context
pub async fn get_project_context(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<AgentContextResponse>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_agent(&role)?;
    let roster_id = resolve_agent_roster_id(&pool, user_id).await?;
    assert_agent_on_project(&pool, roster_id, project_id).await?;

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
    
    let project = row_to_project(row);
    let agent_id = roster_id.to_string();
    let agent_name =
        agent_name_from_json(&project.agents_json, &agent_id).unwrap_or_else(|| "Agent".into());

    let rate_rows = sqlx::query(
        "SELECT role, commission_rate, updated_at FROM public.commission_rates ORDER BY role ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load rates")
    })?;

    Ok(Json(AgentContextResponse {
        project,
        rates: rate_rows.into_iter().map(row_to_rate).collect(),
        agent_id,
        agent_name,
    }))
}
