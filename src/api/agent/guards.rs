use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::users::shared::E;

pub(super) fn require_agent(role: &str) -> Result<(), E> {
    if role == "Agent" {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Agent access required"))
    }
}

pub(super) async fn resolve_agent_roster_id(pool: &PgPool, user_id: Uuid) -> Result<Uuid, E> {
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

pub(super) async fn assert_agent_on_project(
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
