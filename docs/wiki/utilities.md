# Utilities

# Utilities Module

The Utilities module provides essential helper functions and data structures that are used across various parts of the `rust-scraper` application. This includes managing user agents, handling file paths, and providing version information.

## User Agent Management

The `user_agent` submodule is responsible for providing realistic User-Agent strings for HTTP requests. It implements a Time-To-Live (TTL) based caching strategy to balance performance and freshness.

### Cache Strategy

The User-Agent cache follows these steps:

1.  **Check Cache:** It first looks for a cached `user_agents.json` file at `~/.cache/rust_scraper/user_agents.json`.
2.  **Validate Cache:** If the cache file exists, it checks the `chrome_version` stored within. If the cached version's year is within the last year of the current year, the cache is considered valid and used.
3.  **Fetch Fresh:** If the cache is invalid (too old or missing), it attempts to download a fresh list of user agents from the configured API URL (`https://raw.githubusercontent.com/user-agents-api/data/main/user-agents.json`).
4.  **Save Cache:** Upon a successful download, the new list of user agents and their metadata (including the Chrome version and download timestamp) are saved to the cache file.
5.  **Fallback:** If downloading from the API fails for any reason, the module falls back to a hardcoded list of user agents.

### Key Components

*   **`UserAgentCache` Struct:**
    *   `agents`: A `Vec<String>` holding the list of user agent strings.
    *   `chrome_version`: The Chrome version associated with the cached user agents.
    *   `downloaded_at`: A `DateTime<Utc>` timestamp indicating when the cache was last updated.

*   **`UserAgentCache::load()`:**
    *   The primary public method for obtaining a list of user agents.
    *   It orchestrates the cache loading, validation, fetching, and fallback logic.
    *   Returns a `Vec<String>` of user agents.

*   **`UserAgentCache::load_from_cache()`:**
    *   Reads and deserializes the user agent cache from the local file system.

*   **`UserAgentCache::fetch_and_cache()`:**
    *   Handles the asynchronous fetching of user agents from the remote API.
    *   Filters for user agents with a Chrome version greater than or equal to `MIN_CHROME_VERSION` (currently 131).
    *   Saves the fetched data to the cache file, creating directories if necessary. Errors during cache writing are silently ignored to handle read-only file systems or container environments gracefully.

*   **`UserAgentCache::fallback_agents()`:**
    *   Provides a static, hardcoded list of user agents. This is used when cache loading or API fetching fails. The fallback list includes recent Chrome and Firefox versions.

*   **`get_random_user_agent_from_pool(pool: &[String]) -> String`:**
    *   A utility function that selects a random user agent string from a given slice of strings.

*   **`get_random_user_agent() -> String` (Deprecated):**
    *   A legacy function that directly uses `fallback_agents()` to get a random user agent. It is deprecated in favor of `UserAgentCache::load()` for better cache management.

### Cache Location

The cache file is stored at: `~/.cache/rust_scraper/user_agents.json`. The `dirs::cache_dir()` function is used to find the appropriate cache directory, with a fallback to the current directory if it cannot be determined.

## Version Information

The module also provides a way to retrieve detailed build information about the `rust-scraper` application.

*   **`version_string() -> String`:**
    *   Returns a formatted string containing the application version, short Git commit hash, and build date. This information is embedded during the build process using the `built` crate.

## Module Structure

```mermaid
graph TD
    A[Utilities Module] --> B_user_agent["B(user_agent"] submodule);
    B --> C{UserAgentCache};
    C --> C1_load["C1(load)"];
    C --> C2_load_from_cache["C2(load_from_cache)"];
    C --> C3_fetch_and_cache["C3(fetch_and_cache)"];
    C --> C4_fallback_agents["C4(fallback_agents)"];
    B --> D_get_random_user_agent_from_pool["D(get_random_user_agent_from_pool)"];
    B --> E_get_random_user_agent["E(get_random_user_agent"] - Deprecated);
    A --> F_version_string["F(version_string)"];
    F --> G_built_info["G(built_info"] module);
```

## Integration with the Codebase

*   **HTTP Client:** The `create_http_client` function in `application/http_client/client.rs` utilizes `UserAgentCache::load()` to set up the `wreq::Client` with appropriate User-Agent headers. In cases where fetching fails, it falls back to `fallback_agents()`.
*   **Testing:** The `user_agent` submodule includes comprehensive unit tests to verify cache loading, fallback mechanisms, and random agent selection. Integration tests also implicitly test the user agent fetching and fallback logic when setting up HTTP clients.
*   **Build System:** The `version_string` function relies on build-time information generated by the `built` crate, which is configured in the `build.rs` file.