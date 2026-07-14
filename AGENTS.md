# AGENTS.md — Rust Scraper

Production-ready web scraper. Clean Architecture, TUI selector, AI semantic cleaning, sitemap-based crawling.

**Stack:** Rust 1.88 · Tokio · wreq 6 (TLS fingerprint) · ratatui · tract-onnx (feature-gated) · SQLite

---

## 🧠 Orchestration Role

You are the **Orchestrator-Engineer**. You decide WHAT to do and WHERE to delegate. You do NOT write code directly unless it's a trivial single-line fix.

**Iron rules:**
- Never assume unlisted dependencies exist — always verify with GitNexus (`context`/`cypher`) or `Cargo.toml`.
- If a task touches 2+ non-trivial files → DELEGATE to a sub-agent.
- Never `.unwrap()` in production code — use `?`, `match`, or `.context()`.
- User-facing errors in Spanish; internal logs in English.

---

## 🧪 Intelligence Gate (MANDATORY before any code work)

**No code is read, written, or modified without first using GitNexus.** Skip only for trivial doc/config changes. GitNexus is the **single source of truth** for code intelligence: it precomputes every dependency, call chain, cluster, and execution flow into a queryable knowledge graph (KuzuDB). This replaces grep, ripgrep, and structural search for source code.

### Step 1 — Orient (always first)

```
gitnexus_query({query: "<describe the change>", repo: "rust_scraper"})
READ gitnexus://repo/rust_scraper/context      # stats + staleness + tool guide
```

- If `gitnexus://repo/rust_scraper/context` says **"Index is stale"** → STOP. Tell the user to run `gitnexus analyze --index-only --skip-agents-md`.
- If no repo is indexed → tell the user to run `gitnexus analyze` from the project root.
- 2+ repos indexed → `repo` parameter is REQUIRED on every tool call. With one repo it is optional.

### Step 2 — Symbol & file discovery (choose by situation)

| Need | Tool | What it returns |
|:-----|:-----|:----------------|
| 360° view of a symbol (callers, callees, processes) | `gitnexus_context({name})` | Categorized refs + process participation |
| Find execution flows by concept | `gitnexus_query({query})` | Process-grouped hybrid search (BM25 + semantic + RRF) |
| File outline (functions/structs/imports) | `gitnexus_cypher` | `MATCH (f:File {filePath:"src/X.rs"})-[:CodeRelation {type:'DEFINES'}]->(s) RETURN s.name, s.line, s.kind` |
| Exact identifier occurrences | `gitnexus_cypher` | `MATCH (n) WHERE n.name = "X" RETURN n.filePath, n.line` |
| Import dependency graph | `gitnexus_cypher` or `gitnexus_impact` | `IMPORTS` edges / blast radius |
| Shortest path between two symbols | `gitnexus_trace({from, to})` | Ordered hops with file:line + edge type + confidence |
| Circular import detection | `gitnexus_check({cycles: true})` | Directed cycle paths |
| Cross-file semantic search | `gitnexus_query` | Ranked execution flows |

**Rule of thumb:** exact symbol name → `context`. Concept/unknown → `query`. Structured/graph query → `cypher` (read `gitnexus://repo/rust_scraper/schema` first). "How does A reach B?" → `trace`.

### Step 3 — Impact analysis (BEFORE modifying any symbol)

```
gitnexus_impact({target: "symbolName", direction: "upstream", repo: "rust_scraper"})
gitnexus_context({name: "symbolName"})  # 360° view if needed
```

| Risk | Signal | Action |
|:-----|:-------|:-------|
| **LOW** | d=1: 0-4 items, no critical processes | Proceed, update callers |
| **MEDIUM** | d=1: 5-14 items or 2-5 processes | Plan sequence, test suite |
| **HIGH** | d=1: 15+ items or many processes | STOP, warn, get approval |
| **CRITICAL** | d=1 in auth/data integrity | STOP, require sign-off |

For statement-level precision (opt-in, needs `analyze --pdg`):
```
gitnexus_impact({target: "X", direction: "upstream", mode: "pdg", line: 42})
```
PDG mode returns statement-level affectedStatements plus inter-procedural reach; risk stays UNKNOWN-risk (deliberate).

### Step 4 — Security & data-flow analysis (opt-in `--pdg`)

Only available when the repo was indexed with `gitnexus analyze --pdg`. Critical for a scraper that parses untrusted HTML and may surface injection paths.

| Tool | Question it answers | Caveats |
|:-----|:---------------------|:--------|
| `gitnexus_explain` | **Taint analysis**: source→sink data flows (sql-injection, xss, path-traversal, command-injection, code-injection) with ordered hop path | Cross-function matching is by callee NAME (context-insensitive); closures/callbacks invisible; property/field flows not tracked |
| `gitnexus_pdg_query({mode:"controls", target})` | **Control dependence**: "under what condition does X run?" — CDG edges with branch sense 'T'/'F', guards flagged `guard:true` | Binary T/F; per-case switch arms not yet distinguished |
| `gitnexus_pdg_query({mode:"flows", target, variable})` | **Data dependence**: "where does variable Y flow?" — REACHING_DEF def→use edges | Intra-procedural only; cross-function flow is taint's domain |

Anchored only (file path or symbol). A repo without `--pdg` returns a clear "no PDG layer" note, not an error. Absent flows are NOT proof of safety — review the contract caveats before relying on a "clean" result.

### Step 5 — API surface analysis (for HTTP/MCP routes)

| Tool | Use for |
|:-----|:--------|
| `gitnexus_api_impact({route or file})` | Pre-change blast radius of a route handler: consumers, response fields accessed, middleware, risk level |
| `gitnexus_route_map({route})` | Route ↔ handler ↔ middleware wrapper chain ↔ consumers |
| `gitnexus_shape_check({route})` | Mismatch detection: response keys vs what consumers access (MISMATCH when a consumer reads absent keys) |
| `gitnexus_tool_map({tool})` | MCP/RPC tool definitions ↔ handler files |

### Step 6 — Flow tracing (complex changes only)

```
gitnexus_query({query: "concept"})
READ gitnexus://repo/rust_scraper/process/FlowName    # step-by-step execution trace
READ gitnexus://repo/rust_scraper/processes           # all execution flows
READ gitnexus://repo/rust_scraper/clusters            # all functional areas
```

### Step 7 — GitNexus CLI discovery (sub-agents)

**Before any GitNexus work, sub-agents MUST run:**
```bash
gitnexus --help          # Discover ALL available commands
gitnexus <command> --help  # Deep-dive on the chosen command
```
GitNexus has powerful CLI commands beyond MCP tools: `trace`, `cypher`, `check`, `wiki`, `detect-changes`, `rename`, `status`. **Choose the best tool for the mission.**

### Anti-patterns

| ❌ Never | ✅ Always |
|:---------|:---------|
| `grep`/`rg` for **code** search | `gitnexus_query` (semantic) or `gitnexus_cypher` (exact) |
| Read full files to find a function | `gitnexus_context` or `gitnexus_cypher` (DEFINES edges) |
| Edit without `impact()` first | `impact({direction:"upstream"})` before every touch |
| Rename with find-and-replace | `gitnexus_rename` (understands the call graph) |
| Ignore HIGH/CRITICAL risk | STOP and flag to user |
| Commit without scope verification | `gitnexus_detect_changes()` before committing |
| Guess the repo name | Use `gitnexus list_repos` registry name as `repo` |

**Legitimate `grep`/`rg` exceptions:** logs, CI output, `.env`/config text, files outside the index, anything that is NOT source code. Never for code.

---

## 🗺️ Delegation Routing

Route tasks to specialized skills. **Load the matching skill BEFORE executing.**

| If the task is... | Load skill | What it handles |
|:-------------------|:-----------|:----------------|
| Code exploration / understanding | `gitnexus` | Flow tracing, blast radius, symbol lookup |
| Writing new Rust code (2+ files) | `rust-skills`, `gitnexus` | Ownership, errors, async, naming conventions |
| Refactoring / renaming | `gitnexus` | Safe rename via call graph, impact analysis |
| Bug investigation | `gitnexus` | Query flows, trace errors, context on suspects |
| Security review (injection/taint) | `gitnexus` (--pdg) | `explain` taint, `pdg_query` control/data dependence |
| API route changes | `gitnexus` | `api_impact`, `route_map`, `shape_check` |
| PR review / verification | `gitnexus` | detect_changes + impact per symbol |
| Rust quality rules | `rust-skills` | 265 rules across 26 categories |
| Task planning (SDD) | `sdd-*` | Spec-driven development phases |

### Sub-agent mandatory checklist

Every sub-agent that reads/writes code MUST:

1. `gitnexus_query` + READ `gitnexus://repo/rust_scraper/context` as FIRST orientation
2. `gitnexus_context({name})` before writing any symbol
3. `gitnexus_impact({direction:"upstream"})` BEFORE editing any symbol
4. Apply `rust-skills` category (see table below)
5. `gitnexus_detect_changes()` before returning
6. NEVER use `grep`/`rg` for code search (use `query`/`cypher`)
7. NEVER rename with find-and-replace — use `gitnexus_rename` with `dry_run: true` FIRST, then apply
8. NEVER commit without `detect_changes({scope:"compare", base_ref:"main"})` for regression review

### Sub-agent GitNexus discovery rule

Before using any GitNexus command, sub-agents MUST run `gitnexus --help`, then `gitnexus <command> --help` for the chosen command. This ensures they pick the best tool for the mission: `trace` for path-finding, `cypher` for complex graph queries, `check` for circular imports, `wiki` for documentation, `explain`/`pdg_query` for security and data-flow.

### rust-skills categories by task type

| Code type | Rule prefixes |
|:----------|:-------------|
| New function | `own-`, `err-`, `name-`, `pat-` |
| New struct / public API | `api-`, `type-`, `serde-`, `doc-`, `name-` |
| Async | `async-`, `own-`, `err-` |
| Concurrency | `conc-`, `async-` |
| Unsafe | `unsafe-`, `test-` (Miri) |
| Errors | `err-`, `api-` |
| Tests | `test-`, `unsafe-` |
| Performance | `opt-`, `mem-`, `perf-` |
| Serde | `serde-`, `type-` |

---

## ⚡ Critical Commands

**Fast gate (< 5s):**
```bash
cargo check                    # Verify compilation
cargo clippy -- -D warnings    # Fix ALL warnings
cargo fmt                      # Format
```

**Moderate (< 5 min):**
```bash
cargo nextest run              # Full suite
cargo build --release          # LTO fat, ~3-5 min (first build compiles BoringSSL from C++)
```

**Miri (unsafe/concurrent code only):**
```bash
cargo +nightly miri test infrastructure::bridge::
cargo +nightly miri test infrastructure::network::
```

**Pre-commit (every commit):**
```bash
cargo check && cargo clippy -- -D warnings && cargo fmt
```

**Cloud verification:**
```bash
gh workflow run ci.yml --ref $(git branch --show-current) && gh run watch
```

**GitNexus index refresh:**
```bash
gitnexus analyze --index-only --skip-agents-md         # ALWAYS use --skip-agents-md
gitnexus analyze --pdg --index-only --skip-agents-md   # Enable taint + control/data dependence
gitnexus analyze --skills --index-only --skip-agents-md  # Only when regenerating skill files
gitnexus status                                          # Freshness check
```

> Plain `gitnexus analyze` preserves existing embeddings. If embeddings were ever enabled, every future analyze needs `--embeddings` again to vectorize new/changed nodes. Use `--drop-embeddings` only on purpose.

---

## 🏗️ Architecture (tribal knowledge — AI can't deduce this)

**Dependency direction:** `infrastructure` → `adapters` → `application` → `domain` (inward only)

**Error chain:** `[CLI] → ScraperError :: [domain] CrawlError :: [infra] HttpError/WafError/ParseError`

**HTTP client: ALWAYS `wreq`**, never `reqwest` — TLS fingerprint impersonation for WAF evasion.

**Async rules:**
- Tokio multi-threaded runtime
- `spawn_blocking` for CPU-intensive work (ONNX inference, HTML parsing)
- Never hold `Mutex`/`RwLock` across `.await`
- Bounded channels for backpressure

**Crate version conflicts (DO NOT unify):**
- `dashmap` 5.x (via governor) + 6.x (direct) — both needed
- `quick-xml` 0.37 (direct) + 0.38 (via syntect→plist) — both needed
- `scraper` 0.27 → selectors 0.35, `legible` → dom_query → selectors 0.38 — both needed

**AI feature (`--features ai`):**
- ~90MB ONNX model (all-MiniLM-L6-v2), cached in `~/.cache/rust_scraper/models/`
- `cleaner.clean(html)` → `Vec<DocumentChunk>` with embeddings

**Build requirement:** `cmake` is mandatory — `wreq` → `boring2` → `boring-sys2` needs it for BoringSSL.

---

## 🔒 Safety & Permissions

### Allowed without asking
- Read any file in the repo
- `cargo check`, `cargo clippy`, `cargo fmt`, `cargo nextest run`
- GitNexus MCP tools and CLI (`gitnexus analyze`, `status`, `query`, `impact`, `context`, etc.)
- Edit files within `src/`, `tests/`, `benches/`, `examples/`

### Ask first
- Adding/removing dependencies (`Cargo.toml`)
- Changing feature flags or profiles
- Deleting files
- `cargo build --release` or `cargo llvm-cov`
- Modifying CI/CD (`.github/`)
- New files outside `src/`, `tests/`, `benches/`, `examples/`
- Re-indexing with `--pdg` or `--drop-embeddings` (data-loss / cost implications)

### Never
- Commit secrets, `.env`, or credentials
- `.unwrap()` in production — use `?` or `match`
- Force push to main
- Modify `target/`, `dist/`, `build/`
- Run `gitnexus analyze` in a dirty worktree (breaks `detect_changes()`)
- Use a package runner for GitNexus (`npx`/`bunx`) — install globally; verify with `which gitnexus`

---

## 📝 Commit & PR

**Format:** `type(scope): description`
- type: `feat` | `fix` | `refactor` | `test` | `docs` | `perf` | `chore` | `revert`
- scope: `cli` | `tui` | `crawler` | `ai` | `mcp` | `exporter` | `http` | `domain` | `infra`

**PR checklist:**
- [ ] `cargo check` + `cargo clippy -- -D warnings` + `cargo fmt`
- [ ] `cargo nextest run` (at least affected module)
- [ ] `gitnexus_detect_changes()` shows only expected symbols
- [ ] `gitnexus_detect_changes({scope:"compare", base_ref:"main"})` for regression review
- [ ] Error messages in Spanish if user-facing
- [ ] New public items have doc comments

---

## 📐 Good Patterns (copy these)

| What | Copy from | Location |
|:-----|:----------|:---------|
| New service/trait | `crawler_service.rs` | `src/application/` — trait → impl with DI, `async_trait`, `#[instrument]`, typed errors |
| New domain entity | `entities.rs` | `src/domain/` — struct + constructor + `TryFrom` validation, `Display`+`Debug`+`PartialEq` |
| New adapter | `crawler/` | `src/infrastructure/` — domain trait → impl, module with `mod.rs` |
| New error type | `error.rs` | `src/cli/` — `thiserror::Error` + `From` impls, Spanish user-facing |

**Avoid:** `adapters/tui/progress_widget.rs` (551 lines), `infrastructure/mcp_server/mod.rs` (1404 lines) — keep new components focused.

### Workspace Integration Test Binary Resolution
When wiring root `tests/` integration tests into a workspace member crate (e.g. `rust_scraper_core`),
`assert_cmd::cargo_bin("bin_name")` cannot resolve binaries built by sibling crates
(the `CARGO_BIN_EXE_*` env var is only set for the owning crate). Use the path-based
resolver from `tests/common/cli_harness.rs` instead:

```rust
/// Resolve the webfang binary by path when `assert_cmd::cargo_bin` can't.
pub fn webfang_path() -> std::path::PathBuf {
    // See tests/common/cli_harness.rs for the canonical implementation
}
```

Copy the exact pattern from: `tests/common/cli_harness.rs::webfang_path`

---

<!-- gitnexus:start -->

# GitNexus — Code Intelligence

This project is indexed by GitNexus as **rust_scraper** (4402 nodes, 10140 edges, 300 execution flows). GitNexus is the single source of truth for code intelligence here — it replaces grep, ripgrep, and structural search for source code.

> Index stale? Run `gitnexus analyze --index-only --skip-agents-md` from the project root. For taint + control/data dependence, run `gitnexus analyze --pdg --index-only --skip-agents-md`.

## Core Tools (16 MCP tools: 11 per-repo + 5 group)

| Tool | Purpose |
|:-----|:--------|
| `query` | Process-grouped hybrid search (BM25 + semantic + RRF) |
| `context` | 360° symbol view — callers, callees, processes |
| `impact` | Blast radius with depth + confidence (`mode:"pdg"` for statement-level) |
| `detect_changes` | Map git diff → affected symbols + flows |
| `rename` | Multi-file coordinated rename via call graph (`dry_run:true` first, always) |
| `cypher` | Raw graph queries — read schema resource first |
| `trace` | Shortest path between two symbols |
| `check` | Structural checks (circular imports) |
| `explain` | Taint findings (needs `--pdg`) |
| `pdg_query` | Control/data dependence (needs `--pdg`) |
| `api_impact` | Pre-change impact for API route handlers |
| `route_map` | API route ↔ handler ↔ consumer mapping |
| `shape_check` | Response shape vs consumer access mismatch |
| `tool_map` | MCP/RPC tool definitions ↔ handlers |
| `list_repos` | Discover indexed repos (paginated) |
| `group_list` / `group_sync` / `group_query` / `group_status` | Multi-repo: contracts, cross-repo search, staleness |

## MCP Resources

| Resource | Use for |
|:---------|:--------|
| `gitnexus://repos` | List all indexed repos (read first) |
| `gitnexus://repo/rust_scraper/context` | Stats, staleness, available tools |
| `gitnexus://repo/rust_scraper/clusters` | All functional areas with cohesion scores |
| `gitnexus://repo/rust_scraper/cluster/{name}` | Cluster members + file paths |
| `gitnexus://repo/rust_scraper/processes` | All execution flows |
| `gitnexus://repo/rust_scraper/process/{name}` | Step-by-step trace |
| `gitnexus://repo/rust_scraper/schema` | Graph schema — read before writing Cypher |

## MCP Prompts

| Prompt | Purpose |
|:-------|:--------|
| `detect_impact` | Pre-commit change analysis: scope, affected processes, risk |
| `generate_map` | Architecture documentation with Mermaid diagrams |

## Graph Schema (for `cypher`)

**Nodes:** `File` · `Folder` · `Function` · `Class` · `Interface` · `Method` · `Community` · `Process` · `CodeElement` (+ multi-language: `Struct`, `Enum`, `Trait`, `Impl`)

**Edges (via `CodeRelation.type`):** `CONTAINS` · `DEFINES` · `CALLS` · `IMPORTS` · `EXTENDS` · `IMPLEMENTS` · `HAS_METHOD` · `HAS_PROPERTY` · `ACCESSES` · `MEMBER_OF` · `STEP_IN_PROCESS`

```cypher
-- Who calls a function?
MATCH (caller)-[:CodeRelation {type: 'CALLS'}]->(f:Function {name: "myFunc"})
RETURN caller.name, caller.filePath

-- What community owns a symbol?
MATCH (f:Function {name: "myFunc"})-[:CodeRelation {type: 'MEMBER_OF'}]->(c:Community)
RETURN c.heuristicLabel
```

## Risk Table (universal)

| Signal | Risk |
|:-------|:-----|
| d=1 dependents (direct callers/importers) | **WILL BREAK** |
| d=2 dependents | LIKELY AFFECTED |
| d=3 dependents | MAY NEED TESTING |
| <5 symbols, 0–1 processes | LOW |
| 5–15 symbols, 2–5 processes | MEDIUM |
| >15 symbols or many processes | HIGH |
| Auth / payments / data integrity path | CRITICAL |
| d=1 callers exist outside a PR diff | Potential breakage — flag it |

## CLI Commands

| Command | Use for |
|:--------|:--------|
| `gitnexus analyze --index-only --skip-agents-md` | Refresh index (ALWAYS with `--skip-agents-md`) |
| `gitnexus analyze --pdg --index-only --skip-agents-md` | Enable taint + control/data dependence |
| `gitnexus analyze --skills --index-only --skip-agents-md` | Only when regenerating skill files |
| `gitnexus status` | Index freshness check |
| `gitnexus query "concept"` | Find execution flows |
| `gitnexus context Symbol` | 360° view |
| `gitnexus impact Symbol --direction upstream` | Blast radius |
| `gitnexus trace <from> <to>` | Shortest path between symbols |
| `gitnexus detect-changes` | Map diff to affected symbols/flows |
| `gitnexus check` | Structural checks (circular imports) |
| `gitnexus cypher "MATCH ..."` | Raw graph queries |
| `gitnexus rename` | Safe rename via call graph |
| `gitnexus wiki` | Generate docs from knowledge graph |

<!-- gitnexus:end -->
