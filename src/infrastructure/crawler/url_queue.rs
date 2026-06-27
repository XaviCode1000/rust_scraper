//! Concurrent URL queue for crawling
//!
//! Thread-safe URL queue with deduplication.
//!
//! # Rules Applied
//!
//! - **async-no-lock-across-await**: The pending-URL `Vec` is guarded by a
//!   `tokio::sync::Mutex` acquired via `.lock().await` (never
//!   `blocking_lock()`). Each guard is held for nanoseconds across no `.await`
//!   point. The dedup `seen` set is a lock-free `DashSet`.
//! - **mem-with-capacity**: Pre-allocates internal structures.
//! - **mem-u64-dedup**: `seen` stores `u64` hashes (8 B) instead of `String`s
//!   (~150 B), keyed by a per-process `ahash::RandomState` seed.

use dashmap::DashSet;
use tokio::sync::Mutex;
use tracing::debug;

use crate::domain::DiscoveredUrl;

/// Thread-safe URL queue with deduplication
///
/// Following **async-no-lock-across-await**: uses `tokio::sync::Mutex` with
/// `.lock().await` for the pending-URL buffer (held across no `.await`), and a
/// lock-free `DashSet<u64, ahash::RandomState>` for the seen set.
/// Following **mem-with-capacity**: pre-allocates internal storage.
pub struct UrlQueue {
    /// Pending URLs to crawl — the only `tokio::sync::Mutex`, held briefly.
    queue: Mutex<Vec<DiscoveredUrl>>,
    /// Set of URL hashes already enqueued or visited (for deduplication).
    /// `u64` keys (8 B) instead of `String` (~150 B).
    seen: DashSet<u64, ahash::RandomState>,
    /// Per-process randomized hash seed (FR-3). Cloned into `seen` at
    /// construction so both use identical keys.
    rs: ahash::RandomState,
}

impl UrlQueue {
    /// Create a new URL queue
    ///
    /// Following **mem-with-capacity**: pre-allocates internal structures and
    /// a per-process randomized hash seed (FR-3 HashDoS resistance).
    #[must_use]
    pub fn new() -> Self {
        let rs = ahash::RandomState::new();
        Self {
            queue: Mutex::new(Vec::with_capacity(100)),
            seen: DashSet::with_capacity_and_hasher(100, rs.clone()),
            rs,
        }
    }

    /// Push a URL to the queue
    ///
    /// Returns `false` if the URL was already seen (duplicate).
    ///
    /// # Arguments
    ///
    /// * `url` - URL to add
    ///
    /// # Returns
    ///
    /// `true` if added, `false` if duplicate
    pub async fn push(&self, url: DiscoveredUrl) -> bool {
        // Lock-free, atomic check-and-insert via DashSet::insert (no Mutex, no
        // .await). Prevents the race where two tasks both pass contains() and
        // both insert. The hash uses the per-process randomized seed (FR-3/FR-5).
        let hash = self.rs.hash_one(url.url.as_str());
        if !self.seen.insert(hash) {
            debug!("Duplicate URL in queue: {}", url.url);
            return false;
        }

        // Pending-URL Vec is the only tokio::sync::Mutex; the guard is held
        // across no .await (AL-2) — acquired, pushed, dropped.
        let mut queue = self.queue.lock().await;
        queue.push(url);

        true
    }

    /// Pop a URL from the queue
    ///
    /// # Returns
    ///
    /// `Some(DiscoveredUrl)` if queue has URLs, `None` if empty
    pub async fn pop(&self) -> Option<DiscoveredUrl> {
        let mut queue = self.queue.lock().await;
        queue.pop()
    }

    /// Drain all pending URLs from the internal queue into a VecDeque.
    ///
    /// Used to transfer discovered links from the deduplicated `UrlQueue` to
    /// the main crawl loop's `VecDeque` work queue.
    ///
    /// # Returns
    ///
    /// VecDeque of all pending URLs (queue is emptied)
    pub async fn drain_all(&self) -> std::collections::VecDeque<DiscoveredUrl> {
        let mut queue = self.queue.lock().await;
        std::collections::VecDeque::from(std::mem::take(&mut *queue))
    }

    /// Get the current queue length
    ///
    /// # Returns
    ///
    /// Number of URLs in the queue
    pub async fn len(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// Check if the queue is empty
    ///
    /// # Returns
    ///
    /// `true` if queue is empty
    #[must_use]
    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }

    /// Get the number of seen URLs
    ///
    /// Reads the lock-free `DashSet` directly — no mutex acquisition, so this
    /// stays synchronous (AL-3 applies only to methods that acquire the lock).
    ///
    /// # Returns
    ///
    /// Number of URLs that have been seen (added or visited)
    #[must_use]
    pub fn seen_count(&self) -> usize {
        self.seen.len()
    }

    /// Clear the queue (but not the seen set)
    pub async fn clear(&self) {
        self.queue.lock().await.clear();
    }

    /// Get all URLs from the queue (for debugging)
    ///
    /// # Returns
    ///
    /// Vec of all URLs currently in the queue
    #[cfg(test)]
    pub async fn get_all(&self) -> Vec<DiscoveredUrl> {
        self.queue.lock().await.clone()
    }
}

impl Default for UrlQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    fn create_test_url(path: &str) -> DiscoveredUrl {
        let url = Url::parse(&format!("https://example.com{}", path)).unwrap();
        let parent = Url::parse("https://example.com/").unwrap();
        DiscoveredUrl::html(url, 0, parent)
    }

    #[tokio::test]
    async fn test_url_queue_new() {
        let queue = UrlQueue::new();
        assert!(queue.is_empty().await);
        assert_eq!(queue.len().await, 0);
        assert_eq!(queue.seen_count(), 0);
    }

    #[tokio::test]
    async fn test_url_queue_push_pop() {
        let queue = UrlQueue::new();

        let url1 = create_test_url("/page1");
        let url2 = create_test_url("/page2");

        assert!(queue.push(url1).await);
        assert!(queue.push(url2).await);

        assert_eq!(queue.len().await, 2);
        assert_eq!(queue.seen_count(), 2);

        let popped = queue.pop().await;
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().url.path(), "/page2"); // LIFO

        assert_eq!(queue.len().await, 1);
    }

    #[tokio::test]
    async fn test_url_queue_duplicate_detection() {
        let queue = UrlQueue::new();

        let url1 = create_test_url("/page1");
        let url2 = create_test_url("/page1"); // Same URL

        assert!(queue.push(url1).await);
        assert!(!queue.push(url2).await); // Duplicate

        assert_eq!(queue.len().await, 1);
        assert_eq!(queue.seen_count(), 1);
    }

    #[tokio::test]
    async fn test_url_queue_empty_pop() {
        let queue = UrlQueue::new();
        assert!(queue.pop().await.is_none());
    }

    #[tokio::test]
    async fn test_url_queue_clear() {
        let queue = UrlQueue::new();

        queue.push(create_test_url("/page1")).await;
        queue.push(create_test_url("/page2")).await;

        assert_eq!(queue.len().await, 2);

        queue.clear().await;

        assert_eq!(queue.len().await, 0);
        assert_eq!(queue.seen_count(), 2); // Seen set not cleared
    }

    #[tokio::test]
    async fn test_url_queue_multiple_urls() {
        let queue = UrlQueue::new();

        for i in 0..10 {
            let url = create_test_url(&format!("/page{}", i));
            assert!(queue.push(url).await);
        }

        assert_eq!(queue.len().await, 10);
        assert_eq!(queue.seen_count(), 10);

        // Pop all
        for _ in 0..10 {
            assert!(queue.pop().await.is_some());
        }

        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_url_queue_drain_all() {
        let queue = UrlQueue::new();

        queue.push(create_test_url("/page1")).await;
        queue.push(create_test_url("/page2")).await;
        queue.push(create_test_url("/page3")).await;

        assert_eq!(queue.len().await, 3);

        let drained = queue.drain_all().await;

        assert_eq!(drained.len(), 3);
        assert!(queue.is_empty().await);

        // Re-pushing same URLs should fail (dedup via seen set)
        assert!(!queue.push(create_test_url("/page1")).await);
        assert!(!queue.push(create_test_url("/page2")).await);
        assert!(!queue.push(create_test_url("/page3")).await);
    }
}
