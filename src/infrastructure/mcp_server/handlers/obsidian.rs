//! Obsidian Integration tools — 4 tools for vault operations
//!
//! Tools: detect_obsidian_vault, build_obsidian_uri,
//! open_in_obsidian, search_obsidian

use rmcp::handler::server::tool::ToolRouter;
use super::McpHandler;

/// Build the partial tool router for Obsidian tools.
pub fn build_router() -> ToolRouter<McpHandler> {
    ToolRouter::new()
}
