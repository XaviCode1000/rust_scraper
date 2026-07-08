//! Obsidian-specific behavior: wiki-links, tags, quick-save.

use crate::BehavioralTest;
use walkdir::WalkDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

const PAGE_WITH_LINKS: &str = r#"
<html><head><title>Wiki Links Test</title></head>
<body><article>
<h1>Wiki Links Test</h1>
<p>Check out <a href="/other-page">this other page</a> for more info.
Also see <a href="/third-page">the third page</a>.</p>
</article></body></html>
"#;

const TAGGED_PAGE: &str = r#"
<html><head><title>Tagged Page</title></head>
<body><article>
<h1>Tagged Page</h1>
<p>Content with obsidian tags for frontmatter testing.</p>
</article></body></html>
"#;

// ---------------------------------------------------------------------------
// --obsidian-wiki-links
// ---------------------------------------------------------------------------

#[tokio::test]
async fn obsidian_wiki_links_produces_wiki_syntax() {
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PAGE_WITH_LINKS))
        .expect(1)
        .mount(&t.server)
        .await;

    t.scraper_cmd()
        .arg("--single-page")
        .arg("--format")
        .arg("markdown")
        .arg("--obsidian-wiki-links")
        .arg("--quiet")
        .assert()
        .success();

    let content = t.read_md_content();
    assert!(
        content.contains("[["),
        "output should contain [[wiki-link]] syntax: {}",
        &content[..content.len().min(500)]
    );
}

#[tokio::test]
async fn obsidian_wiki_links_removes_absolute_urls() {
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(PAGE_WITH_LINKS))
        .expect(1)
        .mount(&t.server)
        .await;

    t.scraper_cmd()
        .arg("--single-page")
        .arg("--format")
        .arg("markdown")
        .arg("--obsidian-wiki-links")
        .arg("--quiet")
        .assert()
        .success();

    let content = t.read_md_content();
    // The frontmatter `url:` field always contains the full URL — that's expected.
    // The wiki-link conversion affects only <a> tags in the body content.
    // Split at the frontmatter end to check only the body.
    let body = content.split_once("---\n").map(|x| x.1).unwrap_or(&content);
    let body = body.split_once("---\n").map(|x| x.1).unwrap_or(body);
    assert!(
        !body.contains("<a href="),
        "body should not contain raw <a> tags after wiki-link conversion: {}",
        &body[..body.len().min(500)]
    );
    assert!(
        body.contains("[["),
        "body should contain [[wiki-link]] syntax: {}",
        &body[..body.len().min(500)]
    );
}

// ---------------------------------------------------------------------------
// --obsidian-tags
// ---------------------------------------------------------------------------

#[tokio::test]
async fn obsidian_tags_appear_in_frontmatter() {
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(TAGGED_PAGE))
        .expect(1)
        .mount(&t.server)
        .await;

    t.scraper_cmd()
        .arg("--single-page")
        .arg("--format")
        .arg("markdown")
        .arg("--obsidian-tags")
        .arg("scraped,web-dev,rust")
        .arg("--quiet")
        .assert()
        .success();

    let content = t.read_md_content();
    assert!(
        content.contains("tags:"),
        "frontmatter should contain tags field"
    );
    assert!(
        content.contains("scraped") && content.contains("web-dev") && content.contains("rust"),
        "frontmatter should contain all specified tags: {}",
        &content[..content.len().min(500)]
    );
}

#[tokio::test]
async fn obsidian_tags_produces_yaml_frontmatter() {
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(TAGGED_PAGE))
        .expect(1)
        .mount(&t.server)
        .await;

    t.scraper_cmd()
        .arg("--single-page")
        .arg("--format")
        .arg("markdown")
        .arg("--obsidian-tags")
        .arg("test")
        .arg("--quiet")
        .assert()
        .success();

    let content = t.read_md_content();
    // YAML frontmatter starts with ---
    assert!(
        content.starts_with("---"),
        "file should start with YAML frontmatter (---): {}",
        &content[..content.len().min(300)]
    );
}

// ---------------------------------------------------------------------------
// --quick-save
// ---------------------------------------------------------------------------

#[tokio::test]
async fn quick_save_creates_files_in_inbox() {
    let t = BehavioralTest::new().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(TAGGED_PAGE))
        .expect(1)
        .mount(&t.server)
        .await;

    // --quick-save requires --vault to determine where _inbox lives
    // Create a mock vault structure in the output dir
    let vault_dir = t.out.path().join("test_vault");
    std::fs::create_dir_all(vault_dir.join(".obsidian")).unwrap();
    std::fs::write(
        vault_dir.join(".obsidian").join("obsidian.json"),
        r#"{"vault":{"fsPath":"/tmp/test","id":"test","name":"Test"}}"#,
    )
    .unwrap();

    t.scraper_cmd()
        .arg("--single-page")
        .arg("--format")
        .arg("markdown")
        .arg("--quick-save")
        .arg("--vault")
        .arg(&vault_dir)
        .arg("--quiet")
        .assert()
        .success();

    // Check that files ended up in _inbox somewhere under the vault
    let has_inbox = WalkDir::new(&vault_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .any(|e| e.path().to_string_lossy().contains("_inbox"));
    assert!(
        has_inbox,
        "--quick-save should place files in _inbox directory"
    );
}
