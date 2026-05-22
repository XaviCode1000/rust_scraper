# Observability

# Observability Module

The Observability module provides essential infrastructure for monitoring and debugging the application. It encompasses structured logging, metrics collection, and a stub for distributed tracing.

## Purpose

This module aims to provide production-grade observability by:

*   **Structured Logging:** Enabling detailed, machine-readable logs (JSON) with file rotation for long-term storage and analysis.
*   **Metrics Collection:** Gathering key performance indicators (KPIs) related to HTTP requests, crawler operations, and bandwidth usage.
*   **Runtime Debugging:** Offering tools like Tokio Console for real-time inspection of the application's asynchronous runtime.
*   **Future Extensibility:** Including a placeholder for OpenTelemetry integration, allowing for future distributed tracing capabilities.

## Key Components

The Observability module is composed of three sub-modules:

### 1. Async Logging (`async_logging.rs`)

This component implements a non-blocking, asynchronous logging mechanism using `tokio::sync::mpsc`. It replaces the potentially blocking `RollingFileAppender` from `tracing-appender` for file writes, ensuring that log entries do not stall the main application threads.

*   **`WriterConfig`**: Defines configuration parameters for the async writer, including buffer capacity, flush capacity in bytes, and flush interval.
*   **`AsyncLogWriter`**: The main struct that holds the `mpsc::Sender` for queuing log entries and the configuration.
    *   **`new()`**: Asynchronously creates a new `AsyncLogWriter`, spawning a background task (`run_writer_task`) to handle actual log writing.
    *   **`write()`**: A non-blocking method to send a log entry to the background writer task. It handles buffer overflow by dropping entries and printing a warning.
    *   **`flush()`**: An asynchronous method to flush pending logs. (Note: The current implementation is a stub and does not perform actual flushing).
*   **`run_writer_task()`**: The background task responsible for receiving log entries from the channel and writing them to disk. The current implementation is a simplified stub.
*   **`init_async_logging()`**: An alias function for initializing the `AsyncLogWriter`.

**Note:** The `flush()` method in `AsyncLogWriter` is currently a stub. The `LogGuard` in `logging.rs` relies on `tracing_appender::non_blocking::WorkerGuard` for flushing, which is a separate mechanism.

### 2. JSON Logging (`logging.rs`)

This sub-module handles the initialization and configuration of structured JSON logging, primarily using the `tracing-subscriber` crate.

*   **`LogGuard`**: A RAII guard that ensures all pending log writes are flushed to disk when the guard is dropped. This is crucial for preventing log data loss on application exit. It wraps a `tracing_appender::non_blocking::WorkerGuard`.
*   **`LogFormat`**: An enum to specify log output format (Text or Json).
*   **`init_json_logging()`**: Initializes logging with JSON format, file rotation, and a specified log level. It configures logs to be written to both `stderr` (in text format) and a rotating file (in JSON format) if a `log_dir` is provided.
*   **`init_json_logging_dual()`**: An extended version allowing for `quiet` mode and `no_color` options.
*   **`init_otel_tracing()`**: A stub function for initializing OpenTelemetry tracing. Full integration is deferred.

**Logging Flow:**

1.  `init_json_logging` or `init_json_logging_dual` is called.
2.  An `EnvFilter` is created based on the provided level and quiet settings.
3.  A `tracing_subscriber::registry` is initialized with the filter.
4.  A `fmt::layer` is configured for `stderr` output (text format).
5.  If `log_dir` is provided:
    *   A `RollingFileAppender` is created for daily rotation.
    *   `tracing_appender::non_blocking` is used to create a non-blocking writer and a `WorkerGuard`.
    *   A `fmt::layer` is configured for JSON output to the non-blocking writer.
    *   The `LogGuard` is created, holding the `WorkerGuard`.
6.  The subscriber is initialized with the configured layers.
7.  The `LogGuard` is returned. When dropped, its `WorkerGuard` flushes logs.

### 3. Metrics Collection (`metrics.rs`)

This component provides an in-memory, thread-safe metrics collector for various application activities.

*   **`MetricsCollector`**: A struct that holds atomic counters and `DashMap`s for thread-safe metric aggregation.
    *   **`http_requests`, `http_errors`, `pages_scraped`, `urls_discovered`**: Atomic counters for overall request and crawler statistics.
    *   **`bandwidth_per_domain`, `latency_sum_per_domain`, `requests_per_domain`**: `DashMap`s to store per-domain metrics, allowing concurrent access from multiple threads.
    *   **`start_time`**: Records the initialization time for calculating runtime duration.
    *   **`new()`**: Creates a new `MetricsCollector` and records the start time.
    *   **`record_request()`**: Records an HTTP request, updating total requests, per-domain stats, and error counts if applicable.
    *   **`record_error()`**: Records an HTTP error, incrementing error counts.
    *   **`record_page_scraped()`**: Increments the count of successfully scraped pages.
    *   **`record_url_discovered()`**: Increments the count of discovered URLs.
    *   **`record_bandwidth()`**: Records bandwidth usage for a specific domain.
    *   **`export()`**: Serializes all collected metrics into a `serde_json::Value` for output.

## Integration with the Codebase

*   **Initialization:** Logging is typically initialized once at the start of the application, often in `main.rs` or a dedicated initialization function. The `LogGuard` returned by `init_json_logging` must be kept alive for the duration of the application.
*   **Logging Usage:** Application code uses the `tracing` crate macros (`info!`, `warn!`, `error!`, `debug!`, `trace!`) to emit log messages. These messages are then processed by the configured `tracing-subscriber`.
*   **Metrics Usage:** Instances of `MetricsCollector` are created and passed around (often via `Arc`) to various parts of the application that perform operations to be measured. Methods like `record_request`, `record_page_scraped`, etc., are called to update the metrics.
*   **Tokio Console:** If the `console` feature is enabled, `init_console()` can be called to enable real-time runtime debugging via `tokio-console`. This requires compiling with `RUSTFLAGS="--cfg tokio_unstable"`.

## Architecture Overview

```mermaid
graph TD
    A[Application Code] --> B_tracing["B(tracing"] Macros);
    B --> C{tracing-subscriber};
    C --> D[Stderr Layer (Text)];
    C --> E{File Layer (JSON)};
    E --> F[AsyncLogWriter];
    F --> G[mpsc Channel];
    G --> H_Background["H(Background"] Writer Task);
    H --> I[File I/O];
    F --> J[WorkerGuard];
    J --> K_LogGuard["K(LogGuard)"];
    K -- RAII --> L[Flush on Drop];

    M[Application Code] --> N_MetricsCollector["N(MetricsCollector)"];
    N --> O[Atomic Counters];
    N --> P[DashMap];
    N -- export --> Q_JSON["Q(JSON"] Output);

    R{Console Feature Enabled} --> S_init_console["S(init_console)"];
    S --> T[console-subscriber];

    subgraph Async Logging
        F
        G
        H
        I
        J
        K
        L
    end

    subgraph JSON Logging
        C
        D
        E
        J
        K
        L
    end

    subgraph Metrics Collection
        N
        O
        P
        Q
    end

    subgraph Tokio Console
        R
        S
        T
    end
```