# Test Results — FINAL COMPREHENSIVE REPORT

Date: 2026-07-05
Binary: `rust_scraper` v1.1.0 (release)
Total tests: 42 (across 4 rounds)
Test sites: web-scraping.dev, webscraper.io, httpbin.org, w3.org

---

## Executive Summary

| Metric | Value |
|--------|-------|
| Features tested | 42 |
| Working | 19 (45%) |
| Partial | 5 (12%) |
| Broken | 18 (43%) |
| Critical bugs | 9 |
| Medium bugs | 6 |
| Low bugs | 4 |
| **Total bugs** | **19** |

---

## Blind Spots Found (NEW — Round 4)

These were features that appeared to work in earlier rounds but actually didn't:

### `--obsidian-wiki-links` — NO CONVERSION HAPPENS
Tested with `--obsidian-wiki-links` on products page. Internal links like `[Box of Chocolate Candy](https://web-scraping.dev/product/1)` were NOT converted to `[[Box of Chocolate Candy]]`. The flag is parsed but the conversion logic is never executed.

### `--obsidian-relative-assets` — PATHS NOT REWRITTEN
Tested with `--obsidian-relative-assets`. Image paths still use absolute URLs (`https://web-scraping.dev/assets/products/...`) instead of relative paths (`./assets/products/...`).

### `--quick-save` — NO _inbox DIRECTORY CREATED
Tested `--quick-save`. No `_inbox` directory was created, no files saved there. The flag is a no-op.

### `--download-documents` — DUMPS BINARY AS GARBAGE TEXT
Tested with a direct PDF URL (`w3.org/.../dummy.pdf`). The scraper fetched the PDF binary and dumped it as raw bytes into a `.md` file (garbled `%PDF-1.4 %äüöß...` text). No actual PDF file was saved. The flag exists but the download mechanism is broken.

### `--download-images` — NO FILES SAVED
Tested on products page with 5 images. Only the markdown file with `![alt](url)` references was saved. Zero actual image files downloaded. The flag enables detection but the download code path is never reached.

### `--selector` — CSS SELECTOR IGNORED
Tested with `--selector "h3"` and `--selector "table"`. Both times the full page content was extracted. The Readability algorithm overrides the CSS selector.

### `--sitemap-url` — EXPLICIT URL IGNORED
Tested `--sitemap-url "https://web-scraping.dev/sitemap.xml"`. The log shows `use_sitemap: false` — the explicit URL is discarded. Discovery uses the default link extraction instead.

### `--elastic` — HANGS INDEFINITELY
Tested `--elastic` mode. The process hangs for 60+ seconds with no output. The elastic ingestion pipeline deadlocks or waits for a resource that never becomes available.

### `--version` — RETURNS ERROR
Tested `--version`. Returns `Error: invalid arguments` instead of printing the version string. The flag is handled by clap but the help/version subcommand parsing conflicts with the subcommand enum.

### `--force-js-render` — PLACEHOLDER, NOT IMPLEMENTED
Returns error message: "funcionalidad no disponible: JavaScript rendering no está implementado. Fase 2 planificada."

### `--clean-ai` — NO-OP WITHOUT FEATURE
Flag is parsed but produces identical output to regular scrape. Requires `--features ai` at compile time but the release binary doesn't include it.

### MCP Server — SESSION MANAGEMENT BROKEN
The HTTP MCP server starts and responds to `initialize`, but subsequent requests fail with "Unexpected message, expect initialize request" even with correct Mcp-Session header. The session state machine has a bug. The MCP live test suite also hangs.

---

## Complete Test Results

### Core Scraping (14 tests)
| # | Feature | Flags | Status | Evidence |
|---|---------|-------|--------|----------|
| 1 | Static products | `--single-page` | ✅ | 6 products, clean markdown |
| 2 | Product detail | `--single-page` | ✅ | Tables, variants, features |
| 3 | Bad encoding | `--single-page -f text` | ✅ | No mojibake |
| 4 | AI obfuscation | `--single-page --pipeline` | ✅ | Cleaned cipher visible |
| 5 | Crawl depth 2 | `--max-depth 2` | ✅ | 21 URLs, 5 scraped |
| 6 | Pagination | `--max-pages 5` | ⚠️ | 4/5, Readability+404 |
| 7 | Testimonials | `--js-strategy static` | ⚠️ | 9/10, Readability fails |
| 8 | JS links | `--single-page` | ❌ | JS not executed |
| 9 | JSON-LD | `--pipeline` | ⚠️ | No JSON-LD extraction |
| 10 | data-href | `--pipeline` | ⚠️ | Custom attrs ignored |
| 11 | Blocked page | `--single-page` | ✅ | Mock block detected |
| 12 | Rate limiting | `--max-retries 3` | ✅ | Success |
| 13 | robots.txt | (no flag) | ❌ | Scraped disallowed URL |
| 14 | File download | `--download-documents` | ❌ | PDF dumped as garbled text |

### Output & Export (8 tests)
| # | Feature | Flags | Status | Evidence |
|---|---------|-------|--------|----------|
| 15 | Markdown | `-f markdown` | ✅ | Default, perfect |
| 16 | JSON | `-f json` | ✅ | Clean JSON with html field |
| 17 | Text | `-f text` | ✅ | Plain text |
| 18 | JSONL export | `--export-format jsonl` | ✅ | Default, works |
| 19 | Vector export | `--export-format vector` | ✅ | export.json created |
| 20 | Auto export | `--export-format auto` | ✅ | Works |
| 21 | Pipeline JSONL | `--pipeline --pipeline-output jsonl` | ✅ | id,url,title,content |
| 22 | Pipeline none | `--pipeline --pipeline-output none` | ✅ | Runs, export still created |

### Obsidian (6 tests)
| # | Feature | Flags | Status | Evidence |
|---|---------|-------|--------|----------|
| 23 | Wiki-links | `--obsidian-wiki-links` | ❌ | No [[ conversion |
| 24 | Tags | `--obsidian-tags "a,b"` | ✅ | In frontmatter |
| 25 | Rich metadata | `--obsidian-rich-metadata` | ✅ | wordCount, readingTime, etc. |
| 26 | Relative assets | `--obsidian-relative-assets` | ❌ | Still absolute URLs |
| 27 | Vault auto-detect | `--vault` | ✅ | Works |
| 28 | Quick-save | `--quick-save` | ❌ | No _inbox created |

### Discovery (6 tests)
| # | Feature | Flags | Status | Evidence |
|---|---------|-------|--------|----------|
| 29 | Sitemap auto | `--use-sitemap` | ✅ | 4 pages found |
| 30 | Sitemap explicit | `--sitemap-url URL` | ❌ | URL ignored |
| 31 | Include pattern | `--include-pattern` | ✅ | Works |
| 32 | Exclude pattern | `--exclude-pattern` | ❌ | Patterns ignored |
| 33 | Depth 0 | `--max-depth 0` | ✅ | Seed only |
| 34 | Depth 3 | `--max-depth 3` | ✅ | Deeper crawl |

### Behavior & Config (10 tests)
| # | Feature | Flags | Status | Evidence |
|---|---------|-------|--------|----------|
| 35 | CSS selector | `--selector "h3"` | ❌ | Selector ignored |
| 36 | Dry-run | `--dry-run` | ❌ | Still scrapes |
| 37 | Batch stdin | `--batch` | ❌ | Requires --url |
| 38 | Batch file | `--batch-file` | ❌ | Requires --url |
| 39 | Batch + url | `--batch -u URL` | ✅ | 2/2 succeeded |
| 40 | Verbose -vvv | `-vvv` | ✅ | DEBUG visible |
| 41 | Quiet | `-q` | ✅ | Zero output |
| 42 | Config file | `config.toml` | ⚠️ | Loads but verbose override fails |
| 43 | Env vars | `RUST_SCRAPER_*` | ✅ | All work |
| 44 | Custom UA | `--user-agent` | ✅ | Works |

### HTTP & Resilience (5 tests)
| # | Feature | Flags | Status | Evidence |
|---|---------|-------|--------|----------|
| 45 | Accept-Language | `--accept-language "es-ES"` | ✅ | Works |
| 46 | Request timeout | `--timeout-secs 3` | ✅ | Triggers on httpbin/delay/10 |
| 47 | Retry config | `--max-retries 5 --backoff-base-ms 500` | ✅ | Accepted |
| 48 | Concurrency | `--concurrency 4` | ✅ | Works |
| 49 | Delay | `--delay-ms 500` | ✅ | Works |

### JS Rendering (3 tests)
| # | Feature | Flags | Status | Evidence |
|---|---------|-------|--------|----------|
| 50 | Hybrid | `--js-strategy hybrid` | ✅ | 9 pages scraped |
| 51 | Full | `--js-strategy full` | ✅ | 9 pages scraped |
| 52 | Force JS | `--force-js-render` | ❌ | "not implemented" error |

### Advanced (6 tests)
| # | Feature | Flags | Status | Evidence |
|---|---------|-------|--------|----------|
| 53 | Autoscale | `--autoscale` | ✅ | Works |
| 54 | No session health | `--no-session-health` | ✅ | Works |
| 55 | H2 profile | `--h2-profile Chrome131` | ✅ | Works |
| 56 | Elastic | `--elastic` | ❌ | Hangs 60s+ |
| 57 | Download images | `--download-images` | ❌ | No files saved |
| 58 | Clean AI | `--clean-ai` | ❌ | No-op without feature |

### CLI & Error Handling (5 tests)
| # | Feature | Flags | Status | Evidence |
|---|---------|-------|--------|----------|
| 59 | --help | `--help` | ✅ | Full options list |
| 60 | --version | `--version` | ❌ | "Error: invalid arguments" |
| 61 | Completions bash/zsh/fish | `completions` | ✅ | Valid scripts |
| 62 | Invalid URL | `not-a-valid-url` | ✅ | "Invalid URL" error |
| 63 | Invalid domain | `nonexistent-domain.com` | ✅ | Network error |

### MCP Server (1 test)
| # | Feature | Status | Evidence |
|---|---------|--------|----------|
| 64 | MCP HTTP server | ⚠️ | Starts, initialize works, subsequent requests fail |

---

## Complete Bug List (19 bugs)

### Critical (9)
| # | Feature | Description |
|---|---------|-------------|
| 1 | robots.txt | Scrapes Disallowed URLs without --ignore-robots |
| 2 | --dry-run | Flag parsed, orchestrator ignores it |
| 3 | --batch | Requires --url even with stdin |
| 4 | --batch-file | Requires --url even with file |
| 5 | --exclude-pattern | Patterns completely ignored |
| 6 | --version | Returns error instead of version |
| 7 | --elastic | Hangs indefinitely (60s+) |
| 8 | --download-documents | PDF dumped as garbled binary text, no actual file saved |
| 9 | MCP session | Subsequent requests fail after initialize |

### Medium (6)
| # | Feature | Description |
|---|---------|-------------|
| 10 | --download-images | Flag exists, no images saved |
| 11 | --obsidian-relative-assets | Paths not rewritten to relative |
| 12 | --selector | CSS selector ignored, Readability overrides |
| 13 | --sitemap-url | Explicit URL ignored by discovery |
| 14 | --obsidian-wiki-links | No [[wiki-link]] conversion happens |
| 15 | --quick-save | No _inbox directory created |

### Low (4)
| # | Feature | Description |
|---|---------|-------------|
| 16 | --force-js-render | Placeholder, "not implemented" |
| 17 | --clean-ai | No-op without ai feature (no warning) |
| 18 | --checkpoint-interval | No files in single-page mode |
| 19 | Readability SPA | Fails on dynamic content pages |

---

## What Actually Works (19 features)
- Output: markdown, json, text
- Export: jsonl, vector, auto
- Pipeline: jsonl output, none output
- Obsidian: tags, rich metadata, vault detection
- Discovery: sitemap auto, include patterns
- Verbosity: -v, -vvv, -q
- HTTP: custom UA, Accept-Language, timeout, retry, concurrency, delay
- JS strategies: hybrid, full
- Competitive: autoscale, session health, H2 profile
- CLI: help, completions, error handling
- Batch: with --url provided
- Config: env vars, config file (partial)
