# Documentation and Examples — docs

# Documentation and Examples — docs

This module serves as the central hub for all documentation and example-related content for the rust_scraper project. It encompasses various markdown files that detail the project's architecture, features, usage, and development guidelines.

## Purpose

The primary purpose of the `docs` module is to provide clear, comprehensive, and up-to-date information for developers, users, and contributors. This includes:

*   **Understanding the Project:** Explaining the overall architecture and design principles.
*   **Feature Documentation:** Detailing specific features like AI-powered cleaning, RAG export, and Obsidian integration.
*   **Usage Guides:** Providing instructions on how to install, build, and use the CLI tool.
*   **Development Guidelines:** Outlining best practices, testing strategies, and contribution workflows.
*   **Changelog and Audits:** Tracking project history, bug fixes, and quality assessments.

## Key Components

The `docs` module is composed of several markdown files, each focusing on a specific aspect of the project:

*   **`AI-SEMANTIC-CLEANING.md`**: Details the AI-powered semantic content extraction feature, including its architecture, installation, usage, model information, performance benchmarks, and bug fixes.
*   **`ARCHITECTURE.md`**: Provides a high-level overview of the project's Clean Architecture, detailing the layers (Domain, Application, Infrastructure, Adapters), module structure, data flow, and key design decisions.
*   **`AUDITORIA.md`**: Contains audit reports, summarizing feature testing, bug findings, quality metrics, and recommendations for releases.
*   **`CHANGES.md`**: A comprehensive changelog documenting the project's version history, key milestones, release notes, closed issues, merged pull requests, and contributor statistics.
*   **`CLI.md`**: Serves as the reference guide for the Command Line Interface (CLI) tool, detailing all available arguments, options, feature flags, environment variables, and usage examples.
*   **`OBSIDIAN.md`**: Specifically documents the integration with Obsidian, explaining features like vault auto-detection, export options, and quick-save functionality.

## Integration with the Project

The `docs` module is not directly integrated into the runtime of the `rust_scraper` application. Instead, it serves as external documentation that is:

1.  **Included in the Repository:** All markdown files are part of the project's source code repository, ensuring documentation stays synchronized with the code.
2.  **Accessible via `README.md`:** The main `README.md` file typically links to these detailed documentation files, guiding users to the relevant information.
3.  **Used for Development:** Developers refer to these documents for understanding the codebase, contributing new features, and ensuring adherence to project standards.

## Usage within the Project

While not a runtime component, the `docs` module is crucial for:

*   **Onboarding New Developers:** Providing a clear entry point to understand the project's structure and features.
*   **User Support:** Offering detailed guides for using the CLI tool and its various options.
*   **Release Management:** Documenting changes and ensuring transparency through the changelog and audit reports.
*   **Feature Explanation:** Detailing complex features like AI integration and Obsidian export.

## Example: `AI-SEMANTIC-CLEANING.md`

This document is a prime example of the detailed technical documentation provided within the `docs` module. It covers:

*   **Feature Overview:** Explains the AI-powered semantic cleaning using local SLMs.
*   **Architecture:** Details the RAG pipeline and module structure with diagrams.
*   **Installation:** Lists requirements and build instructions with feature flags.
*   **Usage:** Provides CLI examples for basic cleaning, RAG export, and available options.
*   **Model Information:** Describes the default model, caching, and manual download.
*   **Performance:** Presents benchmarks and hardware optimizations.
*   **Bug Fixes:** Documents critical fixes with code examples and impact analysis.
*   **Testing:** Outlines how to run AI-specific tests.
*   **Rust-Skills Applied:** Lists the best practices followed.
*   **Programmatic Usage:** Shows library API examples.
*   **Troubleshooting:** Offers solutions for common issues.
*   **Migration Guide:** Explains how to transition to newer versions.
*   **Future Enhancements:** Lists planned and considered improvements.
*   **References:** Links to relevant issues, PRs, and external resources.
*   **Verification Log:** Confirms the accuracy of the documentation against the codebase.

This level of detail is replicated across other documentation files, ensuring comprehensive coverage of the `rust_scraper` project.