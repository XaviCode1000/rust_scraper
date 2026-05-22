# Other

# Other Module

This documentation covers various aspects of the `rust-scraper` project that don't fit neatly into a single, cohesive module. It includes refactoring logs, module descriptions, and details about the main entry point of the application.

## Refactoring Log (Fase 5: Limpieza Final)

This section details the final cleanup phase of the project, focusing on dependency management, code splitting, and security.

### Dependencias

*   **Removed:** 10 unused dependencies were removed: `flate2`, `md5`, `memmap2`, `ndarray`, `ort`, `pulldown-cmark-to-cmark`, `slug`, `tokio-util`, `tracing-appender`, `urlencoding`.
*   **Verification:** `cargo machete` confirmed 0 unused dependencies remain.

### File Splits

Several large files were split into smaller, more manageable modules to improve organization and maintainability:

*   `obsidian.rs` (678 lines) was split into `obsidian.rs` (190 lines) and `wikilinks.rs` (428 lines).
*   `sitemap_parser.rs` (753 lines) was split into `sitemap_parser.rs` (581 lines) and `sitemap_config.rs` (148 lines).
*   `model_cache.rs` (649 lines) was split into `model_cache.rs` (254 lines) and `cache_config.rs` (137 lines).

The following files were *not* split due to minimal refactoring needs:

*   `crawler_service.rs`: Only 2 deprecated functions were identified.
*   `client.rs`: Only 33 lines related to retries were present.

### Seguridad

*   **RUSTSEC-2026-0009 (time):** Blocked due to `tract-linalg`'s upper bound on the `time` crate (`<0.3.42`). The fix requires `time >=0.3.47`. This is pending an upstream update in `tract-onnx`.
*   **RUSTSEC-2026-0097 (rand):** A warning was noted upstream. This is being monitored.

### Métricas Finales

*   **Tests Passing:** 440/440 tests are passing.
*   **Large Files:** 5 files exceed 600 lines, but these are considered justified.
*   **Clippy:** The codebase is clean, with 0 errors and 0 warnings.
*   **Unused Dependencies:** 0 unused dependencies.
*   **GitNexus Index:** 3064 nodes, 6314 edges.

## Module Descriptions

This section provides a brief overview of key modules within the `rust-scraper` project.

### `src/application/crawler/mod.rs`

This module is the entry point for the crawler functionality, refactored from `crawler_service.rs`. It organizes the crawler's components into sub-modules:

*   **`discovery`:** Handles URL discovery and sitemap parsing.
*   **`engine`:** Manages the asynchronous crawl engine and orchestration logic.
*   **`state`:** Contains types for tracking crawl progress and state.

It also re-exports key functions for backward compatibility and convenience.

### `src/application/mod.rs`

The **Application Layer** serves as the core of the use cases and orchestration logic. It depends on both the Domain and Infrastructure layers. This module exposes:

*   **Container:** Application dependency injection.
*   **Crawl Result Repository:** Interface for storing crawl results.
*   **Crawler Service:** Orchestration of crawling tasks.
*   **Deduplicator:** Logic for removing duplicate URLs.
*   **Export Factory/Utils:** Utilities for exporting data.
*   **HTTP Client:** Creation of HTTP clients.
*   **Rate Limiter:** Configuration and shared instances of rate limiters.
*   **Results Channel:** Mechanisms for communicating crawl results.
*   **Scraper Service:** Logic for web scraping, including SPA detection.
*   **URL Filter:** Utilities for filtering and validating URLs.

### `src/cli/mod.rs`

The **CLI Module** acts as an adapter for command-line interface interactions. It handles argument parsing, error reporting, shell completions, and configuration management. Key sub-modules include:

*   **`args`:** Defines the command-line arguments and subcommands.
*   **`commands`:** Specific command implementations.
*   **`completions`:** Generates shell completion scripts.
*   **`config`:** Manages application configuration.
*   **`error`:** Handles CLI-specific error types and exit codes.
*   **`export_flow`:** Orchestrates the export process.
*   **`orchestrator`:** The main entry point for running the CLI application.
*   **`preflight`:** Performs initial checks and configuration merging.
*   **`scrape_flow`:** Orchestrates the scraping process.
*   **`summary`:** Displays a summary of the crawl/scrape operation.
*   **`url_discovery`:** Handles URL discovery from the CLI.
*   **`wizard`:** Interactive configuration wizard.

It also defines the `SelectedUrls` enum to represent the outcome of URL selection.

### `src/domain/repositories.rs`

This module defines the **Repository Interfaces** for domain data persistence. It outlines the contracts that the Infrastructure layer must implement for storing and retrieving domain entities, such as `ScrapedContent`.

The `CrawlResultRepository` trait includes methods for:

*   `save`: Persisting scraped content.
*   `find_by_url`: Retrieving content by its URL.
*   `get_all_urls`: Fetching a list of all crawled URLs.

A mock implementation (`MockRepo`) is provided for testing purposes.

### `src/infrastructure/http/mod.rs`

This module provides the **HTTP Client Infrastructure**. It primarily re-exports the `create_http_client` function from the application layer, maintaining architectural consistency. It also contains the `waf_engine` sub-module.

### `src/infrastructure/mod.rs`

The **Infrastructure Layer** contains the concrete implementations of external concerns. It depends on the Domain layer but not vice versa, adhering to Clean Architecture principles. This layer includes:

*   **`config`:** Infrastructure-specific configuration.
*   **`converter`:** Content conversion utilities (e.g., HTML to Markdown).
*   **`crawler`:** Concrete implementation of the crawler.
*   **`export`:** Concrete implementations for data export.
*   **`http`:** HTTP client implementation details.
*   **`mcp_server`:** (Likely related to a specific service or protocol).
*   **`observability`:** Logging and tracing implementations.
*   **`obsidian`:** Integration with Obsidian (e.g., for note-taking).
*   **`output`:** Handling of output formats.
*   **`scraper`:** Concrete web scraping implementations (e.g., using Readability).
*   **`user_agent`:** User-agent management.
*   **`ai`:** (Feature-gated) AI-powered semantic cleaning functionalities.

## Main Application Entry Point (`src/main.rs`)

The `main.rs` file serves as the **thin entry point** for the `rust-scraper` application. It orchestrates the initial setup and then delegates the core logic to the `orchestrator::run()` function.

The execution flow is as follows:

1.  **Parse CLI Arguments:** Uses `clap::Parser` to parse command-line arguments into an `Args` struct.
2.  **Handle Subcommands:** Processes specific subcommands like `Completions`.
3.  **Config TUI:** If the `--config-tui` flag is present, it launches an interactive configuration wizard using `ratatui` and `crossterm`. The results from the TUI can override CLI arguments.
4.  **URL Handling:** If no URL is provided via arguments, it prompts the user interactively (if running in a TTY) or returns an error (in CI environments or non-interactive shells).
5.  **Load Config File:** Loads configuration from a default TOML file (`~/.config/rust_scraper/config.toml`).
6.  **Apply Config Defaults:** Merges configuration from the file with CLI arguments, prioritizing CLI arguments.
7.  **Initialize Logging:** Sets up dual logging (stderr and potentially a file) using `tracing`, respecting verbosity and quiet flags.
8.  **Delegate to Orchestrator:** Calls `orchestrator::run(args).await` to execute the main application logic.

The `main` function is marked with `#[tokio::main]` to enable asynchronous execution. Helper functions like `is_ci`, `stdin_is_tty`, and `run_config_tui` manage environment checks and TUI interactions.