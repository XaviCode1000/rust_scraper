# Documentation and Examples — typestate-pattern

# Typestate Pattern for Compile-Time Validation

This module introduces the Typestate pattern to enforce compile-time validation of `DocumentChunk` objects before they can be exported. This approach shifts validation from runtime checks to compile-time guarantees, preventing invalid data from reaching export mechanisms.

## Motivation

Previously, `DocumentChunk` objects could be created without explicit validation, leading to potential runtime errors when these unvalidated chunks were passed to exporters like `FileExporter`. The Typestate pattern addresses this by ensuring that a `DocumentChunk` must transition through a `Validated` state before it can be exported.

## Design and Architecture

The core of this pattern relies on Rust's private state pattern using marker types and `PhantomData`.

### Key Components

*   **State Marker Structs**: Private, empty structs (`Draft`, `Validated`, `Exported`) represent the possible states of a `DocumentChunk`. These are not intended to be constructed directly by users.
*   **`DocumentChunk<S>`**: The main entity struct. It now includes a generic type parameter `S` representing its current state, defaulting to `Draft`. `PhantomData<S>` is used to associate the state with the struct without storing any data.
*   **State Transitions**: Methods like `.validate()` and `.export()` are implemented on specific state variants of `DocumentChunk`.
    *   `DocumentChunk<Draft>` has a `.validate()` method that consumes the `Draft` chunk and returns a `DocumentChunk<Validated>`.
    *   `DocumentChunk<Validated>` has an `.export()` method, which is the only state from which export is permitted.
*   **Backward Compatibility**: A type alias `type DocumentChunk = DocumentChunk<Draft>;` is provided to maintain compatibility with existing code that does not explicitly specify the state.

### Architectural Decisions

*   **State Representation**: Private, empty structs are used as marker types for states. This is idiomatic in Rust for typestate and avoids runtime overhead associated with enums or trait objects.
*   **`DocumentChunk` Location**: The `DocumentChunk` struct and its state-related logic remain in `src/domain/entities.rs` to minimize refactoring.
*   **Compile-Time vs. Trait Bounds**: The `Exporter` trait is updated to accept `DocumentChunk<Validated>` via a trait bound (`where S: Validated`). This directly enforces the validation requirement at compile time.
*   **API Backward Compatibility**: A type alias allows existing code to continue using `DocumentChunk` without modification, while new code can explicitly use `DocumentChunk<Draft>` or `DocumentChunk<Validated>`.

## Data Flow

The typical data flow for a `DocumentChunk` is as follows:

```mermaid
graph TD
    A[ScrapedContent] --> B_DocumentChunk["B(DocumentChunk"]<Draft>);
    B --> C{Call .validate()};
    C --> D_DocumentChunk["D(DocumentChunk"]<Validated>);
    D --> E{Call Exporter.export()};
    E --> F[Exported Result];
```

A `ScrapedContent` is initially converted into a `DocumentChunk<Draft>`. This `Draft` chunk must then be explicitly validated using the `.validate()` method, transitioning it to `DocumentChunk<Validated>`. Only the `Validated` chunk can be passed to an `Exporter`'s `.export()` method.

## Integration with Other Modules

*   **`src/domain/entities.rs`**: This is the primary location for the `DocumentChunk` struct, state marker definitions, and transition methods.
*   **`src/domain/exporter.rs`**: The `Exporter` trait is modified to require `DocumentChunk<Validated>` as input for its `export` method.
*   **`src/infrastructure/export/file_exporter.rs`**: The concrete implementation of the `Exporter` trait is updated to align with the modified `Exporter` trait.
*   **`src/export_flow.rs` and `src/export_factory.rs`**: These modules, which consume the `DocumentChunk` for export, are updated to call `.validate()` before calling `.export()`.

## Testing Strategy

*   **Unit Tests**: Focus on verifying the state transitions and ensuring that `DocumentChunk<Draft>` cannot be compiled into the export process. Tests are located in `entities.rs`.
*   **Integration Tests**: Verify the correct functioning of `FileExporter` with `DocumentChunk<Validated>` instances. These are located in `file_exporter.rs`.

## Migration and Rollout

The migration is planned in phases:

1.  **Phase 1**: Introduce state marker structs and modify `DocumentChunk` with `PhantomData`. Add the backward-compatible type alias.
2.  **Phase 2**: Implement the `.validate()` and `.export()` methods, including basic validation logic within `.validate()`.
3.  **Phase 3**: Update the `Exporter` trait and its implementations to require `DocumentChunk<Validated>`.
4.  **Phase 4**: Modify consumer modules (`export_flow.rs`, `export_factory.rs`) to use the new validation and export flow.
5.  **Phase 5**: Comprehensive testing, including compilation checks, clippy, and existing test suites (with necessary updates).
6.  **Phase 6**: Code cleanup, ensuring no unused type warnings and verifying the backward-compatible alias.

A feature flag can be used to enable/disable this functionality, and a simple rollback involves disabling the flag and reverting code changes.

## Open Questions

*   **Specific Validation Logic**: The exact validation checks performed by `.validate()` (e.g., content length, URL format, required fields) need to be clearly defined.
*   **`embeddings` Field State**: It is unclear if the `embeddings` field within `DocumentChunk` also requires state management, although this is considered out of scope for the current proposal.