use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::shared::require_admin;
use crate::api::users::shared::E;

#[derive(Serialize)]
pub struct SubdivisionLayoutResponse {
    pub project_id: Uuid,
    pub version: i32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub blocks: Value,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct UpsertSubdivisionLayoutInput {
    pub project_id: Uuid,
    pub version: i32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub blocks: Value,
}

async fn ensure_project(pool: &PgPool, project_id: Uuid) -> Result<(), E> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM public.projects WHERE id = $1 AND company_id = 1)",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to verify project")
    })?;
    if !exists {
        return Err((StatusCode::NOT_FOUND, "Project not found"));
    }
    Ok(())
}

fn validate_layout(project_id: Uuid, input: &UpsertSubdivisionLayoutInput) -> Result<Value, E> {
    if input.project_id != project_id {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "project_id must match the URL project",
        ));
    }
    if input.version != 1 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "version must be 1",
        ));
    }
    if !input.blocks.is_array() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "blocks must be a JSON array",
        ));
    }
    if !input.width.is_finite()
        || !input.height.is_finite()
        || input.width <= 0.0
        || input.height <= 0.0
    {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "width and height must be positive finite numbers",
        ));
    }

    let stored = serde_json::json!({
        "project_id": input.project_id,
        "version": input.version,
        "x": input.x,
        "y": input.y,
        "width": input.width,
        "height": input.height,
        "rotation": input.rotation,
        "blocks": input.blocks,
    });
    Ok(stored)
}

fn row_to_response(project_id: Uuid, layout: Value, updated_at: i64) -> Result<SubdivisionLayoutResponse, E> {
    let obj = layout.as_object().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Invalid stored subdivision layout",
    ))?;
    Ok(SubdivisionLayoutResponse {
        project_id,
        version: obj
            .get("version")
            .and_then(Value::as_i64)
            .unwrap_or(1) as i32,
        x: obj.get("x").and_then(Value::as_f64).unwrap_or(0.0),
        y: obj.get("y").and_then(Value::as_f64).unwrap_or(0.0),
        width: obj.get("width").and_then(Value::as_f64).unwrap_or(0.0),
        height: obj.get("height").and_then(Value::as_f64).unwrap_or(0.0),
        rotation: obj.get("rotation").and_then(Value::as_f64).unwrap_or(0.0),
        blocks: obj
            .get("blocks")
            .cloned()
            .unwrap_or_else(|| Value::Array(vec![])),
        updated_at,
    })
}

/// GET /projects/{project_id}/subdivision-layout — returns saved map geometry or 404.
pub async fn get_subdivision_layout(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<SubdivisionLayoutResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let row = sqlx::query(
        "SELECT subdivision_layout, updated_at
           FROM public.projects
          WHERE id = $1 AND company_id = 1",
    )
    .bind(project_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load subdivision layout",
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Project not found"))?;

    let layout: Option<Value> = row.try_get("subdivision_layout").ok().flatten();
    let Some(layout) = layout else {
        return Err((StatusCode::NOT_FOUND, "Subdivision layout not found"));
    };

    let updated_at: i64 = row.try_get("updated_at").unwrap_or(0);
    Ok(Json(row_to_response(project_id, layout, updated_at)?))
}

/// PUT /projects/{project_id}/subdivision-layout — upserts map geometry JSON on the project row.
pub async fn put_subdivision_layout(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(input): Json<UpsertSubdivisionLayoutInput>,
) -> Result<Json<SubdivisionLayoutResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let stored = validate_layout(project_id, &input)?;
    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "UPDATE public.projects
            SET subdivision_layout = $1,
                updated_at = $2
          WHERE id = $3 AND company_id = 1
      RETURNING subdivision_layout, updated_at",
    )
    .bind(stored)
    .bind(now)
    .bind(project_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save subdivision layout",
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Project not found"))?;

    let layout: Value = row
        .try_get("subdivision_layout")
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read saved subdivision layout",
            )
        })?;
    let updated_at: i64 = row.try_get("updated_at").unwrap_or(now);
    Ok(Json(row_to_response(project_id, layout, updated_at)?))
}
