use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::admin::contract_split_history::{ContractSplitHistoryResponse, row_to_entry};
use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::guards::{assert_to_owns_project, require_titling_officer};

/// GET /me/to/projects/{id}/contract-split-history — read-only, whole project (a
/// contract's split history isn't scoped to any one subject, same as admin's version;
/// this Titling Officer already sees every contract in the project via list_project_contracts).
/// Needed so this subject's own commission schedule can be recomputed the same way
/// admin's is — without this, a mid-stream split change silently isn't reflected here.
pub async fn list_contract_split_history(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<ContractSplitHistoryResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_titling_officer(&role)?;
    assert_to_owns_project(&pool, user_id, project_id).await?;

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
