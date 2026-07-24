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

const ROLES: [&str; 3] = ["Lead Broker", "Titling Officer", "Agent"];
const STATUSES: [&str; 2] = ["Active", "Inactive"];

#[derive(Serialize)]
pub struct RosterResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub broker_id: Option<Uuid>,
    pub code: String,
    pub prc_license_number: Option<String>,
    pub commission_rate: f64,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct RosterInput {
    pub user_id: Uuid,
    pub role: String,
    pub broker_id: Option<Uuid>,
    pub code: String,
    pub prc_license_number: Option<String>,
    pub commission_rate: f64,
    pub status: String,
}

const ROSTER_COLUMNS: &str = "r.id, r.user_id, u.username, u.email, r.role, r.broker_id, r.code,
                               r.prc_license_number, r.commission_rate, r.status, r.created_at, r.updated_at";

fn row_to_roster(row: sqlx::postgres::PgRow) -> RosterResponse {
    RosterResponse {
        id: row.try_get("id").unwrap_or_default(),
        user_id: row.try_get("user_id").unwrap_or_default(),
        username: row.try_get("username").unwrap_or_default(),
        email: row.try_get("email").unwrap_or_default(),
        role: row.try_get("role").unwrap_or_default(),
        broker_id: row.try_get("broker_id").ok().flatten(),
        code: row.try_get("code").unwrap_or_default(),
        prc_license_number: row.try_get("prc_license_number").ok().flatten(),
        commission_rate: row.try_get("commission_rate").unwrap_or(0.0),
        status: row.try_get("status").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(0),
        updated_at: row.try_get("updated_at").unwrap_or(0),
    }
}

fn validate_roster_input(p: &RosterInput, current_id: Option<Uuid>, agent_pool_cap: f64) -> Result<(), E> {
    if !ROLES.contains(&p.role.as_str()) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid role"));
    }
    if p.code.trim().is_empty() || p.code.len() > 100 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid code"));
    }
    if p.prc_license_number.as_ref().is_some_and(|v| v.len() > 100) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "PRC license number too long",
        ));
    }
    if !STATUSES.contains(&p.status.as_str()) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid status"));
    }

    match p.role.as_str() {
        "Lead Broker" | "Titling Officer" => {
            if p.commission_rate != 0.0 {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Lead Broker and Titling Officer use company baseline rates",
                ));
            }
            if p.broker_id.is_some() {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Only an Agent can report to a broker",
                ));
            }
        }
        "Agent" => {
            if p.broker_id.is_none() {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Agent must have an upline",
                ));
            }
            if !p.commission_rate.is_finite()
                || p.commission_rate < 0.0
                || p.commission_rate > agent_pool_cap
            {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Share must be within the agent pool limit",
                ));
            }
        }
        _ => {}
    }

    if let (Some(broker_id), Some(id)) = (p.broker_id, current_id)
        && broker_id == id
    {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "A roster entry cannot report to itself",
        ));
    }
    Ok(())
}

async fn get_agent_pool_cap(pool: &PgPool) -> Result<f64, E> {
    let cap: Option<f64> = sqlx::query_scalar(
        "SELECT commission_rate FROM public.commission_rates WHERE role = 'Agent'",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?;
    Ok(cap.unwrap_or(12.0))
}

async fn validate_broker_upline(pool: &PgPool, broker_id: Uuid) -> Result<(), E> {
    let role: Option<String> = sqlx::query_scalar(
        "SELECT role FROM public.roster WHERE id = $1",
    )
    .bind(broker_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?;

    match role.as_deref() {
        Some("Lead Broker") | Some("Titling Officer") => Ok(()),
        Some(_) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Upline must be a Lead Broker or Titling Officer",
        )),
        None => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Referenced user or broker not found",
        )),
    }
}

fn normalized_commission_rate(role: &str, commission_rate: f64) -> f64 {
    if role == "Agent" {
        commission_rate
    } else {
        0.0
    }
}

fn user_role_for_roster(roster_role: &str) -> &'static str {
    match roster_role {
        "Lead Broker" => "Lead Broker",
        "Titling Officer" => "Titling Officer",
        "Agent" => "Agent",
        _ => "User",
    }
}

async fn sync_user_role(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    roster_role: &str,
    now: i64,
) -> Result<(), E> {
    let role = user_role_for_roster(roster_role);
    let result = sqlx::query(
        "UPDATE public.users SET role = $1, updated_at = $2
          WHERE id = $3 AND role != 'Admin'",
    )
    .bind(role)
    .bind(now)
    .bind(user_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to sync user role",
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "User not found or role could not be updated",
        ));
    }
    Ok(())
}

async fn revert_user_role(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    now: i64,
) -> Result<(), E> {
    sqlx::query(
        "UPDATE public.users SET role = 'User', updated_at = $1
          WHERE id = $2 AND role != 'Admin'",
    )
    .bind(now)
    .bind(user_id)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to revert user role",
        )
    })?;
    Ok(())
}

fn map_roster_db_error(e: sqlx::Error) -> E {
    if let Some(d) = e.as_database_error() {
        if d.code().as_deref() == Some("23505") {
            return match d.constraint() {
                Some("roster_user_id_unique") => {
                    (StatusCode::CONFLICT, "This user already has a roster role")
                }
                Some("roster_code_unique") => (StatusCode::CONFLICT, "That code already exists"),
                _ => (StatusCode::CONFLICT, "That value already exists"),
            };
        }
        if d.code().as_deref() == Some("23503") {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Referenced user or broker not found",
            );
        }
    }
    tracing::error!("DB: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to save roster entry",
    )
}

async fn fetch_roster_entry(pool: &PgPool, id: Uuid) -> Result<Json<RosterResponse>, E> {
    let row = sqlx::query(&format!(
        "SELECT {ROSTER_COLUMNS} FROM public.roster r
         JOIN public.users u ON u.id = r.user_id
        WHERE r.id = $1",
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Roster entry not found"))?;

    Ok(Json(row_to_roster(row)))
}

pub async fn list_roster(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<RosterResponse>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(&format!(
        "SELECT {ROSTER_COLUMNS} FROM public.roster r
         JOIN public.users u ON u.id = r.user_id
        WHERE r.company_id = 1
     ORDER BY r.role ASC, u.username ASC",
    ))
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load roster")
    })?;

    Ok(Json(rows.into_iter().map(row_to_roster).collect()))
}

pub async fn create_roster_entry(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(p): Json<RosterInput>,
) -> Result<Json<RosterResponse>, E> {
    require_admin(&pool, &headers).await?;
    let agent_pool_cap = get_agent_pool_cap(&pool).await?;
    validate_roster_input(&p, None, agent_pool_cap)?;
    if p.role == "Agent" {
        if let Some(broker_id) = p.broker_id {
            validate_broker_upline(&pool, broker_id).await?;
        }
    }

    let now = Utc::now().timestamp();
    let commission_rate = normalized_commission_rate(&p.role, p.commission_rate);
    let prc_license_number = p
        .prc_license_number
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save roster entry",
        )
    })?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO public.roster
             (company_id, user_id, role, broker_id, code, prc_license_number,
              commission_rate, status, created_at, updated_at)
         VALUES (1, $1, $2, $3, $4, $5, $6, $7, $8, $8)
      RETURNING id",
    )
    .bind(p.user_id)
    .bind(&p.role)
    .bind(p.broker_id)
    .bind(p.code.trim())
    .bind(prc_license_number)
    .bind(commission_rate)
    .bind(&p.status)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_roster_db_error)?;

    sync_user_role(&mut tx, p.user_id, &p.role, now).await?;

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save roster entry",
        )
    })?;

    fetch_roster_entry(&pool, id).await
}

pub async fn update_roster_entry(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(p): Json<RosterInput>,
) -> Result<Json<RosterResponse>, E> {
    require_admin(&pool, &headers).await?;
    let agent_pool_cap = get_agent_pool_cap(&pool).await?;
    validate_roster_input(&p, Some(id), agent_pool_cap)?;
    if p.role == "Agent" {
        if let Some(broker_id) = p.broker_id {
            validate_broker_upline(&pool, broker_id).await?;
        }
    }

    let now = Utc::now().timestamp();
    let commission_rate = normalized_commission_rate(&p.role, p.commission_rate);
    let prc_license_number = p
        .prc_license_number
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save roster entry",
        )
    })?;

    let old_user_id: Uuid = sqlx::query_scalar(
        "SELECT user_id FROM public.roster WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save roster entry",
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Roster entry not found"))?;

    let result = sqlx::query(
        "UPDATE public.roster
            SET user_id = $1, role = $2, broker_id = $3, code = $4, prc_license_number = $5,
                commission_rate = $6, status = $7, updated_at = $8
          WHERE id = $9",
    )
    .bind(p.user_id)
    .bind(&p.role)
    .bind(p.broker_id)
    .bind(p.code.trim())
    .bind(prc_license_number)
    .bind(commission_rate)
    .bind(&p.status)
    .bind(now)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(map_roster_db_error)?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Roster entry not found"));
    }

    if old_user_id != p.user_id {
        revert_user_role(&mut tx, old_user_id, now).await?;
    }
    sync_user_role(&mut tx, p.user_id, &p.role, now).await?;

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save roster entry",
        )
    })?;

    fetch_roster_entry(&pool, id).await
}

pub async fn delete_roster_entry(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let now = Utc::now().timestamp();

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete roster entry",
        )
    })?;

    let user_id: Uuid = sqlx::query_scalar(
        "SELECT user_id FROM public.roster WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete roster entry",
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Roster entry not found"))?;

    let result = sqlx::query("DELETE FROM public.roster WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete roster entry",
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Roster entry not found"));
    }

    revert_user_role(&mut tx, user_id, now).await?;

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete roster entry",
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}
