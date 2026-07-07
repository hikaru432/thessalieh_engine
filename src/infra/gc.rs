use std::time::Duration;

use chrono::Utc;
use sqlx::PgPool;
use tokio::time;

const GC_INTERVAL_SECS: u64 = 60;

pub fn spawn(pool: PgPool) {
    tokio::spawn(async move {
        let mut ticker = time::interval(Duration::from_secs(GC_INTERVAL_SECS));
        ticker.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            let now = Utc::now().timestamp();

            match sqlx::query!(
                "DELETE FROM public.verification_codes WHERE expires_at <= $1",
                now
            )
            .execute(&pool)
            .await
            {
                Ok(r) if r.rows_affected() > 0 => {
                    tracing::info!(purged = r.rows_affected(), "gc: expired verification codes");
                }
                Ok(_) => {}
                Err(e) => tracing::error!("gc verification_codes: {e}"),
            }

            match sqlx::query!("DELETE FROM public.sessions WHERE expires_at <= $1", now)
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