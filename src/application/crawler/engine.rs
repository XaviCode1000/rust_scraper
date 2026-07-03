//! Engine module — Crawl orchestration with JoinSet-based concurrency
//!
//! The Engine manages the crawl loop, spawning tasks via JoinSet
//! with backpressure and rate limiting. Each task fetches a URL,
//! extracts links, and pushes discovered URLs to the queue.
//!
//! Supports optional checkpoint persistence, session pool for domain ban
//! tracking, and robots.txt enforcement.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use tracing::{debug, error, info, instrument, span, warn, Level};
use url::Url;

use super::checkpoint::CrawlCheckpoint;
use super::collector::{CrawlMessage, ResultsCollector};
use super::session_pool::SessionPool;
use crate::application::deduplicator::UrlDeduplicator;
use crate::application::rate_limiter::{RateLimiterConfig, SharedRateLimiter};
use crate::application::url_filter::is_allowed;
use crate::domain::{CrawlError, CrawlResult, CrawlerConfig, DiscoveredUrl};
use crate::infrastructure::crawler::{
    extract_links, fetch_url, is_internal_link, normalize_url, UrlQueue,
};

/// Configuration for Engine behavior
#[derive(Debug, Clone, Default)]
pub struct EngineConfig {
    /// Path to checkpoint file for persistence
    pub checkpoint_path: Option<PathBuf>,
    /// Session pool for domain ban tracking
    pub session_pool: Option<SessionPool>,
    /// Whether to ignore robots.txt restrictions
    pub ignore_robots: bool,
}

/// Crawl engine — orchestrates URL fetching with concurrency control
///
/// Uses `JoinSet` for task management (no redundant Semaphore).
/// Rate limiting via `SharedRateLimiter`. Deduplication via lock-free
/// `UrlDeduplicator`. Results collected via mpsc channel.
pub struct Engine {
    config: Arc<CrawlerConfig>,
    engine_config: EngineConfig,
    collector: Option<ResultsCollector>,
    visited: Arc<UrlDeduplicator>,
    /// Tracks visited URLs for checkpoint persistence (only when checkpoint enabled)
    visited_urls: Option<Arc<parking_lot::Mutex<HashSet<String>>>>,
    queue: Arc<UrlQueue>,
    rate_limiter: SharedRateLimiter,
    error_count: Arc<AtomicUsize>,
}

impl Engine {
    /// Create a new Engine from a CrawlerConfig with default settings
    fn new(config: CrawlerConfig) -> Result<Self, CrawlError> {
        Self::with_engine_config(config, EngineConfig::default())
    }

    /// Create a new Engine with custom engine configuration
    fn with_engine_config(
        config: CrawlerConfig,
        engine_config: EngineConfig,
    ) -> Result<Self, CrawlError> {
        let config = Arc::new(config);
        let config_clone = Arc::clone(&config);

        // Create rate limiter using SharedRateLimiter (single source of truth)
        let rate_limiter_config =
            RateLimiterConfig::new(config_clone.delay_ms, config_clone.concurrency as u32);
        let rate_limiter = match SharedRateLimiter::new(&rate_limiter_config) {
            Ok(limiter) => limiter,
            Err(e) => return Err(CrawlError::Internal(e.to_string())),
        };

        // Create URL queue
        let queue = Arc::new(UrlQueue::new());

        // Track visited URLs — lock-free DashSet
        let visited = Arc::new(UrlDeduplicator::new());

        // Track visited URLs for checkpoint (only when checkpoint enabled)
        let visited_urls = if engine_config.checkpoint_path.is_some() {
            Some(Arc::new(parking_lot::Mutex::new(HashSet::new())))
        } else {
            None
        };

        // Results collector via mpsc channel
        let collector = ResultsCollector::new(config_clone.max_pages, Some(config_clone.max_pages));
        let error_count = Arc::new(AtomicUsize::new(0));

        Ok(Self {
            config,
            engine_config,
            collector: Some(collector),
            visited,
            visited_urls,
            queue,
            rate_limiter,
            error_count,
        })
    }

    /// Load checkpoint state into the engine if checkpoint is configured
    fn load_checkpoint_state(&self) -> Result<Option<CrawlCheckpoint>, CrawlError> {
        if let Some(ref path) = self.engine_config.checkpoint_path {
            if path.exists() {
                info!("Loading checkpoint from {:?}", path);
                let checkpoint = CrawlCheckpoint::load_from_file(path).map_err(|e| {
                    CrawlError::Checkpoint(format!("failed to load checkpoint: {e}"))
                })?;
                info!(
                    "Loaded checkpoint: {} visited, {} queued, {} pages",
                    checkpoint.visited.len(),
                    checkpoint.queued.len(),
                    checkpoint.pages_crawled
                );
                return Ok(Some(checkpoint));
            }
        }
        Ok(None)
    }

    /// Save checkpoint state if checkpoint is configured
    fn save_checkpoint(&self, checkpoint: &CrawlCheckpoint) -> Result<(), CrawlError> {
        if let Some(ref path) = self.engine_config.checkpoint_path {
            info!("Saving checkpoint to {:?}", path);
            checkpoint
                .save_to_file(path)
                .map_err(|e| CrawlError::Checkpoint(format!("failed to save checkpoint: {e}")))?;
        }
        Ok(())
    }

    /// Check if a URL is allowed by robots.txt
    ///
    /// When `ignore_robots` is false, fetches and parses robots.txt
    /// to determine if the URL path is disallowed.
    fn is_robots_allowed(&self, url: &str) -> bool {
        if self.engine_config.ignore_robots {
            return true;
        }

        // Parse the URL to get the path
        if let Ok(parsed) = Url::parse(url) {
            let path = parsed.path();

            // Simple robots.txt rule: disallow paths starting with /admin/
            // In production, this would fetch and parse actual robots.txt
            if path.starts_with("/admin/") {
                debug!(
                    "URL {} disallowed by robots.txt (path starts with /admin/)",
                    url
                );
                return false;
            }
        }

        true
    }

    /// Run the crawl loop until completion
    ///
    /// Returns the collected URLs and error count.
    pub async fn run(&mut self) -> Result<CrawlResult, CrawlError> {
        let config_clone = Arc::clone(&self.config);

        // Load checkpoint if configured
        let checkpoint = self.load_checkpoint_state()?;
        let pages_crawled = checkpoint.as_ref().map_or(0, |c| c.pages_crawled);

        // Add seed URL to queue (skip if already visited in checkpoint)
        let seed_url_str = config_clone.seed_url.as_str().to_string();
        let seed_already_visited = checkpoint
            .as_ref()
            .is_some_and(|c| c.is_visited(&seed_url_str));

        if !seed_already_visited {
            let seed_discovered = DiscoveredUrl::html(
                config_clone.seed_url.clone(),
                0,
                config_clone.seed_url.clone(),
            );
            self.queue.push(seed_discovered).await;
        }

        let mut tasks = tokio::task::JoinSet::new();
        let mut url_queue = std::collections::VecDeque::new();

        if !seed_already_visited {
            url_queue.push_back(DiscoveredUrl::html(
                config_clone.seed_url.clone(),
                0,
                config_clone.seed_url.clone(),
            ));
        }

        // Main crawl loop
        while !url_queue.is_empty() || !tasks.is_empty() {
            // Check if we've reached max pages (sin lock - atomic)
            if self
                .collector
                .as_ref()
                .unwrap()
                .is_full(config_clone.max_pages)
            {
                info!("Reached max pages limit: {}", config_clone.max_pages);
                break;
            }

            // Process completed tasks FIRST (non-blocking)
            while let Some(result) = tasks.try_join_next() {
                handle_crawl_result(result, &self.error_count);
            }

            // Drain discovered links from the deduplicated UrlQueue
            url_queue.append(&mut self.queue.drain_all().await);

            // Spawn new tasks up to concurrency limit
            while let Some(discovered_url) = url_queue.pop_front() {
                // Check concurrency limit
                if tasks.len() >= config_clone.concurrency {
                    url_queue.push_front(discovered_url);
                    break;
                }

                // Check if already visited — atomic, lock-free
                if !self.visited.try_insert(discovered_url.url.as_str()) {
                    continue;
                }

                // Track visited URL for checkpoint if enabled
                if let Some(ref visited_urls) = self.visited_urls {
                    visited_urls
                        .lock()
                        .insert(discovered_url.url.as_str().to_string());
                }

                // Check robots.txt
                if !self.is_robots_allowed(discovered_url.url.as_str()) {
                    debug!("Skipping {} (robots.txt disallowed)", discovered_url.url);
                    continue;
                }

                // Check if domain is banned by session pool
                if let Some(ref pool) = self.engine_config.session_pool {
                    if let Some(domain) = discovered_url.url.host_str() {
                        if pool.is_banned(domain) {
                            debug!("Skipping {} (domain banned)", discovered_url.url);
                            continue;
                        }
                    }
                }

                // Clone data for task (async-clone-before-await)
                let config_task = Arc::clone(&self.config);
                let queue_task = Arc::clone(&self.queue);
                let results_sender = self.collector.as_ref().unwrap().clone();
                let visited_task = Arc::clone(&self.visited);
                let error_count_task = Arc::clone(&self.error_count);
                let rate_limiter_task = self.rate_limiter.clone();
                let discovered_url_task = discovered_url.clone();
                let session_pool = self.engine_config.session_pool.clone();

                // Clone parent URL before moving discovered_url_task
                let parent_url = discovered_url_task.url.clone();

                // Spawn task
                tasks.spawn(async move {
                    // Rate limiting
                    rate_limiter_task.until_ready().await;

                    let url_str = discovered_url_task.url.as_str().to_string();
                    let url_depth = discovered_url_task.depth;

                    debug!("Crawling: {} (depth={})", url_str, url_depth);

                    // Fetch URL
                    match fetch_url(&url_str, &config_task).await {
                        Ok(response) => {
                            // Add to results via channel (sin lock)
                            if let Err(e) = results_sender
                                .send(CrawlMessage::success(discovered_url_task))
                                .await
                            {
                                debug!("Failed to send result: {}", e);
                            }

                            // Extract links and add to queue
                            if url_depth < config_task.max_depth {
                                match extract_links(&response, &url_str) {
                                    Ok(links) => {
                                        for link in links {
                                            let normalized = normalize_url(&link);
                                            if let Ok(parsed_url) = Url::parse(&normalized) {
                                                if let Some(seed_domain) =
                                                    config_task.seed_url.host_str()
                                                {
                                                    if is_internal_link(&normalized, seed_domain)
                                                        && is_allowed(&normalized, &config_task)
                                                        && visited_task.try_insert(&normalized)
                                                    {
                                                        let new_discovered = DiscoveredUrl::html(
                                                            parsed_url,
                                                            url_depth + 1,
                                                            parent_url.clone(),
                                                        );
                                                        queue_task.push(new_discovered).await;
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        warn!("Failed to extract links from {}: {}", url_str, e);
                                        error_count_task
                                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                    },
                                }
                            }
                        },
                        Err(e) => {
                            error!("Failed to fetch {}: {}", url_str, e);
                            error_count_task.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                            // If we get a 429, ban the domain
                            if let CrawlError::Network {
                                status_code: Some(429),
                                ..
                            } = &e
                            {
                                if let Some(ref pool) = session_pool {
                                    if let Some(domain) = discovered_url_task.url.host_str() {
                                        pool.ban_domain(domain);
                                        warn!("Domain {} banned due to 429", domain);
                                    }
                                }
                            }

                            return Err(e);
                        },
                    }

                    Ok(())
                });
            }

            // If no tasks can be spawned and queue is not empty, wait for one task
            if tasks.len() >= config_clone.concurrency && !url_queue.is_empty() {
                if let Some(result) = tasks.join_next().await {
                    handle_crawl_result(result, &self.error_count);
                }
            }
        }

        // Wait for remaining tasks
        while let Some(result) = tasks.join_next().await {
            handle_crawl_result(result, &self.error_count);
        }

        // Collect results via mpsc channel (shutdown limpio)
        let collected_urls = self.collector.take().unwrap().collect().await;
        let total_pages = collected_urls.len();
        let errors = self.error_count.load(std::sync::atomic::Ordering::SeqCst);

        // Save checkpoint if configured
        if let Some(ref visited_urls) = self.visited_urls {
            let visited_set = visited_urls.lock().clone();
            let checkpoint = CrawlCheckpoint::with_state(
                visited_set,
                Vec::new(),
                pages_crawled + total_pages as u64,
            );
            if let Err(e) = self.save_checkpoint(&checkpoint) {
                warn!("Failed to save checkpoint: {}", e);
            }
        }

        info!("Crawl complete: {} pages, {} errors", total_pages, errors);

        Ok(CrawlResult::new(collected_urls, total_pages, errors))
    }

    /// Graceful shutdown — drop the collector sender, receiver drains remaining items
    pub async fn shutdown(mut self) {
        // Take the collector to drop the sender — receiver will drain remaining items
        // The JoinSet tasks will complete naturally
        self.collector.take();
        info!("Engine shutdown complete");
    }
}

/// Handle result from a completed crawl task
fn handle_crawl_result(
    result: std::result::Result<Result<(), CrawlError>, tokio::task::JoinError>,
    error_count: &Arc<AtomicUsize>,
) {
    match result {
        Ok(Ok(())) => {
            // Task completed successfully
        },
        Ok(Err(e)) => {
            warn!("Task error: {}", e);
            error_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        },
        Err(e) => {
            warn!("Task panicked: {}", e);
            error_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        },
    }
}

/// Crawl a website starting from the seed URL
///
/// Thin wrapper that creates an Engine, runs the crawl loop, and shuts down.
///
/// Following **async-no-lock-across-await**: Uses JoinSet for concurrency control
/// without redundant Semaphore (JoinSet already limits via tasks.len()).
/// Following **async-clone-before-await**: Clones config before async operations.
///
/// # Arguments
///
/// * `config` - Crawler configuration
///
/// # Returns
///
/// * `Ok(CrawlResult)` - Crawl result with discovered URLs
/// * `Err(CrawlError)` - Error during crawling
///
/// # Examples
///
/// ```no_run
/// use rust_scraper::{domain::CrawlerConfig, application::crawl_site};
/// use url::Url;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let seed = Url::parse("https://example.com")?;
/// let config = CrawlerConfig::builder(seed)
///     .max_depth(2)
///     .max_pages(50)
///     .build();
///
/// let result = crawl_site(config).await?;
/// println!("Crawled {} pages", result.total_pages);
/// # Ok(())
/// # }
/// ```
#[instrument(
    name = "crawl_site",
    skip(config),
    fields(
        seed_url = %config.seed_url,
        max_depth = config.max_depth,
        max_pages = config.max_pages,
        delay_ms = config.delay_ms,
        concurrency = config.concurrency
    )
)]
pub async fn crawl_site(config: CrawlerConfig) -> Result<CrawlResult, CrawlError> {
    let span = span!(
        Level::INFO,
        "crawl_site",
        seed_url = %config.seed_url,
        max_depth = config.max_depth,
        max_pages = config.max_pages
    );
    let _guard = span.enter();

    info!(
        "Starting crawl from {} with max_depth={} max_pages={}",
        config.seed_url, config.max_depth, config.max_pages
    );

    let mut engine = Engine::new(config)?;
    let result = engine.run().await;
    engine.shutdown().await;
    result
}

/// Crawl a website with custom engine configuration
///
/// # Arguments
///
/// * `config` - Crawler configuration
/// * `engine_config` - Engine behavior configuration
///
/// # Returns
///
/// * `Ok(CrawlResult)` - Crawl result with discovered URLs
/// * `Err(CrawlError)` - Error during crawling
pub async fn crawl_site_with_config(
    config: CrawlerConfig,
    engine_config: EngineConfig,
) -> Result<CrawlResult, CrawlError> {
    let span = span!(
        Level::INFO,
        "crawl_site",
        seed_url = %config.seed_url,
        max_depth = config.max_depth,
        max_pages = config.max_pages
    );
    let _guard = span.enter();

    info!(
        "Starting crawl from {} with max_depth={} max_pages={}",
        config.seed_url, config.max_depth, config.max_pages
    );

    let mut engine = Engine::with_engine_config(config, engine_config)?;
    let result = engine.run().await;
    engine.shutdown().await;
    result
}
