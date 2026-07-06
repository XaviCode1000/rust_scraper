# rust_scraper — Realidad del Código vs Documentación

> Generado analizando el código fuente con GitNexus + CodeDB. Refleja el estado
> actual del repo, no el README.

---

## ¿Qué es esto?

Un web scraper/Crawler production-ready en Rust con Clean Architecture. Tiene 5
modos de ejecución, un servidor MCP con 37 herramientas para LLMs, un TUI
interactivo para configuración, y un pipeline elástico orientado a RAG.

**Stack**: Rust 1.88 · Tokio · wreq 6 (TLS fingerprint) · ratatui · tract-onnx
(feature-gated) · SQLite · MCP (rmcp)

---

## Modos de Operación

1. **Scraping estándar** — `rust_scraper -u URL` — descubre, crawlea, exporta
2. **Single page** — `--single-page` — solo la URL semilla, sin discovery
3. **Batch** — `--batch` / `--batch-file` — lee URLs de stdin o archivo
4. **Config TUI** — `--config-tui` — formulario interactivo ratatui
5. **MCP Server** — `examples/mcp_server.rs` — 37 herramientas para LLMs

---

## Feature Flags (Cargo.toml)

| Feature | Activa | Para qué sirve |
|:--------|:-------|:---------------|
| `images` | `mimetype-detector` | Detección de tipos MIME para descarga de imágenes |
| `documents` | `mimetype-detector` | Lo mismo para documentos (PDFs, etc.) |
| `full` | `images` + `documents` | Ambas capacidades |
| `ai` | `tokenizers`, `tract-onnx`, `hf-hub` | Limpieza semántica con ONNX (all-MiniLM-L6-v2) |
| `console` | `console-subscriber` | Tokio console para debug de runtime |
| `dev-tracing` | `tracing-tree` | Árbol jerárquico de spans en terminal |
| `otel` | OpenTelemetry stack | Distributed tracing vía OTLP HTTP/protobuf |
| `otel-metrics` | `otel` + metrics | Extiende otel con métricas (counters, histograms) |

---

## CLI Flags Completos

### Target
- `-u, --url` (env: `RUST_SCRAPER_URL`) — URL a scrapear. Sin esto, prompt interactivo.
- `-s, --selector` (env: `RUST_SCRAPER_SELECTOR`, default: `body`) — CSS selector.

### Output
- `-o, --output` (env: `RUST_SCRAPER_OUTPUT`, default: `output/`) — Directorio de salida.
- `-f, --format` (env: `RUST_SCRAPER_FORMAT`, default: `markdown`) — Formato: `markdown`, `text`, `json`.
- `--export-format` (env: `RUST_SCRAPER_EXPORT_FORMAT`, default: `jsonl`) — Formato RAG: `jsonl`, `vector`, `auto`.

### Obsidian
- `--obsidian-wiki-links` — Convierte enlaces internos a `[[wiki-link]]`.
- `--obsidian-tags` — Tags YAML frontmatter (comma-separated).
- `--obsidian-relative-assets` — Rutas de assets relativas al `.md`.
- `--vault` — Path al vault Obsidian (auto-detect si se omite).
- `--quick-save` — Guarda directo en `_inbox` del vault.
- `--obsidian-rich-metadata` — Metadata rica en frontmatter.

### Discovery
- `--delay-ms` (default: `1000`) — Delay entre requests en ms.
- `--max-pages` (default: `10`) — Límite de páginas.
- `--concurrency` (default: auto) — Concurrencia: `auto` o número.
- `--use-sitemap` — Usa sitemap XML para descubrir URLs.
- `--sitemap-url` — URL explícita del sitemap (requiere `--use-sitemap`).
- `--sitemap-depth` (default: `3`) — Profundidad máxima de recursión en sitemap indexes.

### Behavior
- `--single-page` — Solo la URL semilla, sin crawling.
- `--resume` — Retoma scraping anterior (skip URLs procesadas).
- `--state-dir` — Directorio custom para estado de resume.
- `--download-images` — Descarga imágenes encontradas.
- `--download-documents` — Descarga documentos encontrados.
- `--interactive` — TUI selector de URLs antes de scrape.
- `--config-tui` — Formulario TUI para configurar todo.
- `--clean_ai` — Limpieza semántica con IA (requiere feature `ai`).
- `--force-js-render` — Renderizado JS para SPAs (placeholder, no implementado).

### Display
- `-v` / `-vv` / `-vvv` — Verbosity creciente.
- `-q, --quiet` — Suprime output info/debug.
- `-n, --dry-run` — Solo descubre URLs, no scrapea.
- `--trace-file` — Escribe OTel spans como JSONL para debug offline.

### Crawler Settings
- `--max-depth` (default: `2`) — Profundidad de crawl (0 = solo semilla).
- `--timeout-secs` (default: `30`) — Timeout por request.
- `--include-pattern` — Glob patterns para incluir URLs.
- `--exclude-pattern` — Glob patterns para excluir URLs.

### HTTP Client
- `--max-retries` (default: `3`) — Reintentos máximos.
- `--backoff-base-ms` (default: `1000`) — Base del backoff exponencial.
- `--backoff-max-ms` (default: `10000`) — Tope del backoff.
- `--accept-language` (default: `en-US,en;q=0.9`) — Header Accept-Language.
- `--user-agent` — User-Agent custom (override del default Chrome 145).

### Download
- `--max-file-size` (default: `50MB`) — Tamaño máximo de archivo.
- `--download-timeout` (default: `30s`) — Timeout individual por asset.

### AI (feature `ai`)
- `--threshold` (default: `0.3`) — Umbral de relevancia semántica (0.0-1.0).
- `--max-tokens` (default: `512`) — Tokens máximos por chunk.
- `--offline` — Modelo ONNX sin conexión.

### Elastic Ingestion
- `--elastic` — Activa pipeline elástico (streaming, SQLite dedup, Rayon CPU bridge).
- `--cpu-cores` — Override para pool Rayon.
- `--ram-budget` — Override para semáforo byte-weighted (ej: `8GB`, `2048MB`).
- `--db-path` — Path custom para SQLite.

### Competitive Features
- `--checkpoint-interval` (default: `100`) — Páginas entre checkpoints (0 = disabled).
- `--no-checkpoint` — Desactiva checkpoints.
- `--ignore-robots` — No respeta robots.txt.
- `--autoscale` — Concurrencia autoscalable según RAM.
- `--no-session-health` — Desactiva health checks del session pool.
- `--h2-profile` (default: `Chrome145`) — Perfil TLS/HTTP2.

### JS Rendering
- `--js-strategy` (default: `static`) — `static` (wreq), `hybrid` (3 capas), `full` (Chromiumoxide).
- `--obscura-binary` (default: `obscura`) — Path al binario obscura.

### Batch Processing
- `--batch` — Lee URLs de stdin (una por línea).
- `--batch-file` — Path a archivo con URLs.
- `--batch-concurrency` (default: `5`) — URLs concurrentes en batch.

### Item Pipeline
- `--pipeline` — Activa pipeline: validate → clean → output.
- `--pipeline-output` (default: `jsonl`) — Formato: `jsonl` o `none`.

---

## Subcommands

- `completions <shell>` — Genera scripts de completado para bash/zsh/fish.

---

## MCP Server — 37 Herramientas

El servidor MCP expone herramientas agrupadas en 6 categorías:

### Scraping Core (8 herramientas)
- `scrape_url` — Scrapea una URL con Readability (Firefox Reader mode)
- `scrape_with_options` — Scrapea con opciones (assets, concurrencia, delay)
- `scrape_batch` — Batch de URLs con concurrencia controlada
- `crawl_site` — Crawlea un sitio completo desde la semilla
- `crawl_with_sitemap` — Crawlea usando sitemap XML
- `discover_urls` — Descubre URLs desde una página semilla
- `detect_spa` — Detecta si una página es SPA (JavaScript-heavy)
- `clean_html` — Limpia HTML boilerplate

### Content Processing (5 herramientas)
- `html_to_markdown` — Convierte HTML a Markdown limpio
- `extract_links` — Extrae enlaces internos/externos de HTML
- `highlight_code` — Syntax highlighting para bloques de código
- `convert_wiki_links` — Convierte enlaces a formato Obsidian `[[wiki-link]]`
- `verify_waf_integrity` — Verifica integridad contra WAF

### URL Utilities (5 herramientas)
- `validate_url` — Valida formato de URL
- `extract_domain` — Extrae dominio de una URL
- `normalize_url` — Normaliza URL (resuelve relativas, quita fragments)
- `match_url_pattern` — Matchea URLs contra patterns glob
- `is_internal_link` — Determina si un enlace es interno al dominio

### Security (1 herramienta)
- `detect_waf` — Detecta protección WAF (Cloudflare, reCAPTCHA, etc.)

### Export (4 herramientas)
- `export_file` — Exporta contenido a archivos
- `export_jsonl` — Exporta como JSON Lines
- `export_vector` — Exporta en formato vectorial para RAG
- `process_export_pipeline` — Pipeline completo de exportación

### Obsidian (3 herramientas)
- `detect_vault` — Auto-detecta vault de Obsidian
- `build_obsidian_uri` — Construye `obsidian://` URIs
- `search_obsidian` — Busca notas en el vault

### Assets (1 herramienta)
- `download_assets` — Descarga imágenes, CSS, JS de una página

### AI (feature-gated, 2 herramientas)
- `generate_frontmatter` — Genera YAML frontmatter con metadata
- `generate_rich_metadata` — Metadata rica para frontmatter

### Security Verification (1 herramienta)
- `verify_waf_integrity` — Verifica que el scraping no fue bloqueado por WAF

---

## Arquitectura (Clean Architecture / Hexagonal)

```
src/
├── main.rs                    # Entry point, CLI parsing
├── cli/                       # CLI layer
│   ├── args.rs               # Clap Args (1421 líneas, todos los flags)
│   ├── orchestrator.rs       # Coordinador principal: batch, standard, elastic
│   ├── scrape_flow.rs        # Fase de scraping
│   ├── export_flow.rs        # Fase de exportación
│   ├── url_discovery.rs      # Descubrimiento de URLs
│   ├── preflight.rs          # Validación pre-ejecución
│   ├── wizard.rs             # Prompt interactivo para URL
│   ├── config.rs             # Config file (TOML)
│   ├── completions.rs        # Shell completions
│   └── error.rs              # Errores CLI (Spanish user-facing)
├── domain/                    # Entidades y reglas de negocio
│   ├── entities.rs           # ScrapedContent, DocumentChunk, etc.
│   ├── crawl_job/            # Entidades del job de crawl
│   ├── site/config.rs        # CrawlerConfig builder
│   └── js_strategy.rs        # Enum: Static, Hybrid, Full
├── application/              # Casos de uso
│   ├── crawler/              # Engine de crawl
│   │   ├── engine.rs         # Core engine (963 líneas)
│   │   └── discovery.rs      # Robots.txt, sitemap discovery
│   ├── http_client/          # Cliente HTTP con retries
│   ├── scraper_service.rs    # Servicio de scraping
│   ├── crawler_service.rs    # Servicio de crawl
│   ├── elastic_ingestion.rs  # Pipeline elástico (SQLite + Rayon)
│   ├── container.rs          # DI container
│   └── rate_limiter.rs       # Rate limiting
├── infrastructure/           # Implementaciones concretas
│   ├── crawler/              # Sitemap parser, batch processor, etc.
│   ├── network/              # Session pool con health checks
│   ├── downloader/           # Resource downloader, cookie bridge
│   ├── export/               # JSONL exporter
│   ├── persistence/          # SQLite, state store
│   ├── checkpoint/           # Checkpoint persistence
│   ├── ai/                   # Semantic cleaner (ONNX), tokenizer
│   ├── mcp_server/           # Servidor MCP (1404 líneas)
│   │   ├── mod.rs            # 37 tools registered
│   │   ├── handlers/         # 7 módulos: scraping, content, url_utils, etc.
│   │   └── server.rs         # start_mcp_server
│   ├── obsidian/             # Vault detection
│   ├── output/               # File saver, Obsidian options
│   ├── observability/        # OTel, metrics, logging
│   ├── autotuning.rs         # Auto-detect CPU/RAM
│   └── config.rs             # ScraperConfig
├── adapters/                  # TUI (ratatui)
│   └── tui/                  # 12+ archivos
│       ├── app.rs            # App principal, event loop
│       ├── url_selector.rs   # Selector interactivo de URLs
│       ├── config_form.rs    # Formulario de configuración
│       ├── progress_widget.rs# Widget de progreso
│       ├── error_log_widget.rs # Log de errores
│       ├── modal.rs          # Help modal
│       └── component.rs      # Header, StatusBar
├── extractor/                 # Extracción de contenido
│   └── mod.rs                # extract_images, extract content
└── lib.rs                    # Re-exports públicos
```

---

## Pipeline Elástico (--elastic)

Modo de ingestión streaming para RAG:
- SQLite para deduplicación y persistencia
- Rayon pool para CPU bridge (configurable con `--cpu-cores`)
- Semáforo byte-weighted para control de RAM (`--ram-budget`)
- Auto-tuning de recursos con `autotuning.rs`

---

## Session Pool y WAF Evasion

- Pool de sesiones HTTP con health checks
- Auto-evicción de sesiones stale
- Cooldown para sesiones baneadas
- Perfiles TLS (Chrome145 default via `--h2-profile`)
- Detección de WAF (Cloudflare, reCAPTCHA)
- User-Agent rotation

---

## Observabilidad

- Tracing estructurado (tracing-subscriber)
- OpenTelemetry distributed tracing (`--features otel`)
- OpenTelemetry metrics (`--features otel-metrics`)
- OTel spans exportables a JSONL (`--trace-file`)
- Tokio console integration (`--features console`)
- Dev tracing con árbol jerárquico (`--features dev-tracing`)

---

## Errores

User-facing messages en **Spanish**. Logs internos en **English**.
Error chain: `[CLI] → ScraperError :: [domain] CrawlError :: [infra] HttpError/WafError/ParseError`
