//! Headless TUI fallback tests (spec S2.2).
//!
//! When the `ui` feature is OFF, the `--tui`, `--config-tui`, and `--interactive`
//! flags MUST print a Spanish message and exit gracefully instead of attempting
//! to render a TUI. These tests run ONLY when `ui` is not enabled, proving the
//! core binary works headless without ratatui/crossterm.

#![cfg(not(feature = "ui"))]

use assert_cmd::Command;

/// Expected Spanish message (spec S2.2 exact wording).
const EXPECTED_MSG: &str = "TUI no disponible: compilar con --features ui";

/// Resolve the path to the `webfang` binary.
///
/// `webfang` is built by the `rust_scraper_cli` crate (a workspace sibling),
/// so `assert_cmd::cargo_bin` cannot resolve it — `CARGO_BIN_EXE_webfang`
/// is only set for the crate that owns the binary. In CI that variable is
/// absent even though the binary was built by a prior step; this fallback
/// searches `target/{debug,release}` and, as a last resort, spawns
/// `cargo build -p rust_scraper_cli --bin webfang`.
fn webfang_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_webfang") {
        return std::path::PathBuf::from(p);
    }
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
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

fn webfang_cmd() -> Command {
    Command::new(webfang_path())
}

#[test]
fn tui_flag_prints_spanish_message_when_ui_off() {
    let output = webfang_cmd()
        .arg("--tui")
        .timeout(std::time::Duration::from_secs(10))
        .output()
        .expect("failed to execute rust_scraper_core");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");

    assert!(
        !output.status.success(),
        "--tui must exit non-zero when ui is OFF; got exit {:?}\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code()
    );
    assert!(
        combined.contains(EXPECTED_MSG),
        "--tui must print the Spanish TUI-unavailable message\nexpected substring: {EXPECTED_MSG}\nstdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn config_tui_flag_prints_spanish_message_when_ui_off() {
    let output = webfang_cmd()
        .arg("--config-tui")
        .timeout(std::time::Duration::from_secs(10))
        .output()
        .expect("failed to execute rust_scraper_core");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");

    assert!(
        !output.status.success(),
        "--config-tui must exit non-zero when ui is OFF; got exit {:?}",
        output.status.code()
    );
    assert!(
        combined.contains(EXPECTED_MSG),
        "--config-tui must print the Spanish TUI-unavailable message\nexpected: {EXPECTED_MSG}\nstderr: {stderr}"
    );
}

#[test]
fn interactive_flag_prints_spanish_message_when_ui_off() {
    let output = webfang_cmd()
        .arg("--interactive")
        .timeout(std::time::Duration::from_secs(10))
        .output()
        .expect("failed to execute rust_scraper_core");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");

    assert!(
        !output.status.success(),
        "--interactive must exit non-zero when ui is OFF; got exit {:?}",
        output.status.code()
    );
    assert!(
        combined.contains(EXPECTED_MSG),
        "--interactive must print the Spanish TUI-unavailable message\nexpected: {EXPECTED_MSG}\nstderr: {stderr}"
    );
}
