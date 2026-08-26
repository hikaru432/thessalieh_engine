use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::admin::upline_role_types::validate_direct_sale_pool_percent;
use crate::api::shared::require_admin;
use crate::api::users::shared::E;

/// A per-project TCP allocation line item. Exactly one row per project has
/// `is_agent_pool = true` — that row's percent is the bucket that
/// `project_upline_role_rates` further splits among the configured upline roles;
/// the rest is whatever a project's downline agents share via their own
/// `agents_json` `sharePercent`. Every other row is a free-form category an admin
/// can add/rename/remove (Legal Counsel, Land Owner, DCF, ...).
#[derive(Serialize, Clone)]
pub struct RateCategoryResponse {
    pub id: Uuid,
    pub label: String,
    pub percent: f64,
    pub is_agent_pool: bool,
    pub sort_order: i32,
    pub updated_at: i64,
}

#[derive(Serialize, Clone)]
pub struct UplineRoleRateResponse {
    pub slug: String,
    pub label: String,
    pub percent: f64,
    pub has_baseline: bool,
    pub direct_sale_pool_percent: Option<f64>,
}

#[derive(Serialize)]
pub struct ProjectRateConfigResponse {
    pub categories: Vec<RateCategoryResponse>,
    pub upline_roles: Vec<UplineRoleRateResponse>,
    /// Derived, never stored: the agent-pool category's percent minus the sum of
    /// every upline role's percent above. This is the shared pool individual
    /// downline agents draw their `sharePercent` from.
    pub agent_pool_percent: f64,
}

fn row_to_category(row: sqlx::postgres::PgRow) -> RateCategoryResponse {
    RateCategoryResponse {
        id: row.try_get("id").unwrap_or_default(),
        label: row.try_get("label").unwrap_or_default(),
        percent: row.try_get("percent").unwrap_or(0.0),
        is_agent_pool: row.try_get("is_agent_pool").unwrap_or(false),
        sort_order: row.try_get("sort_order").unwrap_or(0),
        updated_at: row.try_get("updated_at").unwrap_or(0),
    }
}

pub async fn ensure_project(pool: &PgPool, project_id: Uuid) -> Result<(), E> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM public.projects WHERE id = $1 AND company_id = 1)",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to verify project",
        )
    })?;
    if !exists {
        return Err((StatusCode::NOT_FOUND, "Project not found"));
    }
    Ok(())
}

/// Shared by the admin Rate Config page and every portal's `get_project_context` —
/// each caller passes its own already-authorized `project_id`, so this project's
/// numbers are always what comes back, never a global fallback.
pub async fn fetch_project_rate_config(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<ProjectRateConfigResponse, E> {
    let category_rows = sqlx::query(
        "SELECT id, label, percent, is_agent_pool, sort_order, updated_at
           FROM public.project_rate_categories
          WHERE project_id = $1
       ORDER BY sort_order ASC, label ASC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load rate categories",
        )
    })?;
    let categories: Vec<RateCategoryResponse> =
        category_rows.into_iter().map(row_to_category).collect();

    let upline_rows = sqlx::query(
        "SELECT u.slug, u.label,
                COALESCE(r.percent, u.base_commission_percent) AS percent,
                COALESCE(r.has_baseline, u.has_baseline) AS has_baseline,
                COALESCE(r.direct_sale_pool_percent, u.direct_sale_pool_percent) AS direct_sale_pool_percent
           FROM public.upline_role_types u
      LEFT JOIN public.project_upline_role_rates r
             ON r.project_id = $1 AND r.upline_role_type_slug = u.slug
       ORDER BY u.sort_order ASC, u.label ASC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load upline role rates",
        )
    })?;
    let upline_roles: Vec<UplineRoleRateResponse> = upline_rows
        .into_iter()
        .map(|row| UplineRoleRateResponse {
            slug: row.try_get("slug").unwrap_or_default(),
            label: row.try_get("label").unwrap_or_default(),
            percent: row.try_get("percent").unwrap_or(0.0),
            has_baseline: row.try_get("has_baseline").unwrap_or(true),
            direct_sale_pool_percent: row.try_get("direct_sale_pool_percent").ok().flatten(),
        })
        .collect();

    let agent_pool_total = categories
        .iter()
        .find(|c| c.is_agent_pool)
        .map(|c| c.percent)
        .unwrap_or(0.0);
    let upline_total: f64 = upline_roles.iter().map(|r| r.percent).sum();
    let agent_pool_percent = (agent_pool_total - upline_total).max(0.0);

    Ok(ProjectRateConfigResponse {
        categories,
        upline_roles,
        agent_pool_percent,
    })
}

pub async fn get_project_rate_config(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectRateConfigResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;
    Ok(Json(fetch_project_rate_config(&pool, project_id).await?))
}

fn validate_percent(value: f64) -> Result<(), E> {
    if !value.is_finite() || !(0.0..=100.0).contains(&value) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Percent must be between 0 and 100",
        ));
    }
    Ok(())
}

fn map_category_db_error(e: sqlx::Error) -> E {
    if let Some(d) = e.as_database_error()
        && d.code().as_deref() == Some("23505")
    {
        return (
            StatusCode::CONFLICT,
            "A category with that label already exists on this project",
        );
    }
    tracing::error!("DB: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save category")
}

#[derive(Deserialize)]
pub struct CreateRateCategoryInput {
    pub label: String,
    pub percent: f64,
}

/// POST /projects/{project_id}/rate-config/categories — admin only. Always creates a
/// plain allocation category; `is_agent_pool` is never settable from here (there is
/// exactly one per project, seeded when the project was created).
pub async fn create_rate_category(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(p): Json<CreateRateCategoryInput>,
) -> Result<Json<RateCategoryResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let label = p.label.trim().to_string();
    if label.is_empty() || label.len() > 100 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid label"));
    }
    validate_percent(p.percent)?;

    let now = Utc::now().timestamp();

    let next_sort: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM public.project_rate_categories WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save category")
    })?;

    let row = sqlx::query(
        "INSERT INTO public.project_rate_categories
             (project_id, label, percent, is_agent_pool, sort_order, created_at, updated_at)
         VALUES ($1, $2, $3, false, $4, $5, $5)
      RETURNING id, label, percent, is_agent_pool, sort_order, updated_at",
    )
    .bind(project_id)
    .bind(&label)
    .bind(p.percent)
    .bind(next_sort)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(map_category_db_error)?;

    Ok(Json(row_to_category(row)))
}

#[derive(Deserialize)]
pub struct UpdateRateCategoryInput {
    pub label: String,
    pub percent: f64,
    pub sort_order: i32,
}

/// PATCH /projects/{project_id}/rate-config/categories/{id} — admin only. The
/// agent-pool row's label is fixed (its percent/sort_order can still change).
pub async fn update_rate_category(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path((project_id, id)): Path<(Uuid, Uuid)>,
    Json(p): Json<UpdateRateCategoryInput>,
) -> Result<Json<RateCategoryResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let label = p.label.trim().to_string();
    if label.is_empty() || label.len() > 100 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid label"));
    }
    validate_percent(p.percent)?;

    let existing = sqlx::query("SELECT label, is_agent_pool FROM public.project_rate_categories WHERE id = $1 AND project_id = $2")
        .bind(id)
        .bind(project_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
        })?
        .ok_or((StatusCode::NOT_FOUND, "Category not found"))?;

    let existing_label: String = existing.try_get("label").unwrap_or_default();
    let is_agent_pool: bool = existing.try_get("is_agent_pool").unwrap_or(false);
    if is_agent_pool && label != existing_label {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Cannot rename the agent-pool category",
        ));
    }

    // Shrinking the agent-pool bucket below what the upline roles already carve out
    // of it would silently clamp fetch_project_rate_config's agent_pool_percent to 0
    // with no signal — reject it here instead, mirroring update_upline_role_rate's
    // symmetric check the other way around.
    if is_agent_pool {
        let upline_total: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(percent), 0) FROM public.project_upline_role_rates WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
        })?;
        if p.percent + 0.001 < upline_total {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Agent-pool percent can't be less than what the upline roles already carve out of it",
            ));
        }
    }

    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "UPDATE public.project_rate_categories
            SET label = $1, percent = $2, sort_order = $3, updated_at = $4
          WHERE id = $5 AND project_id = $6
      RETURNING id, label, percent, is_agent_pool, sort_order, updated_at",
    )
    .bind(&label)
    .bind(p.percent)
    .bind(p.sort_order)
    .bind(now)
    .bind(id)
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .map_err(map_category_db_error)?;

    Ok(Json(row_to_category(row)))
}

/// DELETE /projects/{project_id}/rate-config/categories/{id} — admin only. Rejected
/// for the agent-pool row, which every project must always have exactly one of.
pub async fn delete_rate_category(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path((project_id, id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let is_agent_pool: bool = sqlx::query_scalar(
        "SELECT is_agent_pool FROM public.project_rate_categories WHERE id = $1 AND project_id = $2",
    )
    .bind(id)
    .bind(project_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Category not found"))?;

    if is_agent_pool {
        return Err((
            StatusCode::CONFLICT,
            "Cannot delete the agent-pool category",
        ));
    }

    let result = sqlx::query("DELETE FROM public.project_rate_categories WHERE id = $1 AND project_id = $2")
        .bind(id)
        .bind(project_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete category")
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Category not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct UpdateUplineRoleRateInput {
    pub percent: f64,
    pub has_baseline: bool,
    pub direct_sale_pool_percent: Option<f64>,
}

/// PATCH /projects/{project_id}/rate-config/upline-roles/{slug} — admin only. Upserts
/// this project's percent and commission-behavior flags for an existing upline role.
pub async fn update_upline_role_rate(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path((project_id, slug)): Path<(Uuid, String)>,
    Json(p): Json<UpdateUplineRoleRateInput>,
) -> Result<Json<UplineRoleRateResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;
    validate_percent(p.percent)?;
    validate_direct_sale_pool_percent(p.direct_sale_pool_percent)?;

    let role_row = sqlx::query(
        "SELECT label FROM public.upline_role_types WHERE slug = $1",
    )
    .bind(&slug)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Upline role not found"))?;
    let label: String = role_row.try_get("label").unwrap_or_default();

    // The upline roles' percents are carved out of the project's agent-pool category —
    // reject a value that would push their total over that bucket instead of silently
    // letting fetch_project_rate_config's agent_pool_percent clamp to 0 with no signal.
    let agent_pool_bucket: f64 = sqlx::query_scalar(
        "SELECT percent FROM public.project_rate_categories WHERE project_id = $1 AND is_agent_pool",
    )
    .bind(project_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?
    .unwrap_or(0.0);

    let other_upline_total: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(percent), 0) FROM public.project_upline_role_rates
          WHERE project_id = $1 AND upline_role_type_slug != $2",
    )
    .bind(project_id)
    .bind(&slug)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?;

    if other_upline_total + p.percent > agent_pool_bucket + 0.001 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Upline role percents can't exceed the agent-pool category's percent",
        ));
    }

    let now = Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO public.project_upline_role_rates
             (project_id, upline_role_type_slug, percent, has_baseline,
              direct_sale_pool_percent, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $6)
         ON CONFLICT (project_id, upline_role_type_slug) DO UPDATE
            SET percent = EXCLUDED.percent,
                has_baseline = EXCLUDED.has_baseline,
                direct_sale_pool_percent = EXCLUDED.direct_sale_pool_percent,
                updated_at = EXCLUDED.updated_at",
    )
    .bind(project_id)
    .bind(&slug)
    .bind(p.percent)
    .bind(p.has_baseline)
    .bind(p.direct_sale_pool_percent)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save upline role rate",
        )
    })?;

    Ok(Json(UplineRoleRateResponse {
        slug,
        label,
        percent: p.percent,
        has_baseline: p.has_baseline,
        direct_sale_pool_percent: p.direct_sale_pool_percent,
    }))
}

/// Seeds a brand-new project's rate config from today's global defaults
/// (`commission_rates` + `upline_role_types.base_commission_percent`), mirroring the
/// one-time backfill migration. Called inside `create_project`'s transaction.
pub async fn seed_project_rate_config(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project_id: Uuid,
) -> Result<(), E> {
    let now = Utc::now().timestamp();

    const FIXED_CATEGORIES: [(&str, f64, i32); 4] = [
        ("Legal Counsel", 5.0, 0),
        ("Land Owner", 40.0, 1),
        ("Hypomone", 25.0, 2),
        ("Project Dev & Processing", 10.0, 3),
    ];

    for (label, default_percent, sort_order) in FIXED_CATEGORIES {
        let percent: f64 = sqlx::query_scalar(
            "SELECT COALESCE((SELECT commission_rate FROM public.commission_rates WHERE role = $1), $2)",
        )
        .bind(label)
        .bind(default_percent)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to seed rate config")
        })?;

        sqlx::query(
            "INSERT INTO public.project_rate_categories
                 (project_id, label, percent, is_agent_pool, sort_order, created_at, updated_at)
             VALUES ($1, $2, $3, false, $4, $5, $5)",
        )
        .bind(project_id)
        .bind(label)
        .bind(percent)
        .bind(sort_order)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to seed rate config")
        })?;
    }

    let agent_commission_percent: f64 = sqlx::query_scalar(
        "SELECT COALESCE((SELECT SUM(commission_rate) FROM public.commission_rates
                            WHERE role IN ('Lead Broker', 'Titling Officer', 'Agent')), 20)",
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to seed rate config")
    })?;

    sqlx::query(
        "INSERT INTO public.project_rate_categories
             (project_id, label, percent, is_agent_pool, sort_order, created_at, updated_at)
         VALUES ($1, 'Agent Commission', $2, true, 99, $3, $3)",
    )
    .bind(project_id)
    .bind(agent_commission_percent)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to seed rate config")
    })?;

    let role_rows = sqlx::query(
        "SELECT slug, base_commission_percent, has_baseline, direct_sale_pool_percent
           FROM public.upline_role_types",
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to seed rate config")
    })?;

    for row in role_rows {
        let slug: String = row.try_get("slug").unwrap_or_default();
        let percent: f64 = row.try_get("base_commission_percent").unwrap_or(0.0);
        let has_baseline: bool = row.try_get("has_baseline").unwrap_or(true);
        let direct_sale_pool_percent: Option<f64> =
            row.try_get("direct_sale_pool_percent").ok().flatten();
        sqlx::query(
            "INSERT INTO public.project_upline_role_rates
                 (project_id, upline_role_type_slug, percent, has_baseline,
                  direct_sale_pool_percent, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $6)",
        )
        .bind(project_id)
        .bind(&slug)
        .bind(percent)
        .bind(has_baseline)
        .bind(direct_sale_pool_percent)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to seed rate config")
        })?;
    }

    Ok(())
}
