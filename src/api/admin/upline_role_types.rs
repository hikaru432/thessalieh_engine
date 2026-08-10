use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::api::shared::{require_admin, require_session_user};
use crate::api::users::shared::E;

#[derive(Serialize, Clone)]
pub struct UplineRoleTypeResponse {
    pub slug: String,
    pub label: String,
    pub base_commission_percent: f64,
    pub portal_path: String,
    pub sort_order: i32,
    /// Whether this role earns its base % on every sale project-wide (a "baseline"
    /// cut, even outside their own team) or only from their own tree's sales.
    pub has_baseline: bool,
    /// How much of the shared Agent pool % counts as "Direct buyer" income when this
    /// role closes a sale personally (no downline agent involved) — the rest folds
    /// into their Baseline, same as the override they'd keep from handing a sale to a
    /// downline agent. NULL = not configured, so the app keeps the whole pool as
    /// Direct buyer (the historical, unconfigured behavior).
    pub direct_sale_pool_percent: Option<f64>,
    pub created_at: i64,
    pub updated_at: i64,
}

const ROLE_TYPE_COLUMNS: &str = "slug, label, base_commission_percent, portal_path, sort_order, \
                                  has_baseline, direct_sale_pool_percent, created_at, updated_at";

const RESERVED_PORTAL_PATHS: [&str; 9] = [
    "admin",
    "buyer",
    "agent",
    "login",
    "forgot-password",
    "about-us",
    "contact-us",
    "properties",
    "profile",
];

fn row_to_role_type(row: sqlx::postgres::PgRow) -> UplineRoleTypeResponse {
    UplineRoleTypeResponse {
        slug: row.try_get("slug").unwrap_or_default(),
        label: row.try_get("label").unwrap_or_default(),
        base_commission_percent: row.try_get("base_commission_percent").unwrap_or(0.0),
        portal_path: row.try_get("portal_path").unwrap_or_default(),
        sort_order: row.try_get("sort_order").unwrap_or(0),
        has_baseline: row.try_get("has_baseline").unwrap_or(true),
        direct_sale_pool_percent: row.try_get("direct_sale_pool_percent").ok().flatten(),
        created_at: row.try_get("created_at").unwrap_or(0),
        updated_at: row.try_get("updated_at").unwrap_or(0),
    }
}

fn slugify(label: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in label.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn validate_portal_path(portal_path: &str) -> Result<(), E> {
    let valid = !portal_path.is_empty()
        && portal_path.len() <= 40
        && portal_path
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Portal path must be lowercase letters/digits only",
        ))
    }
}

fn map_role_type_db_error(e: sqlx::Error) -> E {
    if let Some(d) = e.as_database_error()
        && d.code().as_deref() == Some("23505")
    {
        return (
            StatusCode::CONFLICT,
            "That label or portal path already exists",
        );
    }
    tracing::error!("DB: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save role type")
}

/// GET /upline-role-types — any authenticated session (login routing needs this
/// before we know whether the caller is an Admin).
pub async fn list_upline_role_types(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<UplineRoleTypeResponse>>, E> {
    require_session_user(&pool, &headers).await?;

    let rows = sqlx::query(&format!(
        "SELECT {ROLE_TYPE_COLUMNS} FROM public.upline_role_types ORDER BY sort_order ASC, label ASC",
    ))
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load upline role types",
        )
    })?;

    Ok(Json(rows.into_iter().map(row_to_role_type).collect()))
}

fn validate_direct_sale_pool_percent(value: Option<f64>) -> Result<(), E> {
    match value {
        Some(v) if !v.is_finite() || v < 0.0 || v > 100.0 => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Direct-sale pool percent must be between 0 and 100",
        )),
        _ => Ok(()),
    }
}

#[derive(Deserialize)]
pub struct CreateUplineRoleTypeInput {
    pub label: String,
    pub base_commission_percent: f64,
    pub portal_path: String,
    pub has_baseline: bool,
    #[serde(default)]
    pub direct_sale_pool_percent: Option<f64>,
}

/// POST /upline-role-types — admin only.
pub async fn create_upline_role_type(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(p): Json<CreateUplineRoleTypeInput>,
) -> Result<Json<UplineRoleTypeResponse>, E> {
    require_admin(&pool, &headers).await?;

    let label = p.label.trim().to_string();
    if label.is_empty() || label.len() > 100 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid label"));
    }
    if label == "Agent" || label == "User" || label == "Admin" {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "That label is reserved for a fixed system role",
        ));
    }
    if !p.base_commission_percent.is_finite()
        || p.base_commission_percent < 0.0
        || p.base_commission_percent > 100.0
    {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Base commission percent must be between 0 and 100",
        ));
    }
    validate_direct_sale_pool_percent(p.direct_sale_pool_percent)?;
    let portal_path = p.portal_path.trim().to_lowercase();
    validate_portal_path(&portal_path)?;
    if RESERVED_PORTAL_PATHS.contains(&portal_path.as_str()) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "That portal path is reserved",
        ));
    }

    let slug = slugify(&label);
    if slug.is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid label"));
    }

    let now = Utc::now().timestamp();

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save role type")
    })?;

    let next_sort: i32 =
        sqlx::query_scalar("SELECT COALESCE(MAX(sort_order), -1) + 1 FROM public.upline_role_types")
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save role type")
            })?;

    sqlx::query(
        "INSERT INTO public.upline_role_types
             (slug, label, base_commission_percent, portal_path, sort_order, has_baseline,
              direct_sale_pool_percent, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)",
    )
    .bind(&slug)
    .bind(&label)
    .bind(p.base_commission_percent)
    .bind(&portal_path)
    .bind(next_sort)
    .bind(p.has_baseline)
    .bind(p.direct_sale_pool_percent)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(map_role_type_db_error)?;

    sqlx::query(
        "INSERT INTO public.commission_rates (role, commission_rate, updated_at)
         VALUES ($1, $2, $3)
         ON CONFLICT (role) DO NOTHING",
    )
    .bind(&label)
    .bind(p.base_commission_percent)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to seed commission rate",
        )
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save role type")
    })?;

    let row = sqlx::query(&format!(
        "SELECT {ROLE_TYPE_COLUMNS} FROM public.upline_role_types WHERE slug = $1",
    ))
    .bind(&slug)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load role type")
    })?;

    Ok(Json(row_to_role_type(row)))
}

#[derive(Deserialize)]
pub struct UpdateUplineRoleTypeInput {
    pub base_commission_percent: f64,
    pub portal_path: String,
    pub sort_order: i32,
    pub has_baseline: bool,
    #[serde(default)]
    pub direct_sale_pool_percent: Option<f64>,
}

/// PATCH /upline-role-types/{slug} — admin only. Label/slug are immutable after
/// creation; renaming would require cascading through roster.role, commission_rates.role,
/// and users.role, which isn't worth the complexity for a role name change.
pub async fn update_upline_role_type(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Json(p): Json<UpdateUplineRoleTypeInput>,
) -> Result<Json<UplineRoleTypeResponse>, E> {
    require_admin(&pool, &headers).await?;

    if !p.base_commission_percent.is_finite()
        || p.base_commission_percent < 0.0
        || p.base_commission_percent > 100.0
    {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Base commission percent must be between 0 and 100",
        ));
    }
    validate_direct_sale_pool_percent(p.direct_sale_pool_percent)?;
    let portal_path = p.portal_path.trim().to_lowercase();
    validate_portal_path(&portal_path)?;
    if RESERVED_PORTAL_PATHS.contains(&portal_path.as_str()) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "That portal path is reserved",
        ));
    }

    let now = Utc::now().timestamp();

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save role type")
    })?;

    let label: String = sqlx::query_scalar(
        "UPDATE public.upline_role_types
            SET base_commission_percent = $1, portal_path = $2, sort_order = $3,
                has_baseline = $4, direct_sale_pool_percent = $5, updated_at = $6
          WHERE slug = $7
      RETURNING label",
    )
    .bind(p.base_commission_percent)
    .bind(&portal_path)
    .bind(p.sort_order)
    .bind(p.has_baseline)
    .bind(p.direct_sale_pool_percent)
    .bind(now)
    .bind(&slug)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_role_type_db_error)?
    .ok_or((StatusCode::NOT_FOUND, "Role type not found"))?;

    sqlx::query(
        "UPDATE public.commission_rates SET commission_rate = $1, updated_at = $2 WHERE role = $3",
    )
    .bind(p.base_commission_percent)
    .bind(now)
    .bind(&label)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to sync commission rate",
        )
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save role type")
    })?;

    let row = sqlx::query(&format!(
        "SELECT {ROLE_TYPE_COLUMNS} FROM public.upline_role_types WHERE slug = $1",
    ))
    .bind(&slug)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load role type")
    })?;

    Ok(Json(row_to_role_type(row)))
}

/// DELETE /upline-role-types/{slug} — admin only. Rejected while any roster entry
/// (active or inactive) is still assigned to this role.
pub async fn delete_upline_role_type(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let label: String =
        sqlx::query_scalar("SELECT label FROM public.upline_role_types WHERE slug = $1")
            .bind(&slug)
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
            })?
            .ok_or((StatusCode::NOT_FOUND, "Role type not found"))?;

    let in_use: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.roster WHERE role = $1)")
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
            "Cannot delete a role while roster members are assigned to it",
        ));
    }

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete role type",
        )
    })?;

    let result = sqlx::query("DELETE FROM public.upline_role_types WHERE slug = $1")
        .bind(&slug)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete role type",
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Role type not found"));
    }

    sqlx::query("DELETE FROM public.commission_rates WHERE role = $1")
        .bind(&label)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete role type",
            )
        })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete role type",
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}
