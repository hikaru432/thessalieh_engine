use base64::{Engine, engine::general_purpose::STANDARD};

#[derive(Clone)]
pub struct SupabaseStorage {
    client: reqwest::Client,
    project_url: String,
    secret_key: String,
    bucket: String,
}

impl SupabaseStorage {
    pub fn new(project_url: String, secret_key: String, bucket: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            project_url,
            secret_key,
            bucket,
        }
    }

    /// Upload raw bytes to `<bucket>/<path>` and return the public URL.
    ///
    /// Sends a year-long immutable `Cache-Control` so any reader of the
    /// returned URL (Cloudflare edge, browser cache, the Worker proxy)
    /// caches aggressively. Storage paths are UUID-keyed by the backend so
    /// the bytes at a given path never change once written — `immutable`
    /// is safe.
    pub async fn upload(
        &self,
        path: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, String> {
        let url = format!(
            "{}/storage/v1/object/{}/{}",
            self.project_url, self.bucket, path
        );

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .header("Content-Type", content_type)
            .header("Content-Disposition", "attachment")
            .header("Cache-Control", "public, max-age=31536000, immutable")
            .header("x-upsert", "true")
            .body(data)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(format!("Storage {status}: {body}"));
        }

        Ok(format!(
            "{}/storage/v1/object/public/{}/{}",
            self.project_url, self.bucket, path
        ))
    }

    /// Delete a single object at `<bucket>/<path>`.
    pub async fn delete(&self, path: &str) -> Result<(), String> {
        let url = format!(
            "{}/storage/v1/object/{}/{}",
            self.project_url, self.bucket, path
        );

        let res = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        // 404 is treated as already-gone so this is idempotent.
        if res.status().as_u16() == 404 {
            return Ok(());
        }
        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(format!("Storage {status}: {body}"));
        }
        Ok(())
    }

    /// Extract the bucket-relative path from a public URL produced by `upload`.
    /// Returns `None` if the URL does not match the expected shape.
    pub fn path_from_url(&self, url: &str) -> Option<String> {
        let prefix = format!(
            "{}/storage/v1/object/public/{}/",
            self.project_url, self.bucket
        );
        url.strip_prefix(&prefix).map(|s| s.to_string())
    }

    /// Best-effort delete of an object identified by its public URL. Logs a
    /// warning on failure but never errors — used as a side effect after a
    /// successful DB write to clean up orphaned storage objects. URLs that
    /// don't match this bucket's prefix are silently skipped.
    pub async fn delete_url(&self, url: &str) {
        let Some(path) = self.path_from_url(url) else {
            return;
        };
        if let Err(e) = self.delete(&path).await {
            tracing::warn!(%url, "storage cleanup delete failed: {e}");
        }
    }

    /// Best-effort batch delete. Convenience over `delete_url` for the
    /// product/review update paths that replace whole child sets at once.
    pub async fn delete_urls<I, S>(&self, urls: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for url in urls {
            self.delete_url(url.as_ref()).await;
        }
    }
}

/// Decode a standard base64 string to bytes.
pub fn decode_b64(s: &str) -> Result<Vec<u8>, String> {
    STANDARD.decode(s).map_err(|e| e.to_string())
}

/// Infer MIME type from file extension.
pub fn content_type(name: &str) -> &'static str {
    match name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "pdf" => "application/pdf",
        "dwg" => "application/acad",
        _ => "application/octet-stream",
    }
}
