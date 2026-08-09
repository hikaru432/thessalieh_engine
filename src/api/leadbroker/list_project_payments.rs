use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use chrono::{NaiveDate, Utc};
use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::agents::{collect_direct_and_downline_seller_ids, load_project_agents};
use super::guards::{assert_lb_owns_project, require_lead_broker};

#[derive(Serialize)]
pub struct LbCashFlowPaymentResponse {
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
    pub selling_agent_id: Option<String>,
}

fn row_to_lb_cashflow_payment(row: sqlx::postgres::PgRow) -> LbCashFlowPaymentResponse {
    LbCashFlowPaymentResponse {
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
        selling_agent_id: row.try_get("selling_agent_id").ok().flatten(),
    }
}

/// GET /me/lb/projects/{id}/payments — direct-buyer + downline collections only.
pub async fn list_project_payments(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<LbCashFlowPaymentResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_lead_broker(&role)?;
    assert_lb_owns_project(&pool, user_id, project_id).await?;

    let (agents_json, lb_roster_id) = load_project_agents(&pool, project_id).await?;
    let seller_ids = collect_direct_and_downline_seller_ids(
        &agents_json,
        lb_roster_id.as_ref().map(|id| id.to_string()).as_deref(),
    );
    if seller_ids.is_empty() {
        return Ok(Json(Page::new(vec![], &page_query, 0)));
    }
    let seller_list: Vec<String> = seller_ids.into_iter().collect();

    let rows = sqlx::query(
        "SELECT p.id, p.contract_id, p.amount, p.method, p.months_covered, p.paid_at,
                p.reference_no, p.bank_name, p.sender_name, p.receiver_name, p.mode_label,
                c.buyer_name, c.lot_block, c.lot_lot, c.term_years, c.term_months,
                c.selling_agent_id, COUNT(*) OVER() AS total_count
           FROM public.payments p
           INNER JOIN public.contracts c ON c.id = p.contract_id
          WHERE c.project_id = $1
            AND c.selling_agent_id = ANY($2)
       ORDER BY p.paid_at DESC, p.created_at DESC
          LIMIT $3 OFFSET $4",
    )
    .bind(project_id)
    .bind(&seller_list)
    .bind(page_query.per_page())
    .bind(page_query.offset())
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load payments")
    })?;

    let total = total_count(&rows);
    Ok(Json(Page::new(
        rows.into_iter().map(row_to_lb_cashflow_payment).collect(),
        &page_query,
        total,
    )))
}
