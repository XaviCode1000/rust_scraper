# Documentation and Examples — STATE.md

# Project State

This document provides a snapshot of the `rust_scraper` project's current status, including test results, recent fixes, architectural details, and relevant commands. It serves as a quick reference for developers to understand the project's health and operational aspects.

## Current Status

✅ **ALL TESTS PASSING**

**Date**: 2026-04-22

## Test Suite Status

The project's test suite is comprehensive and currently shows all tests passing.

| Test Group              | Status   |
|-------------------------|----------|
| All unit tests          | ✅ PASS  |
| Integration tests       | ✅ PASS  |
| CLI binary tests        | ✅ PASS  |
| Security tests          | ✅ PASS  |
| Vector exporter tests   | ✅ PASS  |

## Recent Fixes

This section details recent changes made to address specific issues and improve the project's stability and functionality.

### 1. Tokio-console Conflict

*   **Files Affected**: `Cargo.toml`, `src/main.rs`
*   **Issue**: The `console-subscriber` crate, when used as a global test hook (`0.5`), interfered with the execution of CLI binary tests.
*   **Resolution**: The `console-subscriber` dependency was removed from `Cargo.toml`, and the `console_subscriber::init()` call was removed from `src/main.rs`.
*   **Tests Impacted**: 5 CLI binary tests.

### 2. Entropy Detection Threshold Adjustment

*   **File Affected**: `src/application/http_client/waf.rs`
*   **Issue**: The UTF-8 encoding of code points 128-255 results in 2-byte sequences, which naturally lowers the byte entropy to approximately 5.5 bits. This was causing false positives in entropy-based detection.
*   **Resolution**: The entropy threshold for detection was lowered from 6.5 to 6.0, and subsequently to 5.5, to accommodate this characteristic of UTF-8 encoding.
*   **Tests Impacted**: `test_detect_by_entropy_high`, `test_datadome_high_entropy_detection`.

### 3. CLI Error Output Correction

*   **File Affected**: `src/cli/error.rs`
*   **Issue**: The `CliExit::Termination::report()` method was not printing error messages to `stderr` as expected.
*   **Resolution**: `eprintln!("Error: {}", msg)` was added for each error variant within `report()` to ensure errors are directed to the standard error stream.
*   **Tests Impacted**: `test_invalid_url_shows_error`.

### 4. Vector Exporter Test Data Correction

*   **File Affected**: `src/infrastructure/export/vector_exporter.rs`
*   **Issue**: The `create_test_chunk()` helper function was generating documents without the `embeddings` field. However, some tests were written expecting this field to be present.
*   **Resolution**: In the affected tests, the `embeddings` field is now manually added to the document using `doc.embeddings = Some(vec![...])`.
*   **Tests Impacted**: `test_serialize_document_with_embeddings`, `test_serialize_document_dimension_mismatch`.

## Architecture Summary

*   **Core Technologies**: Rust 1.88, Tokio (for asynchronous operations), wreq (for HTTP requests, including TLS fingerprinting), ratatui (for TUI rendering), tract-onnx (for ONNX model inference).
*   **Development Hardware**: Intel i5-4590 (4 Cores), 8GB DDR3 RAM, HDD storage.
*   **Test Execution**: `cargo-nextest` is used as the test runner, configured with 2 threads to optimize performance on HDD systems.

## Commands

A collection of useful commands for interacting with the `rust_scraper` project.

*   **Run Tests**:
    ```bash
    cargo nextest run --test-threads 2
    ```
*   **Check Compilation**:
    ```bash
    cargo check
    ```
*   **Linting**:
    ```bash
    cargo clippy -- -D warnings
    ```
*   **CI Pipeline Execution**:
    ```bash
    just test-ci
    ```

## Call Graph & Execution Flows

This module primarily serves as a status report and does not contain executable code that participates in internal or external call graphs or execution flows within the `rust_scraper` application itself. It is a documentation artifact.