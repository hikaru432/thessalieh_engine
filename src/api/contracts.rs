use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::shared::require_admin;
use super::users::shared::E;

const PAYMENT_PLANS: [&str; 3] = ["installment", "half", "full"];
const PAYMENT_METHODS: [&str; 4] = ["cash", "card", "gcash", "maya"];

#[derive(Serialize)]
pub struct ContractResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub lot_id: Option<Uuid>,
    pub buyer_name: String,
    pub buyer_address: String,
    pub buyer_gmail: String,
    pub buyer_contact: String,
    pub lot_block: String,
    pub lot_lot: String,
    pub lot_area: f64,
    pub lot_type: String,
    pub lot_rate: f64,
    pub contract_price: f64,
    pub payment_plan: String,
    pub initial_payment: f64,
    pub term_years: i32,
    pub monthly_amortization: f64,
    pub due_day: i32,
    pub next_due_date: NaiveDate,
    pub approval_at: Option<NaiveDate>,
    pub marketing_representative: String,
    pub agent_code: String,
    pub selling_agent_id: Option<String>,
    pub source_of_buyer: Vec<String>,
    pub other_source: String,
    pub particulars: String,
    pub total_paid: f64,
    pub balance: f64,
    pub status: String,
    pub updated_at: i64,
}

#[derive(Serialize)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub amount: f64,
    pub method: String,
    pub months_covered: i32,
    pub paid_at: NaiveDate,
}

#[derive(Serialize)]
pub struct ContractDetail {
    #[serde(flatten)]
    pub contract: ContractResponse,
    pub payments: Vec<PaymentResponse>,
}

#[derive(Deserialize)]
pub struct ContractInput {
    pub buyer_name: String,
    pub buyer_address: String,
    pub buyer_gmail: String,
    pub buyer_contact: String,
    pub lot_id: Option<Uuid>,
    pub lot_block: String,
    pub lot_lot: String,
    pub lot_area: f64,
    pub lot_type: String,
    pub lot_rate: f64,
    pub payment_plan: String,
    pub initial_payment: f64,
    pub term_years: i32,
    pub monthly_amortization: f64,
    pub due_day: i32,
    pub next_due_date: NaiveDate,
    pub approval_at: Option<NaiveDate>,
    pub marketing_representative: String,
    pub agent_code: String,
    pub selling_agent_id: Option<String>,
    pub source_of_buyer: Vec<String>,
    pub other_source: String,
    pub particulars: String,
}

#[derive(Deserialize)]
pub struct RecordPaymentInput {
    pub amount: f64,
    pub method: String,
    pub months_covered: i32,
    pub paid_at: NaiveDate,
    pub particulars: String,
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let (ny, nm) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    NaiveDate::from_ymd_opt(ny, nm, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day()
}

/// Adds `months` to `date`, clamping the day to the target month's length
/// (matches the frontend's `addMonths` helper).
fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let total = date.year() * 12 + date.month() as i32 - 1 + months;
    let year = total.div_euclid(12);
    let month = (total.rem_euclid(12) + 1) as u32;
    let last_day = last_day_of_month(year, month);
    NaiveDate::from_ymd_opt(year, month, date.day().min(last_day)).unwrap()
}

fn validate_contract_input(p: &ContractInput) -> Result<(), E> {
    if p.buyer_name.trim().is_empty() || p.buyer_name.len() > 255 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid buyer name"));
    }
    if p.lot_block.trim().is_empty() || p.lot_lot.trim().is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Block and lot are required"));
    }
    if !PAYMENT_PLANS.contains(&p.payment_plan.as_str()) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid payment plan"));
    }
    if p.lot_area < 0.0 || p.lot_rate < 0.0 || p.initial_payment < 0.0 || p.monthly_amortization < 0.0 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Numeric fields must not be negative"));
    }
    if !(1..=31).contains(&p.due_day) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Due day must be between 1 and 31"));
    }
    Ok(())
}

const CONTRACT_COLUMNS_WITH_TOTALS: &str = "
    c.id, c.project_id, c.lot_id, c.buyer_name, c.buyer_address, c.buyer_gmail, c.buyer_contact,
    c.lot_block, c.lot_lot, c.lot_area, c.lot_type, c.lot_rate,
    c.contract_price, c.payment_plan, c.initial_payment, c.term_years, c.monthly_amortization,
    c.due_day, c.next_due_date, c.approval_at,
    c.marketing_representative, c.agent_code, c.selling_agent_id, c.source_of_buyer, c.other_source,
    c.particulars, c.updated_at,
    COALESCE(SUM(p.amount), 0) AS total_paid";

fn row_to_contract(row: sqlx::postgres::PgRow) -> ContractResponse {
    let contract_price: f64 = row.try_get("contract_price").unwrap_or(0.0);
    let total_paid: f64 = row.try_get("total_paid").unwrap_or(0.0);
    let balance = (contract_price - total_paid).max(0.0);
    let next_due_date: NaiveDate = row
        .try_get("next_due_date")
        .unwrap_or_else(|_| Utc::now().date_naive());

    let status = if balance <= 0.0 {
        "Fully Paid"
    } else if next_due_date < Utc::now().date_naive() {
        "Needs Attention"
    } else {
        "On Track"
    };

    ContractResponse {
        id: row.try_get("id").unwrap_or_default(),
        project_id: row.try_get("project_id").unwrap_or_default(),
        lot_id: row.try_get("lot_id").ok().flatten(),
        buyer_name: row.try_get("buyer_name").unwrap_or_default(),
        buyer_address: row.try_get("buyer_address").unwrap_or_default(),
        buyer_gmail: row.try_get("buyer_gmail").unwrap_or_default(),
        buyer_contact: row.try_get("buyer_contact").unwrap_or_default(),
        lot_block: row.try_get("lot_block").unwrap_or_default(),
        lot_lot: row.try_get("lot_lot").unwrap_or_default(),
        lot_area: row.try_get("lot_area").unwrap_or(0.0),
        lot_type: row.try_get("lot_type").unwrap_or_default(),
        lot_rate: row.try_get("lot_rate").unwrap_or(0.0),
        contract_price,
        payment_plan: row.try_get("payment_plan").unwrap_or_default(),
        initial_payment: row.try_get("initial_payment").unwrap_or(0.0),
        term_years: row.try_get("term_years").unwrap_or(0),
        monthly_amortization: row.try_get("monthly_amortization").unwrap_or(0.0),
        due_day: row.try_get("due_day").unwrap_or(15),
        next_due_date,
        approval_at: row.try_get("approval_at").ok().flatten(),
        marketing_representative: row.try_get("marketing_representative").unwrap_or_default(),
        agent_code: row.try_get("agent_code").unwrap_or_default(),
        selling_agent_id: row.try_get("selling_agent_id").ok().flatten(),
        source_of_buyer: row.try_get("source_of_buyer").unwrap_or_default(),
        other_source: row.try_get("other_source").unwrap_or_default(),
        particulars: row.try_get("particulars").unwrap_or_default(),
        total_paid,
        balance,
        status: status.to_string(),
        updated_at: row.try_get("updated_at").unwrap_or(0),
    }
}

fn row_to_payment(row: sqlx::postgres::PgRow) -> PaymentResponse {
    PaymentResponse {
        id: row.try_get("id").unwrap_or_default(),
        contract_id: row.try_get("contract_id").unwrap_or_default(),
        amount: row.try_get("amount").unwrap_or(0.0),
        method: row.try_get("method").unwrap_or_default(),
        months_covered: row.try_get("months_covered").unwrap_or(1),
        paid_at: row
            .try_get("paid_at")
            .unwrap_or_else(|_| Utc::now().date_naive()),
    }
}

/// Keeps the pricelist in sync: a linked lot shows the buyer and a
/// Reserved/Sold status; unlinking clears it back to Available.
async fn sync_lot_for_contract(
    pool: &PgPool,
    lot_id: Uuid,
    buyer_name: &str,
    payment_plan: &str,
    fully_paid: bool,
) -> Result<(), E> {
    let status = if fully_paid || payment_plan == "full" {
        "Sold"
    } else {
        "Reserved"
    };
    sqlx::query(
        "UPDATE public.lots SET owner_buyer = $1, on_hold = false, status = $2 WHERE id = $3",
    )
    .bind(buyer_name)
    .bind(status)
    .bind(lot_id)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to sync lot")
    })?;
    Ok(())
}

async fn clear_lot(pool: &PgPool, lot_id: Uuid) -> Result<(), E> {
    sqlx::query(
        "UPDATE public.lots SET owner_buyer = NULL, on_hold = false, status = 'Available' WHERE id = $1",
    )
    .bind(lot_id)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to sync lot")
    })?;
    Ok(())
}

pub async fn list_contracts(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ContractResponse>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(&format!(
        "SELECT {CONTRACT_COLUMNS_WITH_TOTALS}
           FROM public.contracts c
           LEFT JOIN public.payments p ON p.contract_id = c.id
          WHERE c.project_id = $1
       GROUP BY c.id
       ORDER BY c.created_at ASC",
    ))
    .bind(project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load contracts")
    })?;

    Ok(Json(rows.into_iter().map(row_to_contract).collect()))
}

pub async fn get_contract(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<ContractDetail>, E> {
    require_admin(&pool, &headers).await?;

    let row = sqlx::query(&format!(
        "SELECT {CONTRACT_COLUMNS_WITH_TOTALS}
           FROM public.contracts c
           LEFT JOIN public.payments p ON p.contract_id = c.id
          WHERE c.id = $1
       GROUP BY c.id",
    ))
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load contract")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Contract not found"))?;

    let payment_rows = sqlx::query(
        "SELECT id, contract_id, amount, method, months_covered, paid_at
           FROM public.payments WHERE contract_id = $1
       ORDER BY paid_at ASC, created_at ASC",
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load payments")
    })?;

    Ok(Json(ContractDetail {
        contract: row_to_contract(row),
        payments: payment_rows.into_iter().map(row_to_payment).collect(),
    }))
}

pub async fn create_contract(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(p): Json<ContractInput>,
) -> Result<Json<ContractResponse>, E> {
    require_admin(&pool, &headers).await?;
    validate_contract_input(&p)?;

    let now = Utc::now().timestamp();
    let contract_price = p.lot_area * p.lot_rate;

    let row = sqlx::query(
        "INSERT INTO public.contracts (
             project_id, lot_id, buyer_name, buyer_address, buyer_gmail, buyer_contact,
             lot_block, lot_lot, lot_area, lot_type, lot_rate,
             contract_price, payment_plan, initial_payment, term_years, monthly_amortization,
             due_day, next_due_date, approval_at,
             marketing_representative, agent_code, selling_agent_id, source_of_buyer, other_source,
             particulars, created_at, updated_at
         )
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$26)
      RETURNING id",
    )
    .bind(project_id)
    .bind(p.lot_id)
    .bind(p.buyer_name.trim())
    .bind(&p.buyer_address)
    .bind(&p.buyer_gmail)
    .bind(&p.buyer_contact)
    .bind(p.lot_block.trim())
    .bind(p.lot_lot.trim())
    .bind(p.lot_area)
    .bind(&p.lot_type)
    .bind(p.lot_rate)
    .bind(contract_price)
    .bind(&p.payment_plan)
    .bind(p.initial_payment)
    .bind(p.term_years)
    .bind(p.monthly_amortization)
    .bind(p.due_day)
    .bind(p.next_due_date)
    .bind(p.approval_at)
    .bind(&p.marketing_representative)
    .bind(&p.agent_code)
    .bind(&p.selling_agent_id)
    .bind(&p.source_of_buyer)
    .bind(&p.other_source)
    .bind(&p.particulars)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create contract")
    })?;

    let contract_id: Uuid = row.try_get("id").unwrap_or_default();

    if p.initial_payment > 0.0 {
        sqlx::query(
            "INSERT INTO public.payments (contract_id, amount, method, months_covered, paid_at)
             VALUES ($1, $2, 'cash', 0, $3)",
        )
        .bind(contract_id)
        .bind(p.initial_payment)
        .bind(p.approval_at.unwrap_or_else(|| Utc::now().date_naive()))
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to record opening payment")
        })?;
    }

    if let Some(lot_id) = p.lot_id {
        sync_lot_for_contract(
            &pool,
            lot_id,
            p.buyer_name.trim(),
            &p.payment_plan,
            p.initial_payment >= contract_price && contract_price > 0.0,
        )
        .await?;
    }

    get_contract(Extension(pool), headers, Path(contract_id))
        .await
        .map(|Json(detail)| Json(detail.contract))
}

pub async fn update_contract(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(p): Json<ContractInput>,
) -> Result<Json<ContractResponse>, E> {
    require_admin(&pool, &headers).await?;
    validate_contract_input(&p)?;

    let previous_lot_id: Option<Uuid> =
        sqlx::query_scalar("SELECT lot_id FROM public.contracts WHERE id = $1")
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
            })?
            .ok_or((StatusCode::NOT_FOUND, "Contract not found"))?;

    let now = Utc::now().timestamp();
    let contract_price = p.lot_area * p.lot_rate;

    let updated = sqlx::query(
        "UPDATE public.contracts SET
             lot_id = $1, buyer_name = $2, buyer_address = $3, buyer_gmail = $4, buyer_contact = $5,
             lot_block = $6, lot_lot = $7, lot_area = $8, lot_type = $9, lot_rate = $10,
             contract_price = $11, payment_plan = $12, initial_payment = $13, term_years = $14,
             monthly_amortization = $15, due_day = $16, next_due_date = $17, approval_at = $18,
             marketing_representative = $19, agent_code = $20, selling_agent_id = $21,
             source_of_buyer = $22, other_source = $23, particulars = $24, updated_at = $25
           WHERE id = $26",
    )
    .bind(p.lot_id)
    .bind(p.buyer_name.trim())
    .bind(&p.buyer_address)
    .bind(&p.buyer_gmail)
    .bind(&p.buyer_contact)
    .bind(p.lot_block.trim())
    .bind(p.lot_lot.trim())
    .bind(p.lot_area)
    .bind(&p.lot_type)
    .bind(p.lot_rate)
    .bind(contract_price)
    .bind(&p.payment_plan)
    .bind(p.initial_payment)
    .bind(p.term_years)
    .bind(p.monthly_amortization)
    .bind(p.due_day)
    .bind(p.next_due_date)
    .bind(p.approval_at)
    .bind(&p.marketing_representative)
    .bind(&p.agent_code)
    .bind(&p.selling_agent_id)
    .bind(&p.source_of_buyer)
    .bind(&p.other_source)
    .bind(&p.particulars)
    .bind(now)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update contract")
    })?;

    if updated.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Contract not found"));
    }

    if previous_lot_id != p.lot_id {
        if let Some(old_lot_id) = previous_lot_id {
            clear_lot(&pool, old_lot_id).await?;
        }
    }
    if let Some(lot_id) = p.lot_id {
        sync_lot_for_contract(&pool, lot_id, p.buyer_name.trim(), &p.payment_plan, false).await?;
    }

    get_contract(Extension(pool), headers, Path(id))
        .await
        .map(|Json(detail)| Json(detail.contract))
}

pub async fn record_payment(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(p): Json<RecordPaymentInput>,
) -> Result<Json<ContractResponse>, E> {
    require_admin(&pool, &headers).await?;

    if !PAYMENT_METHODS.contains(&p.method.as_str()) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid payment method"));
    }
    if p.amount <= 0.0 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Payment amount must be positive"));
    }
    if p.months_covered < 0 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Months covered must not be negative"));
    }

    let row = sqlx::query(
        "SELECT lot_id, buyer_name, payment_plan, contract_price, next_due_date
           FROM public.contracts WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Contract not found"))?;

    let lot_id: Option<Uuid> = row.try_get("lot_id").ok().flatten();
    let buyer_name: String = row.try_get("buyer_name").unwrap_or_default();
    let payment_plan: String = row.try_get("payment_plan").unwrap_or_default();
    let contract_price: f64 = row.try_get("contract_price").unwrap_or(0.0);
    let current_due_date: NaiveDate = row
        .try_get("next_due_date")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let next_due_date = add_months(current_due_date, p.months_covered.max(1));

    sqlx::query(
        "INSERT INTO public.payments (contract_id, amount, method, months_covered, paid_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(p.amount)
    .bind(&p.method)
    .bind(p.months_covered)
    .bind(p.paid_at)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to record payment")
    })?;

    let now = Utc::now().timestamp();
    sqlx::query(
        "UPDATE public.contracts SET next_due_date = $1, particulars = $2, updated_at = $3 WHERE id = $4",
    )
    .bind(next_due_date)
    .bind(&p.particulars)
    .bind(now)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update contract")
    })?;

    if let Some(lot_id) = lot_id {
        let total_paid: f64 =
            sqlx::query_scalar("SELECT COALESCE(SUM(amount), 0) FROM public.payments WHERE contract_id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap_or(0.0);
        sync_lot_for_contract(
            &pool,
            lot_id,
            &buyer_name,
            &payment_plan,
            total_paid >= contract_price && contract_price > 0.0,
        )
        .await?;
    }

    get_contract(Extension(pool), headers, Path(id))
        .await
        .map(|Json(detail)| Json(detail.contract))
}

pub async fn delete_contract(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let lot_id: Option<Uuid> =
        sqlx::query_scalar("SELECT lot_id FROM public.contracts WHERE id = $1")
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
            })?
            .flatten();

    let result = sqlx::query("DELETE FROM public.contracts WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete contract")
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Contract not found"));
    }

    if let Some(lot_id) = lot_id {
        clear_lot(&pool, lot_id).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}
