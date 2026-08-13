use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::admin::roster::{revert_user_role, sync_user_role};
use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::require_admin;
use crate::api::users::shared::E;
use crate::infra::session_cache::SessionCache;

// ---------------------------------------------------------------------------
// Employees — a dedicated, company-wide list of salaried staff, separate from the
// commission-based Agent/Lead Broker/Titling Officer roster.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SalaryEmployeeResponse {
    pub id: Uuid,
    pub name: String,
    pub position: Option<String>,
    pub status: String,
    /// The login account linked to this employee, if any — set via `user_id` below,
    /// which also syncs that account's `users.role` to "Employee" so it can reach the
    /// self-service /me/employee/* portal. Null means this employee has no portal
    /// access (e.g. a role that never needs to log in).
    pub user_id: Option<Uuid>,
    /// Joined from `users.username` purely for display; not stored on this table.
    pub username: Option<String>,
    /// Days of the week this employee doesn't work (0=Sunday..6=Saturday, matching JS
    /// `Date.getUTCDay()`). Used to compute a daily rate when a pay plan only
    /// partially covers a period (e.g. training ends mid-month) — those days are
    /// excluded from both the rate's denominator and the prorated numerator. Empty
    /// means every calendar day counts as a working day.
    pub rest_days: Vec<i16>,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct CreateSalaryEmployeeInput {
    pub name: String,
    #[serde(default)]
    pub position: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub user_id: Option<Uuid>,
    #[serde(default)]
    pub rest_days: Option<Vec<i16>>,
}

#[derive(Deserialize)]
pub struct UpdateSalaryEmployeeInput {
    pub name: String,
    #[serde(default)]
    pub position: Option<String>,
    pub status: String,
    #[serde(default)]
    pub user_id: Option<Uuid>,
    #[serde(default)]
    pub rest_days: Option<Vec<i16>>,
}

pub(crate) fn row_to_employee(row: sqlx::postgres::PgRow) -> SalaryEmployeeResponse {
    SalaryEmployeeResponse {
        id: row.try_get("id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        position: row.try_get("position").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        user_id: row.try_get("user_id").ok().flatten(),
        username: row.try_get("username").ok().flatten(),
        rest_days: row.try_get("rest_days").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(0),
    }
}

pub(crate) const EMPLOYEE_SELECT: &str = "SELECT se.id, se.name, se.position, se.status, se.user_id, \
                                       u.username, se.rest_days, se.created_at \
                                  FROM public.salary_employees se \
                             LEFT JOIN public.users u ON u.id = se.user_id";

fn valid_status(status: &str) -> bool {
    status == "Active" || status == "Inactive"
}

/// Validates and dedupes a rest-days list; `None` (field omitted) means "leave
/// unchanged" is NOT supported here — both create and update always write a
/// concrete list, defaulting to empty, same as every other optional field on this
/// resource (position, user_id).
fn normalize_rest_days(rest_days: Option<Vec<i16>>) -> Result<Vec<i16>, E> {
    let mut days = rest_days.unwrap_or_default();
    if days.iter().any(|d| !(0..=6).contains(d)) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "rest_days values must be between 0 (Sunday) and 6 (Saturday)",
        ));
    }
    days.sort_unstable();
    days.dedup();
    Ok(days)
}

async fn fetch_position_labels(pool: &PgPool) -> Result<Vec<String>, E> {
    sqlx::query_scalar("SELECT label FROM public.employee_position_types")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to verify position")
        })
}

/// `roster` and `salary_employees` are two independent tables that can each sync
/// `users.role` for the same account (see `sync_user_role`), with nothing else
/// stopping both from claiming the same user — linking a current roster member here
/// would silently flip their role to "Employee" while leaving a stale Agent/Lead
/// Broker/etc. row behind in the roster, still counted wherever the roster is read.
/// Guard the salary side of that conflict explicitly.
async fn ensure_user_not_on_roster(pool: &PgPool, user_id: Uuid) -> Result<(), E> {
    let on_roster: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.roster WHERE user_id = $1)")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to verify user")
            })?;
    if on_roster {
        return Err((
            StatusCode::CONFLICT,
            "This user is already a roster member (Agent/Lead Broker/Titling Officer/etc.) — \
             remove them from the roster first",
        ));
    }
    Ok(())
}

fn map_employee_db_error(e: sqlx::Error) -> E {
    if let Some(d) = e.as_database_error()
        && d.code().as_deref() == Some("23505")
    {
        return (
            StatusCode::CONFLICT,
            "This user is already linked to another employee record",
        );
    }
    tracing::error!("DB: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save employee")
}

pub async fn list_salary_employees(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<SalaryEmployeeResponse>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(&format!("{EMPLOYEE_SELECT} ORDER BY se.created_at ASC"))
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load employees")
        })?;

    Ok(Json(rows.into_iter().map(row_to_employee).collect()))
}

pub async fn create_salary_employee(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(p): Json<CreateSalaryEmployeeInput>,
) -> Result<Json<SalaryEmployeeResponse>, E> {
    require_admin(&pool, &headers).await?;

    let name = p.name.trim();
    if name.is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "name is required"));
    }
    let position = p.position.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if let Some(pos) = position {
        let labels = fetch_position_labels(&pool).await?;
        if !labels.iter().any(|l| l == pos) {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid position — add it under Positions first",
            ));
        }
    }
    let status = p.status.as_deref().unwrap_or("Active");
    if !valid_status(status) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "status must be Active or Inactive"));
    }
    let rest_days = normalize_rest_days(p.rest_days)?;
    if let Some(user_id) = p.user_id {
        ensure_user_not_on_roster(&pool, user_id).await?;
    }

    let now = Utc::now().timestamp();

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save employee")
    })?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO public.salary_employees (name, position, status, user_id, rest_days, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING id",
    )
    .bind(name)
    .bind(position)
    .bind(status)
    .bind(p.user_id)
    .bind(&rest_days)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_employee_db_error)?;

    if let Some(user_id) = p.user_id {
        sync_user_role(&mut tx, user_id, "Employee", now).await?;
    }

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save employee")
    })?;
    if let Some(user_id) = p.user_id {
        SessionCache::global().invalidate_user(user_id);
    }

    let row = sqlx::query(&format!("{EMPLOYEE_SELECT} WHERE se.id = $1"))
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load employee")
        })?;

    Ok(Json(row_to_employee(row)))
}

pub async fn update_salary_employee(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(employee_id): Path<Uuid>,
    Json(p): Json<UpdateSalaryEmployeeInput>,
) -> Result<Json<SalaryEmployeeResponse>, E> {
    require_admin(&pool, &headers).await?;

    let name = p.name.trim();
    if name.is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "name is required"));
    }
    if !valid_status(&p.status) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "status must be Active or Inactive"));
    }
    let position = p.position.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if let Some(pos) = position {
        let labels = fetch_position_labels(&pool).await?;
        if !labels.iter().any(|l| l == pos) {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid position — add it under Positions first",
            ));
        }
    }
    let rest_days = normalize_rest_days(p.rest_days)?;
    let now = Utc::now().timestamp();

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update employee")
    })?;

    let old_user_id: Option<Uuid> =
        sqlx::query_scalar("SELECT user_id FROM public.salary_employees WHERE id = $1")
            .bind(employee_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update employee")
            })?
            .ok_or((StatusCode::NOT_FOUND, "Employee not found"))?;

    // Only a genuinely new link needs the roster check — re-saving an employee whose
    // user_id is unchanged shouldn't start failing just because that account was
    // separately (and questionably) also added to the roster in the meantime.
    if let Some(new_user_id) = p.user_id
        && old_user_id != p.user_id
    {
        ensure_user_not_on_roster(&pool, new_user_id).await?;
    }

    let result = sqlx::query(
        "UPDATE public.salary_employees
            SET name = $1, position = $2, status = $3, user_id = $4, rest_days = $5
          WHERE id = $6",
    )
    .bind(name)
    .bind(position)
    .bind(&p.status)
    .bind(p.user_id)
    .bind(&rest_days)
    .bind(employee_id)
    .execute(&mut *tx)
    .await
    .map_err(map_employee_db_error)?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Employee not found"));
    }

    if old_user_id != p.user_id {
        if let Some(old_user_id) = old_user_id {
            revert_user_role(&mut tx, old_user_id, now).await?;
        }
        if let Some(new_user_id) = p.user_id {
            sync_user_role(&mut tx, new_user_id, "Employee", now).await?;
        }
    }

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update employee")
    })?;
    if old_user_id != p.user_id {
        if let Some(old_user_id) = old_user_id {
            SessionCache::global().invalidate_user(old_user_id);
        }
        if let Some(new_user_id) = p.user_id {
            SessionCache::global().invalidate_user(new_user_id);
        }
    }

    let row = sqlx::query(&format!("{EMPLOYEE_SELECT} WHERE se.id = $1"))
        .bind(employee_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load employee")
        })?;

    Ok(Json(row_to_employee(row)))
}

pub async fn delete_salary_employee(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(employee_id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let now = Utc::now().timestamp();

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete employee")
    })?;

    let user_id: Option<Uuid> =
        sqlx::query_scalar("SELECT user_id FROM public.salary_employees WHERE id = $1")
            .bind(employee_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete employee")
            })?
            .ok_or((StatusCode::NOT_FOUND, "Employee not found"))?;

    let result = sqlx::query("DELETE FROM public.salary_employees WHERE id = $1")
        .bind(employee_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete employee")
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Employee not found"));
    }

    if let Some(user_id) = user_id {
        revert_user_role(&mut tx, user_id, now).await?;
    }

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete employee")
    })?;
    if let Some(user_id) = user_id {
        SessionCache::global().invalidate_user(user_id);
    }

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Pay plans — append-only history per employee. Resolved client-side the same way
// commission_split_schedule is: the latest row with start_date <= target date wins.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SalaryPlanResponse {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub kind: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub training_fee: Option<f64>,
    pub monthly_amount: Option<f64>,
    pub schedule_type: Option<String>,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct CreateSalaryPlanInput {
    pub kind: String,
    pub start_date: String,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub training_fee: Option<f64>,
    #[serde(default)]
    pub monthly_amount: Option<f64>,
    #[serde(default)]
    pub schedule_type: Option<String>,
}

fn parse_date(value: &str, field: &'static str) -> Result<NaiveDate, E> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .map_err(|_| (StatusCode::UNPROCESSABLE_ENTITY, field))
}

pub(crate) fn format_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn format_date_opt(d: Option<NaiveDate>) -> Option<String> {
    d.map(format_date)
}

pub(crate) fn row_to_plan(row: sqlx::postgres::PgRow) -> SalaryPlanResponse {
    let start_date: NaiveDate = row.try_get("start_date").unwrap_or_default();
    let end_date: Option<NaiveDate> = row.try_get("end_date").unwrap_or_default();
    SalaryPlanResponse {
        id: row.try_get("id").unwrap_or_default(),
        employee_id: row.try_get("employee_id").unwrap_or_default(),
        kind: row.try_get("kind").unwrap_or_default(),
        start_date: format_date(start_date),
        end_date: format_date_opt(end_date),
        training_fee: row.try_get("training_fee").unwrap_or_default(),
        monthly_amount: row.try_get("monthly_amount").unwrap_or_default(),
        schedule_type: row.try_get("schedule_type").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(0),
    }
}

async fn ensure_employee(pool: &PgPool, employee_id: Uuid) -> Result<(), E> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.salary_employees WHERE id = $1)")
        .bind(employee_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to verify employee")
        })?;
    if !exists {
        return Err((StatusCode::NOT_FOUND, "Employee not found"));
    }
    Ok(())
}

pub async fn list_salary_plans(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(employee_id): Path<Uuid>,
) -> Result<Json<Vec<SalaryPlanResponse>>, E> {
    require_admin(&pool, &headers).await?;
    ensure_employee(&pool, employee_id).await?;

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

pub async fn create_salary_plan(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(employee_id): Path<Uuid>,
    Json(p): Json<CreateSalaryPlanInput>,
) -> Result<Json<SalaryPlanResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_employee(&pool, employee_id).await?;

    let start_date = parse_date(&p.start_date, "start_date must be YYYY-MM-DD")?;

    let (end_date, training_fee, monthly_amount, schedule_type) = match p.kind.as_str() {
        "training" => {
            let end_raw = p
                .end_date
                .as_deref()
                .ok_or((StatusCode::UNPROCESSABLE_ENTITY, "end_date is required for training"))?;
            let end_date = parse_date(end_raw, "end_date must be YYYY-MM-DD")?;
            if end_date < start_date {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "end_date must be on or after start_date",
                ));
            }
            let fee = p
                .training_fee
                .filter(|v| v.is_finite() && *v > 0.0)
                .ok_or((StatusCode::UNPROCESSABLE_ENTITY, "training_fee must be a positive number"))?;
            (Some(end_date), Some(fee), None, None)
        }
        "regular" => {
            let amount = p
                .monthly_amount
                .filter(|v| v.is_finite() && *v > 0.0)
                .ok_or((StatusCode::UNPROCESSABLE_ENTITY, "monthly_amount must be a positive number"))?;
            let schedule = p.schedule_type.clone().filter(|s| {
                matches!(s.as_str(), "weekly" | "semimonthly" | "monthly")
            });
            let schedule = schedule.ok_or((
                StatusCode::UNPROCESSABLE_ENTITY,
                "schedule_type must be weekly, semimonthly, or monthly",
            ))?;
            (None, None, Some(amount), Some(schedule))
        }
        _ => return Err((StatusCode::UNPROCESSABLE_ENTITY, "kind must be training or regular")),
    };

    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "INSERT INTO public.salary_plans (
            employee_id, kind, start_date, end_date, training_fee, monthly_amount,
            schedule_type, created_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
      RETURNING id, employee_id, kind, start_date, end_date, training_fee, monthly_amount,
                schedule_type, created_at",
    )
    .bind(employee_id)
    .bind(&p.kind)
    .bind(start_date)
    .bind(end_date)
    .bind(training_fee)
    .bind(monthly_amount)
    .bind(&schedule_type)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if let Some(d) = e.as_database_error()
            && d.code().as_deref() == Some("23505")
        {
            return (
                StatusCode::CONFLICT,
                "A pay plan already starts on that date for this employee",
            );
        }
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save pay plan")
    })?;

    Ok(Json(row_to_plan(row)))
}

#[derive(Deserialize)]
pub struct UpdateSalaryPlanInput {
    /// Only meaningful (and required) when the plan being edited is "training".
    #[serde(default)]
    pub training_fee: Option<f64>,
    /// Only meaningful (and required) when the plan being edited is "regular".
    #[serde(default)]
    pub monthly_amount: Option<f64>,
    /// Only meaningful (and required) when the plan being edited is "regular".
    #[serde(default)]
    pub schedule_type: Option<String>,
}

/// PATCH /salary-plans/{plan_id} — admin only. Edits the amount (and, for a
/// "regular" plan, the release cadence) of an existing plan row in place, rather
/// than requiring a delete + re-add. `kind` and `start_date` are immutable here —
/// changing either is a structural change (which periods this plan even generates),
/// not an amount correction, so it still goes through delete + a fresh "Training
/// period"/"Salary" entry instead.
pub async fn update_salary_plan(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(plan_id): Path<Uuid>,
    Json(p): Json<UpdateSalaryPlanInput>,
) -> Result<Json<SalaryPlanResponse>, E> {
    require_admin(&pool, &headers).await?;

    let kind: String = sqlx::query_scalar("SELECT kind FROM public.salary_plans WHERE id = $1")
        .bind(plan_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load pay plan")
        })?
        .ok_or((StatusCode::NOT_FOUND, "Pay plan not found"))?;

    let row = match kind.as_str() {
        "training" => {
            let fee = p
                .training_fee
                .filter(|v| v.is_finite() && *v > 0.0)
                .ok_or((StatusCode::UNPROCESSABLE_ENTITY, "training_fee must be a positive number"))?;

            sqlx::query(
                "UPDATE public.salary_plans
                    SET training_fee = $1
                  WHERE id = $2
              RETURNING id, employee_id, kind, start_date, end_date, training_fee, monthly_amount,
                        schedule_type, created_at",
            )
            .bind(fee)
            .bind(plan_id)
            .fetch_optional(&pool)
            .await
        }
        "regular" => {
            let amount = p
                .monthly_amount
                .filter(|v| v.is_finite() && *v > 0.0)
                .ok_or((StatusCode::UNPROCESSABLE_ENTITY, "monthly_amount must be a positive number"))?;
            let schedule = p
                .schedule_type
                .as_deref()
                .filter(|s| matches!(*s, "weekly" | "semimonthly" | "monthly"))
                .ok_or((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "schedule_type must be weekly, semimonthly, or monthly",
                ))?;

            sqlx::query(
                "UPDATE public.salary_plans
                    SET monthly_amount = $1, schedule_type = $2
                  WHERE id = $3
              RETURNING id, employee_id, kind, start_date, end_date, training_fee, monthly_amount,
                        schedule_type, created_at",
            )
            .bind(amount)
            .bind(schedule)
            .bind(plan_id)
            .fetch_optional(&pool)
            .await
        }
        _ => unreachable!("salary_plans.kind is constrained to training/regular at the DB level"),
    }
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update pay plan")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Pay plan not found"))?;

    Ok(Json(row_to_plan(row)))
}

pub async fn delete_salary_plan(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(plan_id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let result = sqlx::query("DELETE FROM public.salary_plans WHERE id = $1")
        .bind(plan_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete pay plan")
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Pay plan not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Release ledger — project-scoped, same shape as commission_release_entries, so a
// release's amount rolls into that project's Cash Out.
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SalaryReleaseEntryResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub employee_id: Uuid,
    pub period_start: String,
    pub period_end: String,
    pub amount: f64,
    pub paid_at: String,
    pub note: Option<String>,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct CreateSalaryReleaseEntryInput {
    pub employee_id: Uuid,
    pub period_start: String,
    pub period_end: String,
    pub amount: f64,
    pub paid_at: String,
    #[serde(default)]
    pub note: Option<String>,
}

fn row_to_release_entry(row: sqlx::postgres::PgRow) -> SalaryReleaseEntryResponse {
    let period_start: NaiveDate = row.try_get("period_start").unwrap_or_default();
    let period_end: NaiveDate = row.try_get("period_end").unwrap_or_default();
    let paid_at: NaiveDate = row.try_get("paid_at").unwrap_or_default();
    SalaryReleaseEntryResponse {
        id: row.try_get("id").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        employee_id: row.try_get("employee_id").unwrap_or_default(),
        period_start: format_date(period_start),
        period_end: format_date(period_end),
        amount: row.try_get("amount").unwrap_or(0.0),
        paid_at: format_date(paid_at),
        note: row.try_get("note").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(0),
    }
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

#[derive(Serialize)]
pub struct SalaryEmployeeReleaseResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    /// Joined from `projects.name` — this view spans every project the employee has
    /// ever been released against, so each row needs to say which one it's from.
    pub project_name: String,
    pub period_start: String,
    pub period_end: String,
    pub amount: f64,
    pub paid_at: String,
    pub note: Option<String>,
    pub created_at: i64,
}

/// GET /salary-employees/{employee_id}/releases — admin only. Every release entry
/// recorded against this employee across every project, for an "overview" of their
/// full release history (unlike `list_salary_release_entries`, which is scoped to
/// one project at a time to match that project's Cash Out).
pub async fn list_salary_employee_releases(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(employee_id): Path<Uuid>,
) -> Result<Json<Vec<SalaryEmployeeReleaseResponse>>, E> {
    require_admin(&pool, &headers).await?;
    ensure_employee(&pool, employee_id).await?;

    let rows = sqlx::query(
        "SELECT sre.id, sre.project_id, p.name AS project_name, sre.period_start, sre.period_end,
                sre.amount, sre.paid_at, sre.note, sre.created_at
           FROM public.salary_release_entries sre
           JOIN public.projects p ON p.id = sre.project_id
          WHERE sre.employee_id = $1
       ORDER BY sre.period_start ASC, sre.paid_at ASC",
    )
    .bind(employee_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load release history",
        )
    })?;

    Ok(Json(
        rows.into_iter()
            .map(|row| {
                let period_start: NaiveDate = row.try_get("period_start").unwrap_or_default();
                let period_end: NaiveDate = row.try_get("period_end").unwrap_or_default();
                let paid_at: NaiveDate = row.try_get("paid_at").unwrap_or_default();
                SalaryEmployeeReleaseResponse {
                    id: row.try_get("id").unwrap_or_default(),
                    project_id: row.try_get("project_id").unwrap_or_default(),
                    project_name: row.try_get("project_name").unwrap_or_default(),
                    period_start: format_date(period_start),
                    period_end: format_date(period_end),
                    amount: row.try_get("amount").unwrap_or(0.0),
                    paid_at: format_date(paid_at),
                    note: row.try_get("note").unwrap_or_default(),
                    created_at: row.try_get("created_at").unwrap_or(0),
                }
            })
            .collect(),
    ))
}

pub async fn list_salary_release_entries(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<SalaryReleaseEntryResponse>>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let rows = sqlx::query(
        "SELECT id, project_id, employee_id, period_start, period_end,
                amount, paid_at, note, created_at, COUNT(*) OVER() AS total_count
           FROM public.salary_release_entries
          WHERE project_id = $1
       ORDER BY period_start ASC, employee_id ASC, paid_at ASC
          LIMIT $2 OFFSET $3",
    )
    .bind(project_id)
    .bind(page_query.per_page())
    .bind(page_query.offset())
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load salary release entries",
        )
    })?;

    let total = total_count(&rows);
    Ok(Json(Page::new(
        rows.into_iter().map(row_to_release_entry).collect(),
        &page_query,
        total,
    )))
}

pub async fn create_salary_release_entry(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(p): Json<CreateSalaryReleaseEntryInput>,
) -> Result<Json<SalaryReleaseEntryResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;
    ensure_employee(&pool, p.employee_id).await?;

    if !p.amount.is_finite() || p.amount <= 0.0 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "amount must be a positive number",
        ));
    }

    let period_start = parse_date(&p.period_start, "period_start must be YYYY-MM-DD")?;
    let period_end = parse_date(&p.period_end, "period_end must be YYYY-MM-DD")?;
    if period_end < period_start {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "period_end must be on or after period_start",
        ));
    }
    let paid_at = parse_date(&p.paid_at, "paid_at must be YYYY-MM-DD")?;
    let note = p.note.as_deref().map(str::trim).filter(|s| !s.is_empty());

    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "INSERT INTO public.salary_release_entries (
            project_id, employee_id, period_start, period_end, amount, paid_at, note, created_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
      RETURNING id, project_id, employee_id, period_start, period_end, amount, paid_at, note, created_at",
    )
    .bind(project_id)
    .bind(p.employee_id)
    .bind(period_start)
    .bind(period_end)
    .bind(p.amount)
    .bind(paid_at)
    .bind(note)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save salary release entry",
        )
    })?;

    Ok(Json(row_to_release_entry(row)))
}

#[derive(Serialize)]
pub struct ProjectSalaryReleaseTotal {
    pub project_id: Uuid,
    pub project_name: String,
    pub total: f64,
}

#[derive(Serialize)]
pub struct SalaryReleaseSummaryResponse {
    pub total: f64,
    pub projects: Vec<ProjectSalaryReleaseTotal>,
}

/// GET /salary-release-summary — admin only. Company-wide salary-release total plus
/// a per-project breakdown, same treatment as commission_release_summary, for the
/// Cash Out card on the Projects overview page.
pub async fn salary_release_summary(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<SalaryReleaseSummaryResponse>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(
        "SELECT p.id AS project_id, p.name AS project_name, COALESCE(SUM(sre.amount), 0) AS total
           FROM public.projects p
           LEFT JOIN public.salary_release_entries sre ON sre.project_id = p.id
          WHERE p.company_id = 1
       GROUP BY p.id, p.name
       ORDER BY p.created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load salary release summary",
        )
    })?;

    let projects: Vec<ProjectSalaryReleaseTotal> = rows
        .into_iter()
        .map(|row| ProjectSalaryReleaseTotal {
            project_id: row.try_get("project_id").unwrap_or_default(),
            project_name: row.try_get("project_name").unwrap_or_default(),
            total: row.try_get("total").unwrap_or(0.0),
        })
        .collect();
    let total = projects.iter().map(|p| p.total).sum();

    Ok(Json(SalaryReleaseSummaryResponse { total, projects }))
}

pub async fn delete_salary_release_entry(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(entry_id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let result = sqlx::query("DELETE FROM public.salary_release_entries WHERE id = $1")
        .bind(entry_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete salary release entry",
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Salary release entry not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}
