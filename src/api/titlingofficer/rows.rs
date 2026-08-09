use serde_json::Value;
use sqlx::Row;

use crate::api::admin::commission_rates::CommissionRateResponse;
use crate::api::admin::projects::ProjectResponse;

pub(super) fn row_to_project(row: sqlx::postgres::PgRow) -> ProjectResponse {
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

pub(super) fn row_to_rate(row: sqlx::postgres::PgRow) -> CommissionRateResponse {
    CommissionRateResponse {
        role: row.try_get("role").unwrap_or_default(),
        commission_rate: row.try_get("commission_rate").unwrap_or(0.0),
        updated_at: row.try_get("updated_at").unwrap_or(0),
    }
}

pub(super) const PROJECT_COLUMNS: &str = "p.id, p.name, p.created_at, p.lead_broker_roster_id, p.titling_officer_roster_id, p.agent_commission_split_months, p.agents_json";
