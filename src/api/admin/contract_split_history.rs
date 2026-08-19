use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::require_admin;
use crate::api::users::shared::E;

#[derive(Serialize)]
pub struct ContractSplitHistoryResponse {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub split_months: i32,
    /// The 1st or the 16th of a month — the exact biweek release period this split
    /// takes effect from, matching the release cycle (1-15 releases the 16th; 16-30/31
    /// releases the 1st of the next month), not a whole-calendar-month granularity.
    pub effective_period_start: String,
    /// How future commission amounts are recomputed from this change forward:
    /// `even_split` (remaining ÷ new months) or `catch_up` (total ÷ new months, minus
    /// paid periods, with overpayment clawback on the first new period).
    pub rebalance_strategy: String,
    pub created_at: i64,
}

pub(crate) fn row_to_entry(row: sqlx::postgres::PgRow) -> ContractSplitHistoryResponse {
    let effective_period_start: NaiveDate = row.try_get("effective_period_start").unwrap_or_default();
    ContractSplitHistoryResponse {
        id: row.try_get("id").unwrap_or_default(),
        contract_id: row.try_get("contract_id").unwrap_or_default(),
        split_months: row.try_get("split_months").unwrap_or_default(),
        effective_period_start: effective_period_start.format("%Y-%m-%d").to_string(),
        rebalance_strategy: row
            .try_get("rebalance_strategy")
            .unwrap_or_else(|_| "catch_up".to_string()),
        created_at: row.try_get("created_at").unwrap_or(0),
    }
}

fn parse_rebalance_strategy(raw: Option<&str>) -> Result<&'static str, E> {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok("even_split"),
        Some("even_split") => Ok("even_split"),
        Some("catch_up") => Ok("catch_up"),
        Some(_) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "rebalance_strategy must be even_split or catch_up",
        )),
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

/// Every split-history row for every contract in the project, in one call — mirrors
/// how contracts/payments/status are already bulk-fetched per project.
pub async fn list_contract_split_history(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<ContractSplitHistoryResponse>>, E> {
    require_admin(&pool, &headers).await?;
    ensure_project(&pool, project_id).await?;

    let rows = sqlx::query(
        "SELECT h.id, h.contract_id, h.split_months, h.effective_period_start,
                h.rebalance_strategy, h.created_at,
                COUNT(*) OVER() AS total_count
           FROM public.contract_split_history h
           JOIN public.contracts c ON c.id = h.contract_id
          WHERE c.project_id = $1
       ORDER BY h.contract_id ASC, h.effective_period_start ASC
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
            "Failed to load contract split history",
        )
    })?;

    let total = total_count(&rows);
    Ok(Json(Page::new(
        rows.into_iter().map(row_to_entry).collect(),
        &page_query,
        total,
    )))
}

#[derive(Deserialize)]
pub struct ChangeContractSplitInput {
    pub split_months: i32,
    /// Optional explicit effective date (YYYY-MM-DD), used by the bulk "Commission
    /// Split" tool so several buyers can be moved onto a new split at the same chosen
    /// boundary. Must land exactly on the 1st or the 16th of a month (the release
    /// cadence) and cannot be earlier than the current still-unreleased period — same
    /// forward-only guarantee as the default now-computed date. Omit to keep the
    /// existing single-buyer behavior (effective from "now").
    #[serde(default)]
    pub effective_period_start: Option<String>,
    /// How future commission amounts are recomputed: `even_split` (default for new
    /// changes) or `catch_up`. Omit to default to `even_split`.
    #[serde(default)]
    pub rebalance_strategy: Option<String>,
}

#[derive(Serialize)]
pub struct ChangeContractSplitResponse {
    pub contract_id: Uuid,
    pub split_months: i32,
    pub history: Vec<ContractSplitHistoryResponse>,
}

/// The exact biweek release period a split change takes effect from, matching this
/// system's real release cadence — a calendar month has two releases: the 1-15 period
/// releases on the 16th, and the 16-30/31 period releases on the 1st of the NEXT
/// month. Saving on or before the 15th takes effect from the current 1-15 period (its
/// own release, the 16th, hasn't happened yet); saving on the 16th or later takes
/// effect from the current 16-30/31 period (its own release, the 1st of next month,
/// hasn't happened yet either) — NOT deferred to periods dated the following calendar
/// month. Either way the effective period is always within the SAME calendar month as
/// the edit, since neither of that month's two releases can have already happened by
/// the time this fires (the edit's own "today" is always before both).
fn effective_period_start_for_now() -> NaiveDate {
    let now = Utc::now().date_naive();
    let day = if now.day() <= 15 { 1 } else { 16 };
    NaiveDate::from_ymd_opt(now.year(), now.month(), day).unwrap_or(now)
}

/// Earliest period the bulk Commission Split picker may target — includes this
/// month's 1-15 half even after the 16th (its release may have just happened, but
/// admins still need to backdate a split to that boundary when re-applying).
fn earliest_selectable_period_start() -> NaiveDate {
    let now = Utc::now().date_naive();
    NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap_or(now)
}

/// Same day<=15-or-16 rule as `effective_period_start_for_now`, applied to an
/// arbitrary anchor date (used for the lazily-backfilled genesis row).
fn period_start_containing(date: NaiveDate) -> NaiveDate {
    let day = if date.day() <= 15 { 1 } else { 16 };
    NaiveDate::from_ymd_opt(date.year(), date.month(), day).unwrap_or(date)
}

/// Prior biweek boundary — used when genesis and a change would share the same
/// effective_period_start so history keeps two distinct rows.
fn previous_biweek_period_start(date: NaiveDate) -> NaiveDate {
    if date.day() == 16 {
        NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date)
    } else {
        let (year, month) = if date.month() == 1 {
            (date.year() - 1, 12)
        } else {
            (date.year(), date.month() - 1)
        };
        NaiveDate::from_ymd_opt(year, month, 16).unwrap_or(date)
    }
}

/// Records a split-months change for an existing contract, effective going forward
/// only — past months keep whatever they already displayed under the old split. The
/// first time this is called for a contract, it also lazily backfills a genesis row
/// (the contract's original split value, effective at its approval month) so history
/// is always complete once any change happens, with no migration needed to backfill
/// every existing contract.
pub async fn change_contract_split(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(contract_id): Path<Uuid>,
    Json(p): Json<ChangeContractSplitInput>,
) -> Result<Json<ChangeContractSplitResponse>, E> {
    require_admin(&pool, &headers).await?;

    if !(1..=120).contains(&p.split_months) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Split months must be between 1 and 120",
        ));
    }

    let rebalance_strategy = parse_rebalance_strategy(p.rebalance_strategy.as_deref())?;

    // Joined against projects/company_id (rather than a bare contract lookup) so this
    // endpoint enforces the same tenant scoping its sibling admin endpoints already do
    // via ensure_project — a contract belonging to a project outside this company must
    // 404 exactly like it would if the project itself didn't exist.
    let contract = sqlx::query(
        "SELECT c.agent_commission_split_months, c.approval_at, c.updated_at
           FROM public.contracts c
           JOIN public.projects p ON p.id = c.project_id
          WHERE c.id = $1 AND p.company_id = 1",
    )
    .bind(contract_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load contract")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Contract not found"))?;

    let current_split_months: i32 = contract
        .try_get("agent_commission_split_months")
        .unwrap_or(36);

    let approval_at: Option<NaiveDate> = contract.try_get("approval_at").ok().flatten();
    let updated_at: i64 = contract.try_get("updated_at").unwrap_or(0);
    let genesis_anchor = approval_at.unwrap_or_else(|| {
        chrono::DateTime::from_timestamp(updated_at, 0)
            .map(|dt| dt.date_naive())
            .unwrap_or_else(|| Utc::now().date_naive())
    });
    let genesis_period_start = period_start_containing(genesis_anchor);

    let earliest_effective = effective_period_start_for_now();
    let earliest_selectable = earliest_selectable_period_start();
    let effective_period_start = match p.effective_period_start.as_deref() {
        Some(raw) => {
            let parsed = NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").map_err(|_| {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "effective_period_start must be YYYY-MM-DD",
                )
            })?;
            if parsed.day() != 1 && parsed.day() != 16 {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "effective_period_start must be the 1st or the 16th of a month",
                ));
            }
            parsed
        }
        None => earliest_effective,
    };
    let genesis_effective = if genesis_period_start == effective_period_start {
        previous_biweek_period_start(effective_period_start)
    } else {
        genesis_period_start
    };
    let now = Utc::now().timestamp();

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to start transaction")
    })?;

    // Locking the contract row here (rather than just re-reading it) closes the
    // TOCTOU gap where two concurrent split changes on the same contract could both
    // read the same "current" state and both proceed — the loser now blocks until
    // the winner's transaction commits, then re-validates against its result.
    sqlx::query("SELECT id FROM public.contracts WHERE id = $1 FOR UPDATE")
        .bind(contract_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to lock contract")
        })?
        .ok_or((StatusCode::NOT_FOUND, "Contract not found"))?;

    let existing_at_effective: Option<(i32, String)> = sqlx::query(
        "SELECT split_months, rebalance_strategy
           FROM public.contract_split_history
          WHERE contract_id = $1 AND effective_period_start = $2",
    )
    .bind(contract_id)
    .bind(effective_period_start)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to check split history")
    })?
    .map(|row| {
        (
            row.try_get("split_months").unwrap_or(current_split_months),
            row
                .try_get::<String, _>("rebalance_strategy")
                .unwrap_or_else(|_| "catch_up".to_string()),
        )
    });

    if existing_at_effective.is_none() && effective_period_start < earliest_selectable {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "effective_period_start cannot be earlier than the first selectable period this month",
        ));
    }

    let existing_history: Vec<(NaiveDate, i32, String)> = sqlx::query(
        "SELECT effective_period_start, split_months, rebalance_strategy
           FROM public.contract_split_history
          WHERE contract_id = $1
       ORDER BY effective_period_start ASC",
    )
    .bind(contract_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to check split history")
    })?
    .into_iter()
    .map(|row| {
        (
            row.try_get::<NaiveDate, _>("effective_period_start")
                .unwrap_or(genesis_effective),
            row.try_get::<i32, _>("split_months").unwrap_or(current_split_months),
            row.try_get::<String, _>("rebalance_strategy")
                .unwrap_or_else(|_| "catch_up".to_string()),
        )
    })
    .collect();
    let has_history = !existing_history.is_empty();

    // What split_months would actually be active at the CHOSEN effective date under
    // the history as it stands right now (before this change) — the correct thing to
    // compare the new value against. Comparing against `contracts.agent_commission_split_months`
    // instead (as this used to) is wrong once any change has ever been scheduled for
    // the future: that flat column reflected the LAST-SAVED value regardless of
    // whether it had actually taken effect yet, so re-targeting the same split months
    // at a corrected effective date was rejected as a false "no change" duplicate.
    let active_before = existing_history
        .iter()
        .filter(|(start, _, _)| *start <= effective_period_start)
        .next_back()
        .map(|(_, months, _)| *months)
        .unwrap_or(current_split_months);
    let strategy_before = existing_at_effective
        .as_ref()
        .map(|(_, strategy)| strategy.as_str())
        .or_else(|| {
            existing_history
                .iter()
                .find(|(start, _, _)| *start == effective_period_start)
                .map(|(_, _, strategy)| strategy.as_str())
        })
        .unwrap_or("catch_up");
    if active_before == p.split_months && strategy_before == rebalance_strategy {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Split months and rebalance strategy are unchanged at the chosen effective date",
        ));
    }
    if active_before == p.split_months && existing_at_effective.is_none() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "New split months must differ from what would be active at the chosen effective date",
        ));
    }

    if !has_history {
        sqlx::query(
            "INSERT INTO public.contract_split_history
                (contract_id, split_months, effective_period_start, rebalance_strategy, created_at)
             VALUES ($1, $2, $3, 'catch_up', $4)
             ON CONFLICT (contract_id, effective_period_start) DO NOTHING",
        )
        .bind(contract_id)
        .bind(current_split_months)
        .bind(genesis_effective)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to record original split",
            )
        })?;
    }

    sqlx::query(
        "INSERT INTO public.contract_split_history
            (contract_id, split_months, effective_period_start, rebalance_strategy, created_at)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (contract_id, effective_period_start) DO UPDATE
            SET split_months = EXCLUDED.split_months,
                rebalance_strategy = EXCLUDED.rebalance_strategy,
                created_at = EXCLUDED.created_at",
    )
    .bind(contract_id)
    .bind(p.split_months)
    .bind(effective_period_start)
    .bind(rebalance_strategy)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to record split change",
        )
    })?;

    // The flat column is a cache of "whatever's actually active right now" for
    // display/validation purposes (the real per-period math always comes from
    // history — see splitSegments.ts). Setting it to `p.split_months` unconditionally
    // was the bug: a FUTURE-dated change (effective_period_start > earliest_effective)
    // isn't active yet, so the column must keep showing whatever's active today until
    // that date actually arrives. Strategy-only updates on an already-effective row
    // leave the flat column unchanged.
    let current_active = if effective_period_start <= earliest_effective {
        p.split_months
    } else {
        active_before
    };

    sqlx::query(
        "UPDATE public.contracts SET agent_commission_split_months = $1, updated_at = $2 WHERE id = $3",
    )
    .bind(current_active)
    .bind(now)
    .bind(contract_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update contract")
    })?;

    let history_rows = sqlx::query(
        "SELECT id, contract_id, split_months, effective_period_start, rebalance_strategy, created_at
           FROM public.contract_split_history
          WHERE contract_id = $1
       ORDER BY effective_period_start ASC",
    )
    .bind(contract_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to reload split history",
        )
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to commit split change")
    })?;

    Ok(Json(ChangeContractSplitResponse {
        contract_id,
        split_months: p.split_months,
        history: history_rows.into_iter().map(row_to_entry).collect(),
    }))
}
