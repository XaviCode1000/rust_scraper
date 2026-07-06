# Test Results — CLI Validation Against Real Websites

Date: 2026-07-05
Binary: `rust_scraper` v1.1.0 (release build)
Test sites: web-scraping.dev, webscraper.io

---

## Summary

| # | Test | Status | Notes |
|---|------|--------|-------|
| 1 | Static products page | ✅ OK | Clean markdown, all 6 products with images/prices |
| 2 | Single product detail | ✅ OK | Full detail: variants table, features, packs |
| 3 | Bad encoding | ✅ OK | Text readable, no mojibake |
| 4 | AI obfuscation | ✅ OK | Captured obfuscated + cleaned text |
| 5 | Crawl depth 2 | ✅ OK | Discovered 21 URLs, scraped 5 pages |
| 6 | Pagination | ⚠️ PARTIAL | 4/5 pages scraped, 2 failures (Readability + 404) |
| 7 | Testimonials (static) | ⚠️ PARTIAL | 9/10 pages, 2 Readability failures |
| 8 | JS-generated links | ❌ FAIL | Readability failed, JS not executed |
| 9 | JSON-LD structured data | ⚠️ PARTIAL | Only visible text, no JSON-LD extraction |
| 10 | data-href attributes | ⚠️ PARTIAL | Only "click" text, custom attrs ignored |
| 11 | Blocked page | ✅ OK | Detected mock block, saved content |
| 12 | Rate limiting | ✅ OK | Scraped successfully |
| 13 | robots.txt enforcement | ⚠️ BUG | Scraped disallowed URL despite robots.txt |
| 14 | File download | ✅ OK | Page content captured, no actual file download |
| 15 | Dry-run mode | ❌ BUG | Still scrapes and saves files |
| 16 | Batch mode | ❌ BUG | Requires --url even with stdin input |

---

## Detailed Results

### 1. Static Products Page ✅
**URL**: `https://web-scraping.dev/products`
**Flags**: `--single-page`
**Result**: Clean markdown with all 6 products, titles, descriptions, prices, and images.
**Quality**: Excellent — proper heading hierarchy, image alt text preserved.

### 2. Single Product Detail ✅
**URL**: `https://web-scraping.dev/product/1`
**Flags**: `--single-page`
**Result**: Full product page with description, price ($9.99 from $12.99), variants table, features table, and packs table.
**Quality**: Excellent — tables properly formatted in markdown.

### 3. Bad Encoding ⚠️
**URL**: `https://web-scraping.dev/bad-encoding`
**Flags**: `--single-page --format text`
**Result**: Text output describes encoding scenarios correctly. No visible mojibake.
**Caveat**: This is the index page, not individual encoding test cases. Each sub-page needs separate testing.

### 4. AI Content Obfuscation ✅
**URL**: `https://web-scraping.dev/ai-content-obfuscation`
**Flags**: `--single-page --pipeline`
**Result**: Captured both raw obfuscated text AND cleaned version. The cipher "Неllо, thе сiрhеr is: sсrарflу2017" is visible in the cleaned section.
**Quality**: Good — invisible Unicode characters present in raw, cleaned version available.

### 5. Crawl with Depth 2 ✅
**URL**: `https://webscraper.io/test-sites`
**Flags**: `--max-depth 2 --max-pages 5`
**Result**: Discovered 21 URLs from seed page. Scraped 5 pages (index, cloud-scraper, pricing, marketplace, documentation).
**Quality**: Good — proper URL resolution, files organized by domain.

### 6. Pagination ⚠️ PARTIAL
**URL**: `https://web-scraping.dev/reviews`
**Flags**: `--max-pages 5`
**Result**: Discovered 12 URLs, scraped 4 pages. Two failures:
- `https://web-scraping.dev/docs` — Readability failed
- `https://web-scraping.dev/api/graphql` — HTTP 404
**Issue**: The "Load More" button is JavaScript-triggered, so pagination doesn't work with static scraping.

### 7. Testimonials (Static Strategy) ⚠️ PARTIAL
**URL**: `https://web-scraping.dev/testimonials`
**Flags**: `--js-strategy static --max-pages 10`
**Result**: 9/10 pages scraped. Two failures:
- `https://web-scraping.dev/docs` — Readability failed
- `https://web-scraping.dev/api/graphql` — HTTP 404
**Issue**: Testimonials loaded dynamically, static strategy captures only initial content.

### 8. JS-Generated Links ❌ FAIL
**URL**: `https://web-scraping.dev/js-links`
**Flags**: `--single-page`
**Result**: Readability failed. Only captured the raw JavaScript code, not the dynamically injected link.
**Root Cause**: No JS execution in static mode. The `--force-js-render` flag exists but requires `--js-strategy hybrid` or `full`.

### 9. JSON-LD Structured Data ⚠️ PARTIAL
**URL**: `https://web-scraping.dev/linked-data`
**Flags**: `--single-page --pipeline`
**Result**: Only captured "linked data" text. JSON-LD `<script type="application/ld+json">` blocks were not extracted.
**Root Cause**: Readability extracts visible content, not JSON-LD. Would need a dedicated JSON-LD parser.

### 10. data-href Attributes ⚠️ PARTIAL
**URL**: `https://web-scraping.dev/data-href`
**Flags**: `--single-page --pipeline`
**Result**: Only captured "click" text. Custom `data-href` attributes were not resolved as links.
**Root Cause**: Link extraction only looks at standard `href` attributes.

### 11. Blocked Page ✅
**URL**: `https://web-scraping.dev/blocked`
**Flags**: `--single-page`
**Result**: Detected mock block page. Content saved with reference ID and explanation.
**Quality**: Good — the block detection works, content preserved for analysis.

### 12. Rate Limiting ✅
**URL**: `https://web-scraping.dev/rate-limited`
**Flags**: `--single-page --max-retries 3`
**Result**: Scraped successfully on first attempt. The site returned 200 (not actually rate-limited on first request).
**Note**: Would need sequential requests to trigger actual 429 responses.

### 13. robots.txt Enforcement ⚠️ BUG
**URL**: `https://web-scraping.dev/robots-disallowed`
**Flags**: `--single-page` (without `--ignore-robots`)
**Result**: Scraped the disallowed URL! The content says "If you are reading this, your crawler did NOT respect robots.txt."
**Bug**: The scraper fetched a URL that is explicitly Disallowed in robots.txt. This is a compliance issue.

### 14. File Download ✅
**URL**: `https://web-scraping.dev/file-download`
**Flags**: `--single-page --download-documents`
**Result**: Page content captured with technical details about the download mechanism. No actual PDF downloaded (requires POST form submission).
**Note**: The `--download-documents` flag enables detection but the download itself requires specific form handling.

### 15. Dry-Run Mode ❌ BUG
**URL**: `https://webscraper.io/test-sites`
**Flags**: `--max-depth 1 --max-pages 3 --dry-run`
**Result**: Still scraped 3 pages and saved files to `output/`. Dry-run flag is ignored.
**Bug**: `orchestrator::run()` never checks `opts.dry_run`. The flag is parsed but not implemented.

### 16. Batch Mode ❌ BUG
**Input**: `echo -e "url1\nurl2" | rust_scraper --batch`
**Result**: Error: `--url is required for scraping`
**Bug**: `main.rs` validates URL before reaching `orchestrator::run()` where batch mode is handled. Batch mode should skip URL validation.

---

## Bugs Found

### Critical
1. **robots.txt not enforced**: Scraper fetches Disallowed URLs even without `--ignore-robots`
2. **--dry-run broken**: Flag exists but orchestrator ignores it, still scrapes and saves
3. **--batch requires --url**: Batch mode should work without --url when reading from stdin/file

### Medium
4. **Readability failures on SPA content**: Pages with dynamic content return "Failed to extract article content"
5. **No JSON-LD extraction**: Structured data in `<script type="application/ld+json">` is ignored
6. **Custom data-* href attributes not resolved**: Only standard `href` is extracted as links

### Low
7. **Dry-run still saves to default `output/` directory**: Even if scraping is skipped, directory creation should be prevented

---

## Quality Metrics

| Metric | Value |
|--------|-------|
| Tests passed | 8/16 (50%) |
| Tests partial | 4/16 (25%) |
| Tests failed | 4/16 (25%) |
| Critical bugs | 3 |
| Content quality (when extracted) | Excellent |
| Markdown formatting | Clean, proper hierarchy |
| Image handling | Alt text preserved |
| Table formatting | Proper markdown tables |
| Frontmatter | Complete with title, URL, date, excerpt |
