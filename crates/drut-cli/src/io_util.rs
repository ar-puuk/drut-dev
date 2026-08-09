//! Broken-pipe-tolerant stdout writing.
//!
//! Discovered via `002-cli-check-format/quickstart.md`'s own T036
//! walkthrough: piping this CLI's output into something that closes the
//! pipe early (`drut format ... --diff | head`, `| less`, etc. — an
//! extremely ordinary way to use a CLI tool) made `println!`/`print!`
//! **panic**, since those macros unconditionally `.unwrap()` the underlying
//! write result. A closed downstream pipe isn't a `drut` failure — it's the
//! consumer choosing to stop reading — so every stdout write in this crate
//! goes through [`write_stdout`] instead, which treats
//! `ErrorKind::BrokenPipe` as a normal, silent shutdown (exit `0`) rather
//! than a crash. Any *other* stdout error is still surfaced loudly, not
//! swallowed.
//!
//! Scoped to stdout only, not stderr's `eprintln!` call sites elsewhere in
//! this crate — stderr is used here only for short notices/errors, not the
//! large, commonly-piped content (formatted file text, diffs, SARIF JSON)
//! that actually triggers this in practice.

use std::io::Write;

/// Writes `text` to stdout verbatim (no added newline). Exits the process
/// directly on `BrokenPipe` (0) or any other stdout write failure (2) —
/// there is no sensible way to keep running once stdout itself is broken.
pub fn write_stdout(text: &str) {
    if let Err(e) = write!(std::io::stdout(), "{text}") {
        handle_stdout_error(e);
    }
}

/// Same as [`write_stdout`], with a trailing newline.
pub fn write_stdout_line(text: &str) {
    if let Err(e) = writeln!(std::io::stdout(), "{text}") {
        handle_stdout_error(e);
    }
}

fn handle_stdout_error(e: std::io::Error) -> ! {
    if e.kind() == std::io::ErrorKind::BrokenPipe {
        std::process::exit(0);
    }
    eprintln!("error: failed to write to stdout: {e}");
    std::process::exit(2);
}
