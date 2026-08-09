use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::users::shared::E;

/// Resolves `role_slug` to its label and checks the session's role matches it.
/// Returns the label (== users.role / roster.role value for this role).
pub(super) async fn require_upline_role(pool: &PgPool, role_slug: &str, session_role: &str) -> Result<String, E> {
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
pub(super) async fn assert_owns_project(
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
