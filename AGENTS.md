# AGENTS.md — Rust Scraper

Production-ready web scraper. Clean Architecture, TUI selector, AI semantic cleaning.

**Stack:** Rust 1.88 · Tokio · wreq (TLS fingerprint) · ratatui · tract-onnx (feature-gated)
**Hardware:** Intel i5-4590 (4C), 8GB DDR3, HDD — cloud CI for heavy work

---

## Commands

### Local (safe, <30s total)

```bash
cargo check                    # Verify compilation
cargo check --features ai      # With AI feature
cargo clippy -- -D warnings    # Lint
cargo fmt --check              # Format check
```

### Forbidden on this machine (HDD + 8GB RAM — WILL freeze system)

| Command | Why | Alternative |
|---------|-----|-------------|
| `cargo nextest run` | 680 tests, 5-10 min, 100% CPU | `gh workflow run ci.yml` |
| `cargo nextest run --all-features` | AI model (90MB) loads | CI |
| `cargo build --release` | 10+ min optimization | CI |
| `cargo build` | Slower than `cargo check` | `cargo check` |
| `just test-ci` | Full gate, 10+ min | `gh workflow run ci.yml` |
| `cargo llvm-cov` | Instrument + test, 15+ min | CI |
| `cargo miri test` | Interprets instructions, 30+ min | CI |

```
RULE: LOCAL = cargo check/clippy/fmt (<30s) | CLOUD = everything else
```

---

## Code Style

- Error messages in **Spanish** (not English)
- HTTP client is **`wreq`** (not `reqwest`) — TLS fingerprint emulation for WAF evasion
- Never use `.unwrap()` in production — use `?` or `match`

---

## Non-Obvious Patterns

### Crate version conflicts (DO NOT unify)

- `dashmap` 5.x (via governor) + 6.x (direct) — both needed
- `quick-xml` 0.37 (direct) + 0.38 (via syntect→plist) — both needed
- `scraper` 0.27 → selectors 0.35, `legible` → dom_query → selectors 0.38 — both needed

### WAF detection on HTTP 200

Responses scanned for WAF signatures (Cloudflare, reCAPTCHA, hCaptcha, DataDome, PerimeterX, Akamai). If detected → UA rotation + retry. Still blocked → `ScraperError::WafBlocked`.

### AI feature (`--features ai`)

- Loads ~90MB ONNX model (all-MiniLM-L6-v2) — async init, reused across pages
- Model cached in `~/.cache/rust_scraper/models/`
- `cleaner.clean(html)` → `Vec<DocumentChunk>` with embeddings

---

## Boundaries

### Always

- `cargo check` before marking tasks complete
- `cargo clippy -- -D warnings` before committing
- Run `cargo fmt` before committing

### Ask first

- Adding or removing dependencies
- Changing feature flags
- Modifying `Cargo.toml` profiles

### Never

- Commit secrets, `.env`, or credentials
- Use `.unwrap()` in production code
- Force push to main
- Modify `target/`, `dist/`, `build/` directories
- Run any command from the forbidden table above

---

## Skills (load before work)

| Purpose | Skill | Load when |
|---------|-------|-----------|
| Code intelligence | `gitnexus-master` | Any code work, editing, debugging |
| Rust quality | `rust-skills` | Writing Rust code |
| SDD workflow | `sdd-*` skills | Planning/verifying changes |

The orchestrator passes exact `SKILL.md` paths to sub-agents. Sub-agents read skills BEFORE task work.

---

## Resources

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — Architecture details
- [DEVELOPMENT.md](DEVELOPMENT.md) — Dev workflow and tooling
- [justfile](justfile) — Task recipes
- [docs/wiki/](docs/wiki/) — GitNexus auto-generated documentation
