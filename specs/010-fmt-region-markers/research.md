# Phase 0 Research: FMT Region Markers

All findings below are measured against the real, current codebase (`crates/voyager-core/src/format.rs`, `token.rs`, `lexer.rs`; `crates/drut-cli/src/format_cmd.rs`; `crates/drut-lsp/src/formatting.rs`, `diagnostics.rs`; `crates/drut-mcp/src/format.rs`), not estimated.

## §1. Marker recognition is a token-stream concern, not a text-scan — and why that matters

**Decision**: Detect `; FMT: OFF` / `; FMT: ON` markers by scanning the existing `Token` stream (`tokenize`'s output) for `TokenKind::LineComment` tokens, not by regex/line-text matching over raw source.

**Rationale**: `LineComment`'s own recognition already handles exactly the cases that would otherwise silently produce a false-positive marker:
- `lexer.rs`'s own tests confirm a `;` inside a quoted string (`LIST=';FMT: OFF\n',`) or inside a `/* ... */` block comment is **not** tokenized as `LineComment` at all — a naive text scan would wrongly treat `PRINT LIST='; FMT: OFF is not a real marker'` as a region boundary. Reusing the tokenizer's own comment recognition means this can't happen, for free, with no new logic duplicating what the lexer already gets right (constitution Principle I — grammar-adjacent recognition stays in one place).
- `Token.text` for a `LineComment` includes the leading `;` and runs to end of physical line (`lexer.rs:140-144`, `text_of(&chars, start, j)` where `start` is the `;`'s own position) — so `token.text.trim_start_matches(';').trim()` gives exactly the content to case-insensitively compare against `"FMT: OFF"`/`"FMT: ON"` (FR-001/FR-002's whitespace-flexible, colon-preserving match).

**"Entire trimmed content of a comment-only line" (FR-001/FR-002) — the whole-line check**: a `LineComment` token is by construction always the *last* token on its physical line (it runs to end-of-line), but something could still precede it on the same line (e.g. `PRINT LIST=1  ; FMT: OFF`). Check: no other token in the stream has `span.start.line == this_token.span.start.line`. If any other token shares that start line, this is a *trailing* comment, not a whole-line marker, and FR-001/FR-002 correctly do not recognize it (per spec.md's Edge Cases: "a trailing `; FMT: OFF` after real statement content on the same line is not recognized as a marker"). This is a simple `BTreeMap<u32, usize>`-style per-line lookup, structurally identical to the one `lexer.rs:270-280` already builds for its own continuation-retagging pass — same pattern, new purpose.

**Alternatives considered**: A raw line-text regex (`^\s*;\s*FMT:\s*OFF\s*$`, case-insensitive) was the "obvious" first approach but was rejected — it would falsely fire inside string literals and block comments, exactly the two cases `lexer.rs`'s own test suite already proves the real tokenizer gets right. Reusing the tokenizer costs nothing extra (source is tokenized once per `format`/`format_bytes` call already) and inherits correctness the text-scan approach would have to re-derive and re-test from scratch.

## §2. Where protection plugs in: gate the existing plan/edit collection, add no new rendering machinery

**Decision**: Compute a `protected_lines: BTreeSet<u32>` (every line number from a recognized `; FMT: OFF` marker through its matching `; FMT: ON` marker or end-of-file, inclusive of both marker lines) once per `render` call, then thread it through the *existing* collection functions as a fourth "don't touch this line" gate — architecturally identical to how `diagnosed_openers: &BTreeSet<Position>` already gates `plan_block`'s children-skip. **Not** a direct reuse of that mechanism (different set shape — `Position` keyed by block-opener identity vs. `u32` keyed by line number — and different trigger — a diagnostic vs. a user-placed marker), exactly as ROADMAP.md's original framing anticipated.

**Four call sites need the gate, all in `format.rs`**:
1. `plan_indentation`'s top-level `plan.insert(node.span().start.line, 0)` (only reached under `Normalize` mode).
2. `plan_block`'s explicit-closer `plan.insert(closer_line, base)`.
3. `plan_block`'s `ELSEIF`/`ELSE` branch `plan.insert(branch_line, base)`.
4. `plan_children`'s per-child `plan.insert(child_line, base + INDENT_WIDTH)`.

Each becomes `if !protected_lines.contains(&line) { plan.insert(line, value); }` — never insert a plan entry for a protected line, full stop.

**Casing edits collapse to a single gate point**: every one of `collect_block_casing_edits`/`collect_statement_casing_edits`'s edit-producing calls already funnels through one function, `push_if_present` (`format.rs:474`). Gating there once — `if !protected_lines.contains(&span.start.line) { ...push the edit... }` — covers block openers, closers, `ELSEIF`/`ELSE` words, and every statement's control-word/pair-keyword edits, with a single change instead of four.

**The render loop itself (`format.rs:170-221`) needs zero changes.** It already does exactly the right thing for a line with no plan entry and no casing edits: reproduce it untouched. Protection falls out entirely from upstream collection producing nothing for those lines — this is the same "no new fallback logic needed" property `008`'s and `009`'s own plan.md documents both found for `computed_indent`'s existing "planned else original" fallback, and it holds here for exactly the same structural reason.

**A subtlety this resolves for free**: if a block's *opener* line is itself protected but a *child* of that block sits outside the protected region, the child's indentation is computed as `base + INDENT_WIDTH`, where `base = computed_indent(plan, lines, opener_line)`. Because the opener line never received a plan entry (protected), `computed_indent` falls back to `original_indent_width` — the opener's *true* on-disk column — not a naively-computed value that would have applied if the opener weren't protected. The child is therefore correctly anchored to what the opener actually looks like after formatting, not to a value that gets silently discarded. This is the same mechanism that already makes `007`'s diagnosed-block-children skip compose correctly with everything around it; no new fallback logic is needed here either.

**Alternatives considered**: Filtering at the final render loop (compute plan/edits normally, ignoring markers, then discard them per-line at output time) was rejected — it produces the "opener residue" bug described above (a child would anchor to a planned-but-never-applied opener value instead of the opener's real on-disk column). Gating at collection time, not render time, is required for correctness, not just a style preference.

## §3. FR-010's notice: a new `voyager-core` function, reused three different ways per adapter

**Decision**: Expose the unmatched-marker detection as its own small public function —

```rust
pub fn unclosed_fmt_off_markers(source: &str) -> Vec<Position>
```

— computed once inside `format`/`format_bytes` and exposed on `FormatResult` as a new field (`pub unclosed_fmt_off_markers: Vec<Position>`), **and** separately callable on its own. Two consumption paths exist because the three adapters need this at two different times:

- **CLI** (`drut-cli/src/format_cmd.rs`) and **MCP** (`drut-mcp/src/format.rs`) both already call `format`/`format_bytes` directly to get their result — they read the new `FormatResult` field, no extra call needed.
  - CLI: `FormatReport` already carries two structurally identical "informational file lists" (`recovered_encoding_files`, `unsafe_encoding_files`, `format_cmd.rs:46-49`), each populated in every mode and printed via a dedicated `eprintln!` block in `print_report` (`format_cmd.rs:208-225`). A third list, `unclosed_fmt_off_files: Vec<(PathBuf, Vec<Position>)>`, follows the exact same pattern — same non-fatal, stderr-only, every-mode treatment, no `derive_exit_outcome` change (this is informational, not an error condition, matching FR-010's "no error occurs" language in spec.md's revised US2 Acceptance Scenario).
  - MCP: `FormatResultDto` (`drut-mcp/src/format.rs:17-24`) gains one new serializable field, e.g. `unclosed_fmt_off_lines: Vec<u32>` (line numbers; `Position`'s column is not meaningful here since a marker always starts at column 1 of a comment-only line).
- **LSP** is different: `formatting.rs`'s `handle` (`textDocument/formatting`) returns only `Vec<TextEdit>` — the LSP protocol has no side channel on a formatting *response* for an informational notice. Diagnostics are a separate, independent publish cycle (`diagnostics.rs`, driven by `did_open`/`did_change`, not by a formatting request) — and semantically, "this file has an unclosed `; FMT: OFF`" is a property of the document's *text*, true whether or not the user ever runs Format Document, so it belongs in that independent cycle, not bolted onto the formatting handler. **Decision**: `diagnostics.rs`'s existing publish path calls `unclosed_fmt_off_markers` directly (the standalone function, not a full `format()` call — no need to compute indent/casing edits just to get this one signal) and publishes each result as an LSP diagnostic at `DiagnosticSeverity::HINT` (or `INFORMATION` — Phase 1 contract picks one), with a distinct `source` string (e.g. `"drut-fmt"` vs. the structural diagnostics' own source tag) so it is visually and programmatically distinguishable from the six real `DiagnosticKind` categories. This stays outside `voyager_core::Diagnostic`/`DiagnosticKind` entirely — LSP's own severity levels already give every adapter a "this is a hint, not an error" channel without touching the core crate's fixed six-category enum (per spec.md's Assumptions and FR-010: no new `DiagnosticKind` variant).

**Alternatives considered**: Bolting the notice onto `formatting.rs`'s `TextEdit` response (e.g. as a comment injected into the edit) was rejected — it would corrupt the formatted text itself, exactly the kind of silent content mutation Principle III explicitly forbids. Adding a new `DiagnosticKind` variant was rejected per the owner's own explicit steer during spec review — architecturally heavier than needed, and would require a constitution-level accounting of "the six categories" language in `CLAUDE.md`/spec docs that assumes that set is closed.

## §4. Marker syntax matching — exact rule

**Decision**: `token.text.trim_start_matches(';')` (LineComment tokens always start with exactly one `;` per `lexer.rs`'s own tokenization, so no loop needed), then `.trim()`, then case-insensitive equality against literal `"FMT: OFF"` / `"FMT: ON"`, **plus** one whitespace-flexibility allowance around the colon (`FMT :OFF`, `FMT:  ON`, etc.) via a small tokenized comparison (split on `:`, trim each side, compare case-insensitively) rather than a single fixed-string `eq_ignore_ascii_case` — matches spec.md's Assumptions ("flexible on whitespace around the colon") without needing a regex dependency (this crate has zero runtime dependencies, FR-027 in `001`'s own spec — a 4-line manual split/trim/compare is well within hand-written-parsing norms already established throughout this crate).

## §5. No lexer/parser/grammar change of any kind

Confirmed directly against `token.rs`/`lexer.rs`: `LineComment` is already exactly the right granularity (whole line from `;` onward), already correctly excludes quoted/block-comment content, and already preserves raw casing in `.text`. **This feature adds zero new `TokenKind` variants, zero new `Node`/`Block`/`Statement` shapes, and zero new `DiagnosticKind` variants** — everything needed already exists in the tokenizer; the only new code is (a) a marker-scan pass over the existing token stream producing a `BTreeSet<u32>` + `Vec<Position>`, and (b) the four/one-point gate described in §2. This is a materially smaller core-crate change than `008`/`009` (which changed `plan_indentation`'s actual indentation *values*) — here, the values computed are unchanged; only *which lines receive a plan/edit entry at all* changes.
