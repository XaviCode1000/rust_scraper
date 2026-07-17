//! Exit code integration tests
//!
//! Verifies that the CLI returns correct exit codes for:
//! - Empty sitemap discovery → exit 2 (EXIT_EMPTY_DISCOVERY)
//! - Network timeout → exit 69 (EXIT_UNAVAILABLE)
//! - Successful crawl → exit 0 (EXIT_SUCCESS)
//!
//! Run with: cargo nextest run --test-threads 2 exit_code_integration

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Resolve the path to the `webfang` binary.
///
/// `webfang` is built by `rust_scraper_cli` (a workspace sibling), so
/// `assert_cmd::cargo_bin` cannot resolve it — `CARGO_BIN_EXE_webfang`
/// is only set for the owning crate.  This fallback searches
/// `target/{debug,release}` and builds the binary on demand.
fn webfang_path() -> PathBuf {
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_webfang") {
        return PathBuf::from(p);
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("resolve workspace root");
    for profile in ["debug", "release"] {
        let mut candidate = workspace_root.join("target").join(profile).join("webfang");
        if cfg!(windows) {
            candidate.set_extension("exe");
        }
        if candidate.exists() {
            return candidate;
        }
    }
    let cargo = option_env!("CARGO").unwrap_or("cargo");
    let status = std::process::Command::new(cargo)
        .args([
            "build",
            "-p",
            "rust_scraper_cli",
            "--bin",
            "webfang",
            "--quiet",
        ])
        .status()
        .expect("spawn cargo to build webfang");
    assert!(status.success(), "cargo build --bin webfang failed");
    let mut built = workspace_root.join("target").join("debug").join("webfang");
    if cfg!(windows) {
        built.set_extension("exe");
    }
    built
}

fn cmd() -> Command {
    Command::new(webfang_path())
}

// ============================================================================
// Tests: Empty sitemap → exit 2
// ============================================================================

/// Empty sitemap (no <loc> entries) returns exit code 2.
#[tokio::test]
async fn test_empty_sitemap_returns_exit_2() {
    let mock_server = MockServer::start().await;

    // Serve an empty sitemap
    Mock::given(method("GET"))
        .and(path("/sitemap.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
</urlset>"#,
        ))
        .mount(&mock_server)
        .await;

    let base_url = format!("{}/", mock_server.uri());
    let sitemap_url = format!("{}/sitemap.xml", mock_server.uri());

    cmd()
        .arg("--url")
        .arg(&base_url)
        .arg("--sitemap-url")
        .arg(&sitemap_url)
        .arg("--use-sitemap")
        .timeout(Duration::from_secs(30))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("No URLs discovered"));
}

// ============================================================================
// Tests: Network timeout → exit 69
// ============================================================================

/// Timeout during sitemap fetch returns exit code 69.
#[tokio::test]
async fn test_timeout_returns_exit_69() {
    let mock_server = MockServer::start().await;

    // Serve a response with a very long delay to trigger timeout
    Mock::given(method("GET"))
        .and(path("/slow-sitemap.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>http://PLACEHOLDER/page1</loc></url>
</urlset>"#,
                )
                .set_delay(Duration::from_secs(120)),
        )
        .mount(&mock_server)
        .await;

    let base_url = format!("{}/", mock_server.uri());
    let sitemap_url = format!("{}/slow-sitemap.xml", mock_server.uri());

    cmd()
        .arg("--url")
        .arg(&base_url)
        .arg("--sitemap-url")
        .arg(&sitemap_url)
        .arg("--use-sitemap")
        .arg("--timeout-secs")
        .arg("1")
        .timeout(Duration::from_secs(60))
        .assert()
        .code(69)
        .stderr(predicate::str::contains("URL discovery failed"));
}

// ============================================================================
// Tests: Successful discovery → exit 0
// ============================================================================

/// Valid sitemap with URLs returns exit code 0 (no regression).
#[tokio::test]
async fn test_valid_sitemap_returns_exit_0() {
    let mock_server = MockServer::start().await;
    let server_uri = mock_server.uri();

    // Serve a valid sitemap with one URL
    Mock::given(method("GET"))
        .and(path("/sitemap.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>{server_uri}/page1</loc></url>
</urlset>"#
        )))
        .mount(&mock_server)
        .await;

    // Serve the page content
    Mock::given(method("GET"))
        .and(path("/page1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(
                "<html><body><h1>Hello World</h1><p>Test content</p></body></html>",
            ),
        )
        .mount(&mock_server)
        .await;

    let base_url = format!("{}/", server_uri);
    let sitemap_url = format!("{}/sitemap.xml", server_uri);

    cmd()
        .arg("--url")
        .arg(&base_url)
        .arg("--sitemap-url")
        .arg(&sitemap_url)
        .arg("--use-sitemap")
        .timeout(Duration::from_secs(30))
        .assert()
        .code(0);
}

// ============================================================================
// Tests: SitemapNotFound propagation (Bug #191 fix)
// ============================================================================

/// When no sitemap is found at any location, crawl_with_sitemap must return
/// Err(CrawlError::SitemapNotFound) — NOT Ok(vec![]) as pre-fix behavior did.
///
/// wiremock returns 404 for all unregistered paths, so discover_sitemap_url
/// exhausts robots.txt + fallback locations and returns SitemapNotFound.
/// crawl_with_sitemap must propagate this instead of silently returning empty.
#[tokio::test]
async fn test_sitemap_not_found_propagates_error() {
    let mock_server = MockServer::start().await;
    let base_url = format!("{}/", mock_server.uri());

    let seed = url::Url::parse(&format!("{base_url}index.html")).expect("valid seed URL");
    let config = rust_scraper_core::domain::CrawlerConfig::builder(seed)
        .max_pages(5)
        .delay_ms(1)
        .timeout_secs(5)
        .build();

    let result = rust_scraper_core::application::crawler::discovery::crawl_with_sitemap(
        &base_url, None, &config,
    )
    .await;

    match result {
        Err(rust_scraper_core::domain::CrawlError::SitemapNotFound(url)) => {
            assert!(url.contains(
                &mock_server
                    .uri()
                    .replace("http://", "")
                    .replace("https://", "")
            ));
        },
        Ok(urls) => {
            panic!(
                "expected SitemapNotFound error, got Ok with {} URLs",
                urls.len()
            );
        },
        Err(e) => {
            panic!("expected SitemapNotFound error, got {:?}", e);
        },
    }
}

// ============================================================================
// Tests: Engine signal handler abort — no hang on shutdown (Bug #191 fix)
// ============================================================================

/// Exercise crawl_site_with_options with a small crawl that completes
/// naturally. The fix ensures the signal handler's JoinHandle is aborted
/// in engine.shutdown(), preventing the runtime from hanging.
///
/// Before the fix: tokio runtime hangs waiting for the orphaned signal
/// handler task to complete (which never happens since no signal is sent).
/// After the fix: shutdown aborts the signal handler, runtime exits cleanly.
#[tokio::test]
async fn test_crawl_completes_without_signal_handler_hang() {
    let mock_server = MockServer::start().await;
    let server_uri = mock_server.uri();

    Mock::given(method("GET"))
        .and(path("/index.html"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("<html><body><h1>Single Page</h1></body></html>"),
        )
        .mount(&mock_server)
        .await;

    let seed = url::Url::parse(&format!("{server_uri}/index.html")).expect("valid URL");
    let config = rust_scraper_core::domain::CrawlerConfig::builder(seed)
        .max_depth(0)
        .max_pages(1)
        .delay_ms(1)
        .concurrency(1)
        .timeout_secs(5)
        .build();

    let options = rust_scraper_core::application::crawler::engine::EngineOptions {
        checkpoint_path: None,
        session_pool_enabled: false,
        ignore_robots: true,
        js_strategy: rust_scraper_core::domain::JsStrategy::Static,
        autoscale_enabled: false,
    };

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        rust_scraper_core::crawl_site_with_options(config, options),
    )
    .await;

    match result {
        Ok(Ok(crawl_result)) => {
            assert!(
                crawl_result.total_pages > 0,
                "crawl should have completed at least 1 page"
            );
        },
        Ok(Err(e)) => {
            panic!("crawl should succeed, got error: {e:?}");
        },
        Err(_elapsed) => {
            panic!("crawl timed out after 10s — possible signal handler hang");
        },
    }
}
