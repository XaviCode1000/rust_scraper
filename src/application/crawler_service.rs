//! Crawler service module (DEPRECATED)
//!
//! ⚠️ **DEPRECATED since v0.5.0** ⚠️
//! This module is kept for backwards compatibility ONLY.
//!
//! # Migration
//!
//! Replace:
//! ```rust
//! use rust_scraper::application::crawler_service::*;
//! ```
//!
//! With:
//! ```rust
//! use rust_scraper::application::crawler::{self, *};
//! ```
//!
//! Or access individual modules:
//! ```rust
//! use rust_scraper::application::crawler::discovery;
//! use rust_scraper::application::crawler::engine;
//! ```

use std::sync::Arc;

use anyhow::Result;
use tracing::{debug, info, warn};
use url::Url;

pub use crate::domain::{
    CorrelationId, CrawlError, CrawlResult, CrawlerConfig, DiscoveredUrl, ScrapedContent, ValidUrl,
};

pub use super::results_channel::{CrawlMessage, ResultsCollector};
pub use super::url_filter::is_allowed;
pub use crate::infrastructure::crawler::{
    extract_links, fetch_url, is_internal_link, normalize_url, UrlQueue,
};

pub use crate::infrastructure::crawler::{SitemapConfig, SitemapParser};

pub use crate::error::{Result as ScraperResult, ScraperError};
pub use crate::infrastructure::scraper::{fallback, readability};
pub use crate::ScraperConfig;

pub use crate::application::rate_limiter::{RateLimiterConfig, SharedRateLimiter};

/// Fetch and parse a sitemap.xml file (legacy - kept for backwards compatibility)
///
/// Following **own-borrow-over-clone**: Accepts `&str`.
/// Following **xml-no-regex**: Uses quick-xml for streaming XML parsing.
///
/// # Arguments
///
/// * `base_url` - Base URL of the website
///
/// # Returns
///
/// * `Ok(Vec<String>)` - List of URLs from sitemap
/// * `Err(CrawlError)` - Error during fetch or parse
#[deprecated(since = "0.4.0", note = "Use crawl_with_sitemap instead")]
pub async fn fetch_sitemap(base_url: &str) -> Result<Vec<String>, CrawlError> {
    info!("Fetching sitemap from {} (legacy method)", base_url);

    // Try common sitemap locations
    let sitemap_urls = [
        format!("{}/sitemap.xml", base_url.trim_end_matches('/')),
        format!("{}/sitemap_index.xml", base_url.trim_end_matches('/')),
        format!("{}/sitemap.xml.gz", base_url.trim_end_matches('/')),
    ];

    let mut all_urls = Vec::new();

    for sitemap_url in &sitemap_urls {
        debug!("Trying sitemap: {}", sitemap_url);

        // Create minimal config for sitemap fetch
        let seed = Url::parse(base_url).map_err(|e| CrawlError::InvalidUrl(e.to_string()))?;
        let config = Arc::new(CrawlerConfig::new(seed.clone()));
        let config_clone = Arc::clone(&config);

        match fetch_url(sitemap_url, &config_clone).await {
            Ok(response) => {
                // Parse sitemap XML using quick-xml (streaming parser)
                // Pass seed as base_url for relative URL resolution
                match super::crawler::parse_sitemap(&response, &seed) {
                    Ok(urls) => {
                        info!("Found {} URLs in {}", urls.len(), sitemap_url);
                        all_urls.extend(urls);
                    },
                    Err(e) => {
                        warn!("Failed to parse sitemap {}: {}", sitemap_url, e);
                    },
                }
            },
            Err(e) => {
                debug!("Sitemap not found at {}: {}", sitemap_url, e);
            },
        }
    }

    if all_urls.is_empty() {
        warn!("No sitemap found for {}", base_url);
    }

    Ok(all_urls)
}
