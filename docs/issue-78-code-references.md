# Issue #78 — Code Reference Grounding

> Generated from codebase analysis on 2026-07-01.
> Maps each of the 11 proposed features to concrete existing code, integration points, and risk.

---

## 1. Content Pruning Filter (Phase 1)

**Goal:** Remove HTML noise before AI cleaning to reduce token waste 40-60%.

### Current State

| What | Where | Line(s) |
|------|-------|---------|
| AI pipeline entry point: `SemanticCleaner::clean(html)` | `src/domain/semantic_cleaner.rs` | :122-125 |
| Full RAG pipeline: chunk → tokenize → embed → score | `src/infrastructure/ai/semantic_cleaner_impl.rs` | :422-507 |
| HTML chunking step | `src/infrastructure/ai/chunker.rs` | :40-50 (struct) |
| Readability extraction (legible crate) | `src/infrastructure/scraper/readability.rs` | :47-59 |
| HTML boilerplate cleaning (pre-Readability) | `src/infrastructure/converter/html_cleaner.rs` | called at `discovery.rs:265` |
| Fallback text extraction | `src/infrastructure/scraper/fallback.rs` | :20-29 |

### Architecture Gap

The pipeline currently runs: **Raw HTML → Readability → Chunker → Tokenize → Embed → Score**.

There is NO content pruning step between Readability and the AI chunker. The `readability::parse()` call at `discovery.rs:268` produces `Article.content` (clean HTML), which is passed directly to the AI pipeline. All HTML noise (nav, sidebar, footer, ads) that legible misses still enters the chunker.

### Integration Point

The pruning filter should be inserted at `src/infrastructure/ai/semantic_cleaner_impl.rs:428-433` (inside `SemanticCleaner::clean()`), **after** receiving the HTML input and **before** passing it to `self.chunker.chunk(html)`:

```rust
// Current (line 430-433):
let chunks = self.chunker.chunk(html)
    .map_err(|e| SemanticError::Tokenize(format!("Chunking failed: {}", e)))?;

// With pruning:
let pruned_html = ContentPruner::new().prune(html);  // NEW STEP
let chunks = self.chunker.chunk(&pruned_html)          // Modified
```

### Existing Structures to Extend/Replace

| Structure | File:Line | Action |
|-----------|-----------|--------|
| `SemanticCleanerImpl` | `semantic_cleaner_impl.rs:202-221` | Add `pruner: ContentPruner` field |
| `HtmlChunker::chunk()` | `chunker.rs:40-50` | No change needed — receives pre-pruned HTML |
| `Article` struct | `readability.rs:12-25` | Alternative: prune `Article.content` before passing to AI |

### Recommended New File

- `src/infrastructure/ai/content_pruner.rs` — uses `readability` crate for extraction, wraps in a `#[cfg(feature = "ai")]` module
- Register in `src/infrastructure/ai/mod.rs` (after line 108, before re-exports)

### Tests to Update/Create

| Test | File | Status |
|------|------|--------|
| `test_model_config_default` | `semantic_cleaner_impl.rs:677` | No change |
| `test_semantic_cleaner_creation_fails_without_model` | `semantic_cleaner_impl.rs:722` | No change |
| New: `test_content_pruner_removes_noise` | New file | Must create |
| New: `test_pruned_html_produces_fewer_chunks` | New file | Must create |

### Risk: LOW
Self-contained addition before the chunker. Does not modify existing AI pipeline behavior unless enabled.

---

## 2. Session Health Pool (Phase 1)

**Goal:** Track HTTP session health (mark good/bad/retire) to reduce WAF ban rate.

### Current State

| What | Where | Line(s) |
|------|-------|---------|
| `HttpClient` struct with UA rotation | `src/application/http_client/client.rs` | :44-53 |
| UA rotation on 403/WAF challenge | `client.rs` | :240-263 (ua_index loop) |
| WAF detection (body + headers) | `src/infrastructure/http/waf_engine.rs` | :157-176, :191-201 |
| `UserAgentCache::fallback_agents()` | `src/infrastructure/user_agent.rs` | Used at `client.rs:117` |
| Rate limiter (governor) | `src/application/rate_limiter.rs` | :101-130 |

### Architecture Gap

Sessions have no health tracking. The `HttpClient` retries with UA rotation (up to `max_retries`), but there is no mechanism to:
- Mark a session/fingerprint as "burned" after repeated WAF hits
- Track success/failure rates per domain
- Retire old sessions proactively
- Share health state across multiple `HttpClient` instances

The current retry loop at `client.rs:188-393` tracks `ua_index` per-request, not per-session.

### Integration Point

Create `src/infrastructure/network/session_pool.rs` with a `HashMap<String, SessionState>` keyed by domain.

Wire into `src/application/http_client/client.rs` at line 140 (inside `HttpClient::new()`):
```rust
// After creating client:
let session_pool = Arc::new(SessionPool::new());
```

And modify the retry loop at `client.rs:188` to check `session_pool.is_healthy(domain)` before attempting.

### Existing Structures to Extend

| Structure | File:Line | Action |
|-----------|-----------|--------|
| `HttpClient` | `client.rs:44-53` | Add `session_pool: Arc<SessionPool>` field |
| `HttpClientConfig` | `config.rs:11-38` | Add `session_health_enabled: bool` field |
| `WafInspector::detect_body()` | `waf_engine.rs:157` | Call `session_pool.mark_bad(domain)` on detection |

### New File

- `src/infrastructure/network/session_pool.rs`
- Register in a new `src/infrastructure/network/mod.rs`

### Tests

| Test | File | Status |
|------|------|--------|
| `test_http_client_creation_default` | `client.rs:436` | Must update to include session_pool |
| New: `test_session_pool_mark_bad_retires` | New file | Must create |
| New: `test_session_pool_healthy_after_cooldown` | New file | Must create |

### Risk: LOW
Adds a new field to `HttpClient` but the pool defaults to "all healthy". Backward compatible.

---

## 3. Checkpoint/Resume (Phase 1)

**Goal:** Serialize crawl state to survive process crashes on HDD hardware.

### Current State

| What | Where | Line(s) |
|------|-------|---------|
| Resume mode flag | `src/application/crawl_options.rs` | :63 (`resume: bool`) |
| `apply_resume_mode()` — URL filtering | `src/cli/scrape_flow.rs` | :17-87 |
| `StateStore` — persistence of export state | `src/infrastructure/export/state_store.rs` | :28-309 |
| `ExportState` — processed URLs tracking | `src/domain/entities.rs` | referenced at `state_store.rs:17` |
| `Engine::run()` — crawl loop (NO checkpoint) | `src/application/crawler/engine.rs` | :73-223 |
| `UrlQueue` — in-memory URL queue | `src/infrastructure/crawler/url_queue.rs` | :27-36 |

### Architecture Gap

Resume mode currently only tracks **export state** (which URLs were exported to disk via `StateStore`). It does NOT checkpoint:
- The `UrlQueue` (pending URLs) — lost on crash
- The `Engine` crawl progress (which URLs were crawled vs pending)
- The `Visited` deduplicator — lost on crash
- Error counts and retry state

The `Engine::run()` loop at `engine.rs:73-223` is entirely in-memory. A crash loses the entire `VecDeque<DiscoveredUrl>` at line 85 and the `UrlDeduplicator`.

### Integration Point

1. **Domain:** Create `src/domain/crawl_job/checkpoint.rs` — `CrawlCheckpoint` struct (serializable via `serde` + `bincode`)
2. **Infrastructure:** Create `src/infrastructure/crawler/checkpoint_store.rs` — persistence layer
3. **Application:** Modify `src/application/crawler/engine.rs`:
   - At line 85: Load checkpoint into `url_queue` if exists
   - At line 106 (after each task completes): Save checkpoint with current queue state
   - At line 217 (before collect): Save final checkpoint

### Existing Structures to Extend

| Structure | File:Line | Action |
|-----------|-----------|--------|
| `Engine` | `engine.rs:27-34` | Add `checkpoint_store: Option<CheckpointStore>` field |
| `CrawlerConfig` | `site/config.rs:15-38` | Add `checkpoint_enabled: bool`, `checkpoint_interval: usize` |
| `CrawlLimits` | `crawl_options.rs:47-70` | Already has `resume: bool` — extend with checkpoint config |
| `UrlQueue` | `url_queue.rs:27-36` | Add `fn serialize_state()` and `fn load_state()` |

### Tests

| Test | File | Status |
|------|------|--------|
| `test_apply_resume_mode_*` | `tests/flag_audit_sitemap_resume_test.rs` | Must update for checkpoint |
| New: `test_checkpoint_survives_crash` | New file | Must create |
| New: `test_checkpoint_queue_serialization` | New file | Must create |

### Risk: MEDIUM
Touches the core `Engine` crawl loop. Requires careful serialization of `UrlQueue` + `UrlDeduplicator` state.

---

## 4. URL Discovery (Phase 1)

**Goal:** Auto-discover URLs from sitemaps and robots.txt without manual input.

### Current State

| What | Where | Line(s) |
|------|-------|---------|
| Sitemap parser (streaming, quick-xml) | `src/infrastructure/crawler/sitemap_parser.rs` | :131-457 |
| `crawl_with_sitemap()` | `src/application/crawler/discovery.rs` | :365-470 |
| `discover_sitemap_url()` — auto-discover | `src/application/crawler/discovery.rs` | :537-633 |
| Robots.txt parsing | `src/application/crawler/discovery.rs` | :540-578 |
| `discover_urls_for_tui()` — TUI workflow | `src/application/crawler/discovery.rs` | :67-142 |
| `SitemapConfig` | `src/infrastructure/crawler/sitemap_config.rs` | used throughout |
| Legacy `fetch_sitemap()` | `src/application/crawler_service.rs` | :41-80 (deprecated) |
| `SitemapParser` sub-components | `src/infrastructure/crawler/` | `batch_processor.rs`, `compression_handler.rs`, etc. |

### Architecture Gap

Discovery is **already well-implemented**. The existing `SitemapParser` handles:
- Streaming XML parsing (zero-allocation)
- Gzip compression
- Sitemap index recursion
- Crawl budget optimization
- Retry policy

**What's missing:**
- robots.txt **Disallow** directives are not parsed/respected during crawling (only `Sitemap:` directives are extracted)
- No `robotstxt` crate — robots.txt parsing is hand-rolled at `discovery.rs:540-578`
- No parallel multi-sitemap discovery from sitemap index files with proper concurrency

### Integration Point

1. **Extend robots.txt parsing** at `discovery.rs:540-578`: Add `Disallow` directive parsing
2. **Wire Disallow rules into URL filtering** at `url_filter.rs:125-133` (`is_allowed()`)
3. **Register in** `src/application/crawler/mod.rs` as a `RobotsTxt` module

### Existing Structures to Extend

| Structure | File:Line | Action |
|-----------|-----------|--------|
| `CrawlerConfig` | `site/config.rs:15-38` | Add `respect_robots_txt: bool` (default: true) |
| `is_allowed()` | `url_filter.rs:125-133` | Check robots.txt Disallow rules |
| `discover_sitemap_url()` | `discovery.rs:537-633` | Refactor to return both sitemap URL and robots.txt rules |

### Tests

| Test | File | Status |
|------|------|--------|
| `test_parse_sitemap_*` | `discovery.rs:709-775` | Existing — no change |
| `test_*_sitemap_parser_*` | `sitemap_parser.rs:466-677` | Existing — no change |
| New: `test_robots_txt_disallow_blocks_urls` | New file | Must create |
| New: `test_robots_txt_sitemap_directive` | New file | Must create |

### Risk: LOW
Adds new functionality without modifying existing sitemap parsing. robots.txt Disallow is additive.

---

## 5. HTTP/2 Fingerprint (Phase 1)

**Goal:** Configure HTTP/2 SETTINGS frame to match Chrome's fingerprint.

### Current State

| What | Where | Line(s) |
|------|-------|---------|
| wreq client builder with Chrome145 emulation | `src/application/http_client/client.rs` | :101-111 |
| `Emulation::Chrome145` TLS profile | `client.rs` | :102, :408 |
| Legacy client builder | `src/infrastructure/crawler/http_client.rs` | :43-52 |
| `HttpClientConfig::tls_emulation` | `src/application/http_client/config.rs` | :35 |
| Client Hints headers (Chrome 145 2026) | `client.rs` | :22-28 |
| Sec-Fetch headers | `client.rs` | :79-99 |

### Architecture Gap

wreq handles TLS fingerprint emulation via `Emulation::Chrome145`, but HTTP/2 SETTINGS frame customization is NOT exposed. The `h2` crate (used by wreq internally) has configurable SETTINGS (e.g., `HEADER_TABLE_SIZE`, `ENABLE_PUSH`, `MAX_CONCURRENT_STREAMS`, `INITIAL_WINDOW_SIZE`, `MAX_FRAME_SIZE`).

Currently, the HTTP/2 handshake uses wreq's defaults, which may not match Chrome 145's exact HTTP/2 SETTINGS frame.

### Integration Point

Modify `src/application/http_client/client.rs` at line 101 (inside `HttpClient::new()`):
```rust
let builder = Client::builder()
    .emulation(config.tls_emulation)
    // NEW: HTTP/2 SETTINGS fingerprint
    .http2_initial_window_size(6_291_456)       // Chrome: 6MB
    .http2_max_frame_size(16_384)               // Chrome: 16KB
    .http2_header_table_size(65_536)            // Chrome: 64KB
    .http2_enable_push(false)                   // Chrome: disabled
    // ... rest unchanged
```

### Existing Structures to Extend

| Structure | File:Line | Action |
|-----------|-----------|--------|
| `HttpClientConfig` | `config.rs:11-38` | Add `http2_settings: Option<Http2Fingerprint>` |
| `HttpClient::new()` | `client.rs:61-146` | Apply HTTP/2 settings to builder |

### Note on wreq API

The actual integration depends on whether wreq exposes `h2` SETTINGS configuration. If not, this may require:
- Forking or wrapping wreq's `ClientBuilder`
- Or using the raw `h2` crate directly for specific endpoints

### Tests

| Test | File | Status |
|------|------|--------|
| `test_http_client_creation_default` | `client.rs:436` | No change (settings are internal) |
| New: `test_http2_settings_match_chrome` | New file | Must create (integration test) |
| New: `test_http2_fingerprint_detection` | New file | Must create |

### Risk: MEDIUM
Depends on wreq API capabilities. May require upstream contribution or workaround.

---

## 6. Interactive Actions (Phase 2)

**Goal:** Support CDP-based actions (click, type, scroll) via chromiumoxide for SPA interaction.

### Current State

| What | Where | Line(s) |
|------|-------|---------|
| `JsRenderer` trait (forward-compatible stub) | `src/domain/js_renderer.rs` | :74-92 |
| `JsRenderError` enum | `src/domain/js_renderer.rs` | :32-54 |
| `--force-js-render` flag (unimplemented) | `src/cli/scrape_flow.rs` | :100-113 |
| `force_js_render` option | `src/application/crawl_options.rs` | :96 |
| `ScraperError::FeatureGated` for JS rendering | referenced at `scrape_flow.rs:107` |

### Architecture Gap

The `JsRenderer` trait exists as a stub but has NO implementation. The `--force-js-render` flag returns a `FeatureGated` error. There is:
- No `chromiumoxide` dependency
- No headless browser pool
- No CDP action sequence support
- No interaction model (click, scroll, type)

### Integration Point

1. **New infrastructure module:** `src/infrastructure/headless/`
   - `chromium_renderer.rs` — implements `JsRenderer` using chromiumoxide
   - `action_sequence.rs` — defines `Click`, `Type`, `Scroll` CDP actions
   - `browser_pool.rs` — manages headless browser instances

2. **Wire into domain trait:** Implement `JsRenderer for ChromiumRenderer` at `infrastructure/headless/chromium_renderer.rs`

3. **Extend `JsRenderer` trait** at `js_renderer.rs:88-91`:
   ```rust
   fn render_with_actions(
       &self,
       url: &url::Url,
       actions: &[Action],  // NEW: interaction sequence
   ) -> impl Future<Output = Result<String, JsRenderError>> + Send;
   ```

### Existing Structures to Extend

| Structure | File:Line | Action |
|-----------|-----------|--------|
| `JsRenderer` trait | `js_renderer.rs:74-92` | Add `render_with_actions()` method |
| `JsRenderError` | `js_renderer.rs:32-54` | Add `ActionFailed(String)` variant |
| `CrawlOptions::NetworkOptions` | `crawl_options.rs:73-97` | Add `js_actions: Vec<JsAction>` |
| `scrape_flow.rs` | :100-113 | Replace `FeatureGated` with actual dispatch |

### Tests

| Test | File | Status |
|------|------|--------|
| New: `test_chromium_renderer_clicks_button` | New file | Must create |
| New: `test_action_sequence_scroll_and_type` | New file | Must create |
| New: `test_browser_pool_recycles_instances` | New file | Must create |

### Risk: HIGH
Requires chromiumoxide integration (heavyweight dependency), browser pool lifecycle management, and CDP protocol handling.

---

## 7. Priority Scheduler (Phase 2)

**Goal:** Replace FIFO URL queue with priority-based scheduling.

### Current State

| What | Where | Line(s) |
|------|-------|---------|
| `UrlQueue` (FIFO via `Vec` + `pop()`) | `src/infrastructure/crawler/url_queue.rs` | :27-36 |
| `UrlQueue::push()` (LIFO behavior via `pop()`) | `url_queue.rs` | :64-79 |
| `UrlQueue::pop()` | `url_queue.rs` | :87-90 |
| `DiscoveredUrl` with `depth` field | `src/domain/crawl_job/entities.rs` | :26-35 |
| `Engine` crawl loop (drains queue) | `src/application/crawler/engine.rs` | :85-111 |
| `BatchProcessor::apply_crawl_budget()` (sorts by depth) | `src/infrastructure/crawler/batch_processor.rs` | :47-68 |

### Architecture Gap

`UrlQueue` uses a plain `Vec<DiscoveredUrl>` with LIFO `pop()`. There is:
- No priority scoring
- No `BinaryHeap` implementation
- No domain-keyed sub-queues
- `BatchProcessor` sorts URLs but only for sitemap batch optimization, not for the main crawl queue

### Integration Point

Replace `UrlQueue` internals at `url_queue.rs:27-36`:
```rust
pub struct UrlQueue {
    // Current:
    // queue: Mutex<Vec<DiscoveredUrl>>,
    // New:
    queue: Mutex<BinaryHeap<PrioritizedUrl>>,  // Priority queue
    domain_quotas: DashMap<String, DomainQuota>, // Per-domain fairness
    seen: DashSet<u64, ahash::RandomState>,     // Keep existing
}
```

Modify `Engine::run()` at `engine.rs:111` to use priority-aware draining:
```rust
// Current:
url_queue.append(&mut self.queue.drain_all().await);
// New:
url_queue = self.queue.drain_by_priority(config_clone.concurrency).await;
```

### Existing Structures to Extend

| Structure | File:Line | Action |
|-----------|-----------|--------|
| `UrlQueue` | `url_queue.rs:27-36` | Replace `Vec` with `BinaryHeap` |
| `DiscoveredUrl` | `crawl_job/entities.rs:26-35` | Add `priority: u8` field |
| `Engine::run()` | `engine.rs:73-223` | Update queue drain logic |

### Tests

| Test | File | Status |
|------|------|--------|
| `test_url_queue_*` | `url_queue.rs:159-272` | Must update for priority behavior |
| `test_batch_processor_*` | `batch_processor.rs:103-194` | No change (sitemap only) |
| New: `test_priority_queue_high_priority_first` | New file | Must create |
| New: `test_domain_fairness_limits` | New file | Must create |

### Risk: MEDIUM
Core crawl loop modification. Requires careful backward compatibility for existing URL ordering.

---

## 8. Item Pipeline (Phase 3)

**Goal:** Extensible sequential pipeline for validate → clean → enrich → output.

### Current State

| What | Where | Line(s) |
|------|-------|---------|
| `ScrapedContent` struct | `src/domain/entities.rs` | :149-168 |
| `DocumentChunk` (typestate pattern) | `src/domain/entities.rs` | :229+ |
| `scraper_service::scrape_single_url_for_tui()` | `src/application/crawler/discovery.rs` | :235-325 |
| Readability → fallback chain | `discovery.rs` | :268-324 |
| `FileExporter`, `JsonlExporter`, `VectorExporter` | `src/infrastructure/export/` | Multiple files |
| `ElasticIngestion` pipeline | `src/application/elastic_ingestion.rs` | Separate pipeline |
| `CrawlResultRepositoryImpl` (append-only log) | `src/application/crawl_result_repository.rs` | Persistence |

### Architecture Gap

Content processing is hardcoded in `scrape_single_url_for_tui()` at `discovery.rs:235-325`:
1. Fetch HTML
2. Clean boilerplate (`html_cleaner::clean_html`)
3. Try Readability
4. Fallback to plain text
5. Build `ScrapedContent`

There is NO extensible pipeline. Adding a new stage (e.g., language detection, content classification, metadata enrichment) requires modifying this function directly.

### Integration Point

Create `src/application/pipeline/mod.rs` with a trait-based pipeline:
```rust
pub trait PipelineStage: Send + Sync {
    fn process(&self, content: ScrapedContent) -> Pin<Box<dyn Future<Output = Result<ScrapedContent>> + Send>>;
}
```

Wire into `src/application/crawler/discovery.rs` at line 235, replacing the hardcoded chain in `scrape_single_url_for_tui()`.

### Existing Structures to Extend

| Structure | File:Line | Action |
|-----------|-----------|--------|
| `ScrapedContent` | `entities.rs:149-168` | Add pipeline metadata field |
| `scrape_single_url_for_tui()` | `discovery.rs:235-325` | Refactor to use pipeline |
| `ElasticIngestion` | `elastic_ingestion.rs` | Candidate for pipeline stage |

### New Files

- `src/application/pipeline/mod.rs` — pipeline trait + orchestrator
- `src/application/pipeline/stages/` — individual stage implementations
- `src/application/pipeline/stages/readability_stage.rs` — extract from discovery.rs:268
- `src/application/pipeline/stages/fallback_stage.rs` — extract from discovery.rs:287

### Tests

| Test | File | Status |
|------|------|--------|
| `test_scrape_single_url_*` | `discovery.rs:709+` | Must update for pipeline |
| New: `test_pipeline_stage_chaining` | New file | Must create |
| New: `test_pipeline_error_propagation` | New file | Must create |

### Risk: MEDIUM
Refactors `scrape_single_url_for_tui()` which is a critical path. Requires careful stage ordering.

---

## 9. Adaptive Selectors (Phase 3)

**Goal:** CSS selectors that adapt when site structure changes.

### Current State

| What | Where | Line(s) |
|------|-------|---------|
| `Selector::parse("a[href]")` (hardcoded) | `src/infrastructure/crawler/link_extractor.rs` | :46-47 |
| `CrawlLimits::selector` field ("body") | `src/application/crawl_options.rs` | :49 |
| `scraper::Html` + `Selector` usage | `link_extractor.rs` | :45-71 |
| Readability (heuristic content extraction) | `readability.rs` | :47-59 |
| `fallback::extract_text()` (basic HTML strip) | `fallback.rs` | :20-29 |

### Architecture Gap

Selectors are hardcoded or configured as static strings:
- Link extraction always uses `Selector::parse("a[href]")` at `link_extractor.rs:46`
- Content extraction uses `CrawlLimits::selector` (default "body") at `crawl_options.rs:49`
- No structural similarity scoring
- No fallback selector chain
- No site-specific adaptation

### Integration Point

Create `src/infrastructure/scraper/adaptive_selector.rs`:
```rust
pub struct AdaptiveSelector {
    primary: Selector,
    fallbacks: Vec<Selector>,
    similarity_threshold: f32,  // strsim threshold
}
```

Wire into `src/application/crawler/discovery.rs` at line 235 (content extraction) and `link_extractor.rs` at line 46 (link extraction).

### Existing Structures to Extend

| Structure | File:Line | Action |
|-----------|-----------|--------|
| `CrawlLimits::selector` | `crawl_options.rs:49` | Change from `String` to `SelectorConfig { primary, fallbacks }` |
| `extract_links()` | `link_extractor.rs:42-75` | Accept `AdaptiveSelector` parameter |
| `scrape_single_url_for_tui()` | `discovery.rs:235-325` | Use adaptive selector for content |

### New Dependencies

- `strsim` (0.11) — string similarity for selector scoring

### Tests

| Test | File | Status |
|------|------|--------|
| `test_extract_links_*` | `link_extractor.rs:191-383` | Must update |
| New: `test_adaptive_selector_fallback_chain` | New file | Must create |
| New: `test_adaptive_selector_survives_redesign` | New file | Must create |

### Risk: LOW
Additive feature with fallback to existing behavior.

---

## 10. Autoscaled Concurrency (Phase 4)

**Goal:** Dynamic concurrency adjustment based on system load.

### Current State

| What | Where | Line(s) |
|------|-------|---------|
| `CrawlerConfig::concurrency` (static) | `src/domain/site/config.rs` | :27 |
| `ConcurrencyConfig` (auto vs manual) | `src/application/crawl_options.rs` | :80 |
| `Engine` concurrency via JoinSet | `src/application/crawler/engine.rs` | :116 (`tasks.len() >= concurrency`) |
| `SharedRateLimiter` (governor) | `src/application/rate_limiter.rs` | :101-130 |
| `RayonCpuPool` (CPU pool) | `src/infrastructure/cpu_pool.rs` | For CPU-bound tasks |
| Hardware autotuning | `src/infrastructure/autotuning.rs` | `ElasticConfig` for elastic pipeline |

### Architecture Gap

Concurrency is set once at startup (`CrawlerConfig::concurrency`) and never changes:
- `engine.rs:116`: `if tasks.len() >= config_clone.concurrency` — static limit
- No runtime adaptation based on CPU/RAM usage
- No backpressure from system metrics
- `ConcurrencyConfig::is_auto()` exists but only sets a static value based on `nproc`

### Integration Point

Create `src/application/crawler/concurrency_controller.rs`:
```rust
pub struct ConcurrencyController {
    current: AtomicUsize,
    min: usize,
    max: usize,
    sysinfo_monitor: Arc<SysinfoMonitor>,
}
```

Modify `Engine::run()` at `engine.rs:114-118`:
```rust
// Current:
if tasks.len() >= config_clone.concurrency {
// New:
let effective_concurrency = concurrency_controller.current_concurrency();
if tasks.len() >= effective_concurrency {
```

### Existing Structures to Extend

| Structure | File:Line | Action |
|-----------|-----------|--------|
| `Engine` | `engine.rs:27-34` | Add `concurrency_controller: ConcurrencyController` |
| `CrawlerConfig` | `site/config.rs:15-38` | Add `autoscale_enabled: bool` |
| `ConcurrencyConfig` | `crawl_options.rs:80` | Extend with min/max bounds |

### New Dependencies

- `sysinfo` (0.30) — cross-platform system metrics

### Tests

| Test | File | Status |
|------|------|--------|
| `test_engine_*` | `engine.rs` | Must update for dynamic concurrency |
| New: `test_concurrency_scales_down_under_load` | New file | Must create |
| New: `test_concurrency_scales_up_when_idle` | New file | Must create |

### Risk: MEDIUM
Modifies the core `Engine` crawl loop. Requires careful backpressure integration.

---

## 11. Batch Processing (Phase 4)

**Goal:** Multi-URL workflow with progress tracking and job management.

### Current State

| What | Where | Line(s) |
|------|-------|---------|
| `BatchProcessor` (crawl budget optimization) | `src/infrastructure/crawler/batch_processor.rs` | :22-95 |
| `scrape_urls()` — sequential URL processing | `src/cli/scrape_flow.rs` | :94-227 |
| `scrape_urls_for_tui()` — buffered concurrent | `src/application/crawler/discovery.rs` | :190-213 |
| `ResultsCollector` (mpsc-based) | `src/application/crawler/collector.rs` | :74-205 |
| `Engine` crawl loop (single-site) | `src/application/crawler/engine.rs` | :73-223 |
| `CrawlResultRepositoryImpl` (append-only log) | `src/application/crawl_result_repository.rs` | Persistence |

### Architecture Gap

Current batch processing is limited:
- `scrape_urls()` at `scrape_flow.rs:146` processes URLs **sequentially** (one `for` loop)
- `scrape_urls_for_tui()` at `discovery.rs:205` uses `buffered()` but only for a single site
- No multi-site batch job management
- No progress persistence (crash loses all progress)
- No job queue with retry and dead-letter support
- `BatchProcessor` is only for sitemap budget optimization, not general batch workflows

### Integration Point

Create `src/application/batch/mod.rs`:
```rust
pub struct BatchJob {
    id: Uuid,
    urls: Vec<Url>,
    status: BatchStatus,
    progress: BatchProgress,
    results: Vec<ScrapedContent>,
    failures: Vec<BatchFailure>,
}

pub struct BatchManager {
    jobs: DashMap<Uuid, BatchJob>,
    job_store: Arc<dyn JobStore>,  // Persistence
    concurrency: usize,
}
```

Wire into `src/cli/scrape_flow.rs` at line 94, replacing the sequential `for` loop with `BatchManager::process()`.

### Existing Structures to Extend

| Structure | File:Line | Action |
|-----------|-----------|--------|
| `scrape_urls()` | `scrape_flow.rs:94-227` | Refactor to use `BatchManager` |
| `ResultsCollector` | `collector.rs:74-205` | Extend with job-level tracking |
| `BatchProcessor` | `batch_processor.rs:22-95` | Extend or replace with general batch manager |
| `CrawlResultRepositoryImpl` | `crawl_result_repository.rs` | Use as `JobStore` backend |

### New Files

- `src/application/batch/mod.rs` — batch job orchestrator
- `src/application/batch/job.rs` — `BatchJob` struct
- `src/application/batch/store.rs` — `JobStore` trait + implementation
- `src/application/batch/progress.rs` — progress tracking

### Tests

| Test | File | Status |
|------|------|--------|
| `test_batch_processor_*` | `batch_processor.rs:103-194` | No change (sitemap-specific) |
| `test_scrape_urls_*` | `scrape_flow.rs:241+` | Must update |
| New: `test_batch_job_crash_recovery` | New file | Must create |
| New: `test_batch_job_progress_persistence` | New file | Must create |
| New: `test_batch_manager_concurrent_jobs` | New file | Must create |

### Risk: MEDIUM
Major refactor of the URL processing pipeline. Requires careful backward compatibility.

---

## Summary Matrix

| # | Feature | Phase | Risk | Key Integration File(s) | Existing Test Count |
|---|---------|-------|------|------------------------|-------------------|
| 1 | Content pruning | P1 | LOW | `semantic_cleaner_impl.rs:428` | 7 tests in module |
| 2 | Session health pool | P1 | LOW | `http_client/client.rs:44` | 8 tests in module |
| 3 | Checkpoint/resume | P1 | MEDIUM | `crawler/engine.rs:73` | 7 tests (flag_audit) |
| 4 | URL discovery | P1 | LOW | `crawler/discovery.rs:537` | 20+ tests |
| 5 | HTTP/2 fingerprint | P1 | MEDIUM | `http_client/client.rs:101` | 5 tests |
| 6 | Interactive actions | P2 | HIGH | `domain/js_renderer.rs:74` | 0 (stub) |
| 7 | Priority scheduler | P2 | MEDIUM | `crawler/url_queue.rs:27` | 8 tests |
| 8 | Item pipeline | P3 | MEDIUM | `crawler/discovery.rs:235` | 3+ tests |
| 9 | Adaptive selectors | P3 | LOW | `crawler/link_extractor.rs:46` | 10 tests |
| 10 | Autoscaled concurrency | P4 | MEDIUM | `crawler/engine.rs:116` | 2 tests |
| 11 | Batch processing | P4 | MEDIUM | `scrape_flow.rs:94` | 3 tests |
