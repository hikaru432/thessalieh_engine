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

use super::guards::{assert_owns_project, require_upline_role};

/// GET /me/upline/{role_slug}/projects/{project_id}/commission-release-credits — subject only, read-only.
pub async fn list_commission_release_credits(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path((role_slug, project_id)): Path<(String, Uuid)>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<CommissionReleaseCreditResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    let _label = require_upline_role(&pool, &role_slug, &role).await?;
    let subject_id = assert_owns_project(&pool, user_id, project_id, &role_slug)
        .await?
        .to_string();

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
