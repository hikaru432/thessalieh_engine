use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::users::shared::E;

pub(super) fn require_lead_broker(role: &str) -> Result<(), E> {
    if role == "Lead Broker" {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Lead Broker access required"))
    }
}

/// Confirms the caller is assigned as Lead Broker on this project AND returns their
/// own roster id — the only correct "subject" for anything this caller fetches under
/// `/me/lb/...`. Never resolve the subject by re-scanning agents_json for "the"
/// lead-broker entry elsewhere: a project can (transiently, e.g. mid-reassignment)
/// carry more than one agents_json entry with role "lead-broker", and picking the
/// first one instead of the caller's own id would leak another Lead Broker's
/// commission data to whoever happens to be listed first.
pub(super) async fn assert_lb_owns_project(pool: &PgPool, user_id: Uuid, project_id: Uuid) -> Result<Uuid, E> {
    let roster_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT r.id
           FROM public.projects p
           JOIN public.roster r ON r.user_id = $2
          WHERE p.id = $1
            AND (
              p.lead_broker_roster_id = r.id
              OR EXISTS (
                SELECT 1
                  FROM jsonb_array_elements(p.agents_json) AS a
                 WHERE a->>'role' = 'lead-broker'
                   AND a->>'id' = r.id::text
              )
            )",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?;

    roster_id.ok_or((StatusCode::FORBIDDEN, "Not assigned as Lead Broker on this project"))
}
