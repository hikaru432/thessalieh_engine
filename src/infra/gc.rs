use std::time::Duration;

use chrono::Utc;
use sqlx::PgPool;
use tokio::time;

use super::limiter::ConcurrencyLimiter;
use super::rate::RateLimiter;
use super::session_cache::SessionCache;

const GC_INTERVAL_SECS: u64 = 60;

pub fn spawn(pool: PgPool, concurrency_limiter: ConcurrencyLimiter, rate_limiter: RateLimiter) {
    tokio::spawn(async move {
        let mut ticker = time::interval(Duration::from_secs(GC_INTERVAL_SECS));
        ticker.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            let now = Utc::now().timestamp();

            SessionCache::global().sweep(now);
            concurrency_limiter.sweep();
            rate_limiter.sweep();

            match sqlx::query("DELETE FROM public.verification_codes WHERE expires_at <= $1")
                .bind(now)
                .execute(&pool)
                .await
            {
                Ok(r) if r.rows_affected() > 0 => {
                    tracing::info!(purged = r.rows_affected(), "gc: expired verification codes");
                }
                Ok(_) => {}
                Err(e) => tracing::error!("gc verification_codes: {e}"),
            }

            match sqlx::query("DELETE FROM public.sessions WHERE expires_at <= $1")
                .bind(now)
                .execute(&pool)
                .await
            {
                Ok(r) if r.rows_affected() > 0 => {
                    tracing::info!(purged = r.rows_affected(), "gc: expired sessions");
                }
                Ok(_) => {}
                Err(e) => tracing::error!("gc sessions: {e}"),
            }
        }
    });
}