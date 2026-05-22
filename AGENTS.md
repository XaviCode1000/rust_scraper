# Agent Instructions — Rust Scraper

Production-ready web scraper with Clean Architecture, TUI selector, and AI semantic cleaning.

**Stack:** Rust 1.88 · Tokio · wreq (TLS fingerprint) · ratatui · tract-onnx (feature-gated)
**Hardware:** Intel i5-4590 (4C), 8GB DDR3, HDD — all commands are HDD-optimized

---

## Key Commands

### ✅ Safe for Local (HDD-friendly, <30s)

```bash
cargo check              # Verify compilation
cargo check --features ai # With AI feature
cargo clippy -- -D warnings # Lint
cargo fmt --check         # Format check
cargo fmt                 # Format
```

### Just Recipes

```bash
just check          # fmt + clippy strict
just check-fast     # cargo check (fastest)
just fmt            # format code
just audit          # audit + deny + machete
```

### 🚫 NUNCA Ejecutar Localmente (HDD + 8GB RAM)

**Estos comandos CONGELAN el sistema y cierran la terminal. Ejecutarlos es un error de disciplina.**

| Comando | Por qué es peligroso | Alternativa |
|---------|---------------------|-------------|
| `cargo nextest run` | 680 tests — 5-10 min, 100% CPU + HDD | `gh workflow run ci.yml` |
| `cargo nextest run --all-features` | Incluye AI (90MB) — congelamiento total | CI en la nube |
| `cargo build --release` | Optimización — 10+ min | CI en la nube |
| `cargo build` | Más lento que `cargo check` | `cargo check` |
| `just test-ci` | Gate completo — 10+ min | `gh workflow run ci.yml` |
| `cargo llvm-cov` | Instrumenta + tests — 15+ min | CI en la nube |
| `cargo miri test` | Interpreta instrucciones — 30+ min | CI en la nube |

**Regla simple para agentes y sub-agentes:**

```
LOCAL:  cargo check, cargo clippy, cargo fmt --check  (<30s total)
NUBE:   gh workflow run ci.yml && gh run watch         (todo lo demás)
```

**Para tests locales** (solo debugging específico, nunca suite completa):
```bash
cargo nextest run --test-threads 2 -E 'test(mi_test_especifico)'
```

> **NOTE:** First `cargo check` takes ~4 min (cold compile, 300 crates). After that, `sccache` makes everything fast.

---

## Code Style

Error messages are in **Spanish** (not English). HTTP client is **`wreq`**, not `reqwest`.

```rust
// src/error.rs — Error messages in Spanish
#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("URL inválida: {0}")]
    InvalidUrl(String),
    #[error("error de red: {0}")]
    Network(String),
    #[error("WAF/CAPTCHA detectado en {url}: {provider}")]
    WafBlocked { url: String, provider: String },
}

// src/application/http_client.rs — wreq, NOT reqwest
use wreq::Client;
use wreq_util::emulation::ClientBuilderExt;

let client = Client::builder()
    .emulate(wreq_util::emulation::KnownVersion::Chrome131)
    .build()?;
```

---

## Project Architecture

```
src/
├── adapters/        # External adapters (HTTP, filesystem)
├── application/    # Use cases, services (CrawlerService, ScraperService)
├── cli/            # CLI argument parsing and commands
├── domain/         # Entities, value objects, domain logic
├── extractor/      # Content extraction (HTML, text)
├── infrastructure/  # AI, Obsidian, detectors, converters
│   ├── ai/         # Semantic cleaning (ONNX embeddings)
│   ├── obsidian/   # Vault detection and parsing
│   └── ...
└── lib.rs          # Main library (ScraperConfig, exports)
```

**Key modules:**
- `src/application/crawler_service.rs` — Crawling with rate limiting
- `src/application/scraper_service.rs` — Page scraping with SPA detection
- `src/infrastructure/ai/semantic_cleaner_impl.rs` — AI content cleaning
- `src/infrastructure/obsidian/` — Obsidian vault integration
- `src/cli/` — CLI commands and TUI

---

## Non-Obvious Patterns

### Crate version conflicts (DO NOT try to unify)

- `dashmap` 5.x (via governor) + 6.x (direct) — both needed
- `quick-xml` 0.37 (direct) + 0.38 (via syntect→plist) — both needed
- `scraper` 0.27 → selectors 0.35, `legible` → dom_query → selectors 0.38 — both needed

### HTTP client: `wreq` not `reqwest`

Uses TLS fingerprint emulation (Chrome 131) for WAF evasion. Layer 2 evasion built in.

### WAF detection on HTTP 200

Responses are scanned for WAF signatures (Cloudflare, reCAPTCHA, hCaptcha, DataDome, PerimeterX, Akamai). If detected, UA is rotated and retried once. Still blocked → `ScraperError::WafBlocked`.

### AI feature (`--features ai`)

- Loads ~90MB ONNX model (all-MiniLM-L6-v2) into memory
- `SemanticCleanerImpl::new()` is **async** — loads model once, reuses
- `cleaner.clean(html)` is **async** — returns `Vec<DocumentChunk>` with embeddings
- One page → multiple chunks when AI cleaning is active
- Model cached in `~/.cache/rust_scraper/models/`

---

## GitNexus — Code Intelligence

Use the `gitnexus-master` skill for all code intelligence operations. The skill is loaded from `~/.config/opencode/skills/gitnexus-master/SKILL.md`.

**Mandatory workflow:**

- **Before editing any symbol:** run `gitnexus_impact({target: "symbolName", direction: "upstream"})`
- **Before committing:** run `gitnexus_detect_changes()`
- **HIGH/CRITICAL risk from impact:** stop and warn the user
- **Before renaming:** use `gitnexus_rename({symbol_name: "old", new_name: "new", dry_run: true})`

**Index:** `gitnexus analyze` · **Status:** `gitnexus status`

---

## Testing Rules

- Write tests for all new functionality
- Tests must be deterministic and isolated
- Mock all external dependencies
- **Never run full suite locally** — use `gh workflow run ci.yml`
- For specific test debugging: `cargo nextest run --test-threads 2 -E 'test(name)'`

### HDD Configuration (CRITICAL)

```toml
# .config/nextest.toml
[profile.default]
threads-required = 2  # MAX 2 threads — prevents thrashing
retries = { backoff = "exponential", count = 2, delay = "1s" }
slow-timeout = { period = "60s", terminate-after = 3 }
```

Perfiles nextest: `dev` (rápido), `agent` (conservador), `ci` (completo)

---

## Boundaries

### ✅ Always

- Run `cargo check` before marking any task complete
- Run `cargo clippy -- -D warnings` before committing
- Use `cargo nextest run` (never `cargo test`)
- Use `cargo llvm-cov` (never `cargo tarpaulin`)
- Use `bacon` for background checking (never `cargo-watch`)
- Use `just` recipes for multi-step tasks (audit, coverage, release)

### ⚠️ Ask first

- Adding or removing dependencies
- Changing feature flag structure
- Modifying `Cargo.toml` profiles

### 🚫 Never

- Commit secrets, `.env` files, or credentials
- Use `.unwrap()` in production code — use `?` or `match`
- Force push to main or protected branches
- Modify `target/`, `dist/`, or `build/` directories
- Run any command from the forbidden table above

---

## SDD Workflow

Spec-Driven Development via skills en `~/.config/opencode/skills/`:

| Skill | Propósito |
|-------|-----------|
| sdd-init | Inicializar contexto, detectar stack |
| sdd-explore | Investigar código existente |
| sdd-propose | Crear propuesta |
| sdd-spec | Escribir especificaciones |
| sdd-design | Diseño técnico |
| sdd-tasks | Lista de tareas |
| sdd-apply | Implementar (con gitnexus_impact) |
| sdd-verify | Verificar contra specs |
| sdd-archive | Archivar cambio |

### Pipeline SDD + GitNexus
1. `gitnexus_impact` → antes de editar
2. `gitnexus_detect_changes` → pre-commit
3. `gh workflow run ci.yml` → verificación en nube
4. `gh run watch` → esperar resultado
5. `git push` → solo después de ✅ verde

---

## Rust Best Practices

Este proyecto incluye reglas de rust-skills en `~/.config/opencode/skills/rust-skills/`:

| Categoría | Ejemplos |
|-----------|----------|
| Memory | mem-zero-copy, mem-smallvec, mem-compact-string |
| Performance | perf-release-profile, perf-profile-first, perf-collect-once |
| API Design | api-typestate, api-non-exhaustive, api-serde-optional |
| Async | async-tokio-runtime, async-no-lock-await |
| Testing | test-integration-dir, test-tokio-async, test-proptest-properties |
| Error Handling | err-question-mark, err-lowercase-msg |

### Auto-load de rust-skills
Cuando el agente escribe código **Rust**, cargar automáticamente:
```
skill(name: "rust-skills")
```

---

## Resources

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — Architecture details
- [DEVELOPMENT.md](DEVELOPMENT.md) — Dev workflow and tooling
- [justfile](justfile) — Task recipes (check, fmt, audit)
- [docs/wiki/](docs/wiki/) — GitNexus auto-generated documentation
