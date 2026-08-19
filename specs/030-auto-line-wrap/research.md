# Research: Automatic Line-Width Wrapping

## §1: `format.rs`'s pipeline is a per-original-line rebuild, not a generic renderer

`render()` (`format.rs:353-555`) iterates `raw_lines` once (line 485) and, for each *original*
source line, applies a merged edit list (casing + operator-spacing edits, each
`(line, char_start, char_end, replacement)`) plus `indent_plan`'s recomputed leading whitespace,
then emits exactly one output line. **One input line always maps to at most one output line
today** — nothing currently splits a line into two. This feature is the first to break that
invariant, which shapes the whole design below.

**Chosen approach**: piggyback on the existing edit mechanism rather than adding a distinct
post-`render()` text pass. A `SpacingEdit`'s `replacement` is already allowed to differ in
length from `end - start` (existing convention, e.g. operator-spacing's own space-insertion
edits) — a wrap point becomes a zero-width insertion edit at a chosen comma's end position,
`(line, comma_end, comma_end, "<line-terminator><continuation-indent>")`. The per-line rebuild
loop already does `rebuilt.extend(replacement.chars())` (line ~510), which mechanically accepts
an embedded line-terminator character fine — confirmed by reading the actual loop, not assumed.

**Real gotcha, must be a requirement, not an afterthought**: the per-line loop appends that
line's own captured `terminator` (CRLF or LF, captured per original line at line 485) *after*
every edit's replacement has already been emitted (`out.push_str(terminator)`, line ~551). A
wrap edit's replacement must therefore use *that same line's* captured terminator for its
embedded break, not a hardcoded `\n` — otherwise a CRLF-terminated file would get one
newly-inserted line ending in bare `\n` while every other line in the same file still ends in
`\r\n`, a real, silent line-ending-consistency bug this feature must not introduce.

**Second real gotcha**: `indent_plan` is keyed by *original* source line number (research §1
above). A wrap-inserted continuation line is synthetic — it has no entry in `indent_plan` at
all, because it didn't exist in the original source. Its indentation must therefore be computed
independently inside the wrap edit's own replacement string (one level deeper than the
statement's own opening line, per FR-006, using whatever `indent_width` is already resolved to
for this run) — it cannot be retrofitted through `indent_plan`'s existing per-line mechanism.

## §2: `FormatOptions`/`FormatConfig` — exact mirroring pattern to follow

`FormatOptions` (`format.rs:173-232`) is a flat struct; `drut-config::FormatConfig`
(`crates/drut-config/src/lib.rs:32+`) mirrors every field name exactly, each wrapped in
`Option<T>`. Resolution per field is `explicit.or(config.format.X).or(client_defaults.X)
.unwrap_or(BUILT_IN_DEFAULT)` (`lib.rs:233-248`) — `resolve_blank_line_cap` is the existing
numeric-field-with-range-validation helper this feature's width field should reuse the shape
of, not reinvent.

Three new fields, mirroring `blank_lines`/`blank_lines_top_cap`/`blank_lines_nested_cap`'s
three-field precedent exactly:
- `line_wrap: LineWrapMode` (`Preserve` `#[default]` / `Auto`) — mirrors `BlankLineMode`'s shape.
- `line_wrap_width: u16` — `u16`, not `u8` like the blank-line caps: a realistic width range
  (double/triple digits, potentially over 255 for an unusually wide house style) doesn't fit
  comfortably in `u8`, unlike a blank-line-run cap which realistically never exceeds a few dozen.
- `line_wrap_style: LineWrapStyle` (`Fill` `#[default]` / `OnePerLine`) — per spec.md's resolved
  Q2, `Fill` is the default, not `OnePerLine`.

## §3: `operator_spacing.rs` is the module shape to mirror

A sibling module, not inlined in `format.rs`, of `pub(crate)` pure functions taking
`&Statement`/`&[Token]` plus the source's char-lines and pushing into a caller-supplied
`&mut Vec<SpacingEdit>` — no self-contained entry point; `format.rs::render` calls the pieces
directly and does its own dedup/merge against the other edit sources. A new `line_wrap.rs`
follows this identical shape: a pure `collect_wrap_edits(stmt, char_lines, options) ->
Vec<SpacingEdit>`-style function (or pushing into a caller-supplied `&mut Vec`, matching
whichever exact calling convention `operator_spacing.rs`'s own functions use), with zero new
architecture invented.

## §4: Rendering unit — reuse `build_statements`'s flat list, not `Node`/`Block`

`build_statements(tokens.clone())` (called at `format.rs:416`) is the flat statement list
operator-spacing already uses specifically because `Block`'s own `opener_pairs`/`opener_tokens`
retain only keyword-adjacent spans for a block-opener's own pairs, not full inter-token spacing
for arbitrary use — the flat `Statement.tokens` list is what actually carries every token's
original position for a `Control` statement's pair list. This feature reuses that exact same
flat list to locate each `Control` statement's top-level comma tokens, rather than walking
`nodes`/`Block` at all.

**Top-level-only requirement**: a comma inside a function call's parentheses or a bracketed
subscript must never be an eligible split point (spec.md FR-003). Implementation must track
paren/bracket depth across `Statement.tokens` (incrementing on `(`/`[`, decrementing on `)`/`]`)
and only consider a `,` `Punctuation` token eligible when depth is zero at that point in the
statement's own token stream — the same "walk the flat token list, track nesting depth" shape
`block_resolution.rs` and other structural code in this project already use elsewhere, applied
here to punctuation instead of block openers/closers.

**Comma-inside-a-quoted-string IS a separate case to guard against — corrected during
implementation**: this research originally claimed a string literal lexes as one atomic token,
reasoning by analogy from `operator_spacing.rs`'s own quoted-literal-safety precedent. Direct
testing during implementation proved this wrong: `'a, b'` lexes as separate `'`/`a`/`,`/`b`/`'`
tokens in this grammar — there is no atomic string token at all. This is the *exact* same
problem `operator_spacing.rs` already solved (its own research.md §9 documents the identical
discovery for operator characters), so `line_wrap.rs` reuses that module's own
`quoted_token_mask` function directly (`pub(crate)`, already exported for this purpose) rather
than duplicating quote-tracking logic — a masked (inside-a-string) `Punctuation` token, comma or
otherwise, is excluded from split-point collection *and* from paren/bracket depth-tracking
(a stray `(`/`)` inside a quoted value must not corrupt depth accounting either). Verified with
a dedicated test, not assumed.

## §5: CLI/MCP wiring pattern

CLI (`crates/drut-cli/src/cli.rs:85-119`, `format_cmd.rs:106`): mode flags are
`#[arg(long, value_enum)] Option<EnumArg>`; numeric companions are `#[arg(long, value_parser =
clap::value_parser!(u8).range(1..=50))] Option<u8>`. This feature mirrors both patterns: two
`value_enum` flags (`--line-wrap=<preserve|auto>`, `--line-wrap-style=<fill|one-per-line>`) and
one ranged numeric flag (`--line-wrap-width`, `u16`-ranged — exact range a planning-phase
decision, e.g. `20..=500`, wide enough to cover any real house style without accepting a
nonsensical value like `0` or `3`).

MCP: `crates/drut-config`'s single resolution function already serves CLI/LSP/MCP identically,
so the MCP `format` tool almost certainly needs only matching `Option<String>`/`Option<u16>`
params threaded into the same `FormatConfig::Explicit`-shaped construction every other
multi-field option already uses — confirm this narrow claim directly against the actual
`drut-mcp` source during implementation (T-level task), not assumed further here.

## §6: Golden fixture convention

`crates/voyager-core/tests/fixtures/` has one subdirectory per formatter feature/mode
combination — `golden_operator_spacing_fixed/`, `golden_operator_spacing_auto/`,
`golden_blank_lines/`, `golden_casing_function_calls/`, `golden_data_references/`,
`golden_normalize/`. Given this feature has two independently meaningful modes for its output
shape (`Fill` vs. `OnePerLine`), two golden subdirectories are added:
`golden_line_wrap_fill/` and `golden_line_wrap_one_per_line/`, mirroring
`operator_spacing`'s exact `_fixed`/`_auto` two-subdirectory precedent rather than inventing a
new naming shape.

**Post-implementation correction (T027)**: at the default width (120), both golden directories
are byte-identical to the plain `golden/real_corpus/` baseline — the current 9-file real corpus
contains zero eligible wrap targets. Every comma-bearing line over 120 chars in the corpus is one
of: a comment, an `Assignment` statement's arithmetic/string expression (out of v1 scope by
design), or a `Control` statement that a human author already hand-wrapped with a trailing
continuation character (correctly left untouched by the `already_continued` guard). This was
verified by hand, line by line, against every such line the corpus contains — it is not an
artifact of a bug in `top_level_split_points`/`plan_wrap`. The golden fixtures are accepted as-is;
they validate "line_wrap=auto causes no unintended side effects on real content," which is a
real and useful guarantee even though the corpus happens not to exercise an actual wrap. Actual
wrap-output correctness (Fill packing, OnePerLine, CRLF, idempotence) is instead demonstrated by
the 13 hand-crafted unit tests in `format.rs`'s `// -- 030-auto-line-wrap` section, which is where
this feature's positive wrapping behavior is actually proven.
