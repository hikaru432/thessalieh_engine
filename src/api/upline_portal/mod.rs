//! Generic self-service portal API for any admin-configured upline role (Lead Broker,
//! Titling Officer, or a custom role added via /upline-role-types). Replaces what used to
//! be two near-identical modules (`api::leadbroker`, `api::titlingofficer`) with one
//! implementation parameterized by `role_slug`, so a new upline role never needs new
//! backend code — just a row in `upline_role_types`.

use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use crate::api::admin::commission_rates::CommissionRateResponse;
use crate::api::admin::commission_status::{
    CommissionPeriodStatusResponse, ListCommissionStatusQuery, row_to_status,
};
use crate::api::admin::contracts::{CONTRACT_COLUMNS_WITH_TOTALS, ContractResponse, row_to_contract};
use crate::api::admin::lots::{
    LOT_COLUMNS, LotResponse, resolve_reserve_meta, resolve_reserved_until, row_to_lot,
};
use crate::api::admin::projects::ProjectResponse;
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

/// Resolves `role_slug` to its label and checks the session's role matches it.
/// Returns the label (== users.role / roster.role value for this role).
async fn require_upline_role(pool: &PgPool, role_slug: &str, session_role: &str) -> Result<String, E> {
    let label: String =
        sqlx::query_scalar("SELECT label FROM public.upline_role_types WHERE slug = $1")
            .bind(role_slug)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
            })?
            .ok_or((StatusCode::NOT_FOUND, "Unknown upline role"))?;

    if session_role == label {
        Ok(label)
    } else {
        Err((
            StatusCode::FORBIDDEN,
            "Access to this upline portal is not permitted for your role",
        ))
    }
}

/// `lead-broker`/`titling-officer` also have a dedicated projects column as a legacy
/// fallback alongside the agents_json check; any other (custom) role relies on agents_json
/// alone, since there's no per-role dedicated column for those.
async fn assert_owns_project(
    pool: &PgPool,
    user_id: Uuid,
    project_id: Uuid,
    role_slug: &str,
) -> Result<(), E> {
    let owned: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1
              FROM public.projects p
              JOIN public.roster r ON r.user_id = $2
             WHERE p.id = $1
               AND (
                 ($3 = 'lead-broker' AND p.lead_broker_roster_id = r.id)
                 OR ($3 = 'titling-officer' AND p.titling_officer_roster_id = r.id)
                 OR EXISTS (
                   SELECT 1
                     FROM jsonb_array_elements(p.agents_json) AS a
                    WHERE a->>'role' = $3
                      AND a->>'id' = r.id::text
                 )
               )
         )",
    )
    .bind(project_id)
    .bind(user_id)
    .bind(role_slug)
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
            "Not assigned to this role on this project",
        ))
    }
}

fn row_to_project(row: sqlx::postgres::PgRow) -> ProjectResponse {
    ProjectResponse {
        id: row.try_get("id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(0),
        lead_broker_roster_id: row.try_get("lead_broker_roster_id").ok().flatten(),
        titling_officer_roster_id: row.try_get("titling_officer_roster_id").ok().flatten(),
        agent_commission_split_months: row.try_get("agent_commission_split_months").unwrap_or(15),
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

const PROJECT_COLUMNS: &str = "p.id, p.name, p.created_at, p.lead_broker_roster_id, p.titling_officer_roster_id, p.agent_commission_split_months, p.agents_json";

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

#[derive(Serialize)]
pub struct UplineContextResponse {
    pub project: ProjectResponse,
    pub rates: Vec<CommissionRateResponse>,
}

/// GET /me/upline/{role_slug}/projects/{project_id}/context
pub async fn get_project_context(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path((role_slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<UplineContextResponse>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_upline_role(&pool, &role_slug, &role).await?;
    assert_owns_project(&pool, user_id, project_id, &role_slug).await?;

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

    Ok(Json(UplineContextResponse {
        project: row_to_project(row),
        rates: rate_rows.into_iter().map(row_to_rate).collect(),
    }))
}

/// GET /me/upline/{role_slug}/projects/{project_id}/lots
pub async fn list_project_lots(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path((role_slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<LotResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_upline_role(&pool, &role_slug, &role).await?;
    assert_owns_project(&pool, user_id, project_id, &role_slug).await?;

    let rows = sqlx::query(&format!(
        "SELECT {LOT_COLUMNS} FROM public.lots
          WHERE project_id = $1
       ORDER BY block ASC, lot ASC",
    ))
    .bind(project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load lots")
    })?;

    Ok(Json(rows.into_iter().map(row_to_lot).collect()))
}

#[derive(Deserialize)]
pub struct UplineReserveLotInput {
    pub status: String,
    pub reserved_until: Option<i64>,
    #[serde(default)]
    pub reserve_agent_id: Option<String>,
    #[serde(default)]
    pub reserve_notes: Option<String>,
}

/// PATCH /me/upline/{role_slug}/lots/{id} — Available ↔ Reserved only (keeps pricing fields).
pub async fn patch_lot_reserve(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path((role_slug, id)): Path<(String, Uuid)>,
    Json(p): Json<UplineReserveLotInput>,
) -> Result<Json<LotResponse>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    let label = require_upline_role(&pool, &role_slug, &role).await?;

    if p.status != "Available" && p.status != "Reserved" {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "May only set Available or Reserved",
        ));
    }

    let existing = sqlx::query(&format!("SELECT {LOT_COLUMNS} FROM public.lots WHERE id = $1"))
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load lot")
        })?
        .ok_or((StatusCode::NOT_FOUND, "Lot not found"))?;

    let lot = row_to_lot(existing);
    assert_owns_project(&pool, user_id, lot.project_id, &role_slug).await?;
    let _ = &label; // role validated above; kept for symmetry with sibling handlers

    if lot.status != "Available" && lot.status != "Hold" && lot.status != "Reserved" {
        return Err((
            StatusCode::CONFLICT,
            "Lot cannot be reserved in its current status",
        ));
    }

    let reserved_until = resolve_reserved_until(&p.status, p.reserved_until)?;
    let (reserve_agent_id, reserve_notes) =
        resolve_reserve_meta(&p.status, p.reserve_agent_id, p.reserve_notes)?;
    let now = Utc::now().timestamp();
    let on_hold = p.status == "Reserved";

    let row = sqlx::query(&format!(
        "UPDATE public.lots
            SET status = $1, reserved_until = $2, reserve_agent_id = $3, reserve_notes = $4,
                on_hold = $5, updated_at = $6
          WHERE id = $7
      RETURNING {LOT_COLUMNS}",
    ))
    .bind(&p.status)
    .bind(reserved_until)
    .bind(&reserve_agent_id)
    .bind(&reserve_notes)
    .bind(on_hold)
    .bind(now)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update lot")
    })?;

    Ok(Json(row_to_lot(row)))
}

/// GET /me/upline/{role_slug}/projects/{project_id}/contracts
pub async fn list_project_contracts(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path((role_slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<ContractResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_upline_role(&pool, &role_slug, &role).await?;
    assert_owns_project(&pool, user_id, project_id, &role_slug).await?;

    let rows = sqlx::query(&format!(
        "SELECT {CONTRACT_COLUMNS_WITH_TOTALS}
           FROM public.contracts c
           LEFT JOIN public.users bu ON bu.id = c.buyer_user_id
           LEFT JOIN public.payments p ON p.contract_id = c.id
          WHERE c.project_id = $1
       GROUP BY c.id
       ORDER BY c.created_at ASC",
    ))
    .bind(project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load contracts")
    })?;

    Ok(Json(rows.into_iter().map(row_to_contract).collect()))
}

fn root_id_from_agents(agents: &Value, role_slug: &str) -> Option<String> {
    agents.as_array()?.iter().find_map(|a| {
        if a.get("role").and_then(|v| v.as_str()) == Some(role_slug) {
            a.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
        } else {
            None
        }
    })
}

fn collect_direct_and_downline_seller_ids(
    agents: &Value,
    role_slug: &str,
    fallback_root: Option<&str>,
) -> HashSet<String> {
    let root =
        root_id_from_agents(agents, role_slug).or_else(|| fallback_root.map(|s| s.to_string()));
    let Some(root) = root else {
        return HashSet::new();
    };

    let mut by_parent: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(arr) = agents.as_array() {
        for a in arr {
            let Some(id) = a.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            if let Some(parent) = a.get("parentId").and_then(|v| v.as_str()) {
                by_parent
                    .entry(parent.to_string())
                    .or_default()
                    .push(id.to_string());
            }
        }
    }

    let mut out = HashSet::new();
    out.insert(root.clone());
    let mut q = VecDeque::new();
    q.push_back(root);
    while let Some(id) = q.pop_front() {
        if let Some(children) = by_parent.get(&id) {
            for child in children {
                if out.insert(child.clone()) {
                    q.push_back(child.clone());
                }
            }
        }
    }
    out
}

/// Legacy per-role project columns only exist for lead-broker/titling-officer; any
/// other role resolves purely from agents_json.
async fn load_project_agents(
    pool: &PgPool,
    project_id: Uuid,
    role_slug: &str,
) -> Result<(Value, Option<Uuid>), E> {
    let column = match role_slug {
        "lead-broker" => "lead_broker_roster_id",
        "titling-officer" => "titling_officer_roster_id",
        _ => "NULL",
    };
    let row = sqlx::query(&format!(
        "SELECT agents_json, {column} AS legacy_root_id FROM public.projects WHERE id = $1",
    ))
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load project")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Project not found"))?;

    Ok((
        row.try_get("agents_json")
            .unwrap_or_else(|_| Value::Array(vec![])),
        row.try_get("legacy_root_id").ok().flatten(),
    ))
}

fn resolve_subject_id(agents: &Value, role_slug: &str, legacy_root_id: Option<Uuid>) -> Option<String> {
    root_id_from_agents(agents, role_slug).or_else(|| legacy_root_id.map(|id| id.to_string()))
}

#[derive(Serialize)]
pub struct UplineCashFlowPaymentResponse {
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

fn row_to_upline_cashflow_payment(row: sqlx::postgres::PgRow) -> UplineCashFlowPaymentResponse {
    UplineCashFlowPaymentResponse {
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

/// GET /me/upline/{role_slug}/projects/{project_id}/payments — direct-buyer + downline collections only.
pub async fn list_project_payments(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path((role_slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<UplineCashFlowPaymentResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_upline_role(&pool, &role_slug, &role).await?;
    assert_owns_project(&pool, user_id, project_id, &role_slug).await?;

    let (agents_json, legacy_root_id) = load_project_agents(&pool, project_id, &role_slug).await?;
    let seller_ids = collect_direct_and_downline_seller_ids(
        &agents_json,
        &role_slug,
        legacy_root_id.as_ref().map(|id| id.to_string()).as_deref(),
    );
    if seller_ids.is_empty() {
        return Ok(Json(vec![]));
    }
    let seller_list: Vec<String> = seller_ids.into_iter().collect();

    let rows = sqlx::query(
        "SELECT p.id, p.contract_id, p.amount, p.method, p.months_covered, p.paid_at,
                p.reference_no, p.bank_name, p.sender_name, p.receiver_name, p.mode_label,
                c.buyer_name, c.lot_block, c.lot_lot, c.term_years, c.term_months,
                c.selling_agent_id
           FROM public.payments p
           INNER JOIN public.contracts c ON c.id = p.contract_id
          WHERE c.project_id = $1
            AND c.selling_agent_id = ANY($2)
       ORDER BY p.paid_at DESC, p.created_at DESC",
    )
    .bind(project_id)
    .bind(&seller_list)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load payments")
    })?;

    Ok(Json(
        rows.into_iter().map(row_to_upline_cashflow_payment).collect(),
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

/// GET /me/upline/{role_slug}/projects/{project_id}/commission-status — subject only, read-only.
pub async fn list_commission_status(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path((role_slug, project_id)): Path<(String, Uuid)>,
    Query(query): Query<ListCommissionStatusQuery>,
) -> Result<Json<Vec<CommissionPeriodStatusResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    let label = require_upline_role(&pool, &role_slug, &role).await?;
    assert_owns_project(&pool, user_id, project_id, &role_slug).await?;

    let (agents_json, legacy_root_id) = load_project_agents(&pool, project_id, &role_slug).await?;
    let subject_id = resolve_subject_id(&agents_json, &role_slug, legacy_root_id).ok_or((
        StatusCode::UNPROCESSABLE_ENTITY,
        "Subject not found on project",
    ))?;
    let _ = &label; // role validated above; kept for symmetry with sibling handlers

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
