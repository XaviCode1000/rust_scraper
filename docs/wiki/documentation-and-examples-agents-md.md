# Documentation and Examples — AGENTS.md

# Agent Instructions — Rust Scraper

This document provides instructions and guidelines for interacting with the Rust Scraper project, particularly for AI agents. It covers development workflows, command usage, project structure, and best practices to ensure efficient and correct development.

---

## Project Overview

The Rust Scraper is a production-ready web scraping tool built with Clean Architecture principles. It features a Terminal User Interface (TUI) for selector assistance and optional AI-powered semantic cleaning for extracted content.

**Technology Stack:**
*   **Language:** Rust 1.88
*   **Concurrency:** Tokio
*   **HTTP Client:** `wreq` (with TLS fingerprint emulation)
*   **TUI Framework:** `ratatui`
*   **AI/ML (feature-gated):** `tract-onnx` for ONNX model inference

**Hardware Considerations:**
The project is optimized for modest hardware (Intel i5-4590, 8GB RAM, HDD). Commands are designed to be HDD-optimized, and explicit timeouts are recommended for heavy operations.

---

## Development Workflow & Commands

This section details the commands and workflows essential for development, emphasizing efficiency and avoiding common pitfalls like HDD timeouts.

### `just` Recipes (Preferred Orchestration)

The `justfile` provides convenient recipes for common development tasks.

*   `just check`: Runs `cargo fmt` and `clippy` with strict settings.
*   `just check-fast`: Executes `cargo check` for the fastest compilation verification.
*   `just test`: Runs tests using `nextest` with 2 threads.
*   `just test-ai`: Runs `nextest` specifically for tests involving AI features.
*   `just audit`: Performs security audits (`audit`, `deny`, `machete`).
*   `just cov`: Generates an HTML coverage report.
*   `just fmt`: Formats the code according to `rustfmt` standards.
*   `just build-release`: Builds the project in release mode with optimizations.

### Raw Commands (When `just` is Unavailable)

Direct `cargo` and `nextest` commands are also supported.

*   **Compilation Verification:**
    *   `cargo check`: Fast check, recommended during development.
    *   `cargo check --features ai`: Check with AI features enabled.
*   **Linting:**
    *   `cargo clippy -- -D warnings`: Quick lint pass.
    *   `cargo clippy --all-targets --all-features -- -D warnings`: Full lint with all features.
*   **Testing:**
    *   `cargo nextest run --test-threads 2`: Run tests (always prefer `nextest` over `cargo test`).
    *   `cargo nextest run --test-threads 2 --features ai`: Run tests with AI features.
*   **Coverage:**
    *   `cargo llvm-cov --html --output-dir coverage-llvm`: Generate LLVM coverage report.
*   **Formatting Check:**
    *   `cargo fmt --check`: Verify code formatting.
*   **Background Checking:**
    *   `bacon`: Auto-runs `clippy` in the background.

**⚠️ HDD Timeout Rules:**
Initial `cargo check` can take ~4 minutes due to cold compilation. `sccache` significantly speeds up subsequent builds. **Always set explicit timeouts** for heavy commands. Prefer `cargo check` over `cargo build` during active development. Avoid `cargo build --release` unless specifically required.

### 🚀 GitNexus + Just Strategy (2026 - Anti-Timeout)

This mandatory strategy is crucial for AI agents to avoid timeouts and ensure efficient development.

#### ✅ Recommended Sequence (10-30 second iterations)

1.  **Before any changes:** Refresh GitNexus index.
    ```bash
    just analyze
    ```
2.  **After each edit:** Run tests only on affected code.
    ```bash
    just test-dev # (Note: This command is listed as 'never use' later, implying watch-dev is the primary auto-mode)
    ```
    *Correction based on later sections: `just watch-dev` is the primary automatic mode. `test-dev` might be a manual trigger if `watch-dev` is not running.*
3.  **If GitNexus provides a specific filter:** Use precise filtering.
    ```bash
    just test-filter 'package(rust_scraper) + test(contains("scraper"))'
    ```
4.  **Only at the END (when instructed "task complete"):** Run the full test suite.
    ```bash
    just test
    ```

#### 🎯 Commands for Agents (Full Pipeline 2026)

| Command                     | Usage                                     | Approx. Time | When to Use                               |
| :-------------------------- | :---------------------------------------- | :----------- | :---------------------------------------- |
| `just analyze`              | Refresh GitNexus index                    | < 8s         | At the start of a session                 |
| `just watch-dev`            | **Primary automatic mode**                | Automatic    | **During ALL development**                |
| `just test-dev-with-impact` | Manual smart tests                        | 8-25s        | If `watch-dev` is not running             |
| `just test-filter 'filter'` | Custom test filtering                     | 10-30s       | When GitNexus provides a specific filter  |
| `just test`                 | Full test suite (lighter alternative)     | 2-6 min      | As a lighter alternative to `test-ci`     |
| `just test-ci`              | **FINAL GATE** (fmt + clippy + tests)     | 2-7 min      | **Only when instructed "task complete"**  |

### 🚨 Prompts for Code Agents (2026 Best Practices)

Clear prompts are essential for AI agents, especially since they start each session without state.

#### 🎯 Definitive Prompt for Agents (Copy & Paste)

> **MANDATORY 2026 Anti-Timeout Strategy (GitNexus + Just):**
> 1. At session start: `just analyze`
> 2. Immediately after: `just watch-dev` (automatic mode)
> 3. The agent **never** runs tests manually. It only edits code and lets `watch-dev` handle the rest.
> 4. Only when I explicitly say **"task complete"** or **"finish"**, then execute `just test-ci`.
> 5. If you need to verify something specific, use `just test-filter 'filter'`.

#### 📋 Specialized Prompts by Task Type

*   **For Debugging:**
    > Use GitNexus for impact analysis before any changes. Execute `gitnexus_impact({target: "function_name"})` and report HIGH/CRITICAL risks.
*   **For Refactoring:**
    > Execute `gitnexus_detect_changes()` before commits. Never rename without `gitnexus_rename({symbol_name: "old", new_name: "new", dry_run: true})`.
*   **For New Features:**
    > Follow Clean Architecture: business logic in `domain/`, use cases in `application/`, external adapters in `infrastructure/`.

#### ⚙️ Prompt Configuration by Tool

*   **Claude Code:**
    ```bash
    ln -sf AGENTS.md CLAUDE.md
    ```
*   **Cursor:**
    ```bash
    echo "AGENTS.md contains complete instructions" > .cursorrules
    ```
*   **GitHub Copilot:**
    ```bash
    mkdir -p .github
    cp AGENTS.md .github/copilot-instructions.md
    ```

#### 📊 Prompt Effectiveness (2026 Data)

*   **Human-authored files:** +4% improvement vs. no context.
*   **Auto-generated files:** -0.5% to -2% performance impact.
*   **Optimal size:** ≤150 lines (60K+ repos adopted AAIF standard).
*   **Hierarchy:** Nested `AGENTS.md` files in subdirectories take precedence.

#### 🎨 Effective Prompt Structure

1.  **Context First:** Stack, tools, conventions.
2.  **Critical Commands:** `just analyze`, `just watch-dev`, `just test-ci`.
3.  **Clear Rules:** Automatic actions vs. requiring permission.
4.  **Concrete Examples:** References to actual repo files.
5.  **Defined Limits:** Forbidden zones, patterns to avoid.

#### ⚠️ NEVER Use These Commands (Cause Timeouts)

*   `cargo nextest run` (❌ Unnecessary full suite)
*   `cargo nextest run 2>&1 | tail -5` (❌ Pipes block output)
*   `just test-ci` (❌ Only for CI)
*   `just test-dev` (❌ Use `watch-dev` instead)

---

## Code Style

*   **Error Messages:** In Spanish.
*   **HTTP Client:** Uses `wreq`, not `reqwest`.

```rust
// Example: src/error.rs — Spanish error messages
#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("URL inválida: {0}")]
    InvalidUrl(String),
    #[error("error de red: {0}")]
    Network(String),
    #[error("WAF/CAPTCHA detectado en {url}: {provider}")]
    WafBlocked { url: String, provider: String },
}

// Example: src/application/http_client.rs — Using wreq
use wreq::Client;
use wreq_util::emulation::ClientBuilderExt;

let client = Client::builder()
    .emulate(wreq_util::emulation::KnownVersion::Chrome131)
    .build()?;
```

---

## Project Architecture

The project follows a Clean Architecture pattern.

```
src/
├── adapters/        # External adapters (HTTP, filesystem)
├── application/    # Use cases, services (CrawlerService, ScraperService)
├── cli/            # CLI argument parsing and commands
├── domain/         # Entities, value objects, domain logic
├── extractor/      # Content extraction (HTML, text)
├── infrastructure/  # AI, Obsidian, detectors, converters
│   ├── ai/         # Semantic cleaning (ONNX embeddings)
│   ├── obsidian/   # Vault detection and parsing
│   └── ...
└── lib.rs          # Main library (ScraperConfig, exports)
```

**Key Modules:**
*   `src/application/crawler_service.rs`: Crawling logic with rate limiting.
*   `src/application/scraper_service.rs`: Page scraping with SPA detection.
*   `src/infrastructure/ai/semantic_cleaner_impl.rs`: AI content cleaning implementation.
*   `src/infrastructure/obsidian/`: Integration with Obsidian vaults.
*   `src/cli/`: Command-line interface and TUI components.

---

## Non-Obvious Patterns

### Crate Version Conflicts

Do not attempt to unify these versions; they are intentionally managed.
*   `dashmap`: 5.x (via `governor`) and 6.x (direct).
*   `quick-xml`: 0.37 (direct) and 0.38 (via `syntect` → `plist`).
*   `scraper`: 0.22 → `selectors` 0.26, `legible` → `dom_query` → `selectors` 0.35.

### HTTP Client: `wreq`

Uses TLS fingerprint emulation (e.g., Chrome 131) for WAF evasion. Includes Layer 2 evasion.

### WAF Detection

Responses are scanned for 19 WAF signatures (Cloudflare, reCAPTCHA, hCaptcha, DataDome, PerimeterX, Akamai) even on HTTP 200 OK. If detected, the User-Agent is rotated and retried once. Persistent blocks result in `ScraperError::WafBlocked`.

### AI Feature (`--features ai`)

*   Loads a ~90MB ONNX model (`all-MiniLM-L6-v2`) into memory.
*   `SemanticCleanerImpl::new()` is `async` and loads the model once for reuse.
*   `cleaner.clean(html)` is `async` and returns `Vec<DocumentChunk>` with embeddings.
*   AI cleaning can result in multiple `DocumentChunk`s per page.
*   The model is cached in `~/.cache/rust_scraper/models/`.

---

## Testing Rules

*   All new functionality must have associated tests.
*   Tests must be deterministic and isolated.
*   External dependencies must be mocked.
*   **Always run `cargo nextest run`** (never `cargo test`) before marking a task complete.
*   Use `--test-threads 2` to mitigate HDD I/O bottlenecks.

### HDD Configuration (CRITICAL)

For Intel i5-4590, 8GB RAM, HDD:

```toml
# .config/nextest.toml
[profile.default]
threads-required = 2 # MAX 2 threads - prevents thrashing
retries = { backoff = "exponential", count = 2, delay = "1s" }
slow-timeout = { period = "60s", terminate-after = 3 }
```

*   **Never** use more than 2 threads during development.
*   `nextest` profiles: `dev` (fast), `agent` (conservative), `ci` (complete).

---

## Boundaries

### ✅ Always

*   Run `cargo check` before task completion.
*   Run `cargo clippy -- -D warnings` before committing.
*   Use `cargo nextest run` (never `cargo test`).
*   Use `cargo llvm-cov` (never `cargo tarpaulin`).
*   Use `bacon` for background checking (never `cargo-watch`).
*   Use `just` recipes for multi-step tasks (audit, coverage, release).

### ⚠️ Ask First

*   Adding or removing dependencies.
*   Changing feature flag structure.
*   Modifying `Cargo.toml` profiles.

### 🚫 Never

*   Commit secrets, `.env` files, or credentials.
*   Use `.unwrap()` in production code; prefer `?` or `match`.
*   Force push to `main` or protected branches.
*   Modify `target/`, `dist/`, or `build/` directories.
*   Run `cargo build --release` during development (use `cargo check`).

---

## Resources

*   [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): Detailed architecture.
*   [DEVELOPMENT.md](DEVELOPMENT.md): Development workflow and tooling.
*   [justfile](justfile): Task recipes.

---

## Search Tools

| Tool            | Usage                               |
| :-------------- | :---------------------------------- |
| `gitnexus_query`| Find code by concept (execution flows). |
| `gitnexus_context`| 360° view of symbols.               |
| `fff_find_files`| Find files by name.                 |
| `fff_grep`      | Find specific content.              |

---

## GitNexus — Code Intelligence

The project is indexed by **rust\_scraper** (3963 symbols, 6951 relationships, 300 execution flows, 118 communities). GitNexus tools are essential for understanding code, assessing impact, and safe navigation.

> If GitNexus reports a stale index, run `gitnexus analyze` first.

### Communities

| Community   | Symbols   | Cohesion |
| :---------- | :-------- | :------- |
| Application | 30        | 0.85     |
| Domain      | 26        | 0.54     |
| Ai          | 24-19     | 0.78-1.0 |
| Export      | 22-13     | 0.95-0.98|
| Downloader  | 16        | 0.94     |
| Crawler     | 12        | 0.63     |
| Tui         | 10        | 0.63     |
| Scraper     | 10        | 0.73     |

### Always Do (GitNexus)

*   **MUST run impact analysis** before editing any symbol: `gitnexus_impact({target: "symbolName", direction: "upstream"})`.
*   **MUST run `gitnexus_detect_changes()`** before committing.
*   **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk.
*   Use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping.

### When Debugging (GitNexus)

1.  `gitnexus_query({query: "<error or symptom>"})` to find execution flows.
2.  `gitnexus_context({name: "<suspect function>"})` to see callers and callees.
3.  `READ gitnexus://repo/rust_scraper/process/{processName}` to trace full flows.

### When Refactoring (GitNexus)

*   **Renaming:** MUST use `gitnexus_rename({symbol_name: "old", new_name: "new", dry_run: true})` first.
*   **Extracting/Splitting:** MUST run `gitnexus_impact` before moving code.

### Tools Quick Reference (GitNexus)

| Tool           | Command                                       |
| :------------- | :-------------------------------------------- |
| `query`        | `gitnexus_query({query: "concept"})`          |
| `context`      | `gitnexus_context({name: "symbolName"})`      |
| `impact`       | `gitnexus_impact({target: "X", direction: "upstream"})` |
| `detect_changes`| `gitnexus_detect_changes({scope: "staged"})`  |
| `rename`       | `gitnexus_rename({symbol_name: "old", new_name: "new", dry_run: true})` |

### Self-Check Before Finishing (GitNexus)

1.  `gitnexus_impact` was run for all modified symbols.
2.  No HIGH/CRITICAL risk warnings were ignored.
3.  `gitnexus_detect_changes()` confirms the expected scope.
4.  All `d=1` (WILL BREAK) dependents were updated.

---

## SDD Workflow

This project supports Spec-Driven Development (SDD) via skills in `./.opencode/skills/`.

| Skill       | Purpose                               |
| :---------- | :------------------------------------ |
| `sdd-init`  | Initialize context, detect stack.     |
| `sdd-explore`| Investigate existing code.            |
| `sdd-propose`| Create a proposal.                    |
| `sdd-spec`  | Write specifications.                 |
| `sdd-design`| Technical design.                     |
| `sdd-tasks` | Generate task list.                   |
| `sdd-apply` | Implement (using `gitnexus_impact`).  |
| `sdd-verify`| Verify against specs.                 |
| `sdd-archive`| Archive changes.                      |

### SDD + GitNexus Pipeline

1.  `just analyze` → Initialize.
2.  `gitnexus_impact` → Before editing.
3.  `gitnexus_detect_changes` → Pre-commit check.
4.  `just test-ci` → Final verification.

---

## Rust Best Practices

This project incorporates 50+ `rust-skills` rules located in `.atl/skills/rust-skills/rules/`.

| Category       | Examples                                      |
| :------------- | :-------------------------------------------- |
| Memory         | `mem-zero-copy`, `mem-smallvec`, `mem-compact-string` |
| Performance    | `perf-release-profile`, `perf-profile-first`, `perf-collect-once` |
| API Design     | `api-typestate`, `api-non-exhaustive`, `api-serde-optional` |
| Async          | `async-tokio-runtime`, `async-no-lock-await`  |
| Testing        | `test-integration-dir`, `test-tokio-async`, `test-proptest-properties` |
| Error Handling | `err-question-mark`, `err-lowercase-msg`      |

### Auto-load `rust-skills`

When writing Rust code, automatically load the skills:
```
skill(name: "rust-skills")
```
(Path: `.atl/skills/rust-skills/`)

---

## Skills

| Skill                       | Location                                             | Trigger                     |
| :-------------------------- | :--------------------------------------------------- | :-------------------------- |
| `rust-skills`               | `~/.config/opencode/skills/rust-skills/SKILL.md`     | Any Rust code               |
| `gitnexus-exploring`        | `.opencode/skills/gitnexus/gitnexus-exploring/SKILL.MD` | "How does X work?"          |
| `gitnexus-impact-analysis`  | `.opencode/skills/gitnexus/gitnexus-impact-analysis/SKILL.MD` | "What breaks if I change X?"|
| `gitnexus-debugging`        | `.opencode/skills/gitnexus/gitnexus-debugging/SKILL.MD` | "Why is X failing?"         |
| `gitnexus-refactoring`      | `.opencode/skills/gitnexus/gitnexus-refactoring/SKILL.MD` | Rename, extract, split      |
| `gitnexus-cli`              | `.opencode/skills/gitnexus/gitnexus-cli/SKILL.MD`     | Index, status, clean        |
| `gitnexus-guide`            | `.opencode/skills/gitnexus/gitnexus-guide/SKILL.MD`   | Tools, resources            |

### Area-Specific Skills

| Area        | Skill File                                  |
| :---------- | :------------------------------------------ |
| Ai (212 symbols) | `.opencode/skills/generated/ai/SKILL.MD`    |
| Application (78 symbols) | `.opencode/skills/generated/application/SKILL.MD` |
| Domain (74 symbols) | `.opencode/skills/generated/domain/SKILL.MD` |
| Export (59 symbols) | `.opencode/skills/generated/export/SKILL.MD` |
| Crawler (51 symbols) | `.opencode/skills/generated/crawler/SKILL.MD` |
| Tui (27 symbols) | `.opencode/skills/generated/tui/SKILL.MD`   |
| Obsidian (24 symbols) | `.opencode/skills/generated/obsidian/SKILL.MD` |
| Scraper (19 symbols) | `.opencode/skills/generated/scraper/SKILL.MD` |

> Index: `gitnexus analyze` · Status: `gitnexus status`