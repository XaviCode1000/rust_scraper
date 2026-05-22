# Core Application Logic — src

# Core Application Logic — src

This module contains the core application logic, including the dependency injection container.

## Dependency Injection Container (`src/di.rs`)

The `di.rs` file defines the `Container` struct, which serves as the application's dependency injection (DI) container. This container is responsible for holding and providing access to the application's core services and configurations.

### Purpose

The `Container` follows the principles of Clean Architecture by residing in the application layer. Its primary purpose is to:

*   **Centralize Dependencies:** Manage all application-level dependencies in a single location.
*   **Promote Reusability:** Ensure that services are instantiated once and reused throughout the application's lifetime, preventing redundant object creation and managing shared state.
*   **Decouple Components:** Allow different parts of the application to access necessary services without needing to know how they are created or managed.

### Components

The `Container` struct holds the following key dependencies:

*   `config`: An `Arc<Config>` that holds the application's overall configuration. `Arc` is used for shared ownership, allowing multiple parts of the application to access the configuration safely.
*   `http_client`: An `Arc<HttpClient>` that provides an HTTP client instance. This client is configured with settings from the application's configuration.

### Functionality

#### `Container::new(config: Config)`

This is the constructor for the `Container`. It takes the application's `Config` as input and performs the following actions:

1.  **Configuration Validation:** It calls `config.validate()?` to ensure the provided configuration is valid before proceeding.
2.  **HTTP Client Initialization:** It creates a new `HttpClient` instance using the HTTP-specific settings from the `config.http` field. This client is then wrapped in an `Arc` for shared access.
3.  **Container Instantiation:** It constructs and returns a new `Container` instance, wrapping the validated `config` and the initialized `http_client` in `Arc`.

#### `Container::config(&self) -> &Config`

Returns a reference to the application's configuration.

#### `Container::http_client(&self) -> &HttpClient`

Returns a reference to the application's HTTP client.

#### `Container::scraper_config(&self) -> &ScraperConfig`

Returns a reference to the scraper-specific configuration, which is a sub-field of the main `config`.

### Integration with the Codebase

The `Container` is instantiated early in the application's startup process, typically when the main server (e.g., MCP server) is being built.

**Execution Flow Example:**

```mermaid
graph TD
    A[start_mcp_server] --> B_build_mcp_router["B(build_mcp_router)"];
    B --> C_Container__new["C(Container::new)"];
    C --> D_HttpClient__new["D(HttpClient::new)"];
    C --> E_config_validate["E(config.validate)"];
    C --> F_Container["F(Container"] instance);
```

As shown in the call graph and execution flows, the `Container::new` function is invoked by `build_mcp_router` within the MCP server setup. This means that once the `Container` is created, it's available to any component that needs it during the server's operation. The `Container` itself does not make outgoing calls to other modules but rather provides access to services that might.