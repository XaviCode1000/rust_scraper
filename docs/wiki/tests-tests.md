# Tests — tests

# Tests — tests

This module contains all the integration and unit tests for the `rust_scraper` crate. It is organized into several sub-modules, each focusing on a specific area of the application:

- `ai_integration.rs`: Tests for AI-powered semantic cleaning features.
- `cli_binary_test.rs`: Integration tests for the `rust_scraper` binary using `assert_cmd`.
- `cli_tests.rs`: Unit tests for the CLI argument parsing logic.
- `concurrency_tests.rs`: Tests for concurrent operations and race conditions.
- `crawler_integration.rs`: Integration tests for the web crawler functionality.
- `exporter_integration_test.rs`: Integration tests for the `FileExporter`, focusing on data integrity.
- `http_client_integration.rs`: Integration tests for the `HttpClient` using `wiremock`.
- `integration_test.rs`: High-level integration tests for the full scraping pipeline.
- `mcp_proptest.rs`: Proptest and feature-gating tests for the MCP server.

## Purpose

The primary purpose of this module is to ensure the correctness, reliability, and robustness of the `rust_scraper` application. It covers:

- **Functionality**: Verifying that core features like crawling, scraping, data extraction, and exporting work as expected.
- **Integration**: Testing how different components interact with each other.
- **Edge Cases**: Identifying and handling potential issues with invalid inputs, network errors, concurrency, and resource constraints.
- **Performance**: Ensuring that operations are efficient and do not lead to excessive resource consumption.
- **API Stability**: Validating that public APIs and CLI behavior remain consistent.

## Key Components and Testing Strategies

This section details the testing strategies employed for different parts of the `rust_scraper` codebase.

### AI Integration Tests (`ai_integration.rs`)

These tests are feature-gated behind the `ai` feature flag. They verify the functionality of AI-powered semantic cleaning, including:

- **Model Cache**: Ensuring the model cache directory is created and managed correctly.
- **Model Download**: Testing the structure and error handling of the model downloader.
- **Model Configuration**: Validating default values and builder patterns for `ModelConfig`.
- **Inference Engine**: Checking thread-safety (`Send + Sync`) and cloning capabilities.
- **Chunking and Tokenization**: Testing `ChunkId`, `SentenceSplitter`, and `HtmlChunker`.
- **Embedding Operations**: Verifying cosine similarity, dot product, normalization, and Euclidean distance.
- **Relevance Scoring**: Testing `RelevanceScorer` and `ThresholdConfig`.
- **Full Pipeline**: Integrating and testing the entire RAG pipeline from HTML parsing to relevance filtering.
- **Error Handling**: Testing scenarios like `ChunkTooLarge` and `OfflineMode`.

**Example Test:** `test_semantic_cleaner_full_pipeline` verifies the end-to-end RAG pipeline by cleaning HTML, tokenizing, embedding, scoring, and filtering.

### CLI Binary Tests (`cli_binary_test.rs`)

These tests use `assert_cmd` to execute the compiled `rust_scraper` binary and assert its behavior. They focus on:

- **Error Handling**: Testing error messages for missing arguments (`--url`), invalid URLs, and other command-line errors.
- **Help and Version**: Verifying that `--help` displays the correct information and `--version` outputs the expected version string.
- **Dry-Run Mode**: Checking that the `--dry-run` flag is accepted.
- **Flag Acceptance**: Ensuring various flags like `--quiet` are parsed correctly.

**Example Test:** `test_no_url_shows_error` asserts that running the binary without the required `--url` argument produces a specific error message on stderr.

### CLI Argument Parsing Tests (`cli_tests.rs`)

These tests focus on the `clap` integration for command-line argument parsing. They use `Args::parse_from` to simulate command-line inputs and verify:

- **Default Values**: Ensuring all arguments have correct default values when not specified.
- **Argument Parsing**: Testing short (`-u`) and long (`--url`) flags, including complex values like URLs with query parameters and complex CSS selectors.
- **Output Formats**: Validating parsing for `OutputFormat` (Markdown, Text, Json) and `ExportFormat` (Jsonl, Vector).
- **Boolean Flags**: Checking that flags like `--dry-run`, `--quiet`, `--obsidian-wiki-links` are parsed correctly.
- **Subcommands**: Verifying the parsing of subcommands like `completions` for different shells.
- **Combined Arguments**: Testing scenarios where multiple arguments are provided.

**Example Test:** `test_args_combined_flags` parses a complex set of arguments to ensure all are correctly interpreted.

### Concurrency Tests (`concurrency_tests.rs`)

These tests are crucial for ensuring thread safety and preventing data corruption in concurrent operations. They cover:

- **Concurrent JSONL Exports**: Verifying that multiple tasks exporting to the same JSONL file do not lose data or introduce corruption.
- **StateStore Concurrency**: Testing that `mark_processed` operations on `ExportState` are thread-safe and do not lead to duplicate entries.
- **Concurrent Vector Exports**: Ensuring that batch exports to the Vector format are safe and do not corrupt the output file.
- **Append Mode Preservation**: Validating that concurrent appends to a JSONL file preserve existing content.

**Example Test:** `concurrent_jsonl_exports_no_data_loss` spawns multiple tasks that write to a shared `JsonlExporter`, then verifies the output file for correctness and completeness.

### Crawler Integration Tests (`crawler_integration.rs`)

These tests focus on the web crawler's functionality and require network access. They are typically ignored by default.

- **Pattern Matching**: Testing `matches_pattern`, `is_excluded`, and `is_internal_link` with various URL patterns and exclusion rules.
- **Crawling**: Verifying `crawl_site` against small, real websites.
- **URL Discovery**: Testing `discover_urls_for_tui`.
- **Sitemap Fetching**: Testing `crawl_with_sitemap`.
- **Configuration**: Testing the `CrawlerConfig` builder and default values.

**Example Test:** `test_is_allowed_complex` verifies that the `is_allowed` function correctly applies include and exclude patterns defined in `CrawlerConfig`.

### Exporter Integration Tests (`exporter_integration_test.rs`)

These tests focus on the `FileExporter` and ensure data integrity during file exports.

- **Markdown Structure**: Verifying the correct structure of exported Markdown files (though the primary export format tested here is JSONL).
- **JSON Export**: Ensuring exported data is valid JSON.
- **Validation**: Testing `DocumentChunk` validation rules, including empty content and titles.
- **Batch Export**: Verifying that exporting multiple documents at once works correctly.
- **Content Preservation**: Testing that special characters and markdown syntax are handled correctly during export.
- **Edge Cases**: Testing scenarios like empty content, special characters in URLs, and memory pressure.

**Example Test:** `test_file_exporter_markdown_structure` checks that a single document is exported correctly into a JSONL file with the expected fields.

### HTTP Client Integration Tests (`http_client_integration.rs`)

These tests use `wiremock` to mock HTTP responses, allowing for deterministic testing of the `HttpClient`.

- **Mock Server Responses**: Testing successful responses (200 OK) and various error codes (404, 500, 429, 503).
- **Rate Limiting**: Verifying that the client handles `429 Too Many Requests` responses, including retries and backoff strategies.
- **Service Unavailable**: Testing `503 Service Unavailable` responses and the effect of the `Retry-After` header.
- **Latency Simulation**: Testing how the client handles slow responses and timeouts.
- **Empty Responses**: Ensuring the client correctly processes empty response bodies.

**Example Test:** `test_mock_server_429_exhausts_retries` simulates a rate-limited endpoint and verifies that the client exhausts its retries and returns an error after appropriate backoff.

### Integration Tests (`integration_test.rs`)

This module contains high-level integration tests that cover the end-to-end scraping pipeline.

- **Simple Fetching**: Testing the ability to fetch content from a real website (`example.com`).
- **Error Handling**: Verifying graceful handling of `404` errors and invalid URLs.
- **Result Saving**: Testing `save_results` with nested directories, special characters in content, and markdown syntax.
- **Asset Downloading**: (Feature-gated with `images` and `documents`) Testing the download of images and documents from real websites.
- **AI Semantic Filtering**: (Feature-gated with `ai`) Testing the preservation of embeddings after AI semantic cleaning.

**Example Test:** `test_ai_embedding_preservation` (when `ai` feature is enabled) verifies that embeddings are correctly generated and retained in the output chunks after AI processing, fixing a known bug.

### Proptest and Feature-Gating Tests (`mcp_proptest.rs`)

This module combines property-based testing (`proptest`) with feature-gating tests.

- **URL Utilities**: Property-based tests for URL parsing, normalization, and internal link checking, covering various edge cases and invariants.
- **Pattern Matching**: Property-based tests for `matches_pattern`.
- **AI Feature Gating**: Tests that verify the AI module's router is correctly built (or empty) based on the presence or absence of the `ai` feature flag.

**Example Test:** `prop_validate_url_valid_schemes` uses proptest to generate valid HTTP/HTTPS URLs and asserts that they always parse successfully.

## Running Tests

- **All Tests**: `cargo test`
- **Specific Module**: `cargo test --test <module_name>` (e.g., `cargo test --test ai_integration`)
- **With Features**: `cargo test --features <feature_name>` (e.g., `cargo test --features ai --test ai_integration`)
- **Network Tests**: Many integration tests require network access and are marked with `#[ignore]`. Run them with: `cargo test -- --ignored` or `cargo test --test <module_name> -- --ignored`.
- **Binary Tests**: `cargo test --test cli_binary_test`
- **Proptests**: `cargo test --test mcp_proptest`

## Directory Structure

The tests are organized within the `tests/` directory, with each file corresponding to a specific testing focus:

```
tests/
├── ai_integration.rs
├── cli_binary_test.rs
├── cli_tests.rs
├── concurrency_tests.rs
├── crawler_integration.rs
├── exporter_integration_test.rs
├── http_client_integration.rs
├── integration_test.rs
├── mcp_proptest.rs
└── progress_tui_integration.rs # (Implied by call graph, not explicitly listed in provided snippets)
```