use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::shared::require_admin;
use crate::api::users::shared::E;

#[derive(Serialize, Clone)]
pub struct EmployeePositionTypeResponse {
    pub id: Uuid,
    pub label: String,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

const COLUMNS: &str = "id, label, sort_order, created_at, updated_at";

fn row_to_position_type(row: sqlx::postgres::PgRow) -> EmployeePositionTypeResponse {
    EmployeePositionTypeResponse {
        id: row.try_get("id").unwrap_or_default(),
        label: row.try_get("label").unwrap_or_default(),
        sort_order: row.try_get("sort_order").unwrap_or(0),
        created_at: row.try_get("created_at").unwrap_or(0),
        updated_at: row.try_get("updated_at").unwrap_or(0),
    }
}

fn map_position_type_db_error(e: sqlx::Error) -> E {
    if let Some(d) = e.as_database_error()
        && d.code().as_deref() == Some("23505")
    {
        return (StatusCode::CONFLICT, "That position already exists");
    }
    tracing::error!("DB: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save position")
}

/// GET /employee-position-types — admin only.
pub async fn list_employee_position_types(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<EmployeePositionTypeResponse>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(&format!(
        "SELECT {COLUMNS} FROM public.employee_position_types ORDER BY sort_order ASC, label ASC",
    ))
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load position types",
        )
    })?;

    Ok(Json(rows.into_iter().map(row_to_position_type).collect()))
}

#[derive(Deserialize)]
pub struct CreateEmployeePositionTypeInput {
    pub label: String,
}

/// POST /employee-position-types — admin only.
pub async fn create_employee_position_type(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(p): Json<CreateEmployeePositionTypeInput>,
) -> Result<Json<EmployeePositionTypeResponse>, E> {
    require_admin(&pool, &headers).await?;

    let label = p.label.trim().to_string();
    if label.is_empty() || label.len() > 100 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid label"));
    }

    let now = Utc::now().timestamp();

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save position")
    })?;

    let next_sort: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM public.employee_position_types",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save position")
    })?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO public.employee_position_types (label, sort_order, created_at, updated_at)
         VALUES ($1, $2, $3, $3)
      RETURNING id",
    )
    .bind(&label)
    .bind(next_sort)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_position_type_db_error)?;

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save position")
    })?;

    let row = sqlx::query(&format!(
        "SELECT {COLUMNS} FROM public.employee_position_types WHERE id = $1",
    ))
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load position")
    })?;

    Ok(Json(row_to_position_type(row)))
}

#[derive(Deserialize)]
pub struct UpdateEmployeePositionTypeInput {
    pub sort_order: i32,
}

/// PATCH /employee-position-types/{id} — admin only. Label is immutable after
/// creation: salary_employees.position stores the label text directly (not this
/// row's id), so renaming here would silently orphan any employee already using
/// the old label. Only `sort_order` (used for the up/down reorder buttons) can
/// change.
pub async fn update_employee_position_type(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(p): Json<UpdateEmployeePositionTypeInput>,
) -> Result<Json<EmployeePositionTypeResponse>, E> {
    require_admin(&pool, &headers).await?;

    let now = Utc::now().timestamp();

    let row = sqlx::query(&format!(
        "UPDATE public.employee_position_types
            SET sort_order = $1, updated_at = $2
          WHERE id = $3
      RETURNING {COLUMNS}",
    ))
    .bind(p.sort_order)
    .bind(now)
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save position")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Position type not found"))?;

    Ok(Json(row_to_position_type(row)))
}

/// DELETE /employee-position-types/{id} — admin only. Rejected while any salary
/// employee still has this label set as their position.
pub async fn delete_employee_position_type(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let label: String =
        sqlx::query_scalar("SELECT label FROM public.employee_position_types WHERE id = $1")
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
            })?
            .ok_or((StatusCode::NOT_FOUND, "Position type not found"))?;

    let in_use: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.salary_employees WHERE position = $1)")
            .bind(&label)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
            })?;

    if in_use {
        return Err((
            StatusCode::CONFLICT,
            "Cannot delete a position while an employee is assigned to it",
        ));
    }

    let result = sqlx::query("DELETE FROM public.employee_position_types WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete position",
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Position type not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}
