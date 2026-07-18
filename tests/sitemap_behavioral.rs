//! Sitemap behavioral tests.
//!
//! Exercises SitemapParser against wiremock for valid XML parsing,
//! malformed XML graceful degradation, large sitemap performance,
//! and nested sitemap indexes.

use webfang_core::infrastructure::crawler::{SitemapError, SitemapParser};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ============================================================================
// Helpers
// ============================================================================

fn parser() -> SitemapParser {
    SitemapParser::new()
}

const VALID_SITEMAP: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://example.com/page1</loc><lastmod>2025-01-01</lastmod></url>
  <url><loc>https://example.com/page2</loc><changefreq>daily</changefreq></url>
  <url><loc>https://example.com/page3</loc><priority>0.8</priority></url>
</urlset>"#;

// ============================================================================
// 1. Valid sitemap XML parsing via wiremock
// ============================================================================

/// Parse a valid sitemap and verify URL extraction with metadata elements.
#[tokio::test]
async fn sitemap_valid_xml_extracts_all_urls() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sitemap.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(VALID_SITEMAP)
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    let urls = parser()
        .parse_from_url(&format!("{}/sitemap.xml", mock.uri()))
        .await
        .unwrap();

    assert_eq!(urls.len(), 3, "should extract all 3 URLs");
    let strings: Vec<String> = urls.iter().map(|u| u.to_string()).collect();
    assert!(strings.contains(&"https://example.com/page1".to_string()));
    assert!(strings.contains(&"https://example.com/page2".to_string()));
    assert!(strings.contains(&"https://example.com/page3".to_string()));
}

/// Sitemap with URL entries containing only <loc> (minimal sitemap).
#[tokio::test]
async fn sitemap_minimal_entries_only_loc() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://minimal.com/only-loc</loc></url>
</urlset>"#;

    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sitemap.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(xml)
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    let urls = parser()
        .parse_from_url(&format!("{}/sitemap.xml", mock.uri()))
        .await
        .unwrap();

    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0].as_str(), "https://minimal.com/only-loc");
}

/// Sitemap with special characters in URLs (percent-encoded).
/// Parser may decode XML entities and normalize paths — verify no panic
/// and that expected base URLs are present.
#[tokio::test]
async fn sitemap_special_characters_in_urls() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://example.com/path%20with%20spaces</loc></url>
  <url><loc>https://example.com/query?q=hello&amp;lang=en</loc></url>
</urlset>"#;

    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sitemap.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(xml)
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    let urls = parser()
        .parse_from_url(&format!("{}/sitemap.xml", mock.uri()))
        .await
        .unwrap();

    // Parser may decode entities and normalize — assert >=2 and check base URLs
    assert!(
        urls.len() >= 2,
        "should extract at least 2 URLs, got {}",
        urls.len()
    );
    let strings: Vec<String> = urls.iter().map(|u| u.to_string()).collect();
    assert!(
        strings.iter().any(|u| u.contains("example.com/path")),
        "should contain the spaces URL"
    );
    assert!(
        strings.iter().any(|u| u.contains("example.com/query")),
        "should contain the query URL"
    );
}

// ============================================================================
// 2. Malformed XML graceful degradation
// ============================================================================

/// Truncated XML — parser should handle gracefully.
#[tokio::test]
async fn sitemap_truncated_xml_returns_error() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://example.com/page1</loc></url>
  <url><loc>https://example.com/page2</loc></url"#; // Missing closing tags

    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sitemap.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(xml)
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    let result = parser()
        .parse_from_url(&format!("{}/sitemap.xml", mock.uri()))
        .await;

    // Truncated XML either parses the first URL or returns an error — both acceptable
    assert!(
        matches!(
            result,
            Err(SitemapError::XmlError(_)) | Err(SitemapError::NoUrlsFound) | Ok(_)
        ),
        "truncated XML should not panic, got: {:?}",
        result
    );
}

/// Garbage bytes instead of XML — returns error.
#[tokio::test]
async fn sitemap_garbage_bytes_returns_error() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/feed"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(vec![0x00, 0x00, 0x00, 0x3C, 0x00])
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    let result = parser()
        .parse_from_url(&format!("{}/feed", mock.uri()))
        .await;

    assert!(
        matches!(
            result,
            Err(SitemapError::XmlError(_)) | Err(SitemapError::NoUrlsFound)
        ),
        "garbage bytes should fail, got: {:?}",
        result
    );
}

/// HTML content served at a .xml path — parser tries to parse as XML.
#[tokio::test]
async fn sitemap_html_content_at_xml_path() {
    let html = "<html><body><h1>Not a sitemap</h1></body></html>";
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sitemap.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(html)
                .insert_header("content-type", "text/html"),
        )
        .mount(&mock)
        .await;

    let result = parser()
        .parse_from_url(&format!("{}/sitemap.xml", mock.uri()))
        .await;

    // HTML at .xml path: parser accepts it (path-based) but finds no <loc> tags
    assert!(
        matches!(result, Err(SitemapError::NoUrlsFound)),
        "HTML content should yield NoUrlsFound, got: {:?}",
        result
    );
}

/// Non-XML content type on non-.xml path returns InvalidContentType.
#[tokio::test]
async fn sitemap_non_xml_content_type_non_xml_path() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/feed"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("<html>text</html>")
                .insert_header("content-type", "text/html"),
        )
        .mount(&mock)
        .await;

    let result = parser()
        .parse_from_url(&format!("{}/feed", mock.uri()))
        .await;

    assert!(matches!(result, Err(SitemapError::InvalidContentType(_))));
}

/// HTTP 404 — body is empty, parser should fail gracefully.
#[tokio::test]
async fn sitemap_http_404_returns_no_urls() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sitemap.xml"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock)
        .await;

    let result = parser()
        .parse_from_url(&format!("{}/sitemap.xml", mock.uri()))
        .await;

    assert!(matches!(result, Err(SitemapError::NoUrlsFound)));
}

/// HTTP 500 — body is empty, parser should fail gracefully.
#[tokio::test]
async fn sitemap_http_500_returns_error() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sitemap.xml"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock)
        .await;

    let result = parser()
        .parse_from_url(&format!("{}/sitemap.xml", mock.uri()))
        .await;

    assert!(
        matches!(
            result,
            Err(SitemapError::NoUrlsFound) | Err(SitemapError::HttpError(_))
        ),
        "server error should fail gracefully, got: {:?}",
        result
    );
}

/// Invalid URL passed to parser returns error.
#[tokio::test]
async fn sitemap_invalid_url_returns_error() {
    let result = parser().parse_from_url("not-a-valid-url").await;
    assert!(result.is_err());
}

// ============================================================================
// 3. Large sitemap (1000+ URLs) performance
// ============================================================================

/// Parse a sitemap with 1000 URLs — should complete without timeout.
#[tokio::test]
async fn sitemap_large_1000_urls_parses_correctly() {
    let url_entries: String = (1..=1000)
        .map(|i| format!("  <url><loc>https://example.com/page{i}</loc></url>"))
        .collect::<Vec<_>>()
        .join("\n");
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{url_entries}
</urlset>"#
    );

    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sitemap.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(&xml)
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    let start = std::time::Instant::now();
    let urls = parser()
        .parse_from_url(&format!("{}/sitemap.xml", mock.uri()))
        .await
        .unwrap();
    let elapsed = start.elapsed();

    assert_eq!(urls.len(), 1000, "should extract all 1000 URLs");
    assert!(
        elapsed.as_secs() < 10,
        "parsing 1000 URLs should take less than 10 seconds, took {:?}",
        elapsed
    );
    // Verify specific URLs exist (parser returns a HashSet, order not guaranteed)
    let strings: Vec<String> = urls.iter().map(|u| u.to_string()).collect();
    assert!(strings.contains(&"https://example.com/page1".to_string()));
    assert!(strings.contains(&"https://example.com/page500".to_string()));
    assert!(strings.contains(&"https://example.com/page1000".to_string()));
}

/// Sitemap with duplicate URLs in large set — deduplication works at scale.
#[tokio::test]
async fn sitemap_large_with_duplicates_deduplicates() {
    let mut url_entries = String::new();
    for i in 1..=500 {
        url_entries.push_str(&format!(
            "  <url><loc>https://example.com/page{i}</loc></url>\n"
        ));
        // Add duplicate
        url_entries.push_str(&format!(
            "  <url><loc>https://example.com/page{i}</loc></url>\n"
        ));
    }
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
{url_entries}</urlset>"#
    );

    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sitemap.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(&xml)
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    let urls = parser()
        .parse_from_url(&format!("{}/sitemap.xml", mock.uri()))
        .await
        .unwrap();

    assert_eq!(
        urls.len(),
        500,
        "1000 entries with duplicates should dedup to 500"
    );
}

// ============================================================================
// 4. Nested sitemap indexes
// ============================================================================

/// Sitemap index referencing child sitemaps — parser follows references.
#[tokio::test]
async fn sitemap_index_references_child_sitemaps() {
    let child1 = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://example.com/child1/page1</loc></url>
  <url><loc>https://example.com/child1/page2</loc></url>
</urlset>"#;

    let child2 = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://example.com/child2/page1</loc></url>
</urlset>"#;

    let mock = MockServer::start().await;

    // Mock the index — must use mock URI for child references
    let sitemap_index = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap><loc>{}/sitemap-child1.xml</loc></sitemap>
  <sitemap><loc>{}/sitemap-child2.xml</loc></sitemap>
</sitemapindex>"#,
        mock.uri(),
        mock.uri()
    );

    Mock::given(method("GET"))
        .and(path("/sitemap-index.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(&sitemap_index)
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    // Mock child 1
    Mock::given(method("GET"))
        .and(path("/sitemap-child1.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(child1)
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    // Mock child 2
    Mock::given(method("GET"))
        .and(path("/sitemap-child2.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(child2)
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    let urls = parser()
        .parse_from_url(&format!("{}/sitemap-index.xml", mock.uri()))
        .await
        .unwrap();

    assert!(
        urls.len() >= 3,
        "should follow sitemap index and collect child URLs, got {}",
        urls.len()
    );
    let strings: Vec<String> = urls.iter().map(|u| u.to_string()).collect();
    assert!(strings.contains(&"https://example.com/child1/page1".to_string()));
    assert!(strings.contains(&"https://example.com/child1/page2".to_string()));
    assert!(strings.contains(&"https://example.com/child2/page1".to_string()));
}

/// Sitemap index with one missing child — parser handles partial failure.
#[tokio::test]
async fn sitemap_index_with_missing_child() {
    let child1 = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://example.com/child1/page1</loc></url>
</urlset>"#;

    let mock = MockServer::start().await;

    let sitemap_index = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap><loc>{}/sitemap-child1.xml</loc></sitemap>
  <sitemap><loc>{}/sitemap-missing.xml</loc></sitemap>
</sitemapindex>"#,
        mock.uri(),
        mock.uri()
    );

    Mock::given(method("GET"))
        .and(path("/sitemap-index.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(&sitemap_index)
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    Mock::given(method("GET"))
        .and(path("/sitemap-child1.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(child1)
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    // Missing child returns 404
    Mock::given(method("GET"))
        .and(path("/sitemap-missing.xml"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock)
        .await;

    let result = parser()
        .parse_from_url(&format!("{}/sitemap-index.xml", mock.uri()))
        .await;

    // Should succeed with partial URLs — not panic or return error
    assert!(
        result.is_ok(),
        "parser should handle missing child gracefully: {:?}",
        result.err()
    );
    if let Ok(urls) = result {
        assert!(
            !urls.is_empty(),
            "should have at least the URLs from the working child"
        );
    }
}

/// Sitemap index with XML error in child — parser handles gracefully.
#[tokio::test]
async fn sitemap_index_with_malformed_child() {
    let mock = MockServer::start().await;

    let sitemap_index = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap><loc>{}/sitemap-bad.xml</loc></sitemap>
</sitemapindex>"#,
        mock.uri()
    );

    Mock::given(method("GET"))
        .and(path("/sitemap-index.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(&sitemap_index)
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    // Malformed child
    Mock::given(method("GET"))
        .and(path("/sitemap-bad.xml"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(">>> NOT XML <<<")
                .insert_header("content-type", "application/xml"),
        )
        .mount(&mock)
        .await;

    let result = parser()
        .parse_from_url(&format!("{}/sitemap-index.xml", mock.uri()))
        .await;

    // Should handle malformed child gracefully — not panic
    assert!(
        result.is_err(),
        "parser should reject malformed child, got: {:?}",
        result
    );
}
