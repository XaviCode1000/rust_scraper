# Competitor Analysis — TCOF Framework

## 1. Crawl4AI (51K, Python) — LLM-optimized crawler with intelligent content extraction

**What it does differently:** PruningContentFilter strips boilerplate using readability heuristics before LLM ingestion, producing cleaner chunks. Four cache backends (memory, SQLite, disk, BM25) enable incremental crawls without re-fetching. Eight hook points let users inject logic at every lifecycle stage.

**Feature → Rust mapping:**

| Feature | Original impl | Rust approach | Crate | Fits in rust_scraper |
|---------|--------------|---------------|-------|---------------------|
| PruningContentFilter | readability score + node trimming | `readability` crate or custom DOM scoring | `readability` (0.2) | Pre-embedding stage in `ai/cleaner.rs` |
| SQLite incremental cache | Python sqlite3, 4 mode dispatch | `rusqlite` with WAL + hash dedup | `rusqlite` (0.31) | New `cache/` module |
| BM25 relevance filter | bm25s Python lib | `tantivy` BM25 for chunk ranking | `tantivy` (0.22) | Post-extraction ranker in `ai/` |

**Redundancy check:** AI cleaning via tract-onnx is orthogonal — PruningContentFilter removes HTML noise *before* embedding, improving chunk signal. No dedup cache exists today. **Complements, not duplicates.**

**Implementation effort:** M (3-5 days). Pruning filter is the high-ROI piece; cache is straightforward rusqlite behind a trait.

---

## 2. Crawlee/Apify (23K, TS/Python) — Production crawler with autoscaling and session lifecycle

**What it does differently:** AutoscaledPool monitors CPU/RAM via Snapshotter and adjusts concurrency dynamically. SessionPool tracks per-session health (mark_good/bad/retire), rotating out burned fingerprints before WAF bans hit. AdaptivePlaywrightCrawler detects JS requirements mid-flight and switches from HTTP to browser.

**Feature → Rust mapping:**

| Feature | Original impl | Rust approach | Crate | Fits in rust_scraper |
|---------|--------------|---------------|-------|---------------------|
| Session health tracking | Counter-based good/bad/retire | `HashMap<String, SessionState>` with thresholds | native std | `network/session_pool.rs` — new module |
| Adaptive HTTP→Browser | Runtime detection + fallback | SPA heuristic already exists; wire to chromiumoxide | existing + `chromiumoxide` | Extend `scraper_service.rs` |
| Autoscaled concurrency | CPU/RAM sampling loop | `tokio::sync::Semaphore` + `sysinfo` | `sysinfo` (0.30) | `crawler/pool.rs` — new module |

**Redundancy check:** SPA two-pass heuristic covers detection. Session quality tracking and adaptive concurrency are net-new and directly reduce ban rate.

**Implementation effort:** S (1-2 days) for session pool; M (3-5 days) for sysinfo-driven autoscaling.

---

## 3. Scrapy (62K, Python) — Battle-tested framework with middleware pipeline architecture

**What it does differently:** Dual middleware layers separate transport concerns (downloader MW: retries, cookies, compression) from parsing concerns (spider MW: dedup, depth limiting). Item Pipeline enforces sequential processing stages with drop/pause. Scheduler manages priority queues with per-domain rate limiting.

**Feature → Rust mapping:**

| Feature | Original impl | Rust approach | Crate | Fits in rust_scraper |
|---------|--------------|---------------|-------|---------------------|
| Middleware chain | Ordered middleware list, process_request/response | Trait objects in `Vec<Box<dyn Middleware>>` with `tower::ServiceBuilder` | `tower` (0.5) | `infrastructure/http/middleware.rs` |
| Item Pipeline | Sequential stages, drop semantics | `Vec<Box<dyn PipelineStage<T>>` with typed items | native trait objects | `application/pipeline.rs` |
| Priority Scheduler | Heap-based priority queue | `binaryheap` + domain-keyed sub-queues | native `std::collections::BinaryHeap` | `crawler/scheduler.rs` |

**Redundancy check:** No middleware abstraction exists. Current HTTP client is monolithic. Pipeline pattern would replace ad-hoc post-processing. **Net-new architecture.**

**Implementation effort:** L (1-2 weeks) for full middleware + pipeline; S (1-2 days) for priority scheduler alone.

---

## 4. Scrapling (67K, Python) — Adaptive element tracking that survives site redesigns

**What it does differently:** Adaptive Element Tracking uses similarity scoring (Levenshtein + structural attributes) to relocate selectors when sites change their HTML structure. Multi-Session Spider routes between HTTP and browser sessions by session ID. Checkpoint system enables pause/resume for long crawls.

**Feature → Rust mapping:**

| Feature | Original impl | Rust approach | Crate | Fits in rust_scraper |
|---------|--------------|---------------|-------|---------------------|
| Adaptive selectors | Levenshtein + structural similarity scoring | `strsim` for distance + attribute-based scoring | `strsim` (0.11) | `extractor/adaptive_selector.rs` |
| Checkpoint pause/resume | Serialized spider state to disk | `serde` + `bincode` snapshot of crawl queue + counters | `serde` + `bincode` | `crawler/checkpoint.rs` |
| Domain/Ad blocking | Pattern-based URL filter | Regex + domain list filter | `regex` (1.10) | `crawler/url_filter.rs` |

**Redundancy check:** Selectors are currently hand-written CSS. Adaptive tracking would reduce maintenance when target sites restructure. Checkpoint/resume is absent and critical for long crawls on HDD hardware.

**Implementation effort:** M (3-5 days) for checkpoint; L (1-2 weeks) for full adaptive selector engine.

---

## 5. AutoScraper (7.4K, Python) — Automatic pattern learning from examples

**What it does differently:** Learns CSS/XPath patterns from 2-3 example pages by comparing DOM structures and extracting common patterns. Persists learned models for reuse.

**Feature → Rust mapping:**

| Feature | Original impl | Rust approach | Crate | Fits in rust_scraper |
|---------|--------------|---------------|-------|---------------------|
| Pattern learning | DOM diff + common ancestor extraction | `ego-tree` for DOM traversal + custom LCA algorithm | `ego-tree` (0.6) | `extractor/pattern_learner.rs` |

**Redundancy check:** Scrapling's Adaptive Element Tracking covers this use case more robustly with similarity scoring. AutoScraper's learning is simpler but less maintainable. **Skip — superseded by #4.**

**Implementation effort:** N/A (recommendation: skip).

---

## 6. curl-impersonate (6.4K, C/Python) — TLS and HTTP/2 fingerprint impersonation

**What it does differently:** Impersonates browser TLS handshakes AND HTTP/2 SETTINGS frames, priority, pseudo-header order, and frame pacing. Covers Chrome, Firefox, Safari, Edge at both layers.

**Feature → Rust mapping:**

| Feature | Original impl | Rust approach | Crate | Fits in rust_scraper |
|---------|--------------|---------------|-------|---------------------|
| HTTP/2 fingerprint emulation | Custom HTTP/2 SETTINGS + pseudo-header order | `h2` crate with configurable settings frame | `h2` (0.4) with custom config | `infrastructure/http/h2_fingerprint.rs` |
| TLS fingerprint | boring-sys2 BoringSSL bindings | **Already have via wreq** | `wreq` (existing) | No change needed |

**Redundancy check:** TLS fingerprinting is covered by wreq. The gap is HTTP/2 SETTINGS frame customization — wreq defaults to standard values that sophisticated WAFs can fingerprint. **Partial overlap; HTTP/2 tuning is the gap.**

**Implementation effort:** S (1-2 days) for HTTP/2 settings customization if h2 crate exposes the config; otherwise blocked on wreq internals.

---

## 7. Firecrawl (142K, TypeScript) — API-first scraping platform with interactive extraction

**What it does differently:** Interact endpoint performs actions (click, type, scroll) before extraction — handles pagination and infinite scroll. Map endpoint discovers URLs via sitemap/robots.txt/crawling without fetching page content. Batch scrape processes URLs asynchronously with webhook callbacks.

**Feature → Rust mapping:**

| Feature | Original impl | Rust approach | Crate | Fits in rust_scraper |
|---------|--------------|---------------|-------|---------------------|
| URL discovery (Map) | Sitemap + robots.txt parsing + BFS | `sitemap` crate + `robotstxt` + tokio BFS queue | `sitemap` (0.4) + `robotstxt` | `crawler/discovery.rs` — new module |
| Interactive actions | Browser automation before extract | chromiumoxide with action sequences | `chromiumoxide` (0.5, feature-gated) | Extend Phase 2 JS rendering |
| Batch async processing | Webhook + job queue | tokio channels + `dashmap` job tracker | native tokio + `dashmap` | `crawler/batch.rs` |

**Redundancy check:** URL discovery is absent — currently requires manual URL input. Interactive actions belong in Phase 2 with chromiumoxide. Batch processing is relevant for multi-URL workflows. **URL discovery is the key net-new capability.**

**Implementation effort:** S (1-2 days) for sitemap/robots.txt discovery; M (3-5 days) for interactive actions (Phase 2).

---

## Master Adoption Matrix

| # | Feature | Source | Rust Implementation | Effort | Phase | Critical? |
|---|---------|--------|---------------------|--------|-------|-----------|
| 1 | Content pruning filter | Crawl4AI | `readability` crate in `ai/cleaner.rs` | M | 1 (now) | YES — improves embedding quality |
| 2 | Session health pool | Crawlee | `HashMap<String, SessionState>` + thresholds | S | 1 (now) | YES — reduces WAF bans |
| 3 | SQLite incremental cache | Crawl4AI | `rusqlite` WAL + hash dedup | M | 1 (now) | NO — nice to have |
| 4 | Checkpoint pause/resume | Scrapling | `serde` + `bincode` snapshot | M | 1 (now) | YES — critical for HDD + long crawls |
| 5 | URL discovery (sitemap/robots.txt) | Firecrawl | `sitemap` + `robotstxt` crates | S | 1 (now) | YES — eliminates manual URL entry |
| 6 | HTTP/2 SETTINGS fingerprint | curl-impersonate | `h2` crate custom config | S | 1 (now) | MAYBE — only if WAFs fingerprint h2 |
| 7 | Priority scheduler | Scrapy | `BinaryHeap` + domain sub-queues | S | 2 (JS) | NO — current FIFO works for CLI |
| 8 | Middleware chain | Scrapy | `tower::ServiceBuilder` trait objects | L | 2 (JS) | NO — premature abstraction now |
| 9 | Item Pipeline | Scrapy | `Vec<Box<dyn PipelineStage>>` | L | 3 (hooks) | NO — needed when output formats grow |
| 10 | Adaptive selectors | Scrapling | `strsim` + structural scoring | L | 3 (hooks) | NO — only if targeting unstable sites |
| 11 | Autoscaled concurrency | Crawlee | `tokio::Semaphore` + `sysinfo` | M | 4 (deep crawl) | NO — single-URL CLI doesn't need it |
| 12 | Domain/Ad blocking | Scrapling | `regex` + domain list | S | 1 (now) | NO — current WAF filter covers basics |
| 13 | Interactive actions | Firecrawl | `chromiumoxide` action sequences | M | 2 (JS) | NO — Phase 2 scope already planned |
| 14 | Batch async processing | Firecrawl | tokio channels + `dashmap` | M | 4 (deep crawl) | NO — multi-URL is future feature |
| 15 | Pattern learning | AutoScraper | superseded by Scrapling adaptive | — | — | SKIP |

### Recommended Priority (Phase 1, immediate value):

1. **Content pruning filter** — biggest quality boost for existing AI pipeline
2. **Session health pool** — low effort, high ban-reduction ROI
3. **Checkpoint/resume** — essential for your HDD hardware reality
4. **URL discovery** — sitemap + robots.txt parsing eliminates manual input
5. **HTTP/2 fingerprint tuning** — test first; only adopt if WAFs respond to h2 fingerprinting
