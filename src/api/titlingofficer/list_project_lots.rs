use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::admin::lots::{LOT_COLUMNS, LotResponse, row_to_lot};
use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::require_session_user;
use crate::api::users::shared::E;

use super::guards::{assert_to_owns_project, require_titling_officer};

/// GET /me/to/projects/{id}/lots
pub async fn list_project_lots(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(page_query): Query<PageQuery>,
) -> Result<Json<Page<LotResponse>>, E> {
    let (user_id, role) = require_session_user(&pool, &headers).await?;
    require_titling_officer(&role)?;
    assert_to_owns_project(&pool, user_id, project_id).await?;

    let rows = sqlx::query(&format!(
        "SELECT {LOT_COLUMNS}, COUNT(*) OVER() AS total_count FROM public.lots
          WHERE project_id = $1
       ORDER BY block ASC, lot ASC
          LIMIT $2 OFFSET $3",
    ))
    .bind(project_id)
    .bind(page_query.per_page())
    .bind(page_query.offset())
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load lots")
    })?;

    let total = total_count(&rows);
    Ok(Json(Page::new(
        rows.into_iter().map(row_to_lot).collect(),
        &page_query,
        total,
    )))
}
