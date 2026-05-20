//! MCP Handler modules — tool implementations organized by category
//!
//! Each module provides a `build_router()` function that returns a partial
//! `ToolRouter<McpHandler>`. All routers are combined with the `+` operator.
//!
//! Note: All 37 tools are defined in the parent mod.rs #[tool_router] block.
//! These submodules exist for future modularization but currently return
//! empty routers.

use rmcp::handler::server::tool::ToolRouter;
use super::McpHandler;

pub mod scraping;
pub mod content;
pub mod export;
pub mod url_utils;
pub mod security;
pub mod obsidian;
pub mod ai;
pub mod assets;

/// Build the combined ToolRouter from all 8 category modules.
pub fn build_tool_router() -> ToolRouter<McpHandler> {
    scraping::build_router()
        + content::build_router()
        + export::build_router()
        + url_utils::build_router()
        + security::build_router()
        + obsidian::build_router()
        + assets::build_router()
        + ai::build_router()
}
