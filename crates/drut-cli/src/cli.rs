//! `clap`-derive CLI surface (data-model.md §2; contracts/cli-contract.md).

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "drut",
    about = "Structural linter/formatter for Cube Voyager control-statement scripts"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Report structural diagnostics for every `.s`/`.block` file under <PATH>.
    Check {
        /// A file or directory (FR-001).
        path: PathBuf,
        /// text (default, FR-008) or sarif (FR-009). Default holds in every
        /// context, interactive or not (FR-010).
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Normalize whitespace (and, opt-in, keyword casing) for every `.s`/
    /// `.block` file under <PATH>.
    Format {
        /// Same traversal/filtering rules as `check` (FR-001–FR-003).
        path: PathBuf,
        /// Overwrite each matched file in place (FR-017).
        #[arg(long, conflicts_with_all = ["check", "diff"])]
        write: bool,
        /// Report which files would change; write nothing (FR-018).
        #[arg(long, conflicts_with_all = ["write", "diff"])]
        check: bool,
        /// Print a unified diff per changed file; write nothing (FR-019).
        #[arg(long, conflicts_with_all = ["write", "check"])]
        diff: bool,
        /// Opt-in keyword-casing convention for the control-words category
        /// (things like `IF`, `ENDIF`, `LOOP`, `ENDLOOP`) — must be
        /// `preserve`, `upper`, or `lower` when given; no bare
        /// `--casing-control-words` (FR-015, amended by
        /// 014-casing-preserve-mode FR-006; 017-casing-categories-indent-width
        /// FR-001). A flat `--casing` covering this category and
        /// `--casing-pair-keywords` together once existed — removed once
        /// this granular flag and the one below fully superseded it.
        #[arg(long, value_enum)]
        casing_control_words: Option<CasingArg>,
        /// Independent override for the pair-keywords category
        /// (017-casing-categories-indent-width FR-001) — keyword names
        /// inside a `Control` statement's `keyword=value` pairs.
        #[arg(long, value_enum)]
        casing_pair_keywords: Option<CasingArg>,
        /// Independent override for the data-references category — Matrix/
        /// Line/Node/Zone/Database abbreviations, the output-record and
        /// link-endpoint tokens, and the two reserved loop-index
        /// identifiers (017-casing-categories-indent-width FR-004).
        #[arg(long, value_enum)]
        casing_data_references: Option<CasingArg>,
        /// Independent override for the function-calls category
        /// (025-function-casing) — a Cube Voyager built-in function name
        /// immediately followed by `(`.
        #[arg(long, value_enum)]
        casing_function_calls: Option<CasingArg>,
        /// Spaces per nesting level of block indentation
        /// (017-casing-categories-indent-width FR-009). `Option`-typed like
        /// `--casing` — omitting the flag means "consult `drut.toml`, then
        /// the built-in default (4)." Must be between 1 and 16 when given.
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=16))]
        indent_width: Option<u8>,
        /// Top-level (depth-0) indentation policy — `preserve` (default,
        /// FR-001) leaves it exactly as written; `auto` forces every
        /// top-level line to column 0 (FR-002/FR-003,
        /// 009-top-level-indent-toggle; named `normalize` before this value
        /// was renamed for `preserve`/`auto` naming consistency with
        /// `--operator-spacing`/`--blank-lines`). As of 012-toml-configuration,
        /// `Option`-typed like `--casing` — omitting the flag means "consult
        /// `drut.toml`, then the built-in default," which requires
        /// distinguishing "not passed" from "explicitly `preserve`"
        /// (research.md §1). Behavior with no `drut.toml` anywhere is
        /// unchanged: absent still resolves to `Preserve`.
        #[arg(long, value_enum)]
        indent_top_level: Option<IndentTopLevelArg>,
        /// Whitespace normalization around operators/commas/bracket-paren
        /// interiors (018-operator-spacing) — `preserve` (default) leaves it
        /// exactly as written; `fixed` normalizes to one space each side of
        /// every recognized operator; `auto` does everything `fixed` does
        /// plus vertically aligns consecutive `Assignment` statements'
        /// `=`. Same `Option`-typed shape as `--casing`/`--indent-top-level`
        /// — omitting the flag means "consult `drut.toml`, then the
        /// built-in default (preserve)."
        #[arg(long, value_enum)]
        operator_spacing: Option<OperatorSpacingArg>,
        /// Whether/how excessive blank-line runs are contracted
        /// (019-blank-line-normalization) — `preserve` (default) leaves
        /// every run exactly as written, however long; `auto` contracts a
        /// run down to the applicable cap (see the two flags below) only
        /// when it exceeds that cap. Same `Option`-typed, "requires an
        /// explicit value, no bare flag" shape every other format mode flag
        /// already uses.
        #[arg(long, value_enum)]
        blank_lines: Option<BlankLinesArg>,
        /// The maximum number of consecutive blank lines `auto` allows
        /// between top-level statements/blocks before contracting the run
        /// (019-blank-line-normalization FR-002). `Option`-typed like
        /// `--indent-width` — omitting the flag means "consult
        /// `drut.toml`, then the built-in default (2)."
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=50))]
        blank_lines_top_cap: Option<u8>,
        /// The maximum number of consecutive blank lines `auto` allows
        /// inside any block's own body, uniformly regardless of nesting
        /// depth, before contracting the run (019-blank-line-normalization
        /// FR-002/FR-008). Same shape as `--blank-lines-top-cap`,
        /// independently — built-in default `1`.
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=50))]
        blank_lines_nested_cap: Option<u8>,
        /// Whether an over-width `Control` statement's `keyword=value` pair
        /// list is wrapped across multiple physical lines
        /// (030-auto-line-wrap) — `preserve` (default) leaves it exactly as
        /// written, however long; `auto` wraps at top-level commas once the
        /// configured width is exceeded. Same `Option`-typed, "requires an
        /// explicit value, no bare flag" shape every other format mode flag
        /// already uses.
        #[arg(long, value_enum)]
        line_wrap: Option<LineWrapArg>,
        /// The maximum line width `auto` wraps toward (030-auto-line-wrap
        /// FR-002). `Option`-typed like `--indent-width` — omitting the flag
        /// means "consult `drut.toml`, then the built-in default (120)."
        /// Only consulted when `--line-wrap=auto`.
        #[arg(long, value_parser = clap::value_parser!(u16).range(20..=500))]
        line_wrap_width: Option<u16>,
        /// How pairs are distributed across continuation lines under `auto`
        /// (030-auto-line-wrap FR-002a) — `fill` (default) packs as many
        /// pairs as fit per line; `one-per-line` places exactly one pair
        /// per line. Only consulted when `--line-wrap=auto`.
        #[arg(long, value_enum)]
        line_wrap_style: Option<LineWrapStyleArg>,
        /// Skip `drut.toml` discovery entirely for this run, using built-in
        /// defaults plus any other explicit flags (012-toml-configuration
        /// US3, mirroring Ruff's `--isolated`).
        #[arg(long)]
        isolated: bool,
    },
    /// Speak the Language Server Protocol over stdio (003-lsp-vscode-extension
    /// FR-001) — no flags; launchable by an LSP client with no configuration
    /// beyond pointing it at this binary.
    Server,
    /// Speak the Model Context Protocol over stdio (004-mcp-server FR-001)
    /// — no flags; launchable by any MCP-capable client with no
    /// configuration beyond pointing it at this binary. Exposes four
    /// read-only tools (diagnose/format/query_structure/lookup_keyword)
    /// over `voyager-core`, entirely independent of `Server` above (no
    /// shared state, no dependency on a running LSP session, FR-011).
    Mcp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Sarif,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CasingArg {
    Preserve,
    Upper,
    Lower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum IndentTopLevelArg {
    Preserve,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OperatorSpacingArg {
    Preserve,
    Fixed,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BlankLinesArg {
    Preserve,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LineWrapArg {
    Preserve,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LineWrapStyleArg {
    Fill,
    OnePerLine,
}

// The CasingArg -> voyager_core::CasingConvention and IndentTopLevelArg ->
// voyager_core::IndentTopLevelMode conversions live in format_cmd.rs (added
// alongside the voyager-core format module) rather than here, so this
// Foundational-phase module has no dependency on those not-yet-built types.
