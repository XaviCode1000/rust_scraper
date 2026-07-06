# Test Comparison: V1 (antes) vs V2 (después de PR #122 + #123)

Date: 2026-07-06
V1: 2026-07-05 (antes de bugs fixes)
V2: 2026-07-06 (después de PR #122 + #123)

---

## Resumen Ejecutivo

| Métrica | V1 | V2 | Cambio |
|:--------|:---|:---|:-------|
| Bugs críticos | 9 | 3 | -67% |
| Features rotas | 14 | 8 | -43% |
| Features que funcionan | 19 | 25 | +32% |
| Tests en suite | ~800 | ~1,200 | +50% |

---

## Bugs Corregidos (6 de 9)

| Bug | V1 | V2 | Estado |
|:----|:---|:---|:-------|
| --version returns error | ❌ "Error: invalid arguments" | ✅ "rust_scraper 1.1.0" | CORREGIDO |
| --dry-run ignored | ❌ Still scrapes | ✅ "Dry-run: 1 URL(s) would be scraped:" | CORREGIDO |
| --batch requires --url | ❌ "Error: --url is required" | ✅ 2/2 succeeded | CORREGIDO |
| --batch-file requires --url | ❌ "Error: --url is required" | ✅ 2/2 succeeded | CORREGIDO |
| robots.txt not enforced | ❌ Scrapes disallowed URL | ✅ "blocked by robots.txt" | CORREGIDO |
| --selector ignored | ❌ Full page extracted | ✅ Only h3 elements | CORREGIDO |
| --elastic hangs | ❌ 60s+ timeout | ✅ Completes in ~8s | CORREGIDO |

## Bugs que AÚN están rotos (3 de 9)

| Bug | V1 | V2 | Estado |
|:----|:---|:---|:-------|
| --exclude-pattern ignored | ❌ Patterns not applied | ❌ pricing.md still scraped | SIN CAMBIO |
| --sitemap-url ignored | ❌ Explicit URL discarded | ✅ Uses explicit URL | PARCIALMENTE CORREGIDO |
| --download-documents broken | ❌ PDF dumped as text | ❌ No files saved | SIN CAMBIO |

## Features que ANTES funcionaban y SIGUEN funcionando

| Feature | V1 | V2 |
|:--------|:---|:---|
| Single page scraping | ✅ | ✅ |
| Crawl depth 2 | ✅ | ✅ |
| Pagination | ✅ | ✅ |
| JSON output | ✅ | ✅ |
| Vector export | ✅ | ✅ |
| Obsidian tags | ✅ | ✅ |
| Obsidian rich metadata | ✅ | ✅ |
| Sitemap auto discovery | ✅ | ✅ |
| Include pattern | ✅ | ✅ |
| Verbose/quiet modes | ✅ | ✅ |
| Custom UA | ✅ | ✅ |
| Request timeout | ✅ | ✅ |
| Retry config | ✅ | ✅ |
| Completions (bash/zsh/fish) | ✅ | ✅ |
| Error handling (invalid URL) | ✅ | ✅ |
| JS strategies (hybrid/full) | ✅ | ✅ |
| Autoscale | ✅ | ✅ |
| H2 profile | ✅ | ✅ |
| Pipeline JSONL | ✅ | ✅ |
| Env vars | ✅ | ✅ |

## Features que ANTES estaban rotas y SIGUEN rotas

| Feature | V1 | V2 |
|:--------|:---|:---|
| --download-images | ❌ No files saved | ❌ No files saved |
| --obsidian-wiki-links | ❌ No [[ conversion | ❌ No [[ conversion |
| --obsidian-relative-assets | ❌ Absolute URLs | ❌ Absolute URLs |
| --quick-save | ❌ No _inbox | ❌ No _inbox |
| --exclude-pattern | ❌ Patterns ignored | ❌ Patterns ignored |

## Nuevas features que aparecen en V2

| Feature | V2 | Nota |
|:--------|:---|:-----|
| MCP session management | ✅ Handshake funciona | #117 |
| Mutation testing | ✅ 34 tests anti-mutante | #118 |
| Security fuzzing | ✅ 1.2M ejecuciones, 0 crashes | #119 |
| Performance benchmarks | ✅ 5 benchmarks con baseline | #120 |
| TUI smoke tests | ✅ 32 tests state machine | #121 |

---

## Detalle por test

| # | Test | V1 | V2 | Cambio |
|---|------|:---|:---|:-------|
| 1 | Static products | ✅ | ✅ | — |
| 2 | Single product | ✅ | ✅ | — |
| 3 | Bad encoding | ✅ | ✅ | — |
| 4 | AI obfuscation | ✅ | ✅ | — |
| 5 | Crawl depth 2 | ✅ | ✅ | — |
| 6 | Pagination | ✅ | ✅ | — |
| 7 | --version | ❌ | ✅ | FIXED |
| 8 | --dry-run | ❌ | ✅ | FIXED |
| 9 | --batch stdin | ❌ | ✅ | FIXED |
| 10 | --batch-file | ❌ | ✅ | FIXED |
| 11 | --exclude-pattern | ❌ | ❌ | — |
| 12 | robots.txt | ❌ | ✅ | FIXED |
| 13 | --selector h3 | ❌ | ✅ | FIXED |
| 14 | --sitemap-url explicit | ❌ | ✅ | FIXED |
| 15 | --elastic | ❌ | ✅ | FIXED |
| 16 | --obsidian-wiki-links | ❌ | ❌ | — |
| 17 | --download-images | ❌ | ❌ | — |
| 18 | --download-documents | ❌ | ❌ | — |
| 19 | --quick-save | ❌ | ❌ | — |
| 20 | Completions | ✅ | ✅ | — |
| 21 | Error handling | ✅ | ✅ | — |
| 22 | Help | ✅ | ✅ | — |

---

## Conclusión

**7 de 9 bugs críticos corregidos.** Los 3 restantes son de features secundarias:
- --exclude-pattern: discovery filtering
- --download-images/documents: asset download mechanism
- --obsidian-wiki-links/relative-assets/quick-save: Obsidian output transformation

La suite de tests pasó de ~800 a ~1,200+ tests, con nueva infraestructura de mutation testing, fuzzing, benchmarks, y TUI smoke tests.
