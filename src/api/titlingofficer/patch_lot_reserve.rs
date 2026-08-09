use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::admin::lots::{
    LOT_COLUMNS, LotResponse, resolve_reserve_meta, resolve_reserved_until, row_to_lot,
};
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::guards::{assert_to_owns_project, require_titling_officer};

#[derive(Deserialize)]
pub struct ToReserveLotInput {
    pub status: String,
    pub reserved_until: Option<i64>,
    #[serde(default)]
    pub reserve_agent_id: Option<String>,
    #[serde(default)]
    pub reserve_notes: Option<String>,
}

/// PATCH /me/to/lots/{id} — Available ↔ Reserved only (keeps pricing fields).
pub async fn patch_lot_reserve(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(p): Json<ToReserveLotInput>,
) -> Result<Json<LotResponse>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_titling_officer(&role)?;

    if p.status != "Available" && p.status != "Reserved" {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Titling Officer may only set Available or Reserved",
        ));
    }

    let existing = sqlx::query(&format!(
        "SELECT {LOT_COLUMNS} FROM public.lots WHERE id = $1",
    ))
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load lot")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Lot not found"))?;

    let lot = row_to_lot(existing);
    assert_to_owns_project(&pool, user_id, lot.project_id).await?;

    if lot.status != "Available" && lot.status != "Hold" && lot.status != "Reserved" {
        return Err((
            StatusCode::CONFLICT,
            "Lot cannot be reserved in its current status",
        ));
    }

    let reserved_until = resolve_reserved_until(&p.status, p.reserved_until)?;
    let (reserve_agent_id, reserve_notes) =
        resolve_reserve_meta(&p.status, p.reserve_agent_id, p.reserve_notes)?;
    let now = Utc::now().timestamp();
    let on_hold = p.status == "Reserved";

    let row = sqlx::query(&format!(
        "UPDATE public.lots
            SET status = $1, reserved_until = $2, reserve_agent_id = $3, reserve_notes = $4,
                on_hold = $5, updated_at = $6
          WHERE id = $7
      RETURNING {LOT_COLUMNS}",
    ))
    .bind(&p.status)
    .bind(reserved_until)
    .bind(&reserve_agent_id)
    .bind(&reserve_notes)
    .bind(on_hold)
    .bind(now)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update lot")
    })?;

    Ok(Json(row_to_lot(row)))
}
