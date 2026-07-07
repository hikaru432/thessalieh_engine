use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::api::users::shared::extract_cookie_value;

const CSRF_HEADER: &str = "x-csrf-token";

pub async fn enforce_csrf(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    if is_safe_method(req.method()) || is_exempt_path(path) || is_automation_path(path) {
        return Ok(next.run(req).await);
    }

    let headers = req.headers();
    let cookie_token = extract_cookie_value(headers, "csrf");
    let header_token = headers
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty());

    match (cookie_token.as_deref(), header_token) {
        (Some(cookie), Some(header)) if cookie == header => Ok(next.run(req).await),
        _ => Err(StatusCode::FORBIDDEN),
    }
}

fn is_safe_method(method: &Method) -> bool {
    matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS)
}

fn is_exempt_path(path: &str) -> bool {
    matches!(
        path,
        "/auth/register"
            | "/auth/verify"
            | "/auth/login"
            | "/auth/logout"
            | "/auth/password-reset/request"
            | "/auth/password-reset/confirm"
            | "/enquiries/quick"
            | "/enquiries/detailed"
    )
}

/// Every `/automation/*` endpoint is server-to-server (called by n8n) and
/// protected by a Bearer-token check in its handler (see
/// `api::automation::shared::require_automation_token`). It can never carry
/// a CSRF cookie + header pair, so the browser-targeted CSRF guard would
/// always 403 these calls. Prefix-match means new automation endpoints
/// don't need to remember to update this list.
fn is_automation_path(path: &str) -> bool {
    path.starts_with("/automation/")
}
