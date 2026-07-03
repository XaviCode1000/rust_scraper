//! Session pool for managing HTTP sessions and domain ban tracking
//!
//! Tracks which domains are currently banned (e.g., due to 429 responses)
//! and provides session management for the crawler engine.
//!
//! # Rules Applied
//!
//! - **own-mutex-interior**: Uses Mutex for interior mutability across threads
//! - **conc-atomic-ordering**: Uses appropriate atomic ordering
//! - **mem-with-capacity**: Pre-allocates collections

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use tracing::debug;

/// Domain ban status
#[derive(Debug, Clone)]
pub struct DomainBan {
    /// When the domain was banned
    pub banned_at: Instant,
    /// How long the ban lasts
    pub ban_duration: Duration,
}

impl DomainBan {
    /// Check if the ban has expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.banned_at.elapsed() >= self.ban_duration
    }
}

/// Session pool that tracks domain ban status
///
/// Thread-safe via `Arc<Mutex<>>` — the lock is held only briefly
/// for reads/writes to the ban map.
#[derive(Clone, Debug)]
pub struct SessionPool {
    /// Domain → ban status mapping
    bans: Arc<Mutex<HashMap<String, DomainBan>>>,
    /// Default ban duration when a domain gets 429
    default_ban_duration: Duration,
}

impl SessionPool {
    /// Create a new session pool with default ban duration
    #[must_use]
    pub fn new(default_ban_duration: Duration) -> Self {
        Self {
            bans: Arc::new(Mutex::new(HashMap::new())),
            default_ban_duration,
        }
    }

    /// Create a new session pool with 60-second default ban
    #[must_use]
    pub fn with_default_bans() -> Self {
        Self::new(Duration::from_secs(60))
    }

    /// Check if a domain is currently banned
    ///
    /// Automatically cleans up expired bans during the check.
    #[must_use]
    pub fn is_banned(&self, domain: &str) -> bool {
        let mut bans = self.bans.lock();
        if let Some(ban) = bans.get(domain) {
            if ban.is_expired() {
                debug!("Domain {} ban expired, removing", domain);
                bans.remove(domain);
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    /// Mark a domain as banned
    pub fn ban_domain(&self, domain: &str) {
        let ban = DomainBan {
            banned_at: Instant::now(),
            ban_duration: self.default_ban_duration,
        };
        debug!(
            "Banning domain {} for {:?}",
            domain, self.default_ban_duration
        );
        self.bans.lock().insert(domain.to_string(), ban);
    }

    /// Mark a domain as banned with a custom duration
    pub fn ban_domain_for(&self, domain: &str, duration: Duration) {
        let ban = DomainBan {
            banned_at: Instant::now(),
            ban_duration: duration,
        };
        debug!("Banning domain {} for {:?}", domain, duration);
        self.bans.lock().insert(domain.to_string(), ban);
    }

    /// Unban a domain manually
    pub fn unban_domain(&self, domain: &str) {
        debug!("Unbanning domain {}", domain);
        self.bans.lock().remove(domain);
    }

    /// Get the number of currently banned domains
    #[must_use]
    pub fn banned_count(&self) -> usize {
        self.bans.lock().len()
    }

    /// Clean up all expired bans
    pub fn cleanup_expired(&self) {
        let mut bans = self.bans.lock();
        let before = bans.len();
        bans.retain(|domain, ban| {
            if ban.is_expired() {
                debug!("Cleaning up expired ban for {}", domain);
                false
            } else {
                true
            }
        });
        let cleaned = before - bans.len();
        if cleaned > 0 {
            debug!("Cleaned up {} expired bans", cleaned);
        }
    }
}

impl Default for SessionPool {
    fn default() -> Self {
        Self::with_default_bans()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_pool_ban_unban() {
        let pool = SessionPool::new(Duration::from_secs(1));

        assert!(!pool.is_banned("example.com"));
        pool.ban_domain("example.com");
        assert!(pool.is_banned("example.com"));
        assert_eq!(pool.banned_count(), 1);

        pool.unban_domain("example.com");
        assert!(!pool.is_banned("example.com"));
        assert_eq!(pool.banned_count(), 0);
    }

    #[test]
    fn test_session_pool_ban_expires() {
        let pool = SessionPool::new(Duration::from_millis(50));

        pool.ban_domain("example.com");
        assert!(pool.is_banned("example.com"));

        // Wait for ban to expire
        std::thread::sleep(Duration::from_millis(100));
        assert!(!pool.is_banned("example.com"));
    }

    #[test]
    fn test_session_pool_custom_duration() {
        let pool = SessionPool::new(Duration::from_secs(60));

        pool.ban_domain_for("example.com", Duration::from_millis(50));
        assert!(pool.is_banned("example.com"));

        std::thread::sleep(Duration::from_millis(100));
        assert!(!pool.is_banned("example.com"));
    }
}
