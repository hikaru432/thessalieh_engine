use axum::http::{HeaderMap, HeaderValue, StatusCode};
use serde::Serialize;
use uuid::Uuid;

pub type E = (StatusCode, &'static str);

pub const SESSION_MAX_AGE: i64 = 24 * 3600; // 1 day

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub phone: Option<String>,
    pub role: String,
    pub expires_at: i64,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: &'static str,
}

pub fn extract_session_id(headers: &HeaderMap) -> Option<Uuid> {
    extract_cookie_value(headers, "session")?
        .parse::<Uuid>()
        .ok()
}

pub fn extract_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie_str = headers.get("cookie")?.to_str().ok()?;
    let prefix = format!("{name}=");
    for part in cookie_str.split(';') {
        if let Some(val) = part.trim().strip_prefix(&prefix) {
            return Some(val.trim().to_string());
        }
    }
    None
}

fn cookie_security_attrs() -> &'static str {
    if std::env::var("COOKIE_SECURE").is_ok() {
        "SameSite=None; Secure"
    } else {
        "SameSite=Lax"
    }
}

pub fn session_cookie(session_id: Uuid) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "session={session_id}; HttpOnly; {}; Path=/; Max-Age={SESSION_MAX_AGE}",
        cookie_security_attrs()
    ))
    .expect("valid cookie value")
}

pub fn new_csrf_token() -> String {
    Uuid::new_v4().to_string()
}

pub fn csrf_cookie(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "csrf={token}; {}; Path=/; Max-Age={SESSION_MAX_AGE}",
        cookie_security_attrs()
    ))
    .expect("valid cookie value")
}

pub fn clear_session_cookie() -> HeaderValue {
    HeaderValue::from_str(&format!(
        "session=; HttpOnly; {}; Path=/; Max-Age=0",
        cookie_security_attrs()
    ))
    .expect("valid cookie value")
}

pub fn clear_csrf_cookie() -> HeaderValue {
    HeaderValue::from_str(&format!(
        "csrf=; {}; Path=/; Max-Age=0",
        cookie_security_attrs()
    ))
    .expect("valid cookie value")
}

pub fn is_valid_email(email: &str) -> bool {
    let parts: Vec<&str> = email.splitn(2, '@').collect();
    parts.len() == 2 && !parts[0].is_empty() && parts[1].contains('.') && parts[1].len() > 2
}

pub fn is_strong_password(pw: &str) -> bool {
    pw.len() >= 8
        && pw.chars().any(|c| c.is_uppercase())
        && pw.chars().any(|c| !c.is_alphanumeric())
}
