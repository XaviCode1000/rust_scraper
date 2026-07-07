//! Single-page scraping: format variants and quiet mode.

use crate::BehavioralTest;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

const SEED_HTML: &str = r#"
<html><head><title>Single Page Test</title></head>
<body><main><article>
<h1>Hello World</h1>
<p>This is meaningful content for the extractor to process.</p>
<p>More paragraphs ensure readability can extract a proper document.</p>
</article></main></body></html>
"#;

// ---------------------------------------------------------------------------
// Markdown output (default)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn single_page_creates_md_file() {
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SEED_HTML))
        .expect(1)
        .mount(&t.server)
        .await;

    t.scraper_cmd()
        .arg("--single-page")
        .arg("--quiet")
        .assert()
        .success();

    let md_files = t.find_files("md");
    assert!(!md_files.is_empty(), "expected at least one .md file");
}

#[tokio::test]
async fn single_page_md_contains_page_content() {
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SEED_HTML))
        .expect(1)
        .mount(&t.server)
        .await;

    t.scraper_cmd()
        .arg("--single-page")
        .arg("--quiet")
        .assert()
        .success();

    let content = t.read_md_content();
    assert!(
        content.contains("Hello World"),
        "md should contain page heading: {}",
        &content[..content.len().min(500)]
    );
}

// ---------------------------------------------------------------------------
// JSON output
// ---------------------------------------------------------------------------

#[tokio::test]
async fn single_page_json_format_creates_json_file() {
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SEED_HTML))
        .expect(1)
        .mount(&t.server)
        .await;

    t.scraper_cmd()
        .arg("--single-page")
        .arg("--format")
        .arg("json")
        .arg("--quiet")
        .assert()
        .success();

    // JSON output goes to results.json at the output root (not in domain subdirs)
    let json_files = t.find_files("json");
    assert!(
        !json_files.is_empty(),
        "expected at least one .json file"
    );
}

#[tokio::test]
async fn single_page_json_has_correct_structure() {
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SEED_HTML))
        .expect(1)
        .mount(&t.server)
        .await;

    t.scraper_cmd()
        .arg("--single-page")
        .arg("--format")
        .arg("json")
        .arg("--quiet")
        .assert()
        .success();

    let json_files = t.find_files("json");
    assert!(!json_files.is_empty());

    let content = std::fs::read_to_string(&json_files[0]).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
    assert!(
        parsed.is_array() || parsed.get("url").is_some() || parsed.get("title").is_some(),
        "JSON should contain url/title fields or be an array: {}",
        &content[..content.len().min(500)]
    );
}

// ---------------------------------------------------------------------------
// Text output
// ---------------------------------------------------------------------------

#[tokio::test]
async fn single_page_text_format_creates_txt_file() {
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SEED_HTML))
        .expect(1)
        .mount(&t.server)
        .await;

    t.scraper_cmd()
        .arg("--single-page")
        .arg("--format")
        .arg("text")
        .arg("--quiet")
        .assert()
        .success();

    let txt_files = t.find_files("txt");
    assert!(
        !txt_files.is_empty(),
        "expected at least one .txt file"
    );
}

#[tokio::test]
async fn single_page_text_is_plain_text() {
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SEED_HTML))
        .expect(1)
        .mount(&t.server)
        .await;

    t.scraper_cmd()
        .arg("--single-page")
        .arg("--format")
        .arg("text")
        .arg("--quiet")
        .assert()
        .success();

    let txt_files = t.find_files("txt");
    assert!(!txt_files.is_empty());

    let content = std::fs::read_to_string(&txt_files[0]).unwrap();
    // Text format should not contain HTML tags
    assert!(
        !content.contains("<html") && !content.contains("<body"),
        "text format should not contain HTML tags: {}",
        &content[..content.len().min(500)]
    );
}

// ---------------------------------------------------------------------------
// Quiet mode
// ---------------------------------------------------------------------------

#[tokio::test]
async fn single_page_quiet_suppresses_stdout() {
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SEED_HTML))
        .expect(1)
        .mount(&t.server)
        .await;

    let output = t.scraper_cmd()
        .arg("--single-page")
        .arg("--quiet")
        .output()
        .expect("run binary");

    assert!(
        output.status.success(),
        "expected success, got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // stdout should be empty or very short in quiet mode
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty() || stdout.lines().count() <= 1,
        "quiet mode should suppress stdout output, got {} lines: {}",
        stdout.lines().count(),
        &stdout[..stdout.len().min(300)]
    );
}
