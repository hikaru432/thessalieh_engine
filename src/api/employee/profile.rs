use axum::{Extension, Json, http::HeaderMap};
use sqlx::PgPool;

use crate::api::admin::salary::{EMPLOYEE_SELECT, SalaryEmployeeResponse, row_to_employee};
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;
use axum::http::StatusCode;

use super::guards::{require_employee, resolve_employee_id};

/// GET /me/employee/profile
pub async fn get_my_profile(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<SalaryEmployeeResponse>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_employee(&role)?;
    let employee_id = resolve_employee_id(&pool, user_id).await?;

    let row = sqlx::query(&format!("{EMPLOYEE_SELECT} WHERE se.id = $1"))
        .bind(employee_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load profile")
        })?;

    Ok(Json(row_to_employee(row)))
}
