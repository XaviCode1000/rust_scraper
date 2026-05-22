# Documentation and Examples

# Documentation and Examples

This module serves as the central repository for all documentation, research, and example code related to the `rust_scraper` project. Its purpose is to provide comprehensive guidance for users, developers, and contributors, ensuring clarity on the project's functionality, architecture, and development practices.

The sub-modules within this group work together to offer a complete picture of the project:

*   **`README.md`**: Acts as the primary entry point, summarizing the project's purpose, installation, and common usage, while linking to more detailed documentation.
*   **`AGENTS.md`**: Provides specific instructions for AI agents interacting with the Rust Scraper, detailing workflows and commands.
*   **`DEVELOPMENT.md`**: Outlines the development workflow, tooling, and best practices for contributing to the project, including setup and testing guidelines.
*   **`STATE.md`**: Offers a snapshot of the project's current status, including test results and recent fixes, serving as a quick health check.
*   **`docs/`**: A directory containing a broad range of documentation, explaining architecture, features, and usage guides.
*   **`research/`**: Houses research findings and proposals, such as specifications for exporting data compatible with knowledge management systems like Obsidian.
*   **`examples/`**: Contains standalone code examples demonstrating how to use and test core components, like the MCP server in different transports.
*   **`mpsc-channel-refactor`**: Explores a specific refactoring proposal to improve concurrency by moving from a mutex-based collector to an mpsc channel.
*   **`typetstate-pattern`**: Details the implementation of the Typestate pattern for compile-time validation of `DocumentChunk` objects before export.
*   **`specs/`**: Contains product and implementation specifications, such as `async-reactive-tui.md`, which defines the async progress UI for the TUI.

Together, these sub-modules ensure that the project's functionality, development processes, and future directions are well-documented and accessible.