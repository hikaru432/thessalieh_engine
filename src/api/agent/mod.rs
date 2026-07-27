use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use chrono::{NaiveDate, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::admin::commission_rates::CommissionRateResponse;
use crate::api::admin::commission_status::{
    CommissionPeriodStatusResponse, ListCommissionStatusQuery, row_to_status,
};
use crate::api::admin::contracts::{
    CONTRACT_COLUMNS_WITH_TOTALS, ContractResponse, row_to_contract,
};
use crate::api::admin::projects::ProjectResponse;
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

fn require_agent(role: &str) -> Result<(), E> {
    if role == "Agent" {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Agent access required"))
    }
}

async fn resolve_agent_roster_id(pool: &PgPool, user_id: Uuid) -> Result<Uuid, E> {
    sqlx::query_scalar(
        "SELECT id FROM public.roster WHERE user_id = $1 AND role = 'Agent' LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?
    .ok_or((StatusCode::FORBIDDEN, "No Agent roster entry for this user"))
}

async fn assert_agent_on_project(
    pool: &PgPool,
    roster_id: Uuid,
    project_id: Uuid,
) -> Result<(), E> {
    let owned: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1
              FROM public.projects p
             WHERE p.id = $1
               AND EXISTS (
                 SELECT 1
                   FROM jsonb_array_elements(p.agents_json) AS a
                  WHERE a->>'id' = $2
               )
         )",
    )
    .bind(project_id)
    .bind(roster_id.to_string())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?;

    if owned {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            "Not assigned as an Agent on this project",
        ))
    }
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

fn row_to_rate(row: sqlx::postgres::PgRow) -> CommissionRateResponse {
    CommissionRateResponse {
        role: row.try_get("role").unwrap_or_default(),
        commission_rate: row.try_get("commission_rate").unwrap_or(0.0),
        updated_at: row.try_get("updated_at").unwrap_or(0),
    }
}

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

const PROJECT_COLUMNS: &str = "p.id, p.name, p.created_at, p.lead_broker_roster_id, p.titling_officer_roster_id, p.agent_commission_split_months, p.agents_json";

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

/// GET /me/agent/projects/{id}/contracts
pub async fn list_project_contracts(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ContractResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_agent(&role)?;
    let roster_id = resolve_agent_roster_id(&pool, user_id).await?;
    assert_agent_on_project(&pool, roster_id, project_id).await?;

    let rows = sqlx::query(&format!(
        "SELECT {CONTRACT_COLUMNS_WITH_TOTALS}
           FROM public.contracts c
           LEFT JOIN public.users bu ON bu.id = c.buyer_user_id
           LEFT JOIN public.payments p ON p.contract_id = c.id
          WHERE c.project_id = $1
            AND c.selling_agent_id = $2
       GROUP BY c.id
       ORDER BY c.created_at ASC",
    ))
    .bind(project_id)
    .bind(roster_id.to_string())
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load contracts")
    })?;

    Ok(Json(rows.into_iter().map(row_to_contract).collect()))
}

#[derive(Serialize)]
pub struct AgentCashFlowPaymentResponse {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub amount: f64,
    pub method: String,
    pub months_covered: i32,
    pub paid_at: NaiveDate,
    pub reference_no: String,
    pub bank_name: String,
    pub sender_name: String,
    pub receiver_name: String,
    pub mode_label: String,
    pub buyer_name: String,
    pub lot_block: String,
    pub lot_lot: String,
    pub term_years: i32,
    pub term_months: i32,
    pub selling_agent_id: Option<String>,
}

fn row_to_agent_cashflow_payment(row: sqlx::postgres::PgRow) -> AgentCashFlowPaymentResponse {
    AgentCashFlowPaymentResponse {
        id: row.try_get("id").unwrap_or_default(),
        contract_id: row.try_get("contract_id").unwrap_or_default(),
        amount: row.try_get("amount").unwrap_or(0.0),
        method: row.try_get("method").unwrap_or_default(),
        months_covered: row.try_get("months_covered").unwrap_or(1),
        paid_at: row
            .try_get("paid_at")
            .unwrap_or_else(|_| Utc::now().date_naive()),
        reference_no: row.try_get("reference_no").unwrap_or_default(),
        bank_name: row.try_get("bank_name").unwrap_or_default(),
        sender_name: row.try_get("sender_name").unwrap_or_default(),
        receiver_name: row.try_get("receiver_name").unwrap_or_default(),
        mode_label: row.try_get("mode_label").unwrap_or_default(),
        buyer_name: row.try_get("buyer_name").unwrap_or_default(),
        lot_block: row.try_get("lot_block").unwrap_or_default(),
        lot_lot: row.try_get("lot_lot").unwrap_or_default(),
        term_years: row.try_get("term_years").unwrap_or(0),
        term_months: row.try_get("term_months").unwrap_or(0),
        selling_agent_id: row.try_get("selling_agent_id").ok().flatten(),
    }
}

/// GET /me/agent/projects/{id}/payments — this agent's sales only.
pub async fn list_project_payments(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<AgentCashFlowPaymentResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_agent(&role)?;
    let roster_id = resolve_agent_roster_id(&pool, user_id).await?;
    assert_agent_on_project(&pool, roster_id, project_id).await?;

    let rows = sqlx::query(
        "SELECT p.id, p.contract_id, p.amount, p.method, p.months_covered, p.paid_at,
                p.reference_no, p.bank_name, p.sender_name, p.receiver_name, p.mode_label,
                c.buyer_name, c.lot_block, c.lot_lot, c.term_years, c.term_months,
                c.selling_agent_id
           FROM public.payments p
           INNER JOIN public.contracts c ON c.id = p.contract_id
          WHERE c.project_id = $1
            AND c.selling_agent_id = $2
       ORDER BY p.paid_at DESC, p.created_at DESC",
    )
    .bind(project_id)
    .bind(roster_id.to_string())
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load payments")
    })?;

    Ok(Json(
        rows.into_iter().map(row_to_agent_cashflow_payment).collect(),
    ))
}

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

/// GET /me/agent/projects/{id}/commission-status — this agent subject only, read-only.
pub async fn list_commission_status(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ListCommissionStatusQuery>,
) -> Result<Json<Vec<CommissionPeriodStatusResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_agent(&role)?;
    let roster_id = resolve_agent_roster_id(&pool, user_id).await?;
    assert_agent_on_project(&pool, roster_id, project_id).await?;

    let subject_id = roster_id.to_string();
    let from = parse_optional_ymd(query.from.as_deref(), "from")?;
    let to = parse_optional_ymd(query.to.as_deref(), "to")?;

    let rows = sqlx::query(
        "SELECT id, project_id, subject_agent_id, period_start, period_end,
                status, partial_amount, partial_paid_at, updated_at
           FROM public.commission_period_status
          WHERE project_id = $1
            AND subject_agent_id = $2
            AND ($3::date IS NULL OR period_start >= $3)
            AND ($4::date IS NULL OR period_start <= $4)
       ORDER BY period_start ASC",
    )
    .bind(project_id)
    .bind(&subject_id)
    .bind(from)
    .bind(to)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load commission status",
        )
    })?;

    Ok(Json(rows.into_iter().map(row_to_status).collect()))
}
