# Quickstart: Validating Automatic Line-Width Wrapping

A runnable validation guide, not an implementation walkthrough — proves this feature against
spec.md's Success Criteria. See `contracts/line-wrap.md` for the exact type/config shape and
`data-model.md`/`research.md` for the full design rationale.

## Prerequisites

- Rust stable toolchain.

## 1. Build

```powershell
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 2. `voyager-core` `line_wrap` unit tests — validates FR-003, FR-004, FR-005, research.md §4

```powershell
cargo test -p voyager-core line_wrap
```

Expected: all green, including —
- A comma inside a function call's parentheses or a bracketed subscript is never an eligible
  split point.
- A comma inside a quoted pair-value is never surfaced as a split point at all (spot-checked
  directly, not assumed from tokenizer behavior alone).
- A statement already containing a `ContinuationMarker` produces zero wrap edits, regardless of
  its width.
- A `Control` statement with no eligible comma, or a non-`Control` statement, produces zero wrap
  edits.
- An under-width `Control` statement produces zero wrap edits.
- `Fill` packs multiple pairs per continuation line up to the width budget; `OnePerLine` places
  exactly one pair per line.

## 3. `format.rs` terminator/indentation tests — validates data-model.md §1-2

```powershell
cargo test -p voyager-core format::tests -- line_wrap
```

Expected: all green, including —
- A CRLF-terminated input file's newly-inserted continuation line ends in CRLF, not a bare `\n`.
- A newly-inserted continuation line's indentation is one level deeper than the statement's own
  opening line, correct even though `indent_plan` has no entry for that synthetic line.

## 4. Golden fixtures + idempotence — validates SC-001, SC-002, SC-004

```powershell
cargo test -p voyager-core --test format_corpus
cargo test -p voyager-core --test format_sequence
```

Expected: all green, including —
- `golden_line_wrap_fill/` and `golden_line_wrap_one_per_line/` fixtures match hand-verified
  expected output exactly.
- A dedicated second-pass fixture proves a once-wrapped statement is left untouched on
  reformatting (not a generic re-run-and-diff check alone).
- The full existing golden-fixture set is byte-identical to before this feature existed when no
  `line_wrap` configuration is supplied (SC-003).

## 5. `drut-config`/`drut-cli`/`drut-mcp` — validates FR-002, FR-009, SC-005

```powershell
cargo test -p drut-config
cargo test -p drut-cli --test format_flags
cargo test -p drut-mcp
```

Expected: all green, including —
- `line_wrap`/`line_wrap_style` accept only their documented values as an exact-lowercase-string
  match in `drut.toml` (same case-sensitive shape `casing`/`indent_top_level`/`operator_spacing`
  already use — see `drut-config/src/parse.rs`'s note on this); an invalid value degrades to that
  field's built-in default with a non-blocking notice.
- `line_wrap_width` accepts only its valid range in `drut.toml`; an out-of-range value degrades
  the same way.
- `--line-wrap`/`--line-wrap-width`/`--line-wrap-style` and their MCP equivalents reject an
  invalid value with a clear usage/tool error at that surface's own input point.

## 6. Full workspace re-proof

```powershell
cargo test --release --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Mapping back to spec.md Success Criteria

| Step | Success Criterion |
|---|---|
| 2, 3 | FR-003, FR-004, FR-005, FR-006 |
| 4 | SC-001, SC-002, SC-003, SC-004 |
| 5 | FR-002, FR-009, SC-005 |
| 6 | All, integration re-proof |
