# Test Suite Refactoring Plan

## Current State

| Category | Files | Lines | Action |
|:---------|:------|:------|:-------|
| Plumbing (flag_audit_*) | 4 | 1,745 | DELETE after behavioral replacements |
| Common infrastructure | 1 | 339 | EXTEND with TestEnv, fixtures |
| CLI binary tests | 2 | 696 | EXTEND with behavioral tests |
| Integration (http, crawler, rate) | 5 | 1,690 | AUDIT + fill gaps |
| MCP tests | 2 | 691 | FIX session bug + extend |
| AI integration | 1 | 1,029 | KEEP (feature-gated) |
| Elastic tests | 4 | 1,055 | AUDIT behavioral coverage |
| Other (security, concurrency, etc.) | 9 | 1,760 | AUDIT + fill gaps |
| **Total** | **28** | **8,935** | |

## Strategy: 6 Phases

### Phase 0: Bug Fixes First (prerequisite)
Fix the 9 critical bugs BEFORE writing behavioral tests. Otherwise tests fail for wrong reasons.

| Bug | File to fix |
|:----|:-----------|
| --dry-run ignored | src/cli/orchestrator.rs |
| --batch requires --url | src/main.rs |
| --batch-file requires --url | src/main.rs |
| --exclude-pattern ignored | src/application/crawler/discovery.rs |
| --version returns error | src/main.rs or src/cli/args.rs |
| robots.txt not enforced | src/application/crawler/discovery.rs |
| --selector ignored | src/cli/scrape_flow.rs |
| --sitemap-url ignored | src/application/crawler/discovery.rs |
| --download-documents broken | src/infrastructure/downloader/ |

### Phase 1: Infrastructure (tests/common/mod.rs)
Extend existing TestEnv with behavioral helpers:
- TestEnv struct (server + tempdir + cmd builder)
- HTML fixture functions (products, single product, images, links, etc.)
- File search helpers (by extension, by content pattern)
- Markdown content assertion helpers

### Phase 2: CLI Core Behavioral Tests
Replace flag_audit_core_display_test.rs + flag_audit_downloads_test.rs

| Feature | Test | Verifies |
|:--------|:-----|:---------|
| --version | exit 0, stdout contains version | No error |
| --dry-run | no files created in output dir | Scraping skipped |
| --batch | stdin URLs processed without --url | Batch mode works |
| --batch-file | file URLs processed without --url | Batch file works |
| --exclude-pattern | excluded URLs not scraped | Pattern filtering |
| --download-images | .jpg/.png files exist in output | Images saved |
| --download-documents | PDF files exist in output | Documents saved |
| --selector | only matching elements in output | CSS selection |

### Phase 3: Obsidian Behavioral Tests
Replace flag_audit_obsidian_test.rs

| Feature | Test | Verifies |
|:--------|:-----|:---------|
| --obsidian-wiki-links | [[ ]] syntax in markdown | Link conversion |
| --obsidian-relative-assets | relative paths in img src | Path rewriting |
| --quick-save | files in _inbox directory | Quick save works |
| --obsidian-rich-metadata | wordCount, readingTime in frontmatter | Metadata present |
| --vault (explicit) | files in specified vault path | Vault targeting |

### Phase 4: Crawler Behavioral Tests
Replace flag_audit_sitemap_resume_test.rs + flag_audit_http_crawler_test.rs

| Feature | Test | Verifies |
|:--------|:-----|:---------|
| --use-sitemap | URLs from sitemap used | Sitemap discovery |
| --sitemap-url | explicit sitemap URL respected | Explicit sitemap |
| --max-depth 0 | only seed URL scraped | Depth limiting |
| --ignore-robots | disallowed URL scraped | Robots override |
| robots.txt (default) | disallowed URL skipped | Robots enforcement |
| --resume | previously scraped URLs skipped | Resume mode |
| --concurrency N | N concurrent requests | Concurrency control |
| --delay-ms | minimum delay between requests | Rate limiting |

### Phase 5: Error Path Tests
New file: tests/error_paths.rs

| Scenario | Test | Verifies |
|:---------|:-----|:---------|
| Invalid URL format | exit != 0, "Invalid URL" | Input validation |
| Unreachable domain | exit != 0, timeout error | Network error |
| 404 response | exit != 0, error message | HTTP error |
| 500 response | exit != 0, error message | Server error |
| Empty HTML | graceful fallback | Robustness |
| Disk full / no write perms | exit != 0, error message | Filesystem error |
| Very long URL | handled correctly | Edge case |

### Phase 6: Cleanup
- Delete flag_audit_downloads_test.rs
- Delete flag_audit_obsidian_test.rs
- Delete flag_audit_sitemap_resume_test.rs
- Delete flag_audit_http_crawler_test.rs
- Delete flag_audit_core_display_test.rs
- Update CI to run behavioral tests by default

## Issue Templates

### Epic: Test Suite Refactoring
Parent issue tracking the entire effort.

### Issue: Phase 0 — Fix Critical Bugs
Blocks all behavioral test writing. 9 bugs to fix.

### Issue: Phase 1 — Test Infrastructure
TestEnv, fixtures, helpers.

### Issue: Phase 2 — CLI Core Behavioral Tests
~8 new tests replacing 2 plumbing files.

### Issue: Phase 3 — Obsidian Behavioral Tests
~5 new tests replacing 1 plumbing file.

### Issue: Phase 4 — Crawler Behavioral Tests
~8 new tests replacing 2 plumbing files.

### Issue: Phase 5 — Error Path Tests
~7 new tests in new file.

### Issue: Phase 6 — Cleanup
Delete plumbing files, update CI.

## Estimated Effort

| Phase | Tests | Estimated time |
|:------|:------|:---------------|
| Phase 0 (bug fixes) | 0 | 2-3 hours |
| Phase 1 (infrastructure) | 0 | 1 hour |
| Phase 2 (CLI core) | 8 | 2 hours |
| Phase 3 (Obsidian) | 5 | 1.5 hours |
| Phase 4 (Crawler) | 8 | 2 hours |
| Phase 5 (Error paths) | 7 | 1.5 hours |
| Phase 6 (cleanup) | 0 | 30 min |
| **Total** | **38** | **~10-11 hours** |
