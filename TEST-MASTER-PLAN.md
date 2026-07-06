# Test Master Plan — Clean Architecture Alignment

## Architecture Layers & Test Strategy

```
┌─────────────────────────────────────────────┐
│  ADAPTERS (CLI, TUI, MCP)                   │
│  Test: Behavioral (assert_cmd + wiremock)   │
├─────────────────────────────────────────────┤
│  APPLICATION (use cases, services)          │
│  Test: Unit with mock ports                 │
├─────────────────────────────────────────────┤
│  INFRASTRUCTURE (implementations)           │
│  Test: Integration (real I/O, temp dirs)    │
├─────────────────────────────────────────────┤
│  DOMAIN (entities, rules, value objects)    │
│  Test: Pure unit (no I/O, no mocks)         │
└─────────────────────────────────────────────┘
```

**Rule**: Tests flow inward. Domain tests know nothing about infrastructure.
Adapters tests exercise the full stack. Each layer is tested at its own level.

---

## Layer 1: DOMAIN (Pure Unit Tests)

**Where**: `src/domain/`
**Test location**: `tests/unit/domain/` or inline `#[cfg(test)]` modules
**Dependencies**: NONE — no mocks, no I/O, no network

### What to test

| Module | Entity/Rules | Tests needed |
|:-------|:-------------|:-------------|
| `entities.rs` | ScrapedContent, DocumentChunk | Constructors, validation, TryFrom, Display |
| `crawl_job/entities.rs` | DiscoveredUrl, CrawlJob | State transitions, dedup logic |
| `site/config.rs` | CrawlerConfig builder | Builder pattern, defaults, edge cases |
| `js_strategy.rs` | JsStrategy enum | Parse, display, serde roundtrip |
| `mod.rs` | ExportFormat, OutputFormat | Enum values, conversions |

### Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scraped_content_constructor_validates_url() {
        let result = ScrapedContent::new("not-a-url", "title", "content");
        assert!(result.is_err());
    }

    #[test]
    fn crawler_config_builder_sets_defaults() {
        let config = CrawlerConfig::builder("https://example.com".into()).build();
        assert_eq!(config.max_depth, 2);
        assert_eq!(config.max_pages, 10);
    }

    #[test]
    fn js_strategy_deserialize_valid_variants() {
        assert_eq!("static".parse::<JsStrategy>().unwrap(), JsStrategy::Static);
        assert_eq!("hybrid".parse::<JsStrategy>().unwrap(), JsStrategy::Hybrid);
        assert!("invalid".parse::<JsStrategy>().is_err());
    }
}
```

---

## Layer 2: APPLICATION (Unit Tests with Mock Ports)

**Where**: `src/application/`
**Test location**: `tests/unit/application/`
**Dependencies**: Mock traits (ports), no real I/O

### What to test

| Module | Use Case | Tests needed |
|:-------|:---------|:-------------|
| `scraper_service.rs` | scrape_with_readability | Mock HTTP client, verify extraction |
| `crawler_service.rs` | fetch_sitemap | Mock HTTP, verify sitemap parsing |
| `elastic_ingestion.rs` | ingest | Mock SQLite, verify dedup logic |
| `rate_limiter.rs` | RateLimiterConfig | Config validation, Redis key prefix |
| `batch/processor.rs` | process_batch | Mock scraper, verify concurrency |
| `pipeline/stages/` | validate, clean, output | Each stage independently |
| `http_client/client.rs` | get, retry logic | Mock server for 429, 500, timeout |

### Key principle: trait-based mocking

```rust
// The port (trait)
#[async_trait]
pub trait HttpClient {
    async fn get(&self, url: &Url) -> Result<Response, HttpError>;
}

// The test double
struct MockHttpClient {
    responses: HashMap<String, Response>,
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn get(&self, url: &Url) -> Result<Response, HttpError> {
        self.responses.get(&url.to_string())
            .cloned()
            .ok_or(HttpError::NotFound)
    }
}

#[tokio::test]
async fn scraper_service_extracts_title() {
    let mut mock = MockHttpClient::new();
    mock.insert("https://example.com", Response::new("<h1>Title</h1>"));
    let service = ScraperService::new(mock);

    let result = service.scrape(&url).await.unwrap();
    assert_eq!(result.title, "Title");
}
```

---

## Layer 3: INFRASTRUCTURE (Integration Tests)

**Where**: `src/infrastructure/`
**Test location**: `tests/integration/`
**Dependencies**: Real I/O with temp dirs, mock servers, in-memory DBs

### What to test

| Module | Component | Tests needed |
|:-------|:----------|:-------------|
| `crawler/sitemap_parser.rs` | SitemapParser | Parse real sitemap XML fixtures |
| `crawler/resource_downloader.rs` | ResourceDownloader | Download with mock server, verify files |
| `crawler/batch_processor.rs` | BatchProcessor | Batch with mock, verify ordering |
| `crawler/memory_manager.rs` | MemoryManager | RAM limits, eviction |
| `network/session_pool.rs` | DomainSessionPool | Health checks, stale eviction, cooldown |
| `downloader/cookie_bridge.rs` | CookieBridge | Domain matching, path matching |
| `downloader/hybrid_router.rs` | HybridRouter | Static→Obscura→Chromiumoxide escalation |
| `export/jsonl_exporter.rs` | JsonlExporter | Append, dedup, file creation |
| `persistence/sqlite/` | SqliteVectorRepository | CRUD, dedup, query |
| `checkpoint/store.rs` | BincodeCheckpoint | Save/load roundtrip |
| `obsidian/vault_detector.rs` | detect_vault | Real filesystem detection |
| `output/file_saver.rs` | FileSaver | Write markdown, create dirs |
| `ai/tokenizer.rs` | Tokenizer | Tokenize text, chunk sizing |
| `ai/semantic_cleaner_impl.rs` | SemanticCleaner | Clean with mock ONNX model |

### Example

```rust
#[tokio::test]
async fn sitemap_parser_parses_real_fixture() {
    let xml = load_fixture("sitemap_with_index.xml");
    let parser = SitemapParser::new();
    let urls = parser.parse(&xml).unwrap();
    assert!(urls.len() > 0);
    assert!(urls.iter().all(|u| u.scheme() == "https"));
}

#[tokio::test]
async fn session_pool_evicts_stale_sessions() {
    let pool = DomainSessionPool::new(config);
    // Add sessions, age them, verify eviction
}
```

---

## Layer 4: ADAPTERS (Behavioral Tests)

**Where**: `src/cli/`, `src/adapters/tui/`, `src/infrastructure/mcp_server/`
**Test location**: `tests/behavioral/`
**Dependencies**: assert_cmd, wiremock, TempDir

### What to test

| Adapter | Component | Tests needed |
|:--------|:----------|:-------------|
| CLI | Binary exit codes | --help, --version, invalid URL |
| CLI | Single page | Mock server → verify output files |
| CLI | Crawl mode | Mock server with links → verify discovery |
| CLI | Batch mode | Stdin/file → verify processing |
| CLI | Dry run | Verify no output files created |
| CLI | Output formats | markdown, json, text verification |
| CLI | Export formats | jsonl, vector file content |
| CLI | Obsidian integration | Wiki-links, tags, relative assets |
| CLI | Error handling | Network errors, invalid input |
| TUI | App state machine | Event dispatch, mode transitions |
| TUI | URL selector | Selection, filtering |
| MCP | Tool execution | Each tool via JSON-RPC |
| MCP | Session management | Initialize → list → call flow |

### Example

```rust
#[tokio::test]
async fn crawl_discovers_and_scrapes_linked_pages() {
    let server = MockServer::start().await;
    // Seed page with 2 links
    Mock::given(method("GET")).and(path("/"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(r#"<article><a href="/p1">P1</a><a href="/p2">P2</a></article>"#))
        .mount(&server).await;
    Mock::given(method("GET")).and(path("/p1"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(r#"<article><h1>Page 1</h1><p>Content.</p></article>"#))
        .mount(&server).await;
    Mock::given(method("GET")).and(path("/p2"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(r#"<article><h1>Page 2</h1><p>Content.</p></article>"#))
        .mount(&server).await;

    let out = TempDir::new().unwrap();
    Command::cargo_bin("rust_scraper").unwrap()
        .arg("--url").arg(format!("{}/", server.uri()))
        .arg("--max-pages").arg("3")
        .arg("--output").arg(out.path())
        .arg("--quiet")
        .assert().success();

    // Verify 3 files created (seed + 2 linked)
    let md_files: Vec<_> = WalkDir::new(out.path())
        .into_iter().filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
        .collect();
    assert_eq!(md_files.len(), 3);
}
```

---

## Existing Tests: Keep / Migrate / Delete

### KEEP (already correct level)

| File | Lines | Why keep |
|:-----|:------|:---------|
| `http_client_integration.rs` | 337 | Real integration with wiremock |
| `integration_engine_tests.rs` | 282 | Engine integration with mocks |
| `crawler_integration.rs` | 231 | Crawler with mock servers |
| `rate_limiting_integration.rs` | 356 | Rate limiting with wiremock |
| `security_integration.rs` | 388 | Security checks |
| `concurrency_tests.rs` | 310 | Concurrency behavior |
| `resource_downloader_integration.rs` | ~200 | Download with mocks |
| `elastic_*_test.rs` (4 files) | 1,055 | Elastic pipeline tests |
| `ai_integration.rs` | 1,029 | AI feature (feature-gated) |
| `mcp_proptest.rs` | 456 | MCP property tests |
| `progress_tui_integration.rs` | 324 | TUI progress events |

### MIGRATE (wrong level, needs rewrite)

| File | Lines | Current level | Target level |
|:-----|:------|:-------------|:-------------|
| `flag_audit_core_display_test.rs` | 366 | Plumbing | Delete → behavioral |
| `flag_audit_downloads_test.rs` | 338 | Plumbing | Delete → behavioral |
| `flag_audit_obsidian_test.rs` | 356 | Plumbing | Delete → behavioral |
| `flag_audit_sitemap_resume_test.rs` | 355 | Plumbing | Delete → behavioral |
| `flag_audit_http_crawler_test.rs` | 690 | Plumbing | Delete → behavioral |
| `cli_tests.rs` | 497 | Mixed | Split → behavioral + unit |
| `cli_binary_test.rs` | 199 | Partial behavioral | Extend |
| `mcp_live_test.rs` | 235 | Partial behavioral | Fix session + extend |
| `exporter_integration_test.rs` | 382 | Mixed | Split → unit + integration |

### KEEP AS-IS (inline tests in source)

| File | What |
|:-----|:-----|
| `src/cli/args.rs` | Proptest for Args → CrawlOptions parity |
| `src/infrastructure/crawler/sitemap_parser.rs` | Sitemap parsing unit tests |
| `src/infrastructure/export/jsonl_exporter.rs` | JSONL append/write tests |
| `src/infrastructure/checkpoint/store.rs` | Checkpoint save/load tests |
| `src/application/crawler/engine.rs` | Engine instrument tests |
| `src/infrastructure/network/session_pool.rs` | Session pool state tests |
| `src/infrastructure/downloader/cookie_bridge.rs` | Cookie domain matching |

---

## Issue Structure (GitHub)

| Phase | Issue | Title | Labels |
|:------|:------|:------|:-------|
| Epic | #110 | Clean Architecture Test Suite Alignment | enhancement, testing, roadmap |
| 0 | #111 | Fix 9 critical CLI bugs | bug, phase:0, priority:high |
| 1 | #112 | Domain Layer Unit Tests | testing, phase:1 |
| 2 | #113 | Application Layer Mock Tests | testing, phase:2 |
| 3 | #114 | Infrastructure Integration Tests | testing, phase:3 |
| 4 | #115 | Adapter Behavioral Tests | testing, phase:4 |
| 5 | #116 | Delete Plumbing Tests & Update CI | type:chore, phase:5 |
| 6a | #117 | Fix MCP Server Session Management | bug, testing, priority:high |
| 6b | #118 | Mutation Testing with cargo-mutants | testing, enhancement |
| 6c | #119 | Security Fuzzing for URLs and HTML | testing, bug, priority:medium |
| 6d | #120 | Performance Baseline & Regression | testing, enhancement |
| 6e | #121 | TUI Smoke Tests & State Machine | testing, enhancement |

---

## Extended Phases (6a-6e)

### Phase 6a: MCP Session Management (#117)
- **What:** Fix broken session lifecycle (initialize → notify → tools/list)
- **Why:** MCP server is unusable for LLM integrations without working sessions
- **Effort:** 2 hours, ~5 tests

### Phase 6b: Mutation Testing (#118)
- **What:** Verify tests actually detect code changes using cargo-mutants
- **Why:** A test that passes for mutated code is a useless test
- **Effort:** 3 hours, baseline + fix missed mutants

### Phase 6c: Security Fuzzing (#119)
- **What:** Test URL injection, HTML XSS, XML external entities, header injection
- **Why:** Scraper processes untrusted internet input — vulnerabilities are real
- **Effort:** 2 hours, ~15 tests

### Phase 6d: Performance Baseline (#120)
- **What:** Criterion benchmarks + memory thresholds + CI regression detection
- **Why:** A silent 50% slowdown would go unnoticed without baselines
- **Effort:** 2 hours, ~10 benchmarks

### Phase 6e: TUI Smoke Tests (#121)
- **What:** State machine transitions, component init, event dispatch
- **Why:** TUI has 12+ files with zero tests — validate the logic layer
- **Effort:** 1 hour, ~10 tests

---

## Total Estimate

| Phase | Issue | Tests | Hours |
|:------|:------|:------|:------|
| Phase 0: Bug fixes | #111 | 0 | 3 |
| Phase 1: Domain unit | #112 | 20 | 2 |
| Phase 2: Application mock | #113 | 25 | 3 |
| Phase 3: Infrastructure integration | #114 | 30 | 4 |
| Phase 4: Adapter behavioral | #115 | 35 | 5 |
| Phase 5: Cleanup | #116 | 0 | 1 |
| Phase 6a: MCP session fix | #117 | 5 | 2 |
| Phase 6b: Mutation testing | #118 | 0 | 3 |
| Phase 6c: Security fuzzing | #119 | 15 | 2 |
| Phase 6d: Performance baseline | #120 | 10 | 2 |
| Phase 6e: TUI smoke tests | #121 | 10 | 1 |
| **Total** | | **150** | **28** |

## Execution Order

```
Core (110-116):              Extended (117-121):

Phase 0 ──→ Phase 1-4 ──→ Phase 5    6a ──→ 6b ──→ 6c ──→ 6d ──→ 6e
(bugs)      (parallel)    (cleanup)   (sequential, after core)
```

### Work Distribution (3 developers)

| Developer | Core | Extended | Total hours |
|:----------|:-----|:---------|:------------|
| Dev A | #111 (Phase 0) → #115 (Phase 4) | #117 (MCP) → #121 (TUI) | 5 + 10 = 15h |
| Dev B | #112 (Phase 1) + #113 (Phase 2) | #118 (mutation) + #119 (security) | 5 + 5 = 10h |
| Dev C | #114 (Phase 3) + #116 (Phase 5) | #120 (perf) | 5 + 2 = 7h |
