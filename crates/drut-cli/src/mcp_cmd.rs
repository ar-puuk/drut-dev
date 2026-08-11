//! `drut mcp`: thin dispatch to `drut-mcp` — zero MCP protocol logic here
//! (constitution Principle I; 004-mcp-server FR-001).
//!
//! Every other subcommand (`check`/`format`/`server`) stays fully
//! synchronous — only this one dispatch arm ever constructs a `tokio`
//! runtime, and only when `drut mcp` is the subcommand actually invoked
//! (004-mcp-server/research.md §2). No subcommand pays a runtime-
//! construction cost it doesn't ask for.

pub fn run() -> i32 {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("error: failed to start the async runtime for `drut mcp`: {err}");
            return 1;
        }
    };
    match runtime.block_on(drut_mcp::run()) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("error: {err}");
            1
        }
    }
}
