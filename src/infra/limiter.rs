use axum::extract::ConnectInfo;
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use dashmap::DashMap;
use std::{
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;

/// Idle (no in-flight permit, no new request) entries older than this are
/// dropped by `sweep`, which runs off the periodic gc tick.
const IDLE_TTL: Duration = Duration::from_secs(300);

/// Hard ceiling on distinct IPs tracked at once. Bounds worst-case memory
/// even during a burst that outruns the sweep interval, the same way
/// `api::verified::NonceStore` caps itself instead of growing unbounded.
const MAX_TRACKED_IPS: usize = 50_000;

struct Entry {
    semaphore: Arc<Semaphore>,
    last_access: Instant,
}

#[derive(Clone)]
pub struct ConcurrencyLimiter {
    inner: Arc<DashMap<IpAddr, Entry>>,
    max_per_ip: usize,
}

impl ConcurrencyLimiter {
    pub fn new(max_per_ip: usize) -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
            max_per_ip,
        }
    }

    /// `None` means the IP wasn't already tracked and the tracker is at
    /// capacity, so the caller should reject the request rather than let
    /// the map grow further.
    fn get_semaphore(&self, ip: IpAddr) -> Option<Arc<Semaphore>> {
        if let Some(mut entry) = self.inner.get_mut(&ip) {
            entry.last_access = Instant::now();
            return Some(entry.semaphore.clone());
        }

        if self.inner.len() >= MAX_TRACKED_IPS {
            return None;
        }

        let entry = self.inner.entry(ip).or_insert_with(|| Entry {
            semaphore: Arc::new(Semaphore::new(self.max_per_ip)),
            last_access: Instant::now(),
        });
        Some(entry.semaphore.clone())
    }

    /// Drops IPs with no in-flight permits that haven't been touched within
    /// `IDLE_TTL`. Called from the periodic gc sweep.
    pub fn sweep(&self) {
        let max_per_ip = self.max_per_ip;
        self.inner.retain(|_, e| {
            e.semaphore.available_permits() < max_per_ip || e.last_access.elapsed() < IDLE_TTL
        });
    }
}

pub async fn enforce_concurrency(
    limiter: ConcurrencyLimiter,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = req
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip());

    if let Some(ip) = ip {
        let semaphore = limiter
            .get_semaphore(ip)
            .ok_or(StatusCode::TOO_MANY_REQUESTS)?;
        match semaphore.try_acquire() {
            Ok(permit) => {
                let res = next.run(req).await;
                drop(permit);
                Ok(res)
            }
            Err(_) => Err(StatusCode::TOO_MANY_REQUESTS),
        }
    } else {
        Ok(next.run(req).await)
    }
}
