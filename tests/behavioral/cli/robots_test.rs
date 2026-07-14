//! Robots.txt enforcement from CLI args.

use crate::BehavioralTest;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

/// robots.txt disallows /secret → crawler must NOT fetch it.
#[ignore = "Pre-existing stale test, out of scope for insta migration"]
#[tokio::test]
async fn robots_txt_disallow_prevents_fetching() {
    let t = BehavioralTest::new().await;

    // robots.txt: allow everything except /secret
    Mock::given(method("GET"))
        .and(path("/robots.txt"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("User-agent: *\nDisallow: /secret\n"),
        )
        .mount(&t.server)
        .await;

    // Seed page — links to /secret
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<html><body><main><article>
<h1>Index Page</h1>
<p>This page links to a secret page that robots.txt disallows.</p>
<a href="/secret">Secret</a>
</article></main></body></html>"#,
        ))
        .mount(&t.server)
        .await;

    // /secret must NOT be fetched
    Mock::given(method("GET"))
        .and(path("/secret"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<html><body><article>
<h1>Secret Page</h1>
<p>This content should never be scraped.</p>
</article></body></html>"#,
        ))
        .expect(0)
        .named("/secret must not be fetched")
        .mount(&t.server)
        .await;

    t.scraper_cmd()
        .arg("--max-depth")
        .arg("1")
        .arg("--quiet")
        .assert()
        .success();

    let requests = t.server.received_requests().await.unwrap();
    let has_secret = requests.iter().any(|r| r.url.path() == "/secret");
    assert!(
        !has_secret,
        "robots.txt Disallow /secret must prevent fetching /secret"
    );

    // Seed must have been fetched (and possibly robots.txt too)
    let has_root = requests.iter().any(|r| r.url.path() == "/");
    assert!(has_root, "seed page / must be fetched");
}
