use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::shared::require_admin;
use super::users::shared::E;

const LOT_TYPES: [&str; 4] = ["Inner", "Commercial", "Corner", "Commercial / Corner"];
const STATUSES: [&str; 4] = ["Available", "Hold", "Reserved", "Sold"];

#[derive(Serialize)]
pub struct LotResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub block: String,
    pub lot: String,
    pub lot_type: String,
    pub area: f64,
    pub rate: f64,
    pub contract_price: f64,
    pub owner_buyer: Option<String>,
    pub on_hold: bool,
    pub status: String,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct CreateLotInput {
    pub block: String,
    pub lot: String,
    pub lot_type: String,
    pub area: f64,
    pub rate: f64,
}

#[derive(Deserialize)]
pub struct UpdateLotInput {
    pub block: String,
    pub lot: String,
    pub lot_type: String,
    pub area: f64,
    pub rate: f64,
    pub owner_buyer: Option<String>,
    pub on_hold: bool,
    pub status: String,
}

fn row_to_lot(row: sqlx::postgres::PgRow) -> LotResponse {
    LotResponse {
        id: row.try_get("id").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        block: row.try_get("block").unwrap_or_default(),
        lot: row.try_get("lot").unwrap_or_default(),
        lot_type: row.try_get("lot_type").unwrap_or_default(),
        area: row.try_get("area").unwrap_or(0.0),
        rate: row.try_get("rate").unwrap_or(0.0),
        contract_price: row.try_get("contract_price").unwrap_or(0.0),
        owner_buyer: row.try_get("owner_buyer").ok().flatten(),
        on_hold: row.try_get("on_hold").unwrap_or(false),
        status: row.try_get("status").unwrap_or_default(),
        updated_at: row.try_get("updated_at").unwrap_or(0),
    }
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .is_some_and(|d| d.code().as_deref() == Some("23505"))
}

fn validate_block_lot(block: &str, lot: &str) -> Result<(), E> {
    if block.trim().is_empty() || block.len() > 20 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid block"));
    }
    if lot.trim().is_empty() || lot.len() > 20 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid lot"));
    }
    Ok(())
}

fn validate_lot_type(lot_type: &str) -> Result<(), E> {
    if !LOT_TYPES.contains(&lot_type) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid lot type"));
    }
    Ok(())
}

fn validate_status(status: &str) -> Result<(), E> {
    if !STATUSES.contains(&status) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid status"));
    }
    Ok(())
}

fn validate_area_rate(area: f64, rate: f64) -> Result<(), E> {
    if !area.is_finite() || area <= 0.0 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid area"));
    }
    if !rate.is_finite() || rate <= 0.0 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid rate"));
    }
    Ok(())
}

async fn project_exists(pool: &PgPool, project_id: Uuid) -> Result<bool, E> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.projects WHERE id = $1)")
        .bind(project_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
        })
}

const LOT_COLUMNS: &str = "id, project_id, block, lot, lot_type, area, rate, contract_price,
                            owner_buyer, on_hold, status, updated_at";

pub async fn list_lots(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<LotResponse>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(&format!(
        "SELECT {LOT_COLUMNS} FROM public.lots
          WHERE project_id = $1
       ORDER BY block ASC, lot ASC",
    ))
    .bind(project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load lots")
    })?;

    Ok(Json(rows.into_iter().map(row_to_lot).collect()))
}

pub async fn create_lot(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(p): Json<CreateLotInput>,
) -> Result<Json<LotResponse>, E> {
    require_admin(&pool, &headers).await?;

    validate_block_lot(&p.block, &p.lot)?;
    validate_lot_type(&p.lot_type)?;
    validate_area_rate(p.area, p.rate)?;

    if !project_exists(&pool, project_id).await? {
        return Err((StatusCode::NOT_FOUND, "Project not found"));
    }

    let now = Utc::now().timestamp();
    let contract_price = p.area * p.rate;

    let row = sqlx::query(&format!(
        "INSERT INTO public.lots
             (project_id, block, lot, lot_type, area, rate, contract_price, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
      RETURNING {LOT_COLUMNS}",
    ))
    .bind(project_id)
    .bind(p.block.trim())
    .bind(p.lot.trim())
    .bind(&p.lot_type)
    .bind(p.area)
    .bind(p.rate)
    .bind(contract_price)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if is_unique_violation(&e) {
            (
                StatusCode::CONFLICT,
                "That block/lot already exists for this project",
            )
        } else {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create lot")
        }
    })?;

    Ok(Json(row_to_lot(row)))
}

pub async fn update_lot(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(p): Json<UpdateLotInput>,
) -> Result<Json<LotResponse>, E> {
    require_admin(&pool, &headers).await?;

    validate_block_lot(&p.block, &p.lot)?;
    validate_lot_type(&p.lot_type)?;
    validate_area_rate(p.area, p.rate)?;
    validate_status(&p.status)?;

    let owner_buyer = p
        .owner_buyer
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    if owner_buyer.as_ref().is_some_and(|v| v.len() > 255) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Owner/buyer name too long",
        ));
    }

    let now = Utc::now().timestamp();
    let contract_price = p.area * p.rate;

    let row = sqlx::query(&format!(
        "UPDATE public.lots
            SET block = $1, lot = $2, lot_type = $3, area = $4, rate = $5,
                contract_price = $6, owner_buyer = $7, on_hold = $8, status = $9,
                updated_at = $10
          WHERE id = $11
      RETURNING {LOT_COLUMNS}",
    ))
    .bind(p.block.trim())
    .bind(p.lot.trim())
    .bind(&p.lot_type)
    .bind(p.area)
    .bind(p.rate)
    .bind(contract_price)
    .bind(&owner_buyer)
    .bind(p.on_hold)
    .bind(&p.status)
    .bind(now)
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        if is_unique_violation(&e) {
            (
                StatusCode::CONFLICT,
                "That block/lot already exists for this project",
            )
        } else {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update lot")
        }
    })?
    .ok_or((StatusCode::NOT_FOUND, "Lot not found"))?;

    Ok(Json(row_to_lot(row)))
}

pub async fn delete_lot(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let result = sqlx::query("DELETE FROM public.lots WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete lot")
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Lot not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}
