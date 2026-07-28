pub mod api;
pub mod infra;
mod routes;

use std::{env, sync::Arc, time::Duration};

use axum::http::HeaderValue;
use axum::{
    extract::Extension,
    http::Method,
    middleware::{self},
};
use dashmap::DashMap;
use dotenvy::dotenv;
use infra::csrf::enforce_csrf;
use infra::db::init_db_pool;
use infra::limiter::{ConcurrencyLimiter, enforce_concurrency};
use infra::rate::{RateLimiter, enforce_rate_limit};
use routes::api_routes;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let limiter = ConcurrencyLimiter::new(30);

    let device_secret = env::var("DEVICE_SECRET").unwrap_or_default();
    let rate_limiter = RateLimiter::new(100000, Duration::from_secs(60), device_secret);

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let client_url_raw =
        env::var("CLIENT_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let origins: Vec<HeaderValue> = client_url_raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            HeaderValue::from_str(s).unwrap_or_else(|_| panic!("Invalid CLIENT_URL entry: {s}"))
        })
        .collect();

    if origins.is_empty() {
        panic!("CLIENT_URL must contain at least one origin");
    }

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::HeaderName::from_static("x-wallet"),
            axum::http::header::HeaderName::from_static("x-csrf-token"),
            axum::http::header::HeaderName::from_static("x-region"),
        ])
        .expose_headers([
            axum::http::header::HeaderName::from_static("x-blake3-hash"),
            axum::http::header::HeaderName::from_static("x-document-id"),
            axum::http::header::HeaderName::from_static("x-csrf-token"),
        ])
        .allow_credentials(true);

    let nonce_store: api::verified::NonceStore = Arc::new(DashMap::new());

    let db_pool = init_db_pool().await;

    infra::gc::spawn(db_pool.clone());

    let supabase_url = env::var("SUPABASE_URL").expect("SUPABASE_URL must be set");
    let supabase_jwt =
        env::var("SUPABASE_SERVICE_ROLE_JWT").expect("SUPABASE_SERVICE_ROLE_JWT must be set");
    let supabase_bucket =
        env::var("SUPABASE_STORAGE_BUCKET").unwrap_or_else(|_| "loghouse".to_string());
    let storage = infra::storage::SupabaseStorage::new(supabase_url, supabase_jwt, supabase_bucket);

    let app = api_routes::routes()
        .layer(Extension(nonce_store))
        .layer(Extension(db_pool))
        .layer(Extension(storage))
        .layer(middleware::from_fn(move |req, next| {
            enforce_rate_limit(rate_limiter.clone(), req, next)
        }))
        .layer(middleware::from_fn(move |req, next| {
            enforce_concurrency(limiter.clone(), req, next)
        }))
        .layer(middleware::from_fn(enforce_csrf))
        .layer(cors)
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static("default-src 'none'"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        ))
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{port}");
    println!("Server running on http://{addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}
