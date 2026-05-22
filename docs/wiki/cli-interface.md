# CLI Interface

# CLI Interface Module

The CLI Interface module is the primary entry point for interacting with the `rust_scraper` binary. It handles parsing command-line arguments, orchestrating the scraping workflow, and managing user feedback through logging and progress indicators.

## Purpose

This module's main responsibilities are:

1.  **Argument Parsing**: Define and parse all command-line arguments using `clap`. This includes configuration options, subcommands, and feature flags.
2.  **Workflow Orchestration**: Coordinate the high-level execution flow of the scraper, including URL discovery, content scraping, and data export.
3.  **Configuration Management**: Load default configurations from files and merge them with command-line arguments.
4.  **User Feedback**: Provide informative output to the user through logging, progress bars, and summary reports.
5.  **Error Handling**: Gracefully handle errors and exit with appropriate status codes using `sysexits`.

## Key Components

### `Args` Struct (`src/cli/args.rs`)

This struct, defined using `clap`'s derive macros, represents all possible command-line arguments. It's organized into logical sections like "Target", "Output", "Discovery", "Behavior", etc., making it easier to understand and use.

*   **Target**: Specifies the starting URL and CSS selector.
*   **Output**: Defines output directories and formats (individual file output and RAG export).
*   **Obsidian**: Options for integrating with Obsidian, such as wiki-link conversion and asset path rewriting.
*   **Discovery**: Controls how URLs are found, including delay, concurrency, and sitemap usage.
*   **Behavior**: Modifies the scraping process, like enabling resume mode, downloading assets, or interactive TUI.
*   **Display**: Controls verbosity and output formatting.
*   **Crawler Settings**: Parameters for the web crawler, like depth and timeouts.
*   **HTTP Client Settings**: Configuration for the underlying HTTP client, including retries and backoff.
*   **Download Settings**: Limits for asset downloads.
*   **AI Settings**: Feature-gated options for AI-powered content cleaning.
*   **Sitemap Settings**: Specific configurations for sitemap processing.

### `Commands` Enum (`src/cli/args.rs`)

This enum defines subcommands for the `rust_scraper` binary. Currently, it only supports `Completions` for generating shell completion scripts.

### `Orchestrator` (`src/cli/orchestrator.rs`)

The `run` function within this module is the central orchestrator. It takes the parsed `Args` and orchestrates the entire scraping process:

1.  **Argument Validation**: Parses the target URL and checks for basic validity.
2.  **Configuration Building**: Creates `CrawlerConfig` and `ScraperConfig` from the `Args`.
3.  **URL Discovery**: Calls `discover_urls` to find all relevant URLs.
4.  **Scraping**: Invokes `scrape_urls` to fetch and process the content of discovered URLs.
5.  **Exporting**: Uses `run_export` to handle the RAG export (standard or AI-cleaned).
6.  **File Saving**: Calls `save_files` to save individual content files (e.g., Markdown).

### `Scrape Flow` (`src/cli/scrape_flow.rs`)

This module contains the logic for the scraping phase:

*   **`apply_resume_mode`**: Filters discovered URLs based on a `StateStore` if resume mode is enabled.
*   **`scrape_urls`**: Iterates through the URLs to be scraped, creates an HTTP client, and calls the core scraping logic for each URL. It also handles progress reporting via an MPSC channel.

### `Export Flow` (`src/cli/export_flow.rs`)

This module manages the export of scraped data:

*   **`run_export`**: Acts as a dispatcher, calling either `run_standard_export` or `run_ai_export` based on the `--clean-ai` flag and feature availability.
*   **`run_standard_export`**: Uses the `export_factory` to process results into the specified export format (e.g., JSONL).
*   **`run_ai_export`**: (Feature-gated) Initializes and uses the `SemanticCleaner` to process content before exporting.
*   **`save_files`**: Handles saving individual files in formats like Markdown or Text, with Obsidian integration.

### `URL Discovery` (`src/cli/url_discovery.rs`)

This module is responsible for finding URLs:

*   **`discover_urls`**: Orchestrates the URL discovery process, showing a progress spinner if not in quiet mode.
*   **`select_urls`**: Determines which URLs will actually be scraped, based on interactive selection (TUI), quick-save mode, or headless operation.

### `Configuration` (`src/cli/config.rs`)

*   **`ConfigDefaults`**: Defines default configuration values that can be loaded from a TOML file.
*   **`load`**: Reads and parses the TOML configuration file.
*   **`is_no_color` / `should_emit_emoji`**: Helper functions to manage terminal output formatting based on the `NO_COLOR` environment variable.
*   **`init_logging` / `init_logging_dual`**: Configures the `tracing` logging subscriber.

### `Preflight` (`src/cli/preflight.rs`)

This module performs checks before the main scraping begins:

*   **`apply_config_defaults`**: Merges default configurations from a TOML file into the `Args` struct, respecting CLI argument precedence.
*   **`apply_tui_config`**: Merges configuration values obtained from an interactive TUI session.
*   **`preflight_check`**: Performs an HTTP HEAD (or GET fallback) request to verify basic network connectivity to the target URL.
*   **`icon`**: Helper to conditionally display emojis or ASCII characters.

### `Error Handling` (`src/cli/error.rs`)

*   **`CliError`**: An enum defining categorized CLI errors with user-friendly messages and suggestions.
*   **`CliExit`**: An enum mapping errors to `sysexits` exit codes, implementing the `Termination` trait for proper program termination.

### `Summary` (`src/cli/summary.rs`)

*   **`ScrapeSummary`**: A struct to hold statistics about a scraping run (URLs discovered, scraped, failed, duration, etc.).
*   **`display`**: Formats the summary for console output, with options for emoji or ASCII display.

### `Completions` (`src/cli/completions.rs`)

*   **`generate_completions`**: Uses `clap_complete` to generate shell completion scripts for various shells.

### `Wizard` (`src/cli/wizard.rs`)

*   **`TtyDetector`**: Traits and structs for detecting terminal capabilities, used for interactive prompts.
*   **`WizardPrompt`**: Enum defining different types of interactive prompts (confirmation, text input, selection, etc.).

## Execution Flow

The typical execution flow for the `rust_scraper` binary is as follows:

1.  **`main` function**:
    *   Initializes logging (`init_logging_dual`).
    *   Parses command-line arguments using `Args::parse()`.
    *   If the `completions` subcommand is used, calls `handle_completions`.
    *   Loads configuration defaults (`ConfigDefaults::load`).
    *   Applies configuration defaults (`apply_config_defaults`).
    *   If `config_tui` is enabled, runs the TUI configuration wizard and applies its settings (`apply_tui_config`).
    *   Performs a preflight network check (`preflight_check`).
    *   Calls the main orchestrator `run` function.
    *   Reports any `CliExit` status.

2.  **`orchestrator::run`**:
    *   Validates the target URL.
    *   Builds `CrawlerConfig` and `ScraperConfig`.
    *   Calls `discover_urls` to get a list of URLs to potentially scrape.
    *   Calls `select_urls` to filter the discovered URLs based on interactive mode, quick-save, etc.
    *   Calls `apply_resume_mode` to filter URLs if resume mode is active.
    *   Calls `scrape_urls` to perform the actual web scraping.
    *   Saves individual files using `save_files`.
    *   Calls `run_export` to handle the RAG export.
    *   Returns the final `CliExit` status.

```mermaid
graph TD
    A[main.rs] --> B_Args__parse["B(Args::parse)"];
    B --> C{Subcommand?};
    C -- Completions --> D_handle_completions["D(handle_completions)"];
    C -- No Subcommand --> E_init_logging["E(init_logging)"];
    E --> F_ConfigDefaults__load["F(ConfigDefaults::load)"];
    F --> G_apply_config_defaults["G(apply_config_defaults)"];
    G --> H{Config TUI?};
    H -- Yes --> I_run["I(run"] config TUI);
    I --> J_apply_tui_config["J(apply_tui_config)"];
    J --> K_preflight_check["K(preflight_check)"];
    H -- No --> K;
    K --> L_orchestrator__run["L(orchestrator::run)"];

    L --> M_discover_urls["M(discover_urls)"];
    M --> N_select_urls["N(select_urls)"];
    N --> O_apply_resume_mode["O(apply_resume_mode)"];
    O --> P_scrape_urls["P(scrape_urls)"];
    P --> Q_save_files["Q(save_files)"];
    Q --> R_run_export["R(run_export)"];
    R --> S_CliExit["S(CliExit)"];

    D --> S;
    L --> S;

    subgraph CLI Interface
        B; C; D; E; F; G; H; I; J; K; L; M; N; O; P; Q; R; S;
    end
```

## Integration with Other Modules

*   **`clap`**: Used extensively for argument parsing in `src/cli/args.rs`.
*   **`tracing`**: Used for logging throughout the CLI module and integrated via `init_logging` in `src/cli/config.rs`.
*   **`url`**: Used for parsing and manipulating URLs, especially in `orchestrator.rs` and `url_discovery.rs`.
*   **`indicatif`**: Used for progress bars in `url_discovery.rs` and `scrape_flow.rs`.
*   **`thiserror` / `anyhow`**: Used for defining and propagating errors in `src/cli/error.rs`.
*   **`sysexits`**: Used via `CliExit` for standard program exit codes.
*   **`wreq`**: Used in `preflight.rs` for network connectivity checks.
*   **`clap_complete`**: Used in `completions.rs` for shell completion generation.
*   **Application Layer**: The CLI module acts as a thin layer, calling functions from the `application` layer (e.g., `discover_urls_for_tui`, `scrape_single_url_for_tui`, `export_factory`) to perform the core logic.
*   **Infrastructure Layer**: Interacts with infrastructure components like `StateStore`, `SemanticCleaner`, and HTTP clients.
*   **Domain Layer**: Consumes and produces domain entities like `ScrapedContent`.