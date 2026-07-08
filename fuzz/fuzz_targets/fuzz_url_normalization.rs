#![no_main]
use libfuzzer_sys::fuzz_target;

// Fuzz URL normalization — normalizes URLs for deduplication.
// Processes untrusted URLs from HTML. Panic = DoS during crawl.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // normalize_url returns String, never panics
        let normalized = rust_scraper::infrastructure::crawler::link_extractor::normalize_url(s);

        // Idempotency check: only valid when both parse successfully.
        // When Url::parse() fails, the function returns raw input —
        // a second pass may parse differently due to url crate leniency.
        if url::Url::parse(&normalized).is_ok() {
            let double = rust_scraper::infrastructure::crawler::link_extractor::normalize_url(&normalized);
            assert_eq!(normalized, double, "URL normalization is not idempotent for valid URLs!");
        }
    }
});
