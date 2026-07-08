# Mutation Testing Baseline Report

**Date:** 2026-07-08
**Tool:** cargo-mutants v27.1.0
**Rust:** 1.88.0

---

## Executive Summary

| Module | Mutants | Caught | Missed | Unviable | Mutation Score | Notes |
|--------|---------|--------|--------|----------|----------------|-------|
| `domain/entities.rs` | 28 | 20 | 0 | 8 | **100%** | All meaningful mutants caught |
| `domain/site/config.rs` | 19 | 4 | 0 | 15 | **100%** | 15/19 mutants are type-level unviable |
| `application/scraper_service.rs` | 20 | 10 | 4 | 6 | **71.4%** | 4 missed mutants need better tests |
| `infrastructure/crawler/sitemap_parser.rs` | 46 | ~11 | ~34 | ~1 | **~24%** | Heaviest testing gap — XML parsing logic |

**Overall weighted score:** ~61% (weighted by mutant count)

**Note:** Doc-tests excluded from baseline due to 14 pre-existing broken doctests (unrelated to mutation testing). Behavioral test suite excluded due to 4 pre-existing broken download tests.

---

## Detailed Analysis

### 1. `src/domain/entities.rs` — Score: 100%

**28 mutants, 20 caught, 8 unviable**

This module has excellent test coverage. All meaningful mutants are caught:
- `ExportFormat::parse_str`, `extension()`, `name()` — fully tested
- `DocumentChunk::validate()` — validation logic thoroughly tested
- `ExportState` — all operations tested
- `DocumentChunk` constructor variants — all paths covered

**Unviable mutants (8):** Type-level changes that can't compile (e.g., changing return types, altering struct fields). These are acceptable.

**No action needed.**

---

### 2. `src/domain/site/config.rs` — Score: 100%

**19 mutants, 4 caught, 15 unviable**

This module has a high unviable count because the builder pattern generates many type-level mutations:
- `CrawlerConfig::matches_include()` and `matches_exclude()` — caught correctly
- `CrawlerConfigBuilder` setter methods — most are type-unviable (return `Self` -> `Self` mutations that don't affect behavior)

**Unviable mutants (15):** Builder method mutations that can't change observable behavior (e.g., changing field assignment internals).

**No action needed.**

---

### 3. `src/application/scraper_service.rs` — Score: 71.4%

**20 mutants, 10 caught, 4 missed, 6 unviable**

**4 missed mutants requiring attention:**

| Line | Mutation | Impact | Recommended Fix |
|------|----------|--------|-----------------|
| `220:36` | `replace > with ==` in `scrape_with_config` | Boundary check at MAX_INSTRUMENTED_BODY_SIZE | Add test with body exactly at/above threshold |
| `220:36` | `replace > with <` in `scrape_with_config` | Inverted boundary check | Same boundary test covers this |
| `220:36` | `replace > with >=` in `scrape_with_config` | Off-by-one at boundary | Test with body = exactly 1MB |
| `418:5` | Replace `download_assets_if_enabled` return with `Ok(vec![])` | Asset download always returns empty | Add integration test verifying assets actually get downloaded |

**Key functions covered:**
- `extract_with_selector()` — CSS selector extraction (caught)
- `detect_spa_content()` — SPA detection logic (caught)
- `scrape_with_config()` — main scraping pipeline (partially caught)
- `scrape_multiple_with_limit()` — batch scraping (caught)

---

### 4. `src/infrastructure/crawler/sitemap_parser.rs` — Score: ~24%

**46 mutants, ~11 caught, ~34 missed, ~1 timeout**

This is the most significant testing gap. The XML sitemap parsing logic has deep internal state that unit tests don't fully exercise.

**Key missed categories:**

**A. `parse_with_depth()` — depth tracking and recursion (15 missed mutants)**
- Lines 225-231: `||` operator replacements in depth checks
- Line 247: `+=` counter mutations
- Line 248: `>` comparison mutations
- Line 284: arithmetic mutations in depth calculation

**Recommendation:** Add unit tests for depth exhaustion and recursive sitemap index parsing with mock HTTP responses.

**B. `parse_xml_sitemap()` — XML event matching (10 missed mutants)**
- Lines 333, 356: Match guard `e.name().as_ref() == b"loc"` replaced with `true`/`false`
- Line 336: `in_loc` guard replacement
- Line 360: Error match arm deletion

**Recommendation:** Add tests with XML that has non-loc elements alongside loc elements, to verify the `== b"loc"` guard matters.

**C. `parse_gzip_sitemap()` — gzip decompression (1 missed mutant)**
- Line 301: Replace entire function return with `Ok(vec![])`

**Recommendation:** Add test verifying gzip sitemap actually returns URLs (currently only `is_sitemap_index` is tested for gzip).

**D. `resolve_url()` — URL resolution (1 missed mutant)**
- Line 123: `||` to `&&` in scheme check

**Recommendation:** Test with URLs that fail one scheme check but pass another.

**E. Timeout (1)**
- Line 359: `Ok(Event::Eof)` arm deletion — causes hang in XML parsing loop

**Recommendation:** This is actually a correct catch — the EOF arm prevents infinite loops. Consider adding a test that verifies the parser terminates on well-formed XML without EOF.

---

## Pre-existing Issues Discovered

### Broken Doc-tests (14 failures)
Several doc-tests fail due to outdated code examples:
- `src/lib.rs` — `scrape_with_readability` signature mismatch
- `src/cli/args.rs` — `Args.url` type mismatch (Option<String> vs &str)
- `src/application/batch/processor.rs` — missing `?` on `BatchProcessor::new`
- `src/infrastructure/downloader/` — missing `tokio_test` dependency
- `src/infrastructure/user_agent.rs` — import path changed
- Others: `export_utils`, `export_factory`, `collector`, `progress_widget`, `waf_engine`

### Broken Behavioral Tests (4 failures)
Download feature tests in `tests/behavioral/cli/download_test.rs` fail:
- `download_images_saves_png_file`
- `download_images_png_content_is_valid`
- `download_documents_saves_pdf_file`
- `download_documents_pdf_content_is_valid`

---

## Prioritized Recommendations

### P0 — Fix pre-existing test failures
Fix the 14 broken doc-tests and 4 behavioral test failures to have a clean baseline.

### P1 — Improve `scraper_service.rs` mutation score
Add boundary tests for `MAX_INSTRUMENTED_BODY_SIZE` (1MB) and integration test for asset downloading.

### P2 — Improve `sitemap_parser.rs` mutation score
The biggest testing opportunity:
1. Add mock HTTP server tests for recursive sitemap index parsing with depth tracking
2. Add tests with mixed XML elements (loc + non-loc) to verify guard conditions
3. Add gzip sitemap URL extraction test (not just index detection)
4. Add depth exhaustion test

### P3 — Add `mutants` CI step
Once doc-test/behavioral issues are fixed, add mutation testing to CI with a minimum threshold:
```
cargo mutants --ci --timeout 120 -- --lib
```

---

## How to Reproduce

```bash
# Install
cargo install cargo-mutants

# Run per-module (requires fixing doc-tests first for entities/config)
CARGO_ENCODED_RUSTFLAGS="" cargo mutants --file src/domain/entities.rs --no-shuffle --in-place -- --lib --tests

CARGO_ENCODED_RUSTFLAGS="" cargo mutants --file src/domain/site/config.rs --no-shuffle --in-place -- --lib --tests

CARGO_ENCODED_RUSTFLAGS="" cargo mutants --file src/application/scraper_service.rs --no-shuffle --in-place --timeout 120 -- --lib

CARGO_ENCODED_RUSTFLAGS="" cargo mutants --file src/infrastructure/crawler/sitemap_parser.rs --no-shuffle --in-place --timeout 120 -- --lib
```

**Note:** `CARGO_ENCODED_RUSTFLAGS=""` overrides the global `rustflags` from `~/.cargo/config.toml` which includes mold linker flags that conflict with cargo-mutants' temp directory builds.
