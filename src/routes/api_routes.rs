use axum::Json;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, patch, post, put},
};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::api::{company, lots, projects, users};

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
    // All other routes — auth, session, enquiry reads/updates — capped at 5 MB
    let core_routes = Router::new()
        .route("/keepalive", get(keepalive))
        .route("/auth/register", post(users::register))
        .route("/auth/verify", post(users::verify))
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
        );

    Router::new()
        .merge(core_routes)
}
