use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::shared::require_admin;
use crate::api::users::shared::E;

const PROJECT_COLUMNS: &str = "id, name, created_at, lead_broker_roster_id, titling_officer_roster_id, agent_commission_split_months, agents_json";

#[derive(Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: i64,
    pub lead_broker_roster_id: Option<Uuid>,
    pub titling_officer_roster_id: Option<Uuid>,
    pub agent_commission_split_months: i32,
    pub agents_json: Value,
}

#[derive(Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateProjectAgentsInput {
    pub agents: Value,
}

fn row_to_project(row: sqlx::postgres::PgRow) -> ProjectResponse {
    ProjectResponse {
        id: row.try_get("id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(0),
        lead_broker_roster_id: row.try_get("lead_broker_roster_id").ok().flatten(),
        titling_officer_roster_id: row
            .try_get("titling_officer_roster_id")
            .ok()
            .flatten(),
        agent_commission_split_months: row
            .try_get("agent_commission_split_months")
            .unwrap_or(15),
        agents_json: row
            .try_get("agents_json")
            .unwrap_or_else(|_| Value::Array(vec![])),
    }
}

fn validate_agents_json(agents: &Value) -> Result<(), E> {
    let Some(items) = agents.as_array() else {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "agents must be a JSON array",
        ));
    };
    for item in items {
        let Some(obj) = item.as_object() else {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Each agent must be an object",
            ));
        };
        if !obj.get("id").and_then(Value::as_str).is_some_and(|s| !s.is_empty()) {
            return Err((StatusCode::UNPROCESSABLE_ENTITY, "Agent id is required"));
        }
        if !obj.get("name").and_then(Value::as_str).is_some_and(|s| !s.is_empty()) {
            return Err((StatusCode::UNPROCESSABLE_ENTITY, "Agent name is required"));
        }
        if !obj.get("role").and_then(Value::as_str).is_some_and(|s| !s.is_empty()) {
            return Err((StatusCode::UNPROCESSABLE_ENTITY, "Agent role is required"));
        }
    }
    Ok(())
}

pub async fn list_projects(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProjectResponse>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(&format!(
        "SELECT {PROJECT_COLUMNS} FROM public.projects
          WHERE company_id = 1
       ORDER BY created_at ASC",
    ))
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load projects")
    })?;

    Ok(Json(rows.into_iter().map(row_to_project).collect()))
}

pub async fn create_project(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(p): Json<CreateProjectInput>,
) -> Result<Json<ProjectResponse>, E> {
    require_admin(&pool, &headers).await?;

    let name = p.name.trim();
    if name.is_empty() || name.len() > 255 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid project name"));
    }

    let now = Utc::now().timestamp();

    let row = sqlx::query(&format!(
        "INSERT INTO public.projects (company_id, name, created_at, updated_at, agent_commission_split_months, agents_json)
         VALUES (1, $1, $2, $2, 15, '[]'::jsonb)
      RETURNING {PROJECT_COLUMNS}",
    ))
    .bind(name)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create project")
    })?;

    Ok(Json(row_to_project(row)))
}

pub async fn update_project_agents(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(p): Json<UpdateProjectAgentsInput>,
) -> Result<Json<ProjectResponse>, E> {
    require_admin(&pool, &headers).await?;
    validate_agents_json(&p.agents)?;

    let now = Utc::now().timestamp();

    let row = sqlx::query(&format!(
        "UPDATE public.projects
            SET agents_json = $1, updated_at = $2
          WHERE id = $3 AND company_id = 1
      RETURNING {PROJECT_COLUMNS}",
    ))
    .bind(p.agents)
    .bind(now)
    .bind(project_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save project agents")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Project not found"))?;

    Ok(Json(row_to_project(row)))
}
