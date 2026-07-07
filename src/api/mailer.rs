pub async fn send_code(email: &str, code: &str) -> Result<(), String> {
    let mut req = reqwest::Client::new()
        .post("https://loghouse-mailer.reveriontech.workers.dev")
        // .post("https://loghouse-mailer-ms.reveriontech.workers.dev")
        // .post("https://loghouse-mailer-brevo.reveriontech.workers.dev")
        .json(&serde_json::json!({ "email": email, "code": code }));

    // Attach shared secret so the worker rejects calls not from this backend
    if let Ok(secret) = std::env::var("WORKER_SECRET") {
        if !secret.is_empty() {
            req = req.header("X-Worker-Secret", secret);
        }
    }

    let res = req.send().await.map_err(|e| e.to_string())?;
    let status = res.status();
    if status.is_success() {
        Ok(())
    } else {
        let body = res.text().await.unwrap_or_default();
        Err(format!("Worker {status}: {body}"))
    }
}
