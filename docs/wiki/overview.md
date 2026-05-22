# rust_scraper — Wiki

# 🕷️ Rust Scraper

Welcome to the `rust_scraper` repository! This project provides a production-ready web scraping tool built with Clean Architecture principles. It's designed to extract, clean, and export web content efficiently, offering features like an interactive TUI selector, sitemap support, and AI-powered content cleaning.

## What it Does

Rust Scraper is a command-line utility that allows you to:

*   **Interactively explore and select URLs** from a website before scraping.
*   **Extract relevant content**, intelligently ignoring boilerplate like menus and ads.
*   **Export scraped data** into various formats, including Markdown, JSON, JSONL (ideal for RAG), and Vector embeddings.
*   **Integrate seamlessly with Obsidian**, saving articles directly into your vault with wiki-links and metadata.
*   **Download associated assets** like images and documents (e.g., PDFs).
*   **Leverage sitemaps** to discover all pages on a site automatically.

## Architecture Overview

The project adheres to Clean Architecture, separating concerns into distinct layers. The core business logic resides in the `domain` and `application` layers, while infrastructure concerns like HTTP clients, file system access, and the CLI are handled in the `infrastructure` and `cli` layers, respectively.

```mermaid
graph TD
    CLI[CLI Interface] --> App[Core Application Logic]
    App --> Domain[Domain Models and Entities]
    App --> Crawl[Crawling Engine]
    App --> Extract[Data Extraction and Detection]
    App --> Export[Output and Exporting]
    App --> Config[Configuration]
    App --> Obs[Observability]
    Crawl --> HTTP[HTTP Client]
    Extract --> HTTP
    Extract --> Detect[Infrastructure Adapters]
    Export --> Detect
    CLI --> TUI[Terminal User Interface (TUI)]
    CLI --> Config
    App --> AI[AI and Machine Learning]
    Extract --> AI
    Export --> Obsidian[Obsidian Integration]
    WAF[WAF (Web Application Firewall)] --> HTTP
```

### Key Modules

*   **[CLI Interface](cli-interface.md)**: The entry point for users, handling argument parsing and orchestrating the scraping workflow.
*   **[Core Application Logic](core-application-logic.md)**: Orchestrates high-level use cases like crawling and scraping, managing services and repositories.
*   **[Domain Models and Entities](domain-models-and-entities.md)**: Defines the fundamental business types and data structures, independent of any infrastructure.
*   **[Crawling Engine](crawling-engine.md)**: Manages the discovery and fetching of web pages, handling concurrency and URL queues.
*   **[HTTP Client](http-client.md)**: Provides a robust HTTP client with features like retries, user-agent rotation, and WAF detection.
*   **[Data Extraction and Detection](data-extraction-and-detection.md)**: Focuses on extracting relevant data and identifying assets like images and documents.
*   **[Text Conversion](text-conversion.md)**: Transforms raw HTML into clean Markdown, preserving structure and removing boilerplate.
*   **[Output and Exporting](output-and-exporting.md)**: Implements concrete logic for exporting data into various formats.
*   **[Terminal User Interface (TUI)](terminal-user-interface-tui.md)**: Provides an interactive command-line experience for configuration and monitoring.
*   **[AI and Machine Learning](ai-and-machine-learning.md)**: Powers advanced features like semantic chunking and text embedding generation for RAG.
*   **[Obsidian Integration](obsidian-integration.md)**: Facilitates saving scraped content directly into Obsidian vaults.
*   **[Configuration](configuration.md)**: Manages application settings and validation.
*   **[Observability](observability.md)**: Handles logging, metrics, and tracing for monitoring.
*   **[WAF (Web Application Firewall)](waf-web-application-firewall.md)**: Detects and mitigates against WAFs.
*   **[Infrastructure Adapters](infrastructure-adapters.md)**: Contains infrastructure-facing implementations like asset downloading and URL path translation.

## Installation

The recommended way to install is using Cargo:

```bash
cd rust-scraper
cargo install --path .
```

This will build and install the `rust_scraper` binary on your system. For more detailed installation instructions and feature flags, please refer to the [README](README.md).

## Key End-to-End Flows

1.  **Basic Scraping**: The **[CLI Interface](cli-interface.md)** parses arguments, the **[Core Application Logic](core-application-logic.md)** orchestrates a scraping task, the **[Crawling Engine](crawling-engine.md)** discovers URLs, the **[HTTP Client](http-client.md)** fetches pages, **[Data Extraction and Detection](data-extraction-and-detection.md)** pulls out content, **[Text Conversion](text-conversion.md)** cleans it, and **[Output and Exporting](output-and-exporting.md)** saves it.
2.  **Interactive Scraping**: The **[CLI Interface](cli-interface.md)** launches the **[Terminal User Interface (TUI)](terminal-user-interface-tui.md)**. The TUI interacts with the **[Core Application Logic](core-application-logic.md)** to display site structure and allow user selection. Once selections are made, the workflow proceeds as in basic scraping.
3.  **Asset Downloading**: During scraping, if enabled, the **[Data Extraction and Detection](data-extraction-and-detection.md)** module identifies assets. The **[Infrastructure Adapters](infrastructure-adapters.md)**, specifically the downloader, uses the **[HTTP Client](http-client.md)** to fetch these assets, saving them to disk.
4.  **Obsidian Export**: After scraping and content processing, the **[Output and Exporting](output-and-exporting.md)** module utilizes the **[Obsidian Integration](obsidian-integration.md)** module to format and save the content, including metadata and wiki-links, into the user's Obsidian vault.
5.  **AI-Powered Cleaning**: For advanced cleaning, the **[Core Application Logic](core-application-logic.md)** can invoke services from the **[AI and Machine Learning](ai-and-machine-learning.md)** module. This might involve semantic chunking or embedding generation, influencing the final extracted content or its export format.

This overview provides a starting point for understanding the `rust_scraper` project. Please explore the individual module documentation for more in-depth information.