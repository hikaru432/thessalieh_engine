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

pub(super) async fn assert_lb_owns_project(pool: &PgPool, user_id: Uuid, project_id: Uuid) -> Result<(), E> {
    let owned: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1
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
               )
         )",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?;

    if owned {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Not assigned as Lead Broker on this project"))
    }
}
