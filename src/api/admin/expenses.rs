use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::pagination::{Page, PageQuery, total_count};
use crate::api::shared::require_admin;
use crate::api::users::shared::E;

#[derive(Serialize)]
pub struct ExpenseCategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct CreateExpenseCategoryInput {
    pub name: String,
}

#[derive(Serialize)]
pub struct ExpenseResponse {
    pub id: Uuid,
    pub category_id: Uuid,
    pub paid_to: String,
    pub description: Option<String>,
    pub amount: f64,
    pub paid_at: String,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct CreateExpenseInput {
    pub category_id: Uuid,
    pub paid_to: String,
    #[serde(default)]
    pub description: Option<String>,
    pub amount: f64,
    pub paid_at: String,
}

#[derive(Deserialize)]
pub struct UpdateExpenseInput {
    pub category_id: Uuid,
    pub paid_to: String,
    #[serde(default)]
    pub description: Option<String>,
    pub amount: f64,
    pub paid_at: String,
}

#[derive(Deserialize)]
pub struct ListExpensesQuery {
    pub category_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct ExpenseCategoryTotal {
    pub category_id: Uuid,
    pub name: String,
    pub total: f64,
}

#[derive(Serialize)]
pub struct ExpensesSummaryResponse {
    pub categories: Vec<ExpenseCategoryTotal>,
    pub total: f64,
}

fn parse_date(value: &str, field: &'static str) -> Result<NaiveDate, E> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            match field {
                "paid_at" => "paid_at must be YYYY-MM-DD",
                _ => "Date must be YYYY-MM-DD",
            },
        )
    })
}

fn format_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn row_to_category(row: sqlx::postgres::PgRow) -> ExpenseCategoryResponse {
    ExpenseCategoryResponse {
        id: row.try_get("id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(0),
    }
}

fn row_to_expense(row: sqlx::postgres::PgRow) -> ExpenseResponse {
    let paid_at: NaiveDate = row.try_get("paid_at").unwrap_or_default();
    ExpenseResponse {
        id: row.try_get("id").unwrap_or_default(),
        category_id: row.try_get("category_id").unwrap_or_default(),
        paid_to: row.try_get("paid_to").unwrap_or_default(),
        description: row.try_get("description").unwrap_or_default(),
        amount: row.try_get("amount").unwrap_or(0.0),
        paid_at: format_date(paid_at),
        created_at: row.try_get("created_at").unwrap_or(0),
    }
}

async fn ensure_category_exists(pool: &PgPool, category_id: Uuid) -> Result<(), E> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM public.expense_categories WHERE id = $1)",
    )
    .bind(category_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to verify expense category",
        )
    })?;
    if !exists {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "category_id does not exist"));
    }
    Ok(())
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::Database(db) if db.code().as_deref() == Some("23505"))
}

pub async fn list_expense_categories(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<ExpenseCategoryResponse>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(
        "SELECT id, name, created_at
           FROM public.expense_categories
       ORDER BY created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load expense categories",
        )
    })?;

    Ok(Json(rows.into_iter().map(row_to_category).collect()))
}

pub async fn create_expense_category(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(p): Json<CreateExpenseCategoryInput>,
) -> Result<Json<ExpenseCategoryResponse>, E> {
    require_admin(&pool, &headers).await?;

    let name = p.name.trim();
    if name.is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "name is required"));
    }

    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "INSERT INTO public.expense_categories (name, created_at)
         VALUES ($1, $2)
      RETURNING id, name, created_at",
    )
    .bind(name)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if is_unique_violation(&e) {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                "A category with this name already exists",
            );
        }
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save expense category",
        )
    })?;

    Ok(Json(row_to_category(row)))
}

pub async fn delete_expense_category(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(category_id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let result = sqlx::query("DELETE FROM public.expense_categories WHERE id = $1")
        .bind(category_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete expense category",
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Expense category not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_expenses(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Query(page_query): Query<PageQuery>,
    Query(filter): Query<ListExpensesQuery>,
) -> Result<Json<Page<ExpenseResponse>>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(
        "SELECT id, category_id, paid_to, description, amount, paid_at, created_at,
                COUNT(*) OVER() AS total_count
           FROM public.expenses
          WHERE ($3::uuid IS NULL OR category_id = $3)
       ORDER BY paid_at DESC, created_at DESC
          LIMIT $1 OFFSET $2",
    )
    .bind(page_query.per_page())
    .bind(page_query.offset())
    .bind(filter.category_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load expenses")
    })?;

    let total = total_count(&rows);
    Ok(Json(Page::new(
        rows.into_iter().map(row_to_expense).collect(),
        &page_query,
        total,
    )))
}

pub async fn create_expense(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(p): Json<CreateExpenseInput>,
) -> Result<Json<ExpenseResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_category_exists(&pool, p.category_id).await?;

    let paid_to = p.paid_to.trim();
    if paid_to.is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "paid_to is required"));
    }
    if !p.amount.is_finite() || p.amount <= 0.0 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "amount must be a positive number",
        ));
    }

    let paid_at = parse_date(&p.paid_at, "paid_at")?;
    let description = p.description.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let now = Utc::now().timestamp();

    let row = sqlx::query(
        "INSERT INTO public.expenses (
            category_id, paid_to, description, amount, paid_at, created_at
         ) VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING id, category_id, paid_to, description, amount, paid_at, created_at",
    )
    .bind(p.category_id)
    .bind(paid_to)
    .bind(description)
    .bind(p.amount)
    .bind(paid_at)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save expense")
    })?;

    Ok(Json(row_to_expense(row)))
}

pub async fn update_expense(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(expense_id): Path<Uuid>,
    Json(p): Json<UpdateExpenseInput>,
) -> Result<Json<ExpenseResponse>, E> {
    require_admin(&pool, &headers).await?;
    ensure_category_exists(&pool, p.category_id).await?;

    let paid_to = p.paid_to.trim();
    if paid_to.is_empty() {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "paid_to is required"));
    }
    if !p.amount.is_finite() || p.amount <= 0.0 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "amount must be a positive number",
        ));
    }

    let paid_at = parse_date(&p.paid_at, "paid_at")?;
    let description = p.description.as_deref().map(str::trim).filter(|s| !s.is_empty());

    let row = sqlx::query(
        "UPDATE public.expenses
            SET category_id = $1, paid_to = $2, description = $3, amount = $4, paid_at = $5
          WHERE id = $6
      RETURNING id, category_id, paid_to, description, amount, paid_at, created_at",
    )
    .bind(p.category_id)
    .bind(paid_to)
    .bind(description)
    .bind(p.amount)
    .bind(paid_at)
    .bind(expense_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update expense")
    })?
    .ok_or((StatusCode::NOT_FOUND, "Expense not found"))?;

    Ok(Json(row_to_expense(row)))
}

pub async fn delete_expense(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Path(expense_id): Path<Uuid>,
) -> Result<StatusCode, E> {
    require_admin(&pool, &headers).await?;

    let result = sqlx::query("DELETE FROM public.expenses WHERE id = $1")
        .bind(expense_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("DB: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete expense")
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Expense not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Per-category totals plus a grand total — the company-wide "Cash Out" overview
/// cards on the Expenses page. A `LEFT JOIN` keeps categories with zero recorded
/// expenses in the list (at total = 0) instead of silently dropping them.
pub async fn expenses_summary(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
) -> Result<Json<ExpensesSummaryResponse>, E> {
    require_admin(&pool, &headers).await?;

    let rows = sqlx::query(
        "SELECT c.id AS category_id, c.name AS name, COALESCE(SUM(e.amount), 0) AS total
           FROM public.expense_categories c
           LEFT JOIN public.expenses e ON e.category_id = c.id
       GROUP BY c.id, c.name
       ORDER BY c.created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DB: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to load expenses summary",
        )
    })?;

    let categories: Vec<ExpenseCategoryTotal> = rows
        .into_iter()
        .map(|row| ExpenseCategoryTotal {
            category_id: row.try_get("category_id").unwrap_or_default(),
            name: row.try_get("name").unwrap_or_default(),
            total: row.try_get("total").unwrap_or(0.0),
        })
        .collect();

    let total = categories.iter().map(|c| c.total).sum();

    Ok(Json(ExpensesSummaryResponse { categories, total }))
}
