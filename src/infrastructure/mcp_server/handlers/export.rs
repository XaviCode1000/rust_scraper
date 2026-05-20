//! Export tools — 4 tools for output format conversion
//!
//! Tools: export_file, export_jsonl, export_vector,
//! process_export_pipeline

use rmcp::handler::server::tool::ToolRouter;
use super::McpHandler;

/// Build the partial tool router for export tools.
pub fn build_router() -> ToolRouter<McpHandler> {
    ToolRouter::new()
}
