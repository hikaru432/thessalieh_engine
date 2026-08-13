use axum::{Extension, Json, http::HeaderMap};
use sqlx::PgPool;

use crate::api::admin::salary::{SalaryPlanResponse, row_to_plan};
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;
use axum::http::StatusCode;

use super::guards::{require_employee, resolve_employee_id};

/// GET /me/employee/plans
pub async fn list_my_plans(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<SalaryPlanResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_employee(&role)?;
    let employee_id = resolve_employee_id(&pool, user_id).await?;

    let rows = sqlx::query(
        "SELECT id, employee_id, kind, start_date, end_date, training_fee, monthly_amount,
                schedule_type, created_at
           FROM public.salary_plans
          WHERE employee_id = $1
       ORDER BY start_date ASC",
    )
    .bind(employee_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load pay plans")
    })?;

    Ok(Json(rows.into_iter().map(row_to_plan).collect()))
}
