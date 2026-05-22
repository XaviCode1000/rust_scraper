# Output and Exporting

# Output and Exporting Module

This module provides the concrete implementations for exporting scraped data into various formats. It acts as the infrastructure layer for the `Exporter` trait defined in the domain, allowing the application to persist scraped content in a structured and usable way.

The primary goals of this module are:

*   **Format Variety**: Support for multiple output formats including JSON Lines (JSONL), structured JSON, Markdown, and plain Text.
*   **Efficiency**: Optimized for large datasets, especially with JSONL and Vector exporters, supporting streaming and append modes.
*   **State Management**: Integration with `StateStore` to manage export progress and enable resume functionality.
*   **Data Integrity**: Ensuring exported data is correctly formatted and, where applicable, includes necessary metadata.

## Components

The module is composed of several distinct exporters, each tailored for a specific output format or use case:

### `FileExporter`

The `FileExporter` handles exporting `DocumentChunkValidated` entities to the local file system. It supports Markdown, Text, and JSON formats, organizing output into a directory structure based on the domain of the URL.

*   **Functionality**:
    *   Creates directories based on the URL's domain.
    *   Generates filenames derived from the URL, sanitizing special characters.
    *   Formats content for Markdown (with YAML frontmatter), Text (structured with headers), and JSON (pretty-printed).
    *   Supports `ExportFormat::Jsonl` for appending JSON objects to a file, one per line.
    *   Supports `ExportFormat::Vector` by delegating to `save_json`.
    *   Defaults to `ExportFormat::Auto` by using `save_json`.
*   **Key Methods**:
    *   `new(config: ExporterConfig)`: Creates an exporter with a given configuration.
    *   `new_with_path(output_dir: PathBuf, format: OutputFormat, filename: impl Into<String>)`: Convenience constructor.
    *   `export(document: DocumentChunkValidated)`: Exports a single document based on the exporter's configured format.
    *   `export_batch(documents: &[DocumentChunkValidated])`: Exports a slice of documents.
    *   `save_md`, `save_txt`, `save_json`: Internal methods for specific format generation.
    *   `output_path(doc: &DocumentChunkValidated, ext: &str)`: Generates the output file path.

### `JsonlExporter`

The `JsonlExporter` is specifically designed for exporting data in the JSON Lines (JSONL) format. This format is highly efficient for large datasets as it writes one JSON object per line, allowing for streaming and easy parsing with tools like `jq` or `pandas`.

*   **Functionality**:
    *   Appends JSON objects to a file, with each object on a new line.
    *   Optimized for streaming writes, avoiding large in-memory buffers.
    *   Acquires an exclusive file lock (`.jsonl.lock`) to prevent concurrent writes.
    *   Handles directory creation and file opening in append or create/truncate modes.
*   **Key Methods**:
    *   `new(config: ExporterConfig)`: Creates an exporter with a given configuration.
    *   `new_with_path(output_dir: PathBuf, filename: impl Into<String>)`: Convenience constructor.
    *   `export(document: DocumentChunkValidated)`: Exports a single document as a JSON line.
    *   `export_batch(documents: &[DocumentChunkValidated])`: Exports multiple documents as JSON lines.
    *   `writer()`: Manages file handle, locking, and directory creation.
    *   `serialize_line(doc: &DocumentChunkValidated)`: Serializes a single document to a JSON string.

### `StateStore`

The `StateStore` is crucial for implementing resume functionality in the export process. It persists the state of exported items (specifically, processed URLs) to disk, typically in the user's cache directory.

*   **Functionality**:
    *   Manages state for a specific domain.
    *   Stores processed URLs in a JSON file (`<domain>.json`).
    *   Uses file locking (`.json.lock`) to ensure atomic read/write operations.
    *   Supports loading existing state or creating a new one if none exists (`load_or_default`).
    *   Provides methods to mark URLs as processed and check their status.
    *   Handles directory creation for the cache.
*   **Key Methods**:
    *   `new(domain: &str)`: Creates a `StateStore` for a given domain.
    *   `set_cache_dir(cache_dir: PathBuf)`: Allows specifying a custom cache directory.
    *   `get_state_path()`: Returns the path to the state file.
    *   `load()`: Loads the state from disk.
    *   `save(state: &ExportState)`: Saves the current state to disk.
    *   `mark_processed(state: &mut ExportState, url: &str)`: Marks a URL as processed within the `ExportState`.
    *   `is_processed(state: &ExportState, url: &str)`: Checks if a URL has been processed.
    *   `load_or_default()`: Loads state or returns a new, empty state.

### `VectorExporter`

The `VectorExporter` is designed for exporting data, particularly embeddings, for use in vector databases or similarity search applications. It outputs data in a structured JSON format with a metadata header.

*   **Functionality**:
    *   Exports documents as a JSON array within a JSON object.
    *   Includes a metadata header with `format_version`, `model_name`, `dimensions`, `total_documents`, and `created_at`.
    *   Supports embedding dimensions validation: it records the dimension of the first embedding encountered and rejects subsequent documents with different embedding dimensions, serializing them without embeddings.
    *   Rejects embeddings containing `NaN` or `Infinity` values.
    *   Supports append mode, where it intelligently truncates the file at the closing `]` of the `documents` array to allow for seamless appending.
    *   Acquires an exclusive file lock.
*   **Key Methods**:
    *   `new(config: ExporterConfig)`: Creates an exporter with a given configuration.
    *   `new_with_path(config: ExporterConfig, output_dir: impl Into<PathBuf>)`: Convenience constructor.
    *   `export(document: DocumentChunkValidated)`: Exports a single document.
    *   `export_batch(documents: &[DocumentChunkValidated])`: Exports a slice of documents.
    *   `writer()`: Manages file handle, locking, directory creation, and truncation for append mode.
    *   `write_metadata_header(...)`: Writes or updates the metadata header.
    *   `serialize_document(doc: &DocumentChunkValidated)`: Serializes a document, performing embedding dimension checks.
    *   `close_json(...)`: Writes the closing `]` and `}` to complete the JSON structure.
    *   `cosine_similarity(a: &[f32], b: &[f32])`: A utility function (defined within this module) to calculate cosine similarity between two vectors.

## Architecture and Integration

This module resides in the `infrastructure` layer, depending on the `domain` layer (specifically `domain::exporter` and `domain::entities`).

```mermaid
graph TD
    subgraph Infrastructure
        A[FileExporter] --> B_Exporter["B(Exporter"] Trait);
        A --> C_ExporterConfig["C(ExporterConfig)"];
        A --> D_DocumentChunkValidated["D(DocumentChunkValidated)"];
        E[JsonlExporter] --> B;
        E --> C;
        E --> D;
        F[StateStore] --> G_ExportState["G(ExportState)"];
        F --> H_ScraperError["H(ScraperError)"];
        I[VectorExporter] --> B;
        I --> C;
        I --> D;
        I --> J_cosine_similarity["J(cosine_similarity)"];
    end

    subgraph Domain
        B; C; D; G; H; J;
    end

    subgraph Application
        K[ExportFactory] --> B;
        K --> F;
        L[ScrapeFlow] --> F;
    end

    K --> A;
    K --> E;
    K --> I;
    L --> M[StateStore];
```

*   **`Exporter` Trait**: The `FileExporter`, `JsonlExporter`, and `VectorExporter` all implement the `Exporter` trait defined in `src/domain/exporter.rs`. This allows for dependency injection and polymorphism, where the application layer can use any `Exporter` implementation interchangeably.
*   **`ExporterConfig`**: This struct, defined in the domain, is used by all exporters to configure their behavior (output directory, format, filename, append mode).
*   **`DocumentChunkValidated`**: The core data structure being exported, defined in `src/domain/entities.rs`.
*   **`ExportState`**: Used by `StateStore` to track processed items.
*   **`StateStore` Integration**: The `StateStore` is used by the application layer (e.g., `ExportFactory`, `ScrapeFlow`) to manage resume capabilities. It's instantiated per domain and used to check if a URL has already been processed before attempting to export it.
*   **Error Handling**: Exporters return `ExportResult<()>`, which is an alias for `Result<(), ExporterError>`. `ExporterError` is an enum defined in the domain, covering various export-related issues like I/O errors, serialization problems, and dimension mismatches. `StateStore` uses `ScraperError` for its operations.

## Usage Patterns

### Basic Export

To export a single document:

```rust
use rust_scraper::domain::entities::DocumentChunkValidated;
use rust_scraper::domain::exporter::Exporter;
use rust_scraper::infrastructure::export::{FileExporter, ExporterConfig, OutputFormat};
use std::path::PathBuf;

// Assume 'validated_document' is a DocumentChunkValidated instance
let config = ExporterConfig::new(PathBuf::from("./output"), OutputFormat::Json, "my_export");
let exporter = FileExporter::new(config);

match exporter.export(validated_document) {
    Ok(_) => println!("Document exported successfully!"),
    Err(e) => eprintln!("Error exporting document: {:?}", e),
}
```

### JSONL Export

For efficient, line-by-line JSON export:

```rust
use rust_scraper::domain::entities::DocumentChunkValidated;
use rust_scraper::domain::exporter::Exporter;
use rust_scraper::infrastructure::export::{JsonlExporter, ExporterConfig};
use std::path::PathBuf;

// Assume 'validated_document' is a DocumentChunkValidated instance
let config = ExporterConfig::new(PathBuf::from("./output"), rust_scraper::domain::ExportFormat::Jsonl, "my_export.jsonl");
let exporter = JsonlExporter::new(config);

match exporter.export(validated_document) {
    Ok(_) => println!("Document exported to JSONL successfully!"),
    Err(e) => eprintln!("Error exporting document to JSONL: {:?}", e),
}
```

### State Management and Resume

To enable resuming exports:

```rust
use rust_scraper::infrastructure::export::StateStore;
use rust_scraper::domain::ExportState;

let domain = "example.com";
let store = StateStore::new(domain);

// Load existing state or create a new one
let mut state: ExportState = match store.load_or_default() {
    Ok(s) => s,
    Err(e) => {
        eprintln!("Failed to load or create state: {:?}", e);
        return;
    }
};

let url_to_process = "https://example.com/new_page";

if !store.is_processed(&state, url_to_process) {
    // ... perform scraping and export ...
    // If export is successful:
    store.mark_processed(&mut state, url_to_process);
    if let Err(e) = store.save(&state) {
        eprintln!("Failed to save state: {:?}", e);
    }
} else {
    println!("URL already processed: {}", url_to_process);
}
```