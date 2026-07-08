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

use crate::api::{company, contracts, lots, projects, users};

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
        .route(
            "/company/settings",
            get(company::get_settings).patch(company::update_settings),
        )
        .route(
            "/projects",
            get(projects::list_projects).post(projects::create_project),
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
            "/contracts/{id}",
            get(contracts::get_contract)
                .patch(contracts::update_contract)
                .delete(contracts::delete_contract),
        )
        .route(
            "/contracts/{id}/payments",
            post(contracts::record_payment),
        )
        .layer(DefaultBodyLimit::max(BODY_LIMIT_BYTES));

    Router::new().merge(auth_routes).merge(data_routes)
}
