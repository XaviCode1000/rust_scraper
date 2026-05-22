# Documentation and Examples — examples

# Documentation and Examples — `examples`

The `examples` module contains standalone entry points for exercising the MCP server in two different transports:

- `examples/mcp_server.rs` — HTTP-based MCP server for testing the Streamable HTTP handshake
- `examples/mcp_server_stdio.rs` — stdio-based MCP server for MCP clients that launch a subprocess

These examples are intentionally small and are meant to validate the wiring between:

- `rust_scraper::config::Config`
- `rust_scraper::di::Container`
- `rust_scraper::infrastructure::mcp_server::{McpState, McpHandler, server::build_mcp_router}`

They are useful for manual testing, integration checks, and for understanding how the MCP server is assembled from the application’s core services.

## What these examples are for

The examples demonstrate two common MCP deployment modes:

1. **Streamable HTTP**
   - Useful for local testing with `curl`
   - Also mirrors how remote MCP servers are often exposed
   - Implemented in `examples/mcp_server.rs`

2. **stdio transport**
   - Required by many MCP clients that start the server as a child process
   - Common for desktop clients such as Claude Desktop, Cursor, OpenCode, and Cline
   - Implemented in `examples/mcp_server_stdio.rs`

Both examples create the same underlying application state from the same container, so they exercise the same business logic and MCP handlers. The only difference is how the server is exposed to the client.

## Shared startup pattern

Both files follow the same high-level boot sequence:

1. Initialize tracing
2. Build a default `Config`
3. Create the dependency injection `Container`
4. Wrap the container in `McpState`
5. Build the transport-specific server
6. Start serving requests

This separation keeps transport concerns out of the application core and makes it easy to test the server in more than one runtime environment.

```mermaid
flowchart LR
    A[Config::default()] --> B[Container::new(config)]
    B --> C[McpState::new(container)]
    C --> D1[build_mcp_router(state)]
    C --> D2[McpHandler::new(state)]
    D2 --> E1[serve(stdin, stdout)]
    D1 --> E2[axum::serve(listener, app)]
```

## `examples/mcp_server.rs`

### Purpose

This example starts an MCP server over HTTP on `127.0.0.1:8080` and mounts the MCP router at `/mcp`.

It is designed for testing the Streamable HTTP handshake manually, especially with tools like `curl`.

### Entry point

The binary uses:

```rust
#[tokio::main]
async fn main()
```

so it runs on the Tokio async runtime.

### Startup flow

The HTTP example performs the following steps:

- Initializes `tracing_subscriber::fmt::init()`
- Constructs a default `Config`
- Builds a `Container` with `Container::new(config).await`
- Creates `McpState::new(container)`
- Builds the Axum router with `build_mcp_router(state)`
- Binds a `tokio::net::TcpListener` to `127.0.0.1:8080`
- Serves the router with `axum::serve(listener, app)`

### Output and testing

On startup, it prints:

- the listening URL
- a sample `curl` request for the MCP `initialize` handshake

That sample request is important because it shows the expected JSON-RPC payload shape and headers for Streamable HTTP:

- `Content-Type: application/json`
- `Accept: application/json, text/event-stream`

### Error handling

The example uses `expect(...)` at each setup step:

- invalid socket address
- container creation failure
- listener bind failure
- server startup failure

This is appropriate for an example binary where failures should crash loudly and directly.

### Connection to the rest of the codebase

The HTTP example depends on the MCP server router implementation in:

- `rust_scraper::infrastructure::mcp_server::server::build_mcp_router`

That means this example is a thin transport wrapper; it does not implement MCP protocol handling itself.

## `examples/mcp_server_stdio.rs`

### Purpose

This example exposes the MCP server over stdio, which is the transport expected by many MCP desktop clients that spawn the server as a subprocess.

This is the correct choice when the client protocol contract is:

- requests are read from `stdin`
- responses are written to `stdout`
- logs must go to `stderr`

### Entry point

Like the HTTP example, this binary uses:

```rust
#[tokio::main]
async fn main()
```

### Startup flow

The stdio example performs these steps:

- Initializes tracing with a writer bound to `stderr`
- Constructs `Config::default()`
- Builds the `Container`
- Creates `McpState`
- Constructs `McpHandler::new(state)`
- Creates a stdio transport from `(tokio::io::stdin(), tokio::io::stdout())`
- Calls `handler.serve(transport).await`
- Waits for the server to finish with `server.waiting().await`

### Why stderr matters

The file includes an explicit warning:

> All logging MUST go to stderr, never stdout.

This is critical because the MCP stdio transport uses `stdout` exclusively for JSON-RPC messages. Any logs or debug output written to `stdout` will corrupt the protocol stream and can cause client disconnects or parse errors.

That is why this example configures tracing with:

```rust
tracing_subscriber::fmt()
    .with_writer(std::io::stderr)
    .init();
```

### Transport lifecycle

The stdio server is started through `rmcp::service::ServiceExt::serve(...)`. After startup, the example blocks on `server.waiting().await`, which keeps the process alive until the client disconnects or stdin closes.

### Connection to the rest of the codebase

This example depends on:

- `rust_scraper::infrastructure::mcp_server::McpHandler`
- `rust_scraper::infrastructure::mcp_server::McpState`

Unlike the HTTP example, it does not build an Axum router. It uses the rmcp service abstraction directly, which is the correct integration point for stdio MCP clients.

## Transport differences

The two binaries are structurally similar, but they target different runtime environments.

| Example | Transport | Intended client style | Routing layer |
|---|---|---|---|
| `mcp_server.rs` | HTTP | Manual testing, remote clients | Axum router via `build_mcp_router` |
| `mcp_server_stdio.rs` | stdio | Subprocess-based MCP clients | `McpHandler::serve(...)` |

### Practical implications

- Use **HTTP** if you want to inspect requests with `curl` or connect over a network endpoint.
- Use **stdio** if you are integrating with a client that launches the server locally.

## Logging and protocol safety

The examples encode an important operational rule:

- **HTTP example:** normal stdout logging is acceptable because the server is HTTP-based
- **stdio example:** stdout is reserved for protocol messages, so logs must be redirected to stderr

If you add more example binaries in this module, follow the same rule:

- stdio transport → never print logs to stdout
- HTTP transport → stdout is fine for startup messages and diagnostics

## Extending these examples

When adding a new example binary, keep the same pattern:

1. Build configuration first
2. Construct the shared dependency container
3. Convert the container into `McpState`
4. Choose the transport layer
5. Keep transport code thin and push behavior into `infrastructure::mcp_server`

Good candidates for future examples include:

- alternative ports or bind addresses
- loading `Config` from environment instead of `Config::default()`
- additional transport variants
- minimal client scripts for smoke testing

## Maintenance notes

### Keep startup code minimal

These examples should remain focused on bootstrapping only. Avoid moving business logic into the example binaries; that logic belongs in the application modules they depend on.

### Preserve transport correctness

If you modify `mcp_server_stdio.rs`, be careful not to:

- write any `println!` output to stdout
- initialize a logger that writes to stdout by default
- accidentally mix transport bytes with diagnostic output

### Keep the sample handshake current

The HTTP example includes an `initialize` request body with:

- `protocolVersion: "2024-11-05"`
- `capabilities: {}`
- `clientInfo: { name: "test", version: "1.0.0" }`

If the MCP protocol version or handshake expectations change in the rest of the codebase, update the comment and sample request so the example remains accurate.