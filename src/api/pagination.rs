use serde::{Deserialize, Serialize};
use sqlx::Row;

pub const DEFAULT_PER_PAGE: i64 = 50;
pub const MAX_PER_PAGE: i64 = 200;

/// `?page=`/`?per_page=` query params shared by every paginated list endpoint.
/// Add as an extra `Query<PageQuery>` extractor alongside any endpoint-specific
/// query struct — axum parses each `Query<T>` independently from the same
/// querystring, so this composes with filters like `from`/`to` without a shared type.
#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl PageQuery {
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(DEFAULT_PER_PAGE).clamp(1, MAX_PER_PAGE)
    }

    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }
}

#[derive(Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
    pub has_more: bool,
}

/// Reads the `COUNT(*) OVER()` window column a paginated query includes
/// alongside its normal columns; 0 for an empty page.
pub fn total_count(rows: &[sqlx::postgres::PgRow]) -> i64 {
    rows.first()
        .and_then(|r| r.try_get::<i64, _>("total_count").ok())
        .unwrap_or(0)
}

impl<T> Page<T> {
    /// `total` is the full row count (pre-LIMIT), typically read off a
    /// `COUNT(*) OVER()` window column included in the same paginated query.
    pub fn new(items: Vec<T>, query: &PageQuery, total: i64) -> Self {
        let per_page = query.per_page();
        let page = query.page();
        let total_pages = if total == 0 {
            1
        } else {
            (total + per_page - 1) / per_page
        };
        Page {
            items,
            page,
            per_page,
            total,
            total_pages,
            has_more: page < total_pages,
        }
    }
}
