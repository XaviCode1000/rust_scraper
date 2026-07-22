//! Batch mode: stdin and file-based URL processing.

use crate::cmd;
use std::time::Duration;
use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// --batch (stdin)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn batch_stdin_processes_urls() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "<html><body><article>\
                 <h1>Batch Stdin Test</h1>\
                 <p>Content from batch stdin processing.</p>\
                 </article></body></html>",
        ))
        .expect(1)
        .mount(&server)
        .await;

    cmd()
        .arg("--batch")
        .write_stdin(format!("{}\n", server.uri()))
        .timeout(Duration::from_secs(30))
        .assert()
        .success();

    let requests = server.received_requests().await.unwrap();
    assert_eq!(
        requests.len(),
        1,
        "batch stdin should fetch exactly the provided URL, got {} requests",
        requests.len()
    );
}

#[test]
fn batch_empty_stdin_exits_64() {
    cmd()
        .arg("--batch")
        .write_stdin("")
        .timeout(Duration::from_secs(5))
        .assert()
        .code(64)
        .stderr(predicates::str::contains("No URLs provided"));
}

// ---------------------------------------------------------------------------
// --batch-file
// ---------------------------------------------------------------------------

#[tokio::test]
async fn batch_file_processes_urls() {
    let server = MockServer::start().await;
    let temp = TempDir::new().unwrap();

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "<html><body><article>\
                 <h1>Batch File Test</h1>\
                 <p>Content from batch file processing.</p>\
                 </article></body></html>",
        ))
        .expect(1)
        .mount(&server)
        .await;

    let batch_file = temp.path().join("urls.txt");
    std::fs::write(&batch_file, format!("{}\n", server.uri())).unwrap();

    cmd()
        .arg("--batch-file")
        .arg(&batch_file)
        .timeout(Duration::from_secs(30))
        .assert()
        .success();

    let requests = server.received_requests().await.unwrap();
    assert_eq!(
        requests.len(),
        1,
        "batch-file should fetch exactly the URL from the file, got {} requests",
        requests.len()
    );
}

#[test]
fn batch_empty_file_exits_64() {
    let temp = TempDir::new().unwrap();
    let batch_file = temp.path().join("urls.txt");
    std::fs::write(&batch_file, "").unwrap();

    cmd()
        .arg("--batch-file")
        .arg(&batch_file)
        .timeout(Duration::from_secs(5))
        .assert()
        .code(64)
        .stderr(predicates::str::contains("No URLs provided"));
}
