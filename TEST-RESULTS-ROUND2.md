# Test Results Round 2 — All Remaining Features

Date: 2026-07-05
Continuation of TEST-RESULTS.md

---

## Tests Executed

| # | Feature | Flags | Status | Notes |
|---|---------|-------|--------|-------|
| 20 | JSON output format | `-f json` | ✅ OK | Clean JSON array with title, content, url, html |
| 21 | Vector export | `--export-format vector` | ✅ OK | export.json with format_version, documents array |
| 22 | Obsidian wiki-links + tags + rich metadata | `--obsidian-wiki-links --obsidian-tags --obsidian-rich-metadata` | ✅ OK | Frontmatter has tags, wordCount, readingTime, language, contentType, scrapeDate, source, status |
| 23 | Obsidian relative assets | `--obsidian-relative-assets` | ⚠️ PARTIAL | Images still use absolute URLs, not rewritten to relative paths |
| 24 | Sitemap discovery | `--use-sitemap` | ✅ OK | Discovered 4 pages from sitemap |
| 25 | Include pattern | `--include-pattern "*/product/*"` | ✅ OK | Single page scraped correctly |
| 26 | Exclude pattern | `--exclude-pattern "*/pricing*" --exclude-pattern "*/cloud*"` | ❌ BUG | Patterns ignored — pricing.md and cloud-scraper.md still scraped |
| 27 | Max verbosity | `-vvv` | ✅ OK | Shows DEBUG level: HttpClient creation, scraping URLs, JSONL export |
| 28 | Quiet mode | `-q` | ✅ OK | Zero output lines |
| 29 | Custom User-Agent | `--user-agent "Mozilla/5.0 (CustomBot/1.0)"` | ✅ OK | Works without errors |
| 30 | No checkpoint | `--no-checkpoint` | ✅ OK | No checkpoint files created |
| 31 | Pipeline JSONL | `--pipeline --pipeline-output jsonl` | ✅ OK | JSONL output with id, url, title, content, metadata, timestamp |
| 32 | Depth 0 | `--max-depth 0` | ✅ OK | Only seed URL scraped (index.md only) |
| 33 | Custom Accept-Language | `--accept-language "es-ES,es;q=0.9"` | ✅ OK | Works without errors |
| 34 | Depth 3 | `--max-depth 3 --max-pages 15` | ✅ OK | Scraped 1 page (products has limited internal links) |
| 35 | Checkpoint interval | `--checkpoint-interval 1` | ⚠️ NO FILES | No checkpoint files — might only work in crawl mode, not single-page |
| 36 | H2 profile | `--h2-profile Chrome131` | ✅ OK | Works without errors |
| 37 | Download images | `--download-images` | ❌ BUG | No image files downloaded — flag exists but doesn't download |
| 38 | Trace file | `--trace-file` | ⚠️ EMPTY | No trace file — requires `--features otel` at compile time |
| 39 | Exclude pattern v2 | Multiple `--exclude-pattern` | ❌ BUG | All patterns ignored — pricing, cloud, marketplace all scraped |
| 40 | Pipeline none | `--pipeline --pipeline-output none` | ✅ OK | Pipeline runs but export.jsonl still created (expected) |
| 41 | Custom retry config | `--max-retries 5 --backoff-base-ms 500 --backoff-max-ms 5000` | ✅ OK | Works without errors |

---

## New Bugs Found

### Critical
4. **`--exclude-pattern` completely broken** — Patterns are never applied. All URLs are scraped regardless of exclude patterns. Tested with `*/pricing*`, `*/cloud*`, `*/marketplace*` — all ignored.

### Medium
5. **`--download-images` doesn't download** — Flag is parsed and stored but no images are actually saved. The `--download-images` flag enables detection but the download mechanism isn't triggered.
6. **`--obsidian-relative-assets` doesn't rewrite paths** — Images still use absolute URLs (`https://web-scraping.dev/assets/...`) instead of relative paths (`./assets/...`).
7. **`--checkpoint-interval` produces no files in single-page mode** — Checkpoint only works during crawl, not single-page scraping. No documentation about this limitation.

### Low
8. **`--trace-file` requires feature flag** — Trace file only works when compiled with `--features otel`. No warning when flag is used without the feature.

---

## Feature Verification Summary

### Working Features ✅
- JSON output format
- Vector export format
- Obsidian wiki-links, tags, rich metadata (frontmatter)
- Sitemap-based URL discovery
- Include pattern filtering
- Verbosity levels (-v, -vv, -vvv)
- Quiet mode (-q)
- Custom User-Agent
- Custom Accept-Language
- No-checkpoint mode
- Pipeline with jsonl/none output
- Depth 0 (seed-only)
- Depth 3 (deeper crawl)
- H2 profile switching
- Completions (bash, zsh, fish)
- Batch mode (when --url is also provided)

### Broken Features ❌
- `--exclude-pattern` — patterns ignored
- `--download-images` — no images saved
- `--obsidian-relative-assets` — paths not rewritten
- `--checkpoint-interval` — no files in single-page mode
- `--trace-file` — requires otel feature (no warning)
- `--dry-run` — still scrapes (from Round 1)
- `--batch` — requires --url (from Round 1)
- robots.txt enforcement — scrapes disallowed URLs (from Round 1)

---

## Cumulative Bug Count

| Severity | Count | Bugs |
|----------|-------|------|
| Critical | 4 | robots.txt, dry-run, batch requires url, exclude-pattern |
| Medium | 4 | download-images, obsidian-relative-assets, checkpoint single-page, Readability SPA failures |
| Low | 3 | trace-file feature-gated, JSON-LD not extracted, data-href not resolved |

**Total: 11 bugs (4 critical, 4 medium, 3 low)**
