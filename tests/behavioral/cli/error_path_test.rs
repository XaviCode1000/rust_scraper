//! Error paths: unreachable host, 404, 500 responses.

use crate::assert_snapshot_redacted;
use crate::cmd;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Unreachable host → exit error, timeout message
// ---------------------------------------------------------------------------

#[test]
fn unreachable_host_exits_error() {
    cmd()
        .arg("--url")
        .arg("http://127.0.0.1:1")
        .arg("--single-page")
        .arg("--timeout-secs")
        .arg("2")
        .arg("--max-retries")
        .arg("0")
        .arg("--quiet")
        .assert()
        .failure();
}

#[test]
fn unreachable_host_exit_code_69() {
    cmd()
        .arg("--url")
        .arg("http://127.0.0.1:1")
        .arg("--single-page")
        .arg("--timeout-secs")
        .arg("2")
        .arg("--max-retries")
        .arg("0")
        .arg("--quiet")
        .assert()
        .code(69);
}

#[test]
fn unreachable_host_stderr_mentions_failure() {
    let output = cmd()
        .arg("--url")
        .arg("http://127.0.0.1:1")
        .arg("--single-page")
        .arg("--timeout-secs")
        .arg("2")
        .arg("--max-retries")
        .arg("0")
        .arg("--quiet")
        .output()
        .expect("run binary");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_snapshot_redacted("unreachable_host_stderr", Path::new("__no_temp__"), stderr);
}

// ---------------------------------------------------------------------------
// Slow server → timeout → exit error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn slow_server_timeout_exits_error() {
    let server = MockServer::start().await;
    let output = TempDir::new().unwrap();

    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("<html><body>slow</body></html>")
                .set_delay(Duration::from_secs(10)),
        )
        .expect(1)
        .mount(&server)
        .await;

    let result = cmd()
        .arg("--url")
        .arg(format!("{}/slow", server.uri()))
        .arg("--single-page")
        .arg("--timeout-secs")
        .arg("1")
        .arg("--max-retries")
        .arg("0")
        .arg("--output")
        .arg(output.path())
        .arg("--quiet")
        .output()
        .expect("run binary");

    assert_eq!(
        result.status.code(),
        Some(69),
        "slow server should time out with exit code 69"
    );
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert_snapshot_redacted("slow_server_timeout_stderr", output.path(), stderr);
}

// ---------------------------------------------------------------------------
// 404 response → exit error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn not_found_response_exits_error() {
    let server = MockServer::start().await;
    let output = TempDir::new().unwrap();

    Mock::given(method("GET"))
        .and(path("/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    cmd()
        .arg("--url")
        .arg(format!("{}/missing", server.uri()))
        .arg("--single-page")
        .arg("--max-retries")
        .arg("0")
        .arg("--output")
        .arg(output.path())
        .arg("--quiet")
        .assert()
        .failure();
}

#[tokio::test]
async fn not_found_response_exit_code_nonzero() {
    let server = MockServer::start().await;
    let output = TempDir::new().unwrap();

    Mock::given(method("GET"))
        .and(path("/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let output_result = cmd()
        .arg("--url")
        .arg(format!("{}/missing", server.uri()))
        .arg("--single-page")
        .arg("--max-retries")
        .arg("0")
        .arg("--output")
        .arg(output.path())
        .arg("--quiet")
        .output()
        .expect("run binary");

    assert_ne!(
        output_result.status.code(),
        Some(0),
        "404 response should produce a non-zero exit code"
    );
}

// ---------------------------------------------------------------------------
// 500 response → exit error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn server_error_response_exits_error() {
    let server = MockServer::start().await;
    let output = TempDir::new().unwrap();

    Mock::given(method("GET"))
        .and(path("/error"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    cmd()
        .arg("--url")
        .arg(format!("{}/error", server.uri()))
        .arg("--single-page")
        .arg("--max-retries")
        .arg("0")
        .arg("--output")
        .arg(output.path())
        .arg("--quiet")
        .assert()
        .failure();
}

#[tokio::test]
async fn server_error_response_exit_code_nonzero() {
    let server = MockServer::start().await;
    let output = TempDir::new().unwrap();

    Mock::given(method("GET"))
        .and(path("/error"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let output_result = cmd()
        .arg("--url")
        .arg(format!("{}/error", server.uri()))
        .arg("--single-page")
        .arg("--max-retries")
        .arg("0")
        .arg("--output")
        .arg(output.path())
        .arg("--quiet")
        .output()
        .expect("run binary");

    assert_ne!(
        output_result.status.code(),
        Some(0),
        "500 response should produce a non-zero exit code"
    );
}
