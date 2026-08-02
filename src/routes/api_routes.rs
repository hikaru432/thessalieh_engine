use axum::Json;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, patch, post},
};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::api::admin::{
    commission_rates, commission_row_meta, commission_split_schedule, commission_status, company,
    contracts, lots, projects, roster, upline_role_types,
};
use crate::api::buyer;
use crate::api::leadbroker;
use crate::api::titlingofficer;
use crate::api::upline_portal;
use crate::api::agent;
use crate::api::users;

const BODY_LIMIT_BYTES: usize = 10 * 1024 * 1024; // 10 MB

async fn keepalive(Extension(pool): Extension<PgPool>) -> (StatusCode, Json<Value>) {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "status": "ok",    "db": "reachable" })),
        ),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "error", "db": e.to_string() })),
        ),
    }
}

pub fn routes() -> Router {
    // Auth & session routes — capped at 10 MB
    let auth_routes = Router::new()
        .route("/keepalive", get(keepalive))
        .route("/auth/register", post(users::register))
        .route("/auth/login", post(users::login))
        .route("/auth/session", get(users::session_handler))
        .route("/auth/logout", post(users::logout))
        .route("/auth/profile", patch(users::update_profile))
        .route(
            "/auth/password-reset/request",
            post(users::password_reset_request),
        )
        .route(
            "/auth/password-reset/confirm",
            post(users::password_reset_confirm),
        )
        .layer(DefaultBodyLimit::max(BODY_LIMIT_BYTES));

    // Company, project, lot, and contract data routes — capped at 10 MB
    let data_routes = Router::new()
        .route("/me/contracts", get(buyer::contracts::list_my_contracts))
        .route("/me/lb/projects", get(leadbroker::list_my_projects))
        .route(
            "/me/lb/projects/{project_id}/context",
            get(leadbroker::get_project_context),
        )
        .route(
            "/me/lb/projects/{project_id}/lots",
            get(leadbroker::list_project_lots),
        )
        .route(
            "/me/lb/projects/{project_id}/contracts",
            get(leadbroker::list_project_contracts),
        )
        .route(
            "/me/lb/projects/{project_id}/payments",
            get(leadbroker::list_project_payments),
        )
        .route(
            "/me/lb/projects/{project_id}/commission-status",
            get(leadbroker::list_commission_status),
        )
        .route("/me/lb/lots/{id}", patch(leadbroker::patch_lot_reserve))
        .route("/me/to/projects", get(titlingofficer::list_my_projects))
        .route(
            "/me/to/projects/{project_id}/context",
            get(titlingofficer::get_project_context),
        )
        .route(
            "/me/to/projects/{project_id}/lots",
            get(titlingofficer::list_project_lots),
        )
        .route(
            "/me/to/projects/{project_id}/contracts",
            get(titlingofficer::list_project_contracts),
        )
        .route(
            "/me/to/projects/{project_id}/payments",
            get(titlingofficer::list_project_payments),
        )
        .route(
            "/me/to/projects/{project_id}/commission-status",
            get(titlingofficer::list_commission_status),
        )
        .route("/me/to/lots/{id}", patch(titlingofficer::patch_lot_reserve))
        .route(
            "/me/upline/{role_slug}/projects",
            get(upline_portal::list_my_projects),
        )
        .route(
            "/me/upline/{role_slug}/projects/{project_id}/context",
            get(upline_portal::get_project_context),
        )
        .route(
            "/me/upline/{role_slug}/projects/{project_id}/lots",
            get(upline_portal::list_project_lots),
        )
        .route(
            "/me/upline/{role_slug}/projects/{project_id}/contracts",
            get(upline_portal::list_project_contracts),
        )
        .route(
            "/me/upline/{role_slug}/projects/{project_id}/payments",
            get(upline_portal::list_project_payments),
        )
        .route(
            "/me/upline/{role_slug}/projects/{project_id}/commission-status",
            get(upline_portal::list_commission_status),
        )
        .route(
            "/me/upline/{role_slug}/lots/{id}",
            patch(upline_portal::patch_lot_reserve),
        )
        .route("/me/agent/projects", get(agent::list_my_projects))
        .route(
            "/me/agent/projects/{project_id}/context",
            get(agent::get_project_context),
        )
        .route(
            "/me/agent/projects/{project_id}/contracts",
            get(agent::list_project_contracts),
        )
        .route(
            "/me/agent/projects/{project_id}/payments",
            get(agent::list_project_payments),
        )
        .route(
            "/me/agent/projects/{project_id}/commission-status",
            get(agent::list_commission_status),
        )
        .route(
            "/company/settings",
            get(company::get_settings).patch(company::update_settings),
        )
        .route(
            "/company/agent-commission-split-months",
            patch(company::update_agent_commission_split_months),
        )
        .route("/users", get(users::list_users).post(users::create_user))
        .route(
            "/roster",
            get(roster::list_roster).post(roster::create_roster_entry),
        )
        .route(
            "/roster/{id}",
            patch(roster::update_roster_entry).delete(roster::delete_roster_entry),
        )
        .route(
            "/commission-rates",
            get(commission_rates::list_commission_rates),
        )
        .route(
            "/commission-rates/{role}",
            patch(commission_rates::update_commission_rate),
        )
        .route(
            "/upline-role-types",
            get(upline_role_types::list_upline_role_types)
                .post(upline_role_types::create_upline_role_type),
        )
        .route(
            "/upline-role-types/{slug}",
            patch(upline_role_types::update_upline_role_type)
                .delete(upline_role_types::delete_upline_role_type),
        )
        .route(
            "/commission-split-schedule",
            get(commission_split_schedule::list_commission_split_schedule)
                .post(commission_split_schedule::create_commission_split_schedule),
        )
        .route(
            "/commission-split-schedule/{id}",
            axum::routing::delete(commission_split_schedule::delete_commission_split_schedule),
        )
        .route(
            "/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        .route(
            "/projects/{project_id}/agents",
            patch(projects::update_project_agents),
        )
        .route(
            "/projects/{project_id}/commission-status",
            get(commission_status::list_commission_status)
                .put(commission_status::upsert_commission_status),
        )
        .route(
            "/projects/{project_id}/commission-row-meta",
            get(commission_row_meta::list_commission_row_meta)
                .put(commission_row_meta::upsert_commission_row_meta),
        )
        .route(
            "/projects/{project_id}/lots",
            get(lots::list_lots).post(lots::create_lot),
        )
        .route(
            "/lots/{id}",
            patch(lots::update_lot).delete(lots::delete_lot),
        )
        .route(
            "/projects/{project_id}/contracts",
            get(contracts::list_contracts).post(contracts::create_contract),
        )
        .route(
            "/projects/{project_id}/payments",
            get(contracts::list_project_payments),
        )
        .route(
            "/contracts/{id}",
            get(contracts::get_contract)
                .patch(contracts::update_contract)
                .delete(contracts::delete_contract),
        )
        .route(
            "/contracts/{id}/payments",
            post(contracts::record_payment),
        )
        .route(
            "/payments/{id}",
            patch(contracts::update_payment),
        )
        .route(
            "/contracts/{id}/penalty-waiver",
            patch(contracts::update_penalty_waiver),
        )
        .layer(DefaultBodyLimit::max(BODY_LIMIT_BYTES));

    Router::new().merge(auth_routes).merge(data_routes)
}
