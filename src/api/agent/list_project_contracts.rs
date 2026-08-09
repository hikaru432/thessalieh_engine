use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::admin::contracts::{
    CONTRACT_COLUMNS_WITH_TOTALS, ContractResponse, row_to_contract,
};
use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::guards::{require_agent, resolve_agent_roster_id, assert_agent_on_project};

/// GET /me/agent/projects/{id}/contracts
pub async fn list_project_contracts(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<ContractResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_agent(&role)?;
    let roster_id = resolve_agent_roster_id(&pool, user_id).await?;
    assert_agent_on_project(&pool, roster_id, project_id).await?;

    let rows = sqlx::query(&format!(
        "SELECT {CONTRACT_COLUMNS_WITH_TOTALS}, COUNT(*) OVER() AS total_count
           FROM public.contracts c
           LEFT JOIN public.users bu ON bu.id = c.buyer_user_id
           LEFT JOIN public.payments p ON p.contract_id = c.id
          WHERE c.project_id = $1
            AND c.selling_agent_id = $2
       GROUP BY c.id
       ORDER BY c.created_at ASC
          LIMIT $3 OFFSET $4",
    ))
    .bind(project_id)
    .bind(roster_id.to_string())
    .bind(page_query.per_page())
    .bind(page_query.offset())
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load contracts")
    })?;

    let total = total_count(&rows);
    Ok(Json(Page::new(
        rows.into_iter().map(row_to_contract).collect(),
        &page_query,
        total,
    )))
}
