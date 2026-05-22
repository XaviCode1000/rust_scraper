# Configuration

# Configuration Module

Centralized application configuration and validation for the scraper/crawler stack.

This module defines the top-level `Config` type used across the application, along with a small `ConfigError` enum for validation failures. It also exposes `AiConfig` behind the `ai` feature flag.

The intent is to keep configuration as an infrastructure concern while still making it easy for application and CLI code to construct, validate, and pass around a single configuration object.

## Responsibilities

- Aggregate the configuration needed by the major subsystems:
  - `ScraperConfig`
  - `CrawlerConfig`
  - `HttpClientConfig`
  - `ObsidianOptions`
  - `AiConfig` when `feature = "ai"`
- Provide sensible defaults through `Config::new()` and `Default`
- Validate cross-cutting invariants that are not owned by the individual config structs
- Surface validation errors through `ConfigError`

## Public API

### `Config`

```rust
pub struct Config {
    pub scraper: ScraperConfig,
    pub crawler: CrawlerConfig,
    pub http: HttpClientConfig,
    pub obsidian: ObsidianOptions,
    #[cfg(feature = "ai")]
    pub ai: AiConfig,
}
```

`Config` is the central bundle of runtime configuration for the application.

The fields are public, so callers can construct or mutate configs directly when needed, but the intended usage is usually:

1. Start from `Config::new()` or `Config::default()`
2. Override one or more sub-configs
3. Call `validate()`
4. Pass the config into application wiring, services, or infrastructure

### `Config::new()`

Creates a configuration with default values for all enabled subsystems.

Key defaults:

- `scraper` uses `ScraperConfig::default()`
- `crawler` is built with:
  - base URL: `"https://example.com"`
  - `CrawlerConfig::builder(...).build()`
- `http` uses `HttpClientConfig::default()`
- `obsidian` uses `ObsidianOptions::default()`
- `ai` defaults are applied when the feature is enabled:
  - `threshold: 0.3`
  - `max_tokens: 512`
  - `offline: false`

`Config::default()` simply delegates to `Config::new()`.

### `Config::validate()`

Validates a few top-level invariants that matter to runtime behavior.

Checks performed:

- `scraper.scraper_concurrency > 0`
- `crawler.max_pages > 0`
- `http.max_retries <= 10`

If any check fails, validation stops at the first error and returns a `ConfigError`.

### `ConfigError`

```rust
pub enum ConfigError {
    InvalidConcurrency,
    InvalidMaxPages,
    InvalidRetries,
}
```

Validation errors are intentionally compact and targeted at the current top-level checks:

- `InvalidConcurrency` — scraper concurrency is zero
- `InvalidMaxPages` — crawler page limit is zero
- `InvalidRetries` — HTTP retry count exceeds the allowed bound

Each variant has a user-facing error message via `thiserror::Error`.

## Feature-gated AI config

When the `ai` feature is enabled, `Config` includes:

```rust
pub struct AiConfig {
    pub threshold: f32,
    pub max_tokens: usize,
    pub offline: bool,
}
```

This struct is deliberately minimal and focused on semantic filtering / offline AI behavior:

- `threshold` controls relevance filtering
- `max_tokens` caps chunk size
- `offline` disables remote AI usage

Because it is feature-gated, any code that accesses `config.ai` must also be compiled with the `ai` feature enabled.

## Validation model

Validation here is intentionally shallow and structural. It does not attempt to re-validate every nested field owned by sub-configs. Instead, it enforces a few global constraints that are easy to violate when wiring the application together.

This keeps the module aligned with clean architecture boundaries:

- Subsystems own their own detailed config semantics
- `Config` enforces top-level runtime safety rules
- CLI/preflight layers can call `validate()` before starting work

### Validation flow

```mermaid
flowchart TD
    A[Config::new / Default] --> B[Caller overrides fields]
    B --> C[Config::validate()]
    C --> D{scraper_concurrency == 0?}
    D -- yes --> E[Err InvalidConcurrency]
    D -- no --> F{crawler.max_pages == 0?}
    F -- yes --> G[Err InvalidMaxPages]
    F -- no --> H{http.max_retries > 10?}
    H -- yes --> I[Err InvalidRetries]
    H -- no --> J[Ok(())]
```

## Integration points in the codebase

This module is not isolated; it is wired into CLI, HTTP client setup, scraper flow, and MCP server initialization.

### CLI / orchestration

The CLI orchestrator constructs and adjusts `Config` using the scraper-related builder methods from `ScraperConfig`:

- `with_images()`
- `with_documents()`
- `with_output_dir(...)`
- `with_max_pages(...)`
- `with_scraper_concurrency(...)`

The orchestration flow also uses `ConcurrencyConfig::resolve()` and `ConcurrencyConfig::is_auto()` from `src/infrastructure/config.rs` during preflight/default application.

### HTTP client

HTTP client creation consumes `Config` directly:

- `create_http_client` uses `Config`
- `create_rate_limited_client` reads values via `Config`/related config accessors such as `get`

The top-level config therefore influences retries, concurrency, and client behavior downstream.

### Scraper and crawler services

Scraper/crawler paths check whether asset downloads are enabled by calling:

- `ScraperConfig::has_downloads()`

This is the main switch used by:

- `download_assets_if_enabled`
- integration-level scrape flows that decide whether to fetch images/documents

### MCP server

The MCP server wiring constructs or defaults `Config` to bootstrap request handling and application behavior.

## How this module relates to `infrastructure::config`

Do not confuse `src/config.rs` with `src/infrastructure/config.rs`.

They solve different problems:

- `src/config.rs` defines the **top-level application configuration bundle**
- `src/infrastructure/config.rs` defines **scraper/runtime configuration primitives**, including:
  - `ScraperConfig`
  - `ConcurrencyConfig`
  - `OutputFormat`

`Config` depends on `ScraperConfig` from the infrastructure module, but it is not a replacement for it.

## Construction patterns

### Recommended: start with defaults

```rust
let mut config = Config::new();
config.scraper = config.scraper.with_images();
config.scraper = config.scraper.with_output_dir("./output".into());

config.validate()?;
```

### Recommended: mutate before startup

Because fields are public, a common pattern is to create the config early, override values from CLI/env/file loading, and then validate before wiring application services.

### Avoid: bypassing validation

Since validation is separate from construction, callers should not assume `Config::new()` implies the runtime is safe for all inputs. Any externally sourced overrides should be validated explicitly.

## Notes on invariants

The current top-level checks are conservative:

- `scraper_concurrency == 0` is rejected even though `ScraperConfig::default()` uses `3`
- `crawler.max_pages == 0` is rejected even though the default builder path likely produces a nonzero or unset value
- `http.max_retries > 10` is rejected to prevent unbounded or excessive retry behavior

If you add new top-level fields that can invalidate runtime behavior, extend `Config::validate()` and add a matching `ConfigError` variant.

## Testing implications

The tests in this area primarily live in `src/infrastructure/config.rs`, but this module’s behavior is exercised indirectly through:

- application container wiring
- HTTP client creation
- asset download flows
- MCP server setup

When changing `Config` defaults or validation rules, verify the following integration points still behave as expected:

- CLI startup paths
- crawler page limits
- HTTP retry behavior
- asset download enablement
- AI feature-gated code paths

## Extending the module

When adding a new configuration field:

1. Add the field to `Config`
2. Initialize it in `Config::new()`
3. Add validation in `Config::validate()` if there is a global invariant
4. Add a corresponding error variant in `ConfigError` when needed
5. Update call sites that construct or destructure `Config`

When adding feature-gated config:

- follow the existing `#[cfg(feature = "...")]` pattern used for `AiConfig`
- keep defaults inside `Config::new()` so the feature remains easy to enable

## Minimal example

```rust
use rust_scraper::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();

    config.scraper = config.scraper.with_images();
    config.scraper = config.scraper.with_scraper_concurrency(5);
    config.scraper = config.scraper.with_max_pages(100);

    config.validate()?;

    Ok(())
}
```

## Summary

`src/config.rs` is the application’s central configuration hub:

- it assembles subsystem configs into one struct
- it provides defaults suitable for bootstrapping the app
- it enforces a small set of global safety checks
- it is the primary handoff object for CLI, services, and infrastructure wiring