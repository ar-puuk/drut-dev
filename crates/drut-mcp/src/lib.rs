//! `drut-mcp`: a thin Model Context Protocol adapter over `voyager-core`
//! (constitution Principle I) — the fourth adapter named in the
//! constitution alongside the CLI, LSP server, and formatter. No grammar/
//! parsing/lint-rule logic lives here; every fact this crate reports is
//! derived from `voyager-core`'s public entry points (see
//! specs/004-mcp-server/plan.md).
//!
//! Every tool is read-only: no tool ever writes to disk (FR-010). Every
//! tool's only effect is the value it returns. Stateless across calls (no
//! open-document tracking the way `drut-lsp`'s `ServerState` needs — each
//! tool call is fully self-contained, plan.md's Technical Context "Storage:
//! N/A").

pub mod diagnose;
pub mod format;
pub mod lookup_keyword;
pub mod query_structure;
pub mod source;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{tool, tool_router, ErrorData as McpError, ServiceExt};

use diagnose::DiagnosticsInput;
use format::FormatInput;
use lookup_keyword::KeywordLookupInput;
use query_structure::StructuralQueryInput;
use source::SourceError;

/// Converts a tool-internal `SourceError` (bad/missing `text`/`path`, or an
/// unreadable file) into a structured MCP tool-call error — never a panic
/// (FR-012), never silently swallowed.
fn source_error_to_mcp(err: SourceError) -> McpError {
    McpError::invalid_params(err.to_string(), None)
}

/// Serializes any of this crate's own DTO result types to a single JSON
/// text content block — the simplest representation guaranteed to render
/// in any MCP client, text-only or not (contracts/mcp-tools.md).
fn json_result<T: serde::Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string(value)
        .map_err(|err| McpError::internal_error(format!("failed to serialize tool result: {err}"), None))?;
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

/// The MCP server itself — stateless (no fields), matching every tool's own
/// "self-contained per call" contract.
#[derive(Debug, Clone, Default)]
pub struct DrutMcpServer;

#[tool_router(server_handler)]
impl DrutMcpServer {
    #[tool(description = "Report every structural diagnostic voyager-core can find for a Cube Voyager script (given as inline text or a file path). Returns an empty list for a structurally valid script.")]
    fn diagnose(
        &self,
        Parameters(input): Parameters<DiagnosticsInput>,
    ) -> Result<CallToolResult, McpError> {
        let diagnostics = diagnose::diagnose(&input).map_err(source_error_to_mcp)?;
        json_result(&diagnostics)
    }

    #[tool(description = "Reformat a Cube Voyager script's whitespace/indentation (and, opt-in via `casing`, keyword casing). Returns the reformatted text and whether anything changed. Idempotent: formatting an already-formatted script reports changed=false.")]
    fn format(&self, Parameters(input): Parameters<FormatInput>) -> Result<CallToolResult, McpError> {
        let result = format::format(&input).map_err(|msg| McpError::invalid_params(msg, None))?;
        json_result(&result)
    }

    #[tool(description = "Report which of the seven block kinds (If/Loop/Run/Process/JLoop/LinkLoop/DistributeMultistep), if any, encloses a given 1-based line/column position in a Cube Voyager script, and where its matched counterpart is (correctly resolved even through Run/Process's implicit-close quirk). Reports kind=null, not an error, when no block encloses the position.")]
    fn query_structure(
        &self,
        Parameters(input): Parameters<StructuralQueryInput>,
    ) -> Result<CallToolResult, McpError> {
        let result = query_structure::query_structure(&input).map_err(source_error_to_mcp)?;
        json_result(&result)
    }

    #[tool(description = "Look up real, corpus-evidenced keyword=value pair name candidates for a given enclosing control word (e.g. RUN), falling back to the general-syntax control-word list when none is given. Optionally also runs a \"did you mean\" spell-check against a supplied token.")]
    fn lookup_keyword(
        &self,
        Parameters(input): Parameters<KeywordLookupInput>,
    ) -> Result<CallToolResult, McpError> {
        let result = lookup_keyword::lookup_keyword(&input);
        json_result(&result)
    }
}

/// Runs the MCP server over stdio until the client disconnects (FR-001).
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service = DrutMcpServer.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
