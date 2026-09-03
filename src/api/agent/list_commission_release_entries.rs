use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::admin::commission_release_entries::{CommissionReleaseEntryResponse, row_to_entry};
use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::guards::{assert_agent_on_project, require_agent, resolve_agent_roster_id};

/// GET /me/agent/projects/{id}/commission-release-entries — this agent subject only, read-only.
pub async fn list_commission_release_entries(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<CommissionReleaseEntryResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_agent(&role)?;
    let roster_id = resolve_agent_roster_id(&pool, user_id).await?;
    assert_agent_on_project(&pool, roster_id, project_id).await?;
    let subject_id = roster_id.to_string();

    let rows = sqlx::query(
        "SELECT id, project_id, subject_agent_id, period_start, period_end,
                amount, paid_at, created_at, share_kind, row_key, buyer_label, note, COUNT(*) OVER() AS total_count
           FROM public.commission_release_entries
          WHERE project_id = $1
            AND subject_agent_id = $2
       ORDER BY period_start ASC, paid_at ASC
          LIMIT $3 OFFSET $4",
    )
    .bind(project_id)
    .bind(&subject_id)
    .bind(page_query.per_page())
    .bind(page_query.offset())
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load commission release entries",
        )
    })?;

    let total = total_count(&rows);
    Ok(Json(Page::new(
        rows.into_iter().map(row_to_entry).collect(),
        &page_query,
        total,
    )))
}
