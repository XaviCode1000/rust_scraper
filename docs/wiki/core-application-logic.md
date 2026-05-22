# Core Application Logic

# Core Application Logic

This module group encapsulates the fundamental business logic and orchestration of the application. It defines the core use cases, manages dependencies, and provides the services necessary for crawling, scraping, and exporting data.

The `application` sub-module acts as the central orchestrator. It defines high-level use cases such as website crawling (`crawler_service`) and individual URL scraping (`scraper_service`). It also manages the persistence of crawl results through the `crawl_result_repository` and handles data export via the `export_factory` and `export_utils`.

The `src` sub-module, specifically `src/di.rs`, houses the `Container` which is the application's dependency injection mechanism. This `Container` centralizes the instantiation and management of all application-level services, ensuring adherence to Clean Architecture principles and promoting reusability.

Key workflows that span these sub-modules include:

*   **Crawling and Scraping**: The `crawler_service` initiates crawls, potentially using sitemaps, and delegates the scraping of individual URLs to the `scraper_service`. The `scraper_service` can be configured to download assets, which involves interaction with asset handling logic.
*   **Data Management**: Crawl results are persisted using the `crawl_result_repository`, which supports operations like saving, finding by URL, and retrieving all results.
*   **Exporting**: The `export_factory` orchestrates the creation of appropriate exporters, leveraging `export_utils` for tasks like determining domain information from URLs.
*   **Dependency Resolution**: The `Container` from `src/di.rs` is used throughout the application to provide instances of services like the `crawler_service`, `scraper_service`, and `crawl_result_repository`, ensuring they are correctly wired with their own dependencies.

```mermaid
graph TD
    subgraph Core Application Logic
        application[application]
        src[src]
    end

    application -->|Defines Use Cases| crawler_service
    application -->|Defines Use Cases| scraper_service
    application -->|Manages Persistence| crawl_result_repository
    application -->|Handles Exporting| export_factory
    application -->|Handles Exporting| export_utils

    src -->|Provides DI Container| Container
    Container -->|Injects Services| crawler_service
    Container -->|Injects Services| scraper_service
    Container -->|Injects Services| crawl_result_repository
    Container -->|Injects Services| export_factory
    Container -->|Injects Services| export_utils

    crawler_service -->|Scrapes URLs| scraper_service
    scraper_service -->|Saves Results| crawl_result_repository
    crawl_result_repository -->|Stores Data| Persistence Layer
    export_factory -->|Uses Utilities| export_utils
```