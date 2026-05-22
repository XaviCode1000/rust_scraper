# Terminal User Interface (TUI)

# Terminal User Interface (TUI) Module

The TUI module provides an interactive command-line interface for the scraper application. It allows users to configure scraping settings, select URLs for scraping, and monitor the scraping process in real-time. This module acts as a delivery mechanism, translating application data into a user-friendly visual format and capturing user input.

## Key Components

The TUI module is composed of several sub-modules, each responsible for a specific aspect of the user interface:

*   **`config_form`**: Manages an interactive form for configuring scraper settings.
*   **`error_log_widget`**: Displays a scrollable list of errors encountered during scraping, with color-coded severity.
*   **`event_loop`**: Implements a robust asynchronous event loop that handles terminal input, application events, and periodic ticks for UI updates.
*   **`progress_types`**: Defines the data structures and enums used for tracking scraping progress and application events.
*   **`progress_view`**: Orchestrates the main progress display, integrating the progress widget and error log.
*   **`progress_widget`**: Renders the real-time scraping progress, including progress bars, ETA, and URL status.
*   **`terminal`**: Handles the setup and teardown of the terminal environment for TUI operation, including raw mode and alternate screen management.
*   **`url_selector`**: Provides an interactive interface for selecting specific URLs from a discovered list.

## Core Functionality

### Configuration Form (`config_form`)

The `config_form` module allows users to configure various scraping parameters through an interactive form.

*   **`ConfigFormState`**: Manages the state of the form, including the underlying `ratatui-form` instance, and flags for submission or cancellation.
*   **`build_form()`**: Constructs the configuration form with predefined fields for output settings, discovery options, download preferences, and Obsidian integration. The AI cleaning option is feature-gated by the `ai` feature flag.
*   **`handle_input()`**: Processes keyboard events to update the form's state and detect submission or cancellation.
*   **`render()`**: Renders the form to the terminal.
*   **`data()`**: Returns the current form data as a JSON `Value`.

### Event Handling (`event_loop`)

The `event_loop` module is central to the TUI's reactivity. It uses `tokio::select!` to concurrently listen to multiple event sources:

*   **Terminal Input**: Captures keyboard events using `crossterm`.
*   **Application Events**: Receives events from other parts of the application via an MPSC channel (`AppEventSender`/`AppEventReceiver`).
*   **Periodic Ticks**: Triggers UI updates at a configurable interval.

The `run_event_loop` function takes a configuration, an event receiver, and a handler function. The handler processes each event and returns `true` to continue the loop or `false` to exit. `run_event_loop_until_quit` is a convenience function that exits the loop upon receiving an `AppEvent::Quit`.

### Progress Tracking (`progress_types`, `progress_widget`, `error_log_widget`)

These modules work together to provide real-time feedback on the scraping process:

*   **`progress_types`**: Defines enums like `ScrapeProgress`, `ScrapeStatus`, and `ScrapeError` to represent the state and outcomes of scraping tasks. `ProgressState` aggregates the status of all URLs, including counts, errors, and ETA calculations.
*   **`progress_widget`**: Renders the main progress display. It shows an overall progress bar, ETA, current URL, and a summary of completed/failed tasks. It utilizes `ProgressIcons` for animated feedback.
*   **`error_log_widget`**: Displays detailed errors from `ProgressState.errors`. It supports color-coding based on `ErrorType` and provides scrolling capabilities.

The `run_progress_view` function orchestrates the display of this information, setting up the terminal, managing the event loop for progress updates, and rendering the UI.

### URL Selection (`url_selector`)

The `url_selector` module enables users to choose specific URLs from a list of discovered ones before scraping begins.

*   **`UrlSelectorState`**: Manages the state of the selection process, including the list of URLs, selection status, cursor position, and scroll offset. It's designed to be testable independently of the UI.
*   **`UrlSelector`**: Handles the rendering of the URL selection interface, displaying checkboxes, cursor indicators, and status messages.
*   **`run_selector()`**: The main function that sets up the terminal, runs the event loop for user input, and renders the `UrlSelector`. It returns the list of selected URLs or an error if the user quits without selection.

### Terminal Management (`terminal`)

The `terminal` module provides essential functions for initializing and cleaning up the terminal environment:

*   **`setup_terminal()`**: Enables raw mode, enters the alternate screen, hides the cursor, and sets up a panic hook to ensure the terminal is restored even if the application panics.
*   **`restore_terminal()`**: Disables raw mode, leaves the alternate screen, and shows the cursor, returning the terminal to its normal state.

## Integration with the Application

*   **Configuration**: The `run_config_tui` function (likely in `src/main.rs` or a similar entry point) uses `ConfigFormState` to present the configuration options to the user. The resulting configuration is then used by the application core.
*   **URL Selection**: Before scraping begins, `run_selector` is called with the discovered URLs. The returned list of selected URLs dictates which targets the scraper will process.
*   **Progress Monitoring**: The application's core scraping logic sends `ScrapeProgress` events to a `mpsc::Sender` channel. `run_progress_view` receives these events and updates the `ProgressState`, which is then rendered by the `ProgressWidget` and `ErrorLogWidget`.
*   **Event Loop**: The `event_loop` module provides the foundation for the TUI's responsiveness, integrating terminal input with application-level events.

## Architecture Overview

```mermaid
graph TD
    subgraph TUI Module
        A[config_form] --> B_ratatui-form["B(ratatui-form)"]
        C[error_log_widget]
        D[event_loop] --> E_tokio__select["E(tokio::select"]!)
        D --> F_crossterm__event["F(crossterm::event)"]
        D --> G_tokio__sync__mpsc["G(tokio::sync::mpsc)"]
        H[progress_types]
        I[progress_view] --> J_Terminal["J(Terminal)"]
        I --> K_ProgressWidget["K(ProgressWidget)"]
        I --> C
        I --> D
        I --> H
        K --> H
        K --> L_ProgressIcons["L(ProgressIcons)"]
        C --> H
        M[progress_widget] --> H
        M --> L
        N[terminal] --> O_crossterm["O(crossterm)"]
        P[url_selector] --> J
        P --> Q_ratatui["Q(ratatui)"]
        P --> H
        R[TuiError]
    end

    subgraph Application Layer
        S[Scraper Core] --> G
        S --> H
        T[URL Discovery] --> P
        U[Configuration] --> A
        V[Main Entrypoint] --> I
        V --> P
        V --> A
        V --> N
    end

    J --> N
    G --> I
    G --> D

    style TUI Module fill:#f9f,stroke:#333,stroke-width:2px
```

This diagram illustrates the TUI module's internal structure and its primary connections to the rest of the application. The `event_loop` is central to handling asynchronous events, while `progress_types` defines the shared data structures. Specific widgets like `config_form`, `url_selector`, `progress_widget`, and `error_log_widget` handle distinct UI responsibilities, all managed by functions like `run_config_tui`, `run_selector`, and `run_progress_view`. The `terminal` module provides the low-level interface to the terminal.