use axum::{
    Extension, Json,
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::insert::{InsertUserInput, insert_user};
use super::shared::E;
use crate::api::shared::require_admin;

#[derive(Deserialize)]
pub struct CreateUserInput {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub access_token: String,
}

#[derive(Serialize)]
pub struct CreatedUserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub phone: Option<String>,
    pub role: String,
}

pub async fn create_user(
    Extension(pool): Extension<PgPool>,
    headers: HeaderMap,
    Json(p): Json<CreateUserInput>,
) -> Result<(StatusCode, Json<CreatedUserResponse>), E> {
    require_admin(&pool, &headers).await?;

    let result = insert_user(
        &pool,
        InsertUserInput {
            username: p.username,
            password: p.password,
            access_token: p.access_token,
        },
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreatedUserResponse {
            id: result.user_id,
            username: result.username,
            email: String::new(),
            phone: None,
            role: result.role.to_string(),
        }),
    ))
}
