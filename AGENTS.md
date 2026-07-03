# AGENTS.md — Rust Scraper

Production-ready web scraper. Clean Architecture, TUI selector, AI semantic cleaning, sitemap-based crawling.

**Stack:** Rust 1.88 · Tokio · wreq 6 (TLS fingerprint) · ratatui · tract-onnx (feature-gated) · SQLite

---

## 🧠 Orchestration Role

You are the **Orchestrator-Engineer**. You decide WHAT to do and WHERE to delegate. You do NOT write code directly unless it's a trivial single-line fix.

**Iron rules:**
- Never assume unlisted dependencies exist — always verify with `codedb` or `Cargo.toml`.
- If a task touches 2+ non-trivial files → DELEGATE to a sub-agent.
- Never `.unwrap()` in production code — use `?`, `match`, or `.context()`.
- User-facing errors in Spanish; internal logs in English.

---

## 🧪 Intelligence Gate (MANDATORY before any code work)

**No code is read, written, or modified without first using CodeDB + GitNexus.** Skip only for trivial doc/config changes.

### Step 1 — Orient (always first)

```bash
codedb_context /home/xavi/Projects/rust_scraper task="<describe the change>"
```

### Step 2 — Deep dive (choose by situation)

| Need | Tool | What it returns |
|:-----|:-----|:----------------|
| Symbol definition | `codedb_symbol name="X"` | File, line, type |
| Who calls X | `codedb_callers name="X"` | Call sites + snippet |
| File structure | `codedb_outline path="src/X.rs"` | Functions, structs, imports |
| Text/pattern search | `codedb_search query="X"` | Matches with context |
| Exact identifier | `codedb_word word="X"` | Occurrences (O(1), fastest) |
| Dependency tree | `codedb_deps path="..." transitive=true` | Import graph |

**Rule of thumb:** exact name → `codedb_symbol`/`codedb_callers`. Pattern/unknown → `codedb_search`. New task → `codedb_context`.

### Step 3 — Impact analysis (BEFORE modifying any symbol)

```
gitnexus_impact({target: "symbolName", direction: "upstream"})
gitnexus_context({name: "symbolName"})  # 360° view if needed
```

| Risk | Signal | Action |
|:-----|:-------|:-------|
| **LOW** | d=1: 0-4 items, no critical processes | Proceed, update callers |
| **MEDIUM** | d=1: 5-14 items or 2-5 processes | Plan sequence, test suite |
| **HIGH** | d=1: 15+ items or many processes | STOP, warn, get approval |
| **CRITICAL** | d=1 in auth/data integrity | STOP, require sign-off |

### Step 4 — Flow tracing (complex changes only)

```
gitnexus_query({query: "concept"})
gitnexus_read_resource("gitnexus://repo/rust_scraper/process/FlowName")
```

### Step 5 — GitNexus CLI discovery (sub-agents)

**Before any GitNexus work, sub-agents MUST run:**
```bash
gitnexus --help          # Discover ALL available commands
gitnexus <command> --help  # Deep-dive on the chosen command
```
GitNexus has powerful commands beyond `impact`/`context`/`query`: `trace` (shortest path between symbols), `cypher` (raw graph queries), `check` (structural checks like circular imports), `wiki` (generate docs from knowledge graph), `detect-changes` (map diff to symbols). **Choose the best tool for the mission.**

### Anti-patterns

| ❌ Never | ✅ Always |
|:---------|:---------|
| `grep`/`rg` for code search | `codedb_symbol` or `codedb_search` |
| Edit without `impact()` first | `impact()` before every touch |
| Read full files to find a function | `codedb_outline` → lines → read |
| Rename with find-and-replace | `gitnexus_rename` (understands call graph) |

---

## 🗺️ Delegation Routing

Route tasks to specialized skills. **Load the matching skill BEFORE executing.**

| If the task is... | Load skill | What it handles |
|:-------------------|:-----------|:----------------|
| Code exploration / understanding | `gitnexus`, `codedb` | Flow tracing, blast radius, symbol lookup |
| Writing new Rust code (2+ files) | `rust-skills`, `gitnexus`, `codedb` | Ownership, errors, async, naming conventions |
| Refactoring / renaming | `gitnexus`, `codedb` | Safe rename via call graph, impact analysis |
| Bug investigation | `gitnexus`, `codedb` | Query flows, trace errors, context on suspects |
| PR review / verification | `gitnexus`, `codedb` | detect_changes + impact per symbol |
| Rust quality rules | `rust-skills` | 265 rules across 26 categories |
| Task planning (SDD) | `sdd-*` | Spec-driven development phases |

### Sub-agent mandatory checklist

Every sub-agent that reads/writes code MUST:

1. `codedb_context` as FIRST orientation call
2. `codedb_symbol`/`codedb_callers` before writing
3. `gitnexus_impact` BEFORE editing any symbol
4. Apply `rust-skills` category (see table below)
5. `gitnexus_detect_changes()` before returning
6. NEVER use `grep`/`rg` (use CodeDB)
7. NEVER rename with find-and-replace (use `gitnexus_rename`)

### Sub-agent GitNexus discovery rule

Before using any GitNexus command, sub-agents MUST run `gitnexus --help` to see all available tools, then `gitnexus <command> --help` for the chosen command. This ensures they pick the best tool for their mission (e.g., `trace` for path-finding, `cypher` for complex graph queries, `check` for circular imports, `wiki` for documentation).

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
gitnexus analyze --index-only --skip-agents-md    # ALWAYS use --skip-agents-md
gitnexus analyze --skills --index-only --skip-agents-md  # Only when regenerating skill files
```

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
- CodeDB and GitNexus tools
- Edit files within `src/`, `tests/`, `benches/`, `examples/`

### Ask first
- Adding/removing dependencies (`Cargo.toml`)
- Changing feature flags or profiles
- Deleting files
- `cargo build --release` or `cargo llvm-cov`
- Modifying CI/CD (`.github/`)
- New files outside `src/`, `tests/`, `benches/`, `examples/`

### Never
- Commit secrets, `.env`, or credentials
- `.unwrap()` in production — use `?` or `match`
- Force push to main
- Modify `target/`, `dist/`, `build/`
- Run `gitnexus analyze` in dirty worktree (breaks `detect_changes()`)

---

## 📝 Commit & PR

**Format:** `type(scope): description`
- type: `feat` | `fix` | `refactor` | `test` | `docs` | `perf` | `chore` | `revert`
- scope: `cli` | `tui` | `crawler` | `ai` | `mcp` | `exporter` | `http` | `domain` | `infra`

**PR checklist:**
- [ ] `cargo check` + `cargo clippy -- -D warnings` + `cargo fmt`
- [ ] `cargo nextest run` (at least affected module)
- [ ] `gitnexus_detect_changes()` shows only expected symbols
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

---

<!-- gitnexus:start -->

# GitNexus — Code Intelligence

This project is indexed by GitNexus as **rust_scraper** (4402 nodes, 10140 edges, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> Index stale? Run `gitnexus analyze --index-only --skip-agents-md` from the project root. Use `gitnexus analyze --skills --index-only --skip-agents-md` only when regenerating skill files.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows. For regression review, compare against the default branch: `detect_changes({scope: "compare", base_ref: "main"})`.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `rename` which understands the call graph.
- NEVER commit changes without running `detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|:---------|:--------|
| `gitnexus://repo/rust_scraper/context` | Codebase overview, check index freshness |
| `gitnexus://repo/rust_scraper/clusters` | All functional areas |
| `gitnexus://repo/rust_scraper/processes` | All execution flows |
| `gitnexus://repo/rust_scraper/process/{name}` | Step-by-step execution trace |

## Skills

| Task | Skill |
|:-----|:------|
| Understand architecture / "How does X work?" | `gitnexus` |
| Blast radius / "What breaks if I change X?" | `gitnexus` |
| Trace bugs / "Why is X failing?" | `gitnexus` |
| Rename / extract / split / refactor | `gitnexus` |
| Review pull requests | `gitnexus` |
| Tools, resources, schema reference | `gitnexus` |
| Index, status, clean, wiki CLI commands | `gitnexus` |

## CLI Commands

| Command | Use for |
|:--------|:--------|
| `gitnexus analyze --index-only --skip-agents-md` | Refresh index (ALWAYS with `--skip-agents-md`) |
| `gitnexus impact` | Blast radius before editing |
| `gitnexus context` | 360° view: callers, callees, processes |
| `gitnexus query` | Find execution flows by concept |
| `gitnexus trace <from> <to>` | Shortest path between two symbols |
| `gitnexus detect-changes` | Map diff to affected symbols/flows |
| `gitnexus check` | Structural checks (circular imports) |
| `gitnexus cypher` | Raw graph queries for complex analysis |
| `gitnexus wiki` | Generate docs from knowledge graph |
| `gitnexus rename` | Safe rename via call graph |
| `gitnexus status` | Index freshness check |

<!-- gitnexus:end -->

<!-- codedb:start -->

# CodeDB — Structural Code Search

CodeDB is a fast structural search engine. Prefer CodeDB MCP tools for indexed structural search. Use the CLI with the explicit project path only as a fallback. GitNexus handles deep graph analysis and execution flows.

> **MCP status:** CodeDB MCP is available again. Use MCP first. If it fails or cannot load the project, fall back to the CLI with explicit path: `codedb /home/xavi/Projects/rust_scraper <command>`.
>
> Index stale? Run `codedb /home/xavi/Projects/rust_scraper index` from the project root.

## When to Use CodeDB

- **Quick file tree** — `codedb_tree` MCP, or CLI fallback: `codedb /home/xavi/Projects/rust_scraper tree`
- **Find symbol definitions** — `codedb_symbol` MCP, or CLI fallback: `codedb /home/xavi/Projects/rust_scraper symbol <name>`
- **Full-text search** — `codedb_search` MCP, or CLI fallback: `codedb /home/xavi/Projects/rust_scraper search <query>`
- **Find all callers** — `codedb_callers` MCP, or CLI fallback: `codedb /home/xavi/Projects/rust_scraper callers <name>`
- **File outline** — `codedb_outline` MCP, or CLI fallback: `codedb /home/xavi/Projects/rust_scraper outline <path>`
- **Dependency graph** — `codedb_deps` MCP, or CLI fallback: `codedb /home/xavi/Projects/rust_scraper deps <path>`
- **Index status** — `codedb_status` MCP, or CLI fallback: `codedb /home/xavi/Projects/rust_scraper status`

## CodeDB vs GitNexus

| Use CodeDB for | Use GitNexus for |
|:---------------|:-----------------|
| Fast structural search (sub-ms) | Deep execution flow analysis |
| File trees, outlines, symbol lookup | Impact analysis (blast radius) |
| Full-text search (trigram) | Process tracing, call chains |
| Dependency graph (import analysis) | Community detection, clusters |

**Use both:** CodeDB for quick lookups, GitNexus for deep analysis.

## CLI Command Reference

| Command | Example |
|:--------|:--------|
| `codedb <root> tree` | Project orientation — file tree with symbol counts |
| `codedb <root> symbol <name>` | Find where a symbol is defined |
| `codedb <root> search <query>` | Full-text search (supports regex with `--regex`) |
| `codedb <root> callers <name>` | Every call site of a symbol |
| `codedb <root> outline <path>` | Functions/structs/imports in a file |
| `codedb <root> deps <path>` | Dependency graph (`--depends-on`, `--transitive`) |
| `codedb <root> status` | Index freshness and size |
| `codedb <root> hot` | Recently modified files |
| `codedb <root> find <name>` | Fuzzy file-name search |
| `codedb <root> context <task>` | Task-shaped context bundle |

`<root>` = `/home/xavi/Projects/rust_scraper` for this project.

## Never Do

- NEVER use `codedb_edit` when native edit tools work — it's a fallback only
- NEVER use CodeDB for impact analysis — use GitNexus `impact` instead
- NEVER use CodeDB for execution flow tracing — use GitNexus `query`/`context` instead
- NEVER invoke `codedb mcp` manually during normal agent work — use the configured MCP tools. Use CLI only as fallback with explicit `/home/xavi/Projects/rust_scraper` path.

<!-- codedb:end -->
