use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::users::shared::E;

pub(super) fn require_employee(role: &str) -> Result<(), E> {
    if role == "Employee" {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Employee access required"))
    }
}

pub(super) async fn resolve_employee_id(pool: &PgPool, user_id: Uuid) -> Result<Uuid, E> {
    sqlx::query_scalar("SELECT id FROM public.salary_employees WHERE user_id = $1 LIMIT 1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
        })?
        .ok_or((StatusCode::FORBIDDEN, "No employee record linked to this user"))
}
