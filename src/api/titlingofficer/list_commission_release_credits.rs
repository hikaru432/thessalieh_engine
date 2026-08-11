use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::admin::commission_release_credits::{CommissionReleaseCreditResponse, row_to_credit};
use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::guards::{assert_to_owns_project, require_titling_officer};

/// GET /me/to/projects/{id}/commission-release-credits — TO subject only, read-only.
pub async fn list_commission_release_credits(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<CommissionReleaseCreditResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_titling_officer(&role)?;
    let subject_id = assert_to_owns_project(&pool, user_id, project_id).await?.to_string();

    let rows = sqlx::query(
        "SELECT id, project_id, subject_agent_id, share_kind, amount, paid_at, note, created_at,
                COUNT(*) OVER() AS total_count
           FROM public.commission_release_credits
          WHERE project_id = $1
            AND subject_agent_id = $2
       ORDER BY paid_at ASC
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
            "Failed to load commission release credits",
        )
    })?;

    let total = total_count(&rows);
    Ok(Json(Page::new(
        rows.into_iter().map(row_to_credit).collect(),
        &page_query,
        total,
    )))
}
