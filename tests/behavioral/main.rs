//! Adapter-layer behavioral tests for the `webfang` binary.
//!
//! Every test uses wiremock (no real network) and TempDir (auto-cleanup).
//! Run with: `cargo nextest run --test behavioral`

mod cli;

use assert_cmd::Command;

/// Resolve the path to the `webfang` binary.
///
/// `webfang` is built by the `rust_scraper_cli` crate (a workspace sibling),
/// so `assert_cmd::cargo_bin` cannot resolve it — `CARGO_BIN_EXE_webfang`
/// is only set for the crate that owns the binary.  In CI that variable is
/// absent even though the binary was built by a prior step; this fallback
/// searches `target/{debug,release}` and, as a last resort, spawns
/// `cargo build -p rust_scraper_cli --bin webfang`.
pub(crate) fn webfang_path() -> std::path::PathBuf {
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

/// Shared binary command builder for tests that don't need a mock server.
pub(crate) fn cmd() -> Command {
    Command::new(webfang_path())
}

/// Shared test harness: one mock server + one temp output directory.
pub(crate) struct BehavioralTest {
    pub server: wiremock::MockServer,
    pub out: tempfile::TempDir,
}

impl BehavioralTest {
    /// Spin up a fresh mock server and temp directory.
    pub async fn new() -> Self {
        Self {
            server: wiremock::MockServer::start().await,
            out: tempfile::TempDir::new().expect("create temp output dir"),
        }
    }

    /// Build a `Command` for the `webfang` binary with `--url` and
    /// `--output` pre-filled to this harness's server and temp dir.
    pub fn scraper_cmd(&self) -> assert_cmd::Command {
        let mut cmd = assert_cmd::Command::new(webfang_path());
        cmd.arg("--url")
            .arg(self.server.uri())
            .arg("--output")
            .arg(self.out.path());
        cmd
    }

    /// Recursively find all files matching the given extension inside the
    /// output directory (files live in domain subdirs).
    pub fn find_files(&self, ext: &str) -> Vec<std::path::PathBuf> {
        walkdir::WalkDir::new(self.out.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some_and(|x| x == ext))
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    /// Read the first `.md` file found in the output directory.
    /// Panics if no `.md` file exists.
    pub fn read_md_content(&self) -> String {
        let md_files = self.find_files("md");
        assert!(
            !md_files.is_empty(),
            "expected at least one .md file in output"
        );
        std::fs::read_to_string(&md_files[0]).expect("read .md file")
    }
}
