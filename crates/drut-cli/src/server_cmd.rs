//! `drut server`: thin dispatch to `drut-lsp` over real stdio — zero LSP
//! protocol logic here (constitution Principle I; 003-lsp-vscode-extension
//! FR-001).

pub fn run() -> i32 {
    let (connection, io_threads) = lsp_server::Connection::stdio();
    drut_lsp::run(connection);
    // `io_threads.join()` blocks until the transport threads finish
    // shutting down, which happens once the client closes the connection.
    let _ = io_threads.join();
    0
}
