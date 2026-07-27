use axum::{Extension, Json, http::HeaderMap, http::StatusCode};
use sqlx::PgPool;

use crate::api::admin::contracts::{
    CONTRACT_COLUMNS_WITH_TOTALS, ContractDetail, row_to_contract, row_to_payment,
};
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

/// Lists contracts owned by the session user (`buyer_user_id`), each with payments.
pub async fn list_my_contracts(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<ContractDetail>>, E> {
    let (user_id, _role) = require_session_user(&pool, &headers).await?;

    let rows = sqlx::query(&format!(
        "SELECT {CONTRACT_COLUMNS_WITH_TOTALS}
           FROM public.contracts c
           LEFT JOIN public.users bu ON bu.id = c.buyer_user_id
           LEFT JOIN public.payments p ON p.contract_id = c.id
          WHERE c.buyer_user_id = $1
       GROUP BY c.id
       ORDER BY c.created_at ASC",
    ))
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load contracts")
    })?;

    let mut details = Vec::with_capacity(rows.len());
    for row in rows {
        let contract = row_to_contract(row);
        let payment_rows = sqlx::query(
            "SELECT id, contract_id, amount, method, months_covered, paid_at,
                    reference_no, bank_name, sender_name, receiver_name, mode_label
               FROM public.payments WHERE contract_id = $1
           ORDER BY paid_at ASC, created_at ASC",
        )
        .bind(contract.id)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load payments")
        })?;

        details.push(ContractDetail {
            contract,
            payments: payment_rows.into_iter().map(row_to_payment).collect(),
        });
    }

    Ok(Json(details))
}
