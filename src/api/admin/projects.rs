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

/// Pull root roster UUIDs from agents_json (agent.id = roster.id).
fn root_roster_ids_from_agents(agents: &Value) -> (Option<Uuid>, Option<Uuid>) {
    let mut lead_broker = None;
    let mut titling_officer = None;
    let Some(items) = agents.as_array() else {
        return (None, None);
    };
    for item in items {
        let Some(obj) = item.as_object() else {
            continue;
        };
        let role = obj.get("role").and_then(Value::as_str).unwrap_or("");
        let id = obj
            .get("id")
            .and_then(Value::as_str)
            .and_then(|s| Uuid::parse_str(s).ok());
        match role {
            "lead-broker" if lead_broker.is_none() => lead_broker = id,
            "titling-officer" if titling_officer.is_none() => titling_officer = id,
            _ => {}
        }
    }
    (lead_broker, titling_officer)
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

#[derive(Serialize)]
pub struct ProjectSummaryTotals {
    pub project_id: Uuid,
    pub buyers: i64,
    pub tcp: f64,
    pub collected: f64,
    pub balance: f64,
}

#[derive(Serialize)]
pub struct MonthlyCollectionPoint {
    pub month: String,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct CompanyProjectsSummary {
    pub projects: Vec<ProjectSummaryTotals>,
    pub monthly_collections: Vec<MonthlyCollectionPoint>,
}

fn row_to_project_summary_totals(row: sqlx::postgres::PgRow) -> ProjectSummaryTotals {
    ProjectSummaryTotals {
        project_id: row.try_get("project_id").unwrap_or_default(),
        buyers: row.try_get("buyers").unwrap_or(0),
        tcp: row.try_get("tcp").unwrap_or(0.0),
        collected: row.try_get("collected").unwrap_or(0.0),
        balance: row.try_get("balance").unwrap_or(0.0),
    }
}

fn row_to_monthly_collection_point(row: sqlx::postgres::PgRow) -> MonthlyCollectionPoint {
    MonthlyCollectionPoint {
        month: row.try_get("month").unwrap_or_default(),
        amount: row.try_get("amount").unwrap_or(0.0),
    }
}

/// Per-project totals + a company-wide monthly collections trend, computed in SQL —
/// replaces what used to be a full contracts+payments fetch per project just to
/// render summary cards on the Projects overview page.
pub async fn list_projects_summary(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<CompanyProjectsSummary>, E> {
    require_admin(&pool, &headers).await?;

    let totals_rows = sqlx::query(
        "SELECT c.project_id,
                COUNT(DISTINCT COALESCE(c.buyer_user_id::text, 'contract:' || c.id::text)) AS buyers,
                COALESCE(SUM(c.contract_price), 0) AS tcp,
                COALESCE(SUM(paid.total_paid), 0) AS collected,
                COALESCE(SUM(GREATEST(c.contract_price - COALESCE(paid.total_paid, 0), 0)), 0) AS balance
           FROM public.contracts c
           JOIN public.projects p ON p.id = c.project_id AND p.company_id = 1
           LEFT JOIN (
               SELECT contract_id, SUM(amount) AS total_paid
                 FROM public.payments
             GROUP BY contract_id
           ) paid ON paid.contract_id = c.id
       GROUP BY c.project_id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load project totals",
        )
    })?;

    let monthly_rows = sqlx::query(
        "SELECT to_char(p.paid_at, 'YYYY-MM') AS month, COALESCE(SUM(p.amount), 0) AS amount
           FROM public.payments p
           JOIN public.contracts c ON c.id = p.contract_id
           JOIN public.projects pr ON pr.id = c.project_id AND pr.company_id = 1
          WHERE p.paid_at >= (((now() AT TIME ZONE 'UTC')::date) - INTERVAL '12 months')
       GROUP BY month
       ORDER BY month",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load monthly collections",
        )
    })?;

    Ok(Json(CompanyProjectsSummary {
        projects: totals_rows
            .into_iter()
            .map(row_to_project_summary_totals)
            .collect(),
        monthly_collections: monthly_rows
            .into_iter()
            .map(row_to_monthly_collection_point)
            .collect(),
    }))
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

    let (lead_broker_roster_id, titling_officer_roster_id) = root_roster_ids_from_agents(&p.agents);
    let now = Utc::now().timestamp();

    let row = sqlx::query(&format!(
        "UPDATE public.projects
            SET agents_json = $1,
                lead_broker_roster_id = $2,
                titling_officer_roster_id = $3,
                updated_at = $4
          WHERE id = $5 AND company_id = 1
      RETURNING {PROJECT_COLUMNS}",
    ))
    .bind(&p.agents)
    .bind(lead_broker_roster_id)
    .bind(titling_officer_roster_id)
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
