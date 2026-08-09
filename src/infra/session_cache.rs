use dashmap::DashMap;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// How long a session lookup stays valid in memory before falling back to the
/// DB. Short enough that logout/expiry are felt within roughly one page load,
/// long enough to absorb the burst of `require_session_user` calls a single
/// page triggers (every admin/agent/buyer/etc. handler calls it).
const TTL: Duration = Duration::from_secs(30);

struct Entry {
    user_id: Uuid,
    role: String,
    expires_at: i64,
    cached_at: Instant,
}

#[derive(Clone)]
pub struct SessionCache(Arc<DashMap<Uuid, Entry>>);

static GLOBAL: OnceLock<SessionCache> = OnceLock::new();

impl SessionCache {
    /// Installed once from `main`. Handlers reach the same cache through
    /// `global()` instead of threading it through every one of the ~90
    /// `require_session_user` call sites as an `Extension`.
    pub fn install() -> Self {
        let cache = Self(Arc::new(DashMap::new()));
        GLOBAL.set(cache.clone()).ok();
        cache
    }

    pub fn global() -> &'static SessionCache {
        GLOBAL.get().expect("SessionCache::install was not called")
    }

    pub fn get(&self, sid: Uuid, now_unix: i64) -> Option<(Uuid, String)> {
        let entry = self.0.get(&sid)?;
        if entry.cached_at.elapsed() > TTL || entry.expires_at <= now_unix {
            return None;
        }
        Some((entry.user_id, entry.role.clone()))
    }

    pub fn insert(&self, sid: Uuid, user_id: Uuid, role: String, expires_at: i64) {
        self.0.insert(
            sid,
            Entry {
                user_id,
                role,
                expires_at,
                cached_at: Instant::now(),
            },
        );
    }

    /// Called on logout for the session being killed.
    pub fn invalidate(&self, sid: Uuid) {
        self.0.remove(&sid);
    }

    /// Called wherever a user's sessions are bulk-deleted by user_id (re-login,
    /// password reset) so a stale cache entry can't outlive the DB row it was
    /// read from.
    pub fn invalidate_user(&self, user_id: Uuid) {
        self.0.retain(|_, e| e.user_id != user_id);
    }

    /// Drops TTL-stale and DB-expired entries. Called from the periodic gc sweep.
    pub fn sweep(&self, now_unix: i64) {
        self.0
            .retain(|_, e| e.cached_at.elapsed() <= TTL && e.expires_at > now_unix);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache() -> SessionCache {
        SessionCache(Arc::new(DashMap::new()))
    }

    #[test]
    fn hit_returns_cached_user_and_role() {
        let c = cache();
        let sid = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        c.insert(sid, user_id, "Admin".to_string(), 9_999_999_999);
        assert_eq!(c.get(sid, 0), Some((user_id, "Admin".to_string())));
    }

    #[test]
    fn miss_when_absent() {
        let c = cache();
        assert_eq!(c.get(Uuid::new_v4(), 0), None);
    }

    #[test]
    fn miss_once_db_expiry_passes() {
        let c = cache();
        let sid = Uuid::new_v4();
        c.insert(sid, Uuid::new_v4(), "Agent".to_string(), 100);
        assert!(c.get(sid, 99).is_some());
        assert_eq!(c.get(sid, 200), None); // now_unix past expires_at
    }

    #[test]
    fn miss_once_ttl_elapses_even_if_db_expiry_is_future() {
        let c = cache();
        let sid = Uuid::new_v4();
        c.0.insert(
            sid,
            Entry {
                user_id: Uuid::new_v4(),
                role: "Admin".to_string(),
                expires_at: 9_999_999_999,
                cached_at: Instant::now() - (TTL + Duration::from_secs(1)),
            },
        );
        assert_eq!(c.get(sid, 0), None);
    }

    #[test]
    fn invalidate_removes_single_session() {
        let c = cache();
        let sid = Uuid::new_v4();
        c.insert(sid, Uuid::new_v4(), "Admin".to_string(), 9_999_999_999);
        c.invalidate(sid);
        assert_eq!(c.get(sid, 0), None);
    }

    #[test]
    fn invalidate_user_clears_only_that_users_sessions() {
        let c = cache();
        let user = Uuid::new_v4();
        let other = Uuid::new_v4();
        let sid_a = Uuid::new_v4();
        let sid_b = Uuid::new_v4();
        let sid_other = Uuid::new_v4();
        c.insert(sid_a, user, "Agent".to_string(), 9_999_999_999);
        c.insert(sid_b, user, "Agent".to_string(), 9_999_999_999);
        c.insert(sid_other, other, "Admin".to_string(), 9_999_999_999);

        c.invalidate_user(user);

        assert_eq!(c.get(sid_a, 0), None);
        assert_eq!(c.get(sid_b, 0), None);
        assert!(c.get(sid_other, 0).is_some());
    }

    #[test]
    fn sweep_purges_db_expired_and_ttl_stale_but_keeps_fresh() {
        let c = cache();
        let db_expired = Uuid::new_v4();
        let ttl_stale = Uuid::new_v4();
        let fresh = Uuid::new_v4();

        c.insert(db_expired, Uuid::new_v4(), "Agent".to_string(), 100);
        c.0.insert(
            ttl_stale,
            Entry {
                user_id: Uuid::new_v4(),
                role: "Agent".to_string(),
                expires_at: 9_999_999_999,
                cached_at: Instant::now() - (TTL + Duration::from_secs(1)),
            },
        );
        c.insert(fresh, Uuid::new_v4(), "Agent".to_string(), 9_999_999_999);

        c.sweep(200);

        assert_eq!(c.0.len(), 1);
        assert!(c.get(fresh, 200).is_some());
    }
}
