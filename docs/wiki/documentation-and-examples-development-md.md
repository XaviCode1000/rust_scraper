# Documentation and Examples — DEVELOPMENT.md

# Development Workflow — Rust Scraper

This document outlines the development workflow, tooling, and best practices for the Rust Scraper project. It serves as a guide for developers to set up their environment, run tests, build the project, and understand performance optimizations.

## 🚀 Quick Start

For a rapid setup and to get started with common development tasks:

```bash
# Install essential tools (one-time setup)
cargo binstall just cargo-nextest cargo-llvm-cov cargo-audit cargo-machete

# Run all tests (optimized for HDD)
just test

# Perform a full code check (formatting and linting)
just check

# Start the background code checker (re-runs checks on file changes)
bacon
```

## 📦 Stack Óptimo 2025-26

The project utilizes a curated set of tools for an efficient and robust development experience.

| Tool             | Version   | Purpose                                     |
|------------------|-----------|---------------------------------------------|
| **Rust**         | 1.88      | Minimum Supported Rust Version (MSRV)       |
| **just**         | 1.49      | Task runner for orchestrating commands      |
| **cargo-nextest**| 0.9+      | Parallel and advanced test runner           |
| **cargo-llvm-cov**| latest    | Native LLVM code coverage                   |
| **cargo-audit**  | latest    | Security vulnerability scanning             |
| **cargo-deny**   | 0.16+     | License and dependency policy enforcement   |
| **cargo-machete**| 0.9+      | Detects unused dependencies                 |
| **sccache**      | 0.14      | Build artifact caching                      |
| **bacon**        | latest    | Background code checker                     |
| **mold**         | 2.40      | High-performance linker                     |

## 🛠️ Commands

The `justfile` provides convenient recipes for common development tasks.

### Just Recipes (Preferred)

```bash
just                # Default: Runs 'check' (fmt + clippy)
just check          # Runs 'cargo fmt' and 'cargo clippy -- -D warnings --all-targets --all-features'
just check-fast     # Runs 'cargo check' for a quicker syntax check
just test           # Runs 'cargo nextest run --test-threads 2'
just test-ai        # Runs 'cargo nextest run' with the 'ai' feature enabled
just audit          # Runs 'cargo audit', 'cargo deny check', and 'cargo machete'
just cov            # Generates an HTML code coverage report using 'cargo llvm-cov'
just build-release  # Builds the project in release mode: 'cargo build --release'
```

### Raw Commands

#### Tests

*   **Nextest (Recommended):**
    ```bash
    cargo nextest run --test-threads 2
    ```
*   **Run only failed tests:**
    ```bash
    cargo nextest run --failed
    ```
*   **Run ignored tests (typically interact with real external resources):**
    ```bash
    cargo nextest run --run-ignored ignored-only
    ```

#### Coverage

*   **LLVM-Cov (Recommended):** Generates an HTML report.
    ```bash
    cargo llvm-cov nextest --html --output-dir coverage-llvm
    ```

#### Build

*   **Standard Release Build:**
    ```bash
    cargo build --release
    ```
*   **With sccache:** `sccache` is typically configured globally via `~/.cargo/config.toml` as a `rustc-wrapper`.
    ```bash
    sccache --show-stats  # View cache statistics
    ```

#### Linting

*   **Clippy with warnings as errors:**
    ```bash
    cargo clippy -- -D warnings
    ```
*   **Auto-fix Clippy suggestions:**
    ```bash
    cargo clippy --fix -- -D warnings
    ```

#### Formatting

*   **Check code format:**
    ```bash
    cargo fmt --check
    ```
*   **Format code:**
    ```bash
    cargo fmt
    ```

#### Background Checking (Bacon)

*   **Run bacon:** Automatically runs `clippy` and `nextest` on file changes.
    ```bash
    bacon
    ```
*   **Keybindings within bacon:** `t` (run tests), `c` (run clippy), `n` (run nextest), `r` (run release build).

#### Audit

*   **Security vulnerabilities:**
    ```bash
    cargo audit
    ```
*   **License and dependency policy:**
    ```bash
    cargo deny check
    ```
*   **Unused dependencies:**
    ```bash
    cargo machete
    ```

## 📊 Performance Comparison

The optimized toolchain significantly improves development cycle times.

| Task         | Traditional (`cargo`) | Optimized 2025-26 (`just`/`nextest`/`sccache`) | Improvement |
|--------------|-----------------------|-------------------------------------------------|-------------|
| **Tests**    | ~30s                  | ~6s                                             | **~5x**     |
| **Coverage** | ~5min (`tarpaulin`)   | ~30s (`llvm-cov`)                               | **~10x**    |
| **Build**    | ~60s (clean)          | ~10s (`sccache`)                                | **~6x**     |
| **Linting**  | Manual                | Instant (`bacon`)                               | **Instant** |
| **Multi-step**| Manual sequential     | Orchestrated (`just audit`)                     | **Orchestrated** |

## 🔧 Configuration

### nextest.toml

Located at the project root, this file configures `cargo-nextest` for optimal performance, especially on HDDs.

```toml
[profile.default]
threads-required = 2
retries = 2
slow-timeout = { period = "60s", terminate-after = 3 }

[profile.ci]
threads-required = 4
retries = 0
```

### bacon.toml

Configuration for the background checker.

```toml
summary = true

[keybindings]
t = "nextest"
f = "nextest --failed"
c = "clippy"
r = "build --release"
```

### `.cargo/config.toml`

This file is intentionally minimal. Global configurations for tools like `sccache`, `mold`, and sparse registry are managed in the user's global `~/.cargo/config.toml` as they are machine-level optimizations.

```toml
# Cargo configuration for local development
# sccache is configured globally (~/.cargo/config.toml) via rustc-wrapper
# CI uses Swatinem/rust-cache@v2 instead
```

### justfile

The `justfile` defines recipes for orchestrating multi-step development tasks, complementing `bacon`'s inner-loop watch mode.

```makefile
check        # fmt + clippy strict
test         # nextest --test-threads 2
audit        # audit + deny + machete
cov          # coverage report
```

### sccache Stats

To manage the build cache:

```bash
# Start the sccache server (usually auto-started)
sccache --start-server

# View cache statistics
sccache --show-stats

# Reset cache statistics
sccache --zero-stats
```

## 📁 Project Structure

A high-level overview of the project's directory layout:

```
rust_scraper/
├── .cargo/
│   └── config.toml          # Minimal; sccache/mold are global
├── src/
│   ├── application/
│   │   └── http_client.rs   # HttpClient wrapper (Option A)
│   ├── domain/
│   ├── infrastructure/
│   └── adapters/
├── tests/
│   ├── http_client_integration.rs  # Real site tests (ignored)
│   └── ai_integration.rs            # AI tests (feature-gated)
├── docs/
├── justfile                 # Task runner recipes
├── nextest.toml             # Test configuration
├── bacon.toml               # Background checker configuration
├── deny.toml                # License/dependency policy
└── Cargo.toml
```

## 🎯 Testing Workflow

### 1. Daily Development

Use `bacon` in one terminal to automatically re-run checks and tests as you save files.

```bash
# Terminal 1: Start background checker
bacon

# Terminal 2: Edit code and observe results in Terminal 1
```

### 2. Before Commit

Ensure code quality and correctness before committing.

*   **Option A (Recommended):** Use the `just` recipe.
    ```bash
    just check && just test
    ```
*   **Option B (Manual):**
    ```bash
    cargo fmt
    cargo clippy -- -D warnings
    cargo nextest run
    ```

### 3. Coverage Check

Generate and review the code coverage report.

```bash
# Generate and open the coverage report
cargo llvm-cov nextest --html
open coverage-llvm/index.html
```

## 🐛 Troubleshooting

### sccache Not Working

If `sccache` is not providing expected caching benefits:

```bash
# Verify the sccache server is running and check stats
sccache --show-stats

# Stop and restart the server
sccache --stop-server
sccache --start-server
```

### Nextest Failures

If tests are failing unexpectedly:

```bash
# Clean previous build artifacts
cargo clean

# Re-run tests
cargo nextest run
```

### Coverage Generation Issues

If `cargo llvm-cov` fails to generate reports:

```bash
# Clean build artifacts and coverage data
cargo clean
# or specifically for coverage:
cargo llvm-cov nextest --clean

# Regenerate the report
cargo llvm-cov nextest --html
```

## ⚡ Build Performance

### Initial Build Time

The first build can take approximately 7 minutes due to the compilation of computationally intensive crates, many of which involve native code compilation or complex C bindings:

| Crate        | Time Estimate | Reason                               |
|--------------|---------------|--------------------------------------|
| `tract-onnx` | ~3 min        | ONNX runtime (C++ codegen)           |
| `syntect`    | ~1-2 min      | Oniguruma regex engine (C bindings)  |
| `tokenizers` | ~1 min        | NLP tokenization                     |
| `ring`       | ~30s          | Cryptography (C bindings)            |

### Speeding Up Builds with sccache

`sccache` caches compilation artifacts, drastically reducing build times for subsequent compilations. Ensure it's configured as your `RUSTC_WRAPPER` globally.

```bash
# Ensure sccache server is running
sccache --start-server

# Build the project
cargo build --release
```

**Expected Improvements:**

*   **First Build:** ~7 min (unchanged, as all code must be compiled initially).
*   **Rebuild after small change:** ~10-30 seconds (due to `sccache` hits).
*   **Rebuild after `git pull`:** ~1-2 minutes (only changed crates need recompilation).

### Building Without AI Features

To reduce build times by excluding AI-related dependencies (saving ~3 minutes):

```bash
# Standard build (excludes AI/ONNX features)
cargo build --release

# Build with all stable features (e.g., images, documents)
cargo build --release --features full
```

### Duplicate Dependencies

`cargo tree` may show duplicate versions of crates like `selectors`, `dashmap`, `lru`, and `quick-xml`. This is **expected and unavoidable** as they are required by different upstream dependencies. Refer to comments in `Cargo.toml` for detailed explanations. Do not attempt to unify these versions.

## 📚 Resources

*   [cargo-nextest docs](https://nexte.st/)
*   [cargo-llvm-cov docs](https://github.com/taiki-e/cargo-llvm-cov)
*   [sccache docs](https://github.com/mozilla/sccache)
*   [bacon docs](https://dystroy.org/bacon/)
*   [just docs](https://github.com/casey/just)
*   [cargo-deny docs](https://embarkstudios.github.io/cargo-deny/)
*   [cargo-audit docs](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
*   [cargo-machete docs](https://github.com/bnjbvr/cargo-machete)
*   [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

---

**Last updated**: 2026-04-12
**Rust version**: 1.88
**Stack**: sccache + mold + nextest + just + bacon + cargo-deny + cargo-audit + cargo-machete
**Tests**: ~271 passing (nextest)