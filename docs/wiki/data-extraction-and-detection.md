# Data Extraction and Detection

# Data Extraction and Detection Module

This module is responsible for extracting relevant data from web pages, specifically focusing on identifying and classifying assets like images and documents. It also provides mechanisms for detecting MIME types and downloading these assets.

## Overview

The Data Extraction and Detection module acts as a bridge between raw HTML content and structured, usable data. It leverages several sub-modules to achieve its goals:

*   **MIME Type Detection (`adapters::detector::mime`):** Identifies the type of a file or data based on its extension or content (magic bytes).
*   **Asset Extraction (`extractor`):** Parses HTML to find and extract URLs of images and documents.
*   **Asset Downloading (`infrastructure::scraper::asset_download`):** Handles the process of downloading identified assets.
*   **Text Extraction (`infrastructure::scraper::fallback` and `infrastructure::scraper::readability`):** Extracts readable text content from HTML, with fallback mechanisms.

## Key Components and Functionality

### 1. MIME Type Detection (`adapters::detector::mime`)

This sub-module provides utilities to determine the type of a resource.

*   **`AssetType` Enum:**
    *   Represents the classified type of an asset: `Image`, `Document`, or `Unknown`.
    *   Provides helper methods `is_image()` and `is_document()`.

*   **Detection Functions:**
    *   **`detect_from_url(url: &str) -> AssetType`**: Detects the asset type by examining the file extension in the URL. It first attempts to parse the URL and then uses `detect_from_path`.
    *   **`detect_from_path(path: &str) -> AssetType`**: Detects the asset type based on the file extension extracted from a given path. It maintains lists of known image and document extensions.
    *   **`detect_from_bytes(data: &[u8]) -> AssetType`**: (Feature-gated by `images` or `documents`) Detects the asset type by analyzing the "magic bytes" (initial bytes of the file content) using the `mimetype-detector` crate. This is a more robust method than extension-based detection.

*   **Helper Functions:**
    *   **`get_extension(url: &str) -> Option<String>`**: Extracts the file extension from a URL.
    *   **`get_mime_type(url: &str) -> Option<&'static str>`**: Attempts to determine the MIME type of a resource from its URL. It prioritizes extension-based detection.

*   **Constants:**
    *   `IMAGE_MIMES`, `DOCUMENT_MIMES`: Lists of known MIME types for images and documents, used by `detect_from_bytes`.
    *   `IMAGE_EXTENSIONS`, `DOCUMENT_EXTENSIONS`: Lists of known file extensions for images and documents, used by `detect_from_path`.

### 2. Asset Extraction (`extractor`)

This sub-module focuses on parsing HTML to find URLs of images and documents.

*   **`AssetUrl` Struct:**
    *   Represents a discovered asset with its `url`, `asset_type`, and optional `alt` text (for images) or description (for documents).

*   **Extraction Functions:**
    *   **`extract_images(document: &Html, base_url: &url::Url) -> Vec<AssetUrl>`**: Extracts image URLs from `<img>` tags (both `src` and `srcset` attributes) and `<source>` tags within `<picture>` elements. It uses pre-compiled CSS selectors for efficiency.
    *   **`extract_documents(document: &Html, base_url: &url::Url) -> Vec<AssetUrl>`**: Extracts document URLs from `<a>` tags. It filters out links that are not actual resources (e.g., fragment identifiers, JavaScript links) and checks if the URL points to a document using `detect_from_url`. The link's text content is used as a description.
    *   **`extract_all_assets(html: &str, base_url: &url::Url) -> Vec<AssetUrl>`**: A convenience function that parses the HTML document once and then calls both `extract_images` and `extract_documents` to gather all asset URLs.

*   **Internal Helpers:**
    *   **`process_asset_src(src: &str, base_url: &url::Url, alt: Option<String>) -> Option<AssetUrl>`**: A core function that takes a source URL, resolves it against a base URL, filters out invalid or non-asset sources (like `data:` URIs), detects the asset type, and returns an `AssetUrl` if valid.
    *   **`parse_srcset(srcset: &str) -> Vec<String>`**: Parses the `srcset` attribute of `<img>` and `<source>` tags, extracting individual URLs.

*   **CSS Selectors:**
    *   `IMG_SELECTOR`, `SRCSET_SELECTOR`, `SOURCE_SELECTOR`, `LINK_SELECTOR`: Pre-compiled `scraper::Selector` instances for efficient HTML element selection. `LazyLock` is used to ensure they are compiled only once.

### 3. Asset Downloading (`infrastructure::scraper::asset_download`)

This feature-gated module handles the actual downloading of assets. It's enabled when the `images` or `documents` features are active.

*   **`download_all(html: &str, base_url: &url::Url, config: &ScraperConfig) -> Result<Vec<DownloadedAsset>>`**: The main entry point for downloading. It first extracts image and document URLs using the `extractor` module and then proceeds to download them in batches with controlled concurrency.
*   **Batch Download Functions:**
    *   **`download_image_batch(...)`**: Downloads a collection of image `AssetUrl`s.
    *   **`download_document_batch(...)`**: Downloads a collection of document `AssetUrl`s.
    *   These functions use `futures::stream::buffer_unordered` to manage concurrent downloads, limiting them by `DOWNLOAD_CONCURRENCY`.
*   **`download_single_asset(url: &str, asset_type: &str, output_dir: &Path) -> Result<DownloadedAsset>`**: Downloads a single asset.
    *   It uses the `wreq` crate with a Chrome user agent for better compatibility.
    *   The downloaded content is hashed using SHA-256 to generate a unique filename.
    *   The file extension is determined using `adapters::detector::mime::get_extension`.
    *   The asset is saved into a subdirectory (`images` or `documents`) within the specified `output_dir`.
    *   File system operations (directory creation and writing) are performed on a separate thread pool using `tokio::task::spawn_blocking` to avoid blocking the async runtime.
*   **`DownloadedAsset` Struct:** (Defined in `src/domain/value_objects.rs`) Represents a successfully downloaded asset, containing its original `url`, `local_path`, `asset_type`, and `size`.

### 4. Text Extraction (`infrastructure::scraper::fallback` and `infrastructure::scraper::readability`)

These modules provide ways to extract clean, readable text content from HTML.

*   **`infrastructure::scraper::readability`:**
    *   **`parse(html: &str, url: Option<&str>) -> Result<Article>`**: Wraps the `legible` crate to apply the Readability algorithm. This algorithm aims to remove boilerplate content (navigation, ads, footers) and extract the main article content.
    *   **`Article` Struct:** Represents the parsed article, containing fields like `title`, `content` (clean HTML), `text_content` (plain text), `excerpt`, `byline`, and `published_time`.

*   **`infrastructure::scraper::fallback`:**
    *   **`extract_text(html: &str) -> String`**: Provides a fallback mechanism for text extraction when the Readability algorithm fails. It primarily uses the `htmd` crate for HTML-to-Markdown conversion. If `htmd` also fails, it performs a very basic line-by-line filtering of the HTML to extract any semblance of text.

## Module Dependencies

The Data Extraction and Detection module has the following key dependencies:

*   **`scraper`**: For parsing HTML and selecting elements using CSS selectors.
*   **`url`**: For URL parsing and resolution.
*   **`mimetype-detector`**: (Optional, feature-gated) For detecting MIME types from file content.
*   **`wreq` / `wreq-util`**: (Optional, feature-gated) For making HTTP requests to download assets.
*   **`sha2`**: (Optional, feature-gated) For generating file hashes.
*   **`legible`**: (Optional, feature-gated) For applying the Readability algorithm.
*   **`htmd`**: (Optional, feature-gated) For HTML to Markdown conversion as a fallback.
*   **`tokio`**: For asynchronous operations, especially `spawn_blocking` for file I/O.
*   **`tracing`**: For logging information and warnings.

## Execution Flow Example: Downloading an Image

1.  **`download_all`** is called with HTML content and a base URL.
2.  Inside `download_all`, the HTML is parsed by `scraper::Html::parse_document`.
3.  **`extract_images`** is called, which uses CSS selectors to find `<img>` and `<source>` tags.
4.  For each image source found, **`process_asset_src`** is called.
5.  `process_asset_src` resolves the URL against the `base_url` and calls **`detect_from_url`**.
6.  `detect_from_url` calls **`detect_from_path`** (or `detect_from_bytes` if enabled and content is available) to classify the asset as `AssetType::Image`.
7.  If classified as an image, an `AssetUrl` is created and added to a list.
8.  Back in `download_all`, the list of `AssetUrl`s for images is passed to **`download_image_batch`**.
9.  `download_image_batch` creates asynchronous tasks for each image URL and uses `buffer_unordered` to download them concurrently via **`download_single_asset`**.
10. `download_single_asset` makes an HTTP GET request using `wreq`, reads the response bytes, determines the filename (using `get_extension` and hashing), creates necessary directories, and writes the file to disk.
11. Successfully downloaded assets are collected into a `Vec<DownloadedAsset>` and returned.

## Architecture Diagram

```mermaid
graph TD
    A[HTML Content] --> B_Scraper["B(Scraper"] Service);
    B --> C{Extractor};
    C --> D_MIME["D(MIME"] Detector);
    C --> E_Asset["E(Asset"] Downloader);
    E --> F_HTTP["F(HTTP"] Client);
    E --> G_File["G(File"] System);
    B --> H{Text Extractor};
    H --> I_Readability["I(Readability)"];
    H --> J_Fallback["J(Fallback"] Text Stripper);
    D --> K_MIME["K(MIME"] Type Lists);
    D --> L_File["L(File"] Extension Lists);
    E --> D; % Downloader uses Detector for extension
    E --> M_Hashing["M(Hashing)"];
    E --> N_Directory["N(Directory"] Creation);

    subgraph Extractor
        C1[extract_images]
        C2[extract_documents]
        C3[process_asset_src]
        C4[parse_srcset]
    end

    subgraph MIME Detector
        D1[detect_from_url]
        D2[detect_from_path]
        D3[detect_from_bytes]
        D4[get_extension]
        D5[get_mime_type]
    end

    subgraph Asset Downloader
        E1[download_all]
        E2[download_image_batch]
        E3[download_document_batch]
        E4[download_single_asset]
    end

    subgraph Text Extractor
        H1[extract_text]
    end

    subgraph Readability
        I1[legible::parse]
    end

    subgraph Fallback Text Stripper
        J1[htmd::convert]
        J2[Basic Line Filtering]
    end

    B -- Uses --> C;
    B -- Uses --> H;
    C -- Uses --> D;
    C -- Uses --> C3;
    C3 -- Uses --> D1;
    D1 -- Uses --> D2;
    C -- Uses --> C4;
    C2 -- Uses --> D1;
    B -- Uses --> E;
    E -- Uses --> C; % Extracts assets first
    E -- Uses --> E1;
    E1 -- Uses --> E2;
    E1 -- Uses --> E3;
    E2 -- Uses --> E4;
    E3 -- Uses --> E4;
    E4 -- Uses --> D4;
    E4 -- Uses --> M;
    E4 -- Uses --> N;
    B -- Uses --> H;
    H -- Uses --> I;
    H -- Uses --> J;
    I -- Uses --> I1;
    J -- Uses --> J1;
    J -- Uses --> J2;
```