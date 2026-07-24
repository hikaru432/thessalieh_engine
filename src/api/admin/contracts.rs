use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::shared::require_admin;
use crate::api::users::shared::E;

const PAYMENT_PLANS: [&str; 3] = ["installment", "half", "full"];
const PAYMENT_METHODS: [&str; 5] = ["cash", "gcash", "maya", "bank", "others"];

#[derive(Serialize)]
pub struct ContractResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub lot_id: Option<Uuid>,
    pub buyer_user_id: Option<Uuid>,
    pub buyer_username: String,
    pub buyer_name: String,
    pub buyer_last_name: String,
    pub buyer_first_name: String,
    pub buyer_middle_name: String,
    pub buyer_address: String,
    pub buyer_gmail: String,
    pub buyer_contact: String,
    pub lot_block: String,
    pub lot_lot: String,
    pub lot_area: f64,
    pub lot_type: String,
    pub lot_rate: f64,
    pub contract_price: f64,
    pub is_promo: bool,
    pub list_price: f64,
    pub payment_plan: String,
    pub initial_payment: f64,
    pub term_years: i32,
    pub term_months: i32,
    pub monthly_amortization: f64,
    pub due_day: i32,
    pub next_due_date: NaiveDate,
    pub approval_at: Option<NaiveDate>,
    pub marketing_representative: String,
    pub agent_code: String,
    pub selling_agent_id: Option<String>,
    pub agent_id: Option<Uuid>,
    pub source_of_buyer: Vec<String>,
    pub other_source: String,
    pub particulars: String,
    pub total_paid: f64,
    pub balance: f64,
    pub status: String,
    pub agent_commission_split_months: i32,
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
    pub reference_no: String,
    pub bank_name: String,
    pub sender_name: String,
    pub receiver_name: String,
    pub mode_label: String,
}

#[derive(Serialize)]
pub struct CashFlowPaymentResponse {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub amount: f64,
    pub method: String,
    pub months_covered: i32,
    pub paid_at: NaiveDate,
    pub reference_no: String,
    pub bank_name: String,
    pub sender_name: String,
    pub receiver_name: String,
    pub mode_label: String,
    pub buyer_name: String,
    pub lot_block: String,
    pub lot_lot: String,
    pub term_years: i32,
    pub term_months: i32,
}

#[derive(Serialize)]
pub struct ContractDetail {
    #[serde(flatten)]
    pub contract: ContractResponse,
    pub payments: Vec<PaymentResponse>,
}

#[derive(Deserialize)]
pub struct ContractInput {
    #[serde(default)]
    pub buyer_name: String,
    #[serde(default)]
    pub buyer_last_name: String,
    #[serde(default)]
    pub buyer_first_name: String,
    #[serde(default)]
    pub buyer_middle_name: String,
    pub buyer_address: String,
    pub buyer_gmail: String,
    pub buyer_contact: String,
    pub lot_id: Option<Uuid>,
    pub buyer_user_id: Option<Uuid>,
    pub lot_block: String,
    pub lot_lot: String,
    pub lot_area: f64,
    pub lot_type: String,
    pub lot_rate: f64,
    pub payment_plan: String,
    pub initial_payment: f64,
    #[serde(default)]
    pub term_years: i32,
    #[serde(default)]
    pub term_months: i32,
    pub monthly_amortization: f64,
    pub due_day: i32,
    pub next_due_date: NaiveDate,
    pub approval_at: Option<NaiveDate>,
    pub marketing_representative: String,
    pub agent_code: String,
    pub selling_agent_id: Option<String>,
    pub agent_id: Option<Uuid>,
    pub source_of_buyer: Vec<String>,
    pub other_source: String,
    pub particulars: String,
    #[serde(default = "default_split_months")]
    pub agent_commission_split_months: i32,
    #[serde(default)]
    pub is_promo: bool,
    /// Effective TCP when `is_promo` is true; ignored otherwise (catalog = area × rate).
    #[serde(default)]
    pub contract_price: f64,
    #[serde(default = "default_opening_method")]
    pub opening_payment_method: String,
    #[serde(default)]
    pub opening_reference_no: String,
    #[serde(default)]
    pub opening_bank_name: String,
    #[serde(default)]
    pub opening_sender_name: String,
    #[serde(default)]
    pub opening_receiver_name: String,
    #[serde(default)]
    pub opening_mode_label: String,
}

fn default_split_months() -> i32 {
    36
}

fn default_opening_method() -> String {
    "cash".to_string()
}

/// Returns (contract_price, list_price, is_promo).
fn resolve_contract_prices(p: &ContractInput) -> Result<(f64, f64, bool), E> {
    let list_price = p.lot_area * p.lot_rate;
    if p.is_promo {
        if p.contract_price <= 0.0 {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Promo TCP must be greater than zero",
            ));
        }
        if p.contract_price > list_price + 1e-6 {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Promo TCP cannot exceed catalog TCP",
            ));
        }
        Ok((p.contract_price, list_price, true))
    } else {
        Ok((list_price, list_price, false))
    }
}

#[derive(Deserialize)]
pub struct RecordPaymentInput {
    pub amount: f64,
    pub method: String,
    pub months_covered: i32,
    pub paid_at: NaiveDate,
    pub particulars: String,
    #[serde(default)]
    pub reference_no: String,
    #[serde(default)]
    pub bank_name: String,
    #[serde(default)]
    pub sender_name: String,
    #[serde(default)]
    pub receiver_name: String,
    #[serde(default)]
    pub mode_label: String,
}

fn normalize_payment_meta(p: &RecordPaymentInput) -> Result<(String, String, String, String, String), E> {
    normalize_payment_fields(
        &p.method,
        &p.reference_no,
        &p.bank_name,
        &p.sender_name,
        &p.receiver_name,
        &p.mode_label,
    )
}

fn normalize_payment_fields(
    method: &str,
    reference_no: &str,
    bank_name: &str,
    sender_name: &str,
    receiver_name: &str,
    mode_label: &str,
) -> Result<(String, String, String, String, String), E> {
    let reference_no = reference_no.trim().to_string();
    let bank_name = bank_name.trim().to_string();
    let sender_name = sender_name.trim().to_string();
    let receiver_name = receiver_name.trim().to_string();
    let mode_label = mode_label.trim().to_string();

    match method {
        "cash" => Ok((String::new(), String::new(), String::new(), String::new(), String::new())),
        "gcash" | "maya" => {
            if reference_no.is_empty() {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Reference / transaction number is required",
                ));
            }
            Ok((reference_no, String::new(), sender_name, receiver_name, String::new()))
        }
        "bank" => {
            if bank_name.is_empty() {
                return Err((StatusCode::UNPROCESSABLE_ENTITY, "Bank name is required"));
            }
            if reference_no.is_empty() {
                return Err((StatusCode::UNPROCESSABLE_ENTITY, "Reference number is required"));
            }
            Ok((reference_no, bank_name, String::new(), String::new(), String::new()))
        }
        "others" => {
            if mode_label.is_empty() {
                return Err((StatusCode::UNPROCESSABLE_ENTITY, "Payment label is required"));
            }
            Ok((reference_no, String::new(), String::new(), String::new(), mode_label))
        }
        _ => Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid payment method")),
    }
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

fn format_buyer_name(last: &str, first: &str, middle: &str) -> String {
    let last = last.trim();
    let first = first.trim();
    let middle = middle.trim();
    if middle.is_empty() {
        format!("{last}, {first}")
    } else {
        format!("{last}, {first} {middle}")
    }
}

enum ResolvedBuyerNames {
    FromParts {
        buyer_name: String,
        last: String,
        first: String,
        middle: String,
    },
    Legacy {
        buyer_name: String,
    },
}

fn resolve_buyer_names(p: &ContractInput) -> Result<ResolvedBuyerNames, E> {
    let last = p.buyer_last_name.trim();
    let first = p.buyer_first_name.trim();
    let middle = p.buyer_middle_name.trim();

    if !last.is_empty() || !first.is_empty() || !middle.is_empty() {
        if last.is_empty() || first.is_empty() {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Last name and first name are required",
            ));
        }
        if last.len() > 255 || first.len() > 255 || middle.len() > 255 {
            return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid buyer name"));
        }
        return Ok(ResolvedBuyerNames::FromParts {
            buyer_name: format_buyer_name(last, first, middle),
            last: last.to_string(),
            first: first.to_string(),
            middle: middle.to_string(),
        });
    }

    let name = p.buyer_name.trim();
    if name.is_empty() || name.len() > 255 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid buyer name"));
    }
    Ok(ResolvedBuyerNames::Legacy {
        buyer_name: name.to_string(),
    })
}

fn map_contract_db_error(action: &'static str) -> impl Fn(sqlx::Error) -> E {
    move |e| {
        if e.as_database_error()
            .is_some_and(|d| d.code().as_deref() == Some("23503"))
        {
            return (StatusCode::UNPROCESSABLE_ENTITY, "Selected agent not found");
        }
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, action)
    }
}

fn validate_contract_input(p: &ContractInput) -> Result<(), E> {
    resolve_buyer_names(p)?;
    if p.buyer_user_id.is_none() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Buyer account is required",
        ));
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
    if !(1..=120).contains(&p.agent_commission_split_months) {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Agent commission split months must be between 1 and 120",
        ));
    }
    Ok(())
}

const CONTRACT_COLUMNS_WITH_TOTALS: &str = "
    c.id, c.project_id, c.lot_id, c.buyer_user_id, MAX(bu.username) AS buyer_username,
    c.buyer_name, c.buyer_last_name, c.buyer_first_name, c.buyer_middle_name,
    c.buyer_address, c.buyer_gmail, c.buyer_contact,
    c.lot_block, c.lot_lot, c.lot_area, c.lot_type, c.lot_rate,
    c.contract_price, c.is_promo, c.list_price, c.payment_plan, c.initial_payment, c.term_years, c.term_months, c.monthly_amortization,
    c.due_day, c.next_due_date, c.approval_at,
    c.marketing_representative, c.agent_code, c.selling_agent_id, c.agent_id,
    c.source_of_buyer, c.other_source,
    c.particulars, c.agent_commission_split_months, c.updated_at,
    COALESCE(SUM(p.amount), 0) AS total_paid";

async fn validate_buyer_user(
    pool: &PgPool,
    buyer_user_id: Uuid,
    project_id: Uuid,
    exclude_contract_id: Option<Uuid>,
) -> Result<(), E> {
    let row = sqlx::query("SELECT role FROM public.users WHERE id = $1")
        .bind(buyer_user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to validate buyer account")
        })?
        .ok_or((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Buyer account not found",
        ))?;

    let role: String = row.try_get("role").unwrap_or_default();
    if role != "User" {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Selected account is not a buyer user",
        ));
    }

    let duplicate = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM public.contracts
          WHERE project_id = $1 AND buyer_user_id = $2 AND ($3::uuid IS NULL OR id != $3)
          LIMIT 1",
    )
    .bind(project_id)
    .bind(buyer_user_id)
    .bind(exclude_contract_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to validate buyer account")
    })?;

    if duplicate.is_some() {
        return Err((
            StatusCode::CONFLICT,
            "Buyer already has a contract in this project",
        ));
    }

    Ok(())
}

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
        buyer_user_id: row.try_get("buyer_user_id").ok().flatten(),
        buyer_username: row.try_get("buyer_username").unwrap_or_default(),
        buyer_name: row.try_get("buyer_name").unwrap_or_default(),
        buyer_last_name: row.try_get("buyer_last_name").unwrap_or_default(),
        buyer_first_name: row.try_get("buyer_first_name").unwrap_or_default(),
        buyer_middle_name: row.try_get("buyer_middle_name").unwrap_or_default(),
        buyer_address: row.try_get("buyer_address").unwrap_or_default(),
        buyer_gmail: row.try_get("buyer_gmail").unwrap_or_default(),
        buyer_contact: row.try_get("buyer_contact").unwrap_or_default(),
        lot_block: row.try_get("lot_block").unwrap_or_default(),
        lot_lot: row.try_get("lot_lot").unwrap_or_default(),
        lot_area: row.try_get("lot_area").unwrap_or(0.0),
        lot_type: row.try_get("lot_type").unwrap_or_default(),
        lot_rate: row.try_get("lot_rate").unwrap_or(0.0),
        contract_price,
        is_promo: row.try_get("is_promo").unwrap_or(false),
        list_price: row.try_get("list_price").unwrap_or(contract_price),
        payment_plan: row.try_get("payment_plan").unwrap_or_default(),
        initial_payment: row.try_get("initial_payment").unwrap_or(0.0),
        term_years: row.try_get("term_years").unwrap_or(0),
        term_months: row.try_get("term_months").unwrap_or(0),
        monthly_amortization: row.try_get("monthly_amortization").unwrap_or(0.0),
        due_day: row.try_get("due_day").unwrap_or(15),
        next_due_date,
        approval_at: row.try_get("approval_at").ok().flatten(),
        marketing_representative: row.try_get("marketing_representative").unwrap_or_default(),
        agent_code: row.try_get("agent_code").unwrap_or_default(),
        selling_agent_id: row.try_get("selling_agent_id").ok().flatten(),
        agent_id: row.try_get("agent_id").ok().flatten(),
        source_of_buyer: row.try_get("source_of_buyer").unwrap_or_default(),
        other_source: row.try_get("other_source").unwrap_or_default(),
        particulars: row.try_get("particulars").unwrap_or_default(),
        total_paid,
        balance,
        status: status.to_string(),
        agent_commission_split_months: row
            .try_get("agent_commission_split_months")
            .unwrap_or(36),
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
        reference_no: row.try_get("reference_no").unwrap_or_default(),
        bank_name: row.try_get("bank_name").unwrap_or_default(),
        sender_name: row.try_get("sender_name").unwrap_or_default(),
        receiver_name: row.try_get("receiver_name").unwrap_or_default(),
        mode_label: row.try_get("mode_label").unwrap_or_default(),
    }
}

fn row_to_cashflow_payment(row: sqlx::postgres::PgRow) -> CashFlowPaymentResponse {
    CashFlowPaymentResponse {
        id: row.try_get("id").unwrap_or_default(),
        contract_id: row.try_get("contract_id").unwrap_or_default(),
        amount: row.try_get("amount").unwrap_or(0.0),
        method: row.try_get("method").unwrap_or_default(),
        months_covered: row.try_get("months_covered").unwrap_or(1),
        paid_at: row
            .try_get("paid_at")
            .unwrap_or_else(|_| Utc::now().date_naive()),
        reference_no: row.try_get("reference_no").unwrap_or_default(),
        bank_name: row.try_get("bank_name").unwrap_or_default(),
        sender_name: row.try_get("sender_name").unwrap_or_default(),
        receiver_name: row.try_get("receiver_name").unwrap_or_default(),
        mode_label: row.try_get("mode_label").unwrap_or_default(),
        buyer_name: row.try_get("buyer_name").unwrap_or_default(),
        lot_block: row.try_get("lot_block").unwrap_or_default(),
        lot_lot: row.try_get("lot_lot").unwrap_or_default(),
        term_years: row.try_get("term_years").unwrap_or(0),
        term_months: row.try_get("term_months").unwrap_or(0),
    }
}

/// Keeps the pricelist in sync with the linked contract.
/// Reserved is only for timed holds on the pricelist (not contracts).
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
        "Installment"
    };
    sqlx::query(
        "UPDATE public.lots
            SET owner_buyer = $1, on_hold = false, status = $2, reserved_until = NULL
          WHERE id = $3",
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
        "UPDATE public.lots
            SET owner_buyer = NULL, on_hold = false, status = 'Available', reserved_until = NULL
          WHERE id = $1",
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
           LEFT JOIN public.users bu ON bu.id = c.buyer_user_id
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
           LEFT JOIN public.users bu ON bu.id = c.buyer_user_id
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
        "SELECT id, contract_id, amount, method, months_covered, paid_at,
                reference_no, bank_name, sender_name, receiver_name, mode_label
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
    let buyer_user_id = p
        .buyer_user_id
        .ok_or((StatusCode::UNPROCESSABLE_ENTITY, "Buyer account is required"))?;
    validate_buyer_user(&pool, buyer_user_id, project_id, None).await?;
    let names = resolve_buyer_names(&p)?;
    let (buyer_name, buyer_last_name, buyer_first_name, buyer_middle_name) = match names {
        ResolvedBuyerNames::FromParts {
            buyer_name,
            last,
            first,
            middle,
        } => (buyer_name, last, first, middle),
        ResolvedBuyerNames::Legacy { buyer_name: _ } => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Last name and first name are required",
            ));
        }
    };

    let now = Utc::now().timestamp();
    let (contract_price, list_price, is_promo) = resolve_contract_prices(&p)?;

    let row = sqlx::query(
        "INSERT INTO public.contracts (
             project_id, lot_id, buyer_user_id, buyer_name, buyer_last_name, buyer_first_name, buyer_middle_name,
             buyer_address, buyer_gmail, buyer_contact,
             lot_block, lot_lot, lot_area, lot_type, lot_rate,
             contract_price, is_promo, list_price, payment_plan, initial_payment, term_years, term_months, monthly_amortization,
             due_day, next_due_date, approval_at,
             marketing_representative, agent_code, selling_agent_id, agent_id,
             source_of_buyer, other_source,
             particulars, agent_commission_split_months, created_at, updated_at
         )
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31,$32,$33,$34,$35,$36)
      RETURNING id",
    )
    .bind(project_id)
    .bind(p.lot_id)
    .bind(buyer_user_id)
    .bind(&buyer_name)
    .bind(&buyer_last_name)
    .bind(&buyer_first_name)
    .bind(&buyer_middle_name)
    .bind(&p.buyer_address)
    .bind(&p.buyer_gmail)
    .bind(&p.buyer_contact)
    .bind(p.lot_block.trim())
    .bind(p.lot_lot.trim())
    .bind(p.lot_area)
    .bind(&p.lot_type)
    .bind(p.lot_rate)
    .bind(contract_price)
    .bind(is_promo)
    .bind(list_price)
    .bind(&p.payment_plan)
    .bind(p.initial_payment)
    .bind(p.term_years)
    .bind(p.term_months)
    .bind(p.monthly_amortization)
    .bind(p.due_day)
    .bind(p.next_due_date)
    .bind(p.approval_at)
    .bind(&p.marketing_representative)
    .bind(&p.agent_code)
    .bind(&p.selling_agent_id)
    .bind(p.agent_id)
    .bind(&p.source_of_buyer)
    .bind(&p.other_source)
    .bind(&p.particulars)
    .bind(p.agent_commission_split_months)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(map_contract_db_error("Failed to create contract"))?;

    let contract_id: Uuid = row.try_get("id").unwrap_or_default();

    if p.initial_payment > 0.0 {
        let method = if p.opening_payment_method.trim().is_empty() {
            "cash"
        } else {
            p.opening_payment_method.trim()
        };
        if !PAYMENT_METHODS.contains(&method) {
            return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid payment method"));
        }
        let (reference_no, bank_name, sender_name, receiver_name, mode_label) =
            normalize_payment_fields(
                method,
                &p.opening_reference_no,
                &p.opening_bank_name,
                &p.opening_sender_name,
                &p.opening_receiver_name,
                &p.opening_mode_label,
            )?;
        sqlx::query(
            "INSERT INTO public.payments (
                 contract_id, amount, method, months_covered, paid_at,
                 reference_no, bank_name, sender_name, receiver_name, mode_label
             )
             VALUES ($1, $2, $3, 0, $4, $5, $6, $7, $8, $9)",
        )
        .bind(contract_id)
        .bind(p.initial_payment)
        .bind(method)
        .bind(p.approval_at.unwrap_or_else(|| Utc::now().date_naive()))
        .bind(&reference_no)
        .bind(&bank_name)
        .bind(&sender_name)
        .bind(&receiver_name)
        .bind(&mode_label)
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
            &buyer_name,
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
    let buyer_user_id = p
        .buyer_user_id
        .ok_or((StatusCode::UNPROCESSABLE_ENTITY, "Buyer account is required"))?;

    let project_id: Uuid =
        sqlx::query_scalar("SELECT project_id FROM public.contracts WHERE id = $1")
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                tracing::error!("DB: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "DB error")
            })?
            .ok_or((StatusCode::NOT_FOUND, "Contract not found"))?;

    validate_buyer_user(&pool, buyer_user_id, project_id, Some(id)).await?;
    let names = resolve_buyer_names(&p)?;

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
    let (contract_price, list_price, is_promo) = resolve_contract_prices(&p)?;

    let buyer_name_for_sync = match &names {
        ResolvedBuyerNames::FromParts { buyer_name, .. } => buyer_name.clone(),
        ResolvedBuyerNames::Legacy { buyer_name } => buyer_name.clone(),
    };

    let updated = match names {
        ResolvedBuyerNames::FromParts {
            buyer_name,
            last,
            first,
            middle,
        } => {
            sqlx::query(
                "UPDATE public.contracts SET
                     lot_id = $1, buyer_user_id = $2, buyer_name = $3, buyer_last_name = $4, buyer_first_name = $5, buyer_middle_name = $6,
                     buyer_address = $7, buyer_gmail = $8, buyer_contact = $9,
                     lot_block = $10, lot_lot = $11, lot_area = $12, lot_type = $13, lot_rate = $14,
                     contract_price = $15, is_promo = $16, list_price = $17, payment_plan = $18, initial_payment = $19, term_years = $20, term_months = $21,
                     monthly_amortization = $22, due_day = $23, next_due_date = $24, approval_at = $25,
                     marketing_representative = $26, agent_code = $27, selling_agent_id = $28, agent_id = $29,
                     source_of_buyer = $30, other_source = $31, particulars = $32,
                     agent_commission_split_months = $33, updated_at = $34
                   WHERE id = $35",
            )
            .bind(p.lot_id)
            .bind(buyer_user_id)
            .bind(&buyer_name)
            .bind(&last)
            .bind(&first)
            .bind(&middle)
            .bind(&p.buyer_address)
            .bind(&p.buyer_gmail)
            .bind(&p.buyer_contact)
            .bind(p.lot_block.trim())
            .bind(p.lot_lot.trim())
            .bind(p.lot_area)
            .bind(&p.lot_type)
            .bind(p.lot_rate)
            .bind(contract_price)
            .bind(is_promo)
            .bind(list_price)
            .bind(&p.payment_plan)
            .bind(p.initial_payment)
            .bind(p.term_years)
            .bind(p.term_months)
            .bind(p.monthly_amortization)
            .bind(p.due_day)
            .bind(p.next_due_date)
            .bind(p.approval_at)
            .bind(&p.marketing_representative)
            .bind(&p.agent_code)
            .bind(&p.selling_agent_id)
            .bind(p.agent_id)
            .bind(&p.source_of_buyer)
            .bind(&p.other_source)
            .bind(&p.particulars)
            .bind(p.agent_commission_split_months)
            .bind(now)
            .bind(id)
            .execute(&pool)
            .await
        }
        ResolvedBuyerNames::Legacy { buyer_name } => {
            sqlx::query(
                "UPDATE public.contracts SET
                     lot_id = $1, buyer_user_id = $2, buyer_name = $3, buyer_address = $4, buyer_gmail = $5, buyer_contact = $6,
                     lot_block = $7, lot_lot = $8, lot_area = $9, lot_type = $10, lot_rate = $11,
                     contract_price = $12, is_promo = $13, list_price = $14, payment_plan = $15, initial_payment = $16, term_years = $17, term_months = $18,
                     monthly_amortization = $19, due_day = $20, next_due_date = $21, approval_at = $22,
                     marketing_representative = $23, agent_code = $24, selling_agent_id = $25, agent_id = $26,
                     source_of_buyer = $27, other_source = $28, particulars = $29,
                     agent_commission_split_months = $30, updated_at = $31
                   WHERE id = $32",
            )
            .bind(p.lot_id)
            .bind(buyer_user_id)
            .bind(&buyer_name)
            .bind(&p.buyer_address)
            .bind(&p.buyer_gmail)
            .bind(&p.buyer_contact)
            .bind(p.lot_block.trim())
            .bind(p.lot_lot.trim())
            .bind(p.lot_area)
            .bind(&p.lot_type)
            .bind(p.lot_rate)
            .bind(contract_price)
            .bind(is_promo)
            .bind(list_price)
            .bind(&p.payment_plan)
            .bind(p.initial_payment)
            .bind(p.term_years)
            .bind(p.term_months)
            .bind(p.monthly_amortization)
            .bind(p.due_day)
            .bind(p.next_due_date)
            .bind(p.approval_at)
            .bind(&p.marketing_representative)
            .bind(&p.agent_code)
            .bind(&p.selling_agent_id)
            .bind(p.agent_id)
            .bind(&p.source_of_buyer)
            .bind(&p.other_source)
            .bind(&p.particulars)
            .bind(p.agent_commission_split_months)
            .bind(now)
            .bind(id)
            .execute(&pool)
            .await
        }
    }
    .map_err(map_contract_db_error("Failed to update contract"))?;

    if updated.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Contract not found"));
    }

    if previous_lot_id != p.lot_id
        && let Some(old_lot_id) = previous_lot_id
    {
        clear_lot(&pool, old_lot_id).await?;
    }
    if let Some(lot_id) = p.lot_id {
        let total_paid: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0) FROM public.payments WHERE contract_id = $1",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap_or(0.0);
        let paid = if total_paid > 0.0 {
            total_paid
        } else {
            p.initial_payment
        };
        sync_lot_for_contract(
            &pool,
            lot_id,
            &buyer_name_for_sync,
            &p.payment_plan,
            paid >= contract_price && contract_price > 0.0,
        )
        .await?;
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
    let (reference_no, bank_name, sender_name, receiver_name, mode_label) =
        normalize_payment_meta(&p)?;

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
    // months_covered == 0: amort paid on approval (cash only) — keep next unpaid preferred due
    let next_due_date = if p.months_covered == 0 {
        current_due_date
    } else {
        add_months(current_due_date, p.months_covered)
    };

    sqlx::query(
        "INSERT INTO public.payments (
             contract_id, amount, method, months_covered, paid_at,
             reference_no, bank_name, sender_name, receiver_name, mode_label
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(id)
    .bind(p.amount)
    .bind(&p.method)
    .bind(p.months_covered)
    .bind(p.paid_at)
    .bind(&reference_no)
    .bind(&bank_name)
    .bind(&sender_name)
    .bind(&receiver_name)
    .bind(&mode_label)
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
        let total_paid: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0) FROM public.payments WHERE contract_id = $1",
        )
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

pub async fn list_project_payments(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<CashFlowPaymentResponse>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(
        "SELECT p.id, p.contract_id, p.amount, p.method, p.months_covered, p.paid_at,
                p.reference_no, p.bank_name, p.sender_name, p.receiver_name, p.mode_label,
                c.buyer_name, c.lot_block, c.lot_lot, c.term_years, c.term_months
           FROM public.payments p
           INNER JOIN public.contracts c ON c.id = p.contract_id
          WHERE c.project_id = $1
       ORDER BY p.paid_at DESC, p.created_at DESC",
    )
    .bind(project_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load payments")
    })?;

    Ok(Json(rows.into_iter().map(row_to_cashflow_payment).collect()))
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
