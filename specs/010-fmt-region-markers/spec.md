# Feature Specification: FMT Region Markers

**Feature Branch**: `010-fmt-region-markers`

**Created**: 2026-08-12

**Status**: Draft

**Input**: User description: "; FMT: OFF and ; FMT: ON region markers, a new
drut format feature. Lets users wrap a range of lines in a Voyager .s/.block
script with a `; FMT: OFF` comment and a matching `; FMT: ON` comment, and
have `drut format` leave every line inside that range completely untouched
(indentation, casing, spacing, anything else the formatter would otherwise
normalize) — an escape hatch for hand-tuned formatting the tool would
otherwise 'fix'. This is queued item 2 in ROADMAP.md (added 2026-08-11):
'Lets users mark a line range to be skipped entirely by `drut format`.
Reference 007's diagnosed-block-skip mechanism
(`diagnosed_block_openers`/`plan_block`'s skip-a-diagnosed-block's-children
logic, `specs/007-.../research.md` §1) as an architectural starting point —
not a direct reuse, since that mechanism skips based on diagnosed block
structure, not user-placed markers.'"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Protect a hand-tuned range from reformatting (Priority: P1)

A script author has a block of lines they've deliberately indented or cased
in a way `drut format` would otherwise "correct" — for example, a table of
`ARRAY`/`LOOKUP` value assignments hand-aligned into visual columns, or a
vendor-supplied snippet they don't want touched. They wrap that range with
`; FMT: OFF` above it and `; FMT: ON` below it. Running `drut format`
normalizes the rest of the file as usual but leaves every line from the
`; FMT: OFF` marker through the `; FMT: ON` marker (inclusive) exactly as
written — same indentation, same keyword casing, same everything.

**Why this priority**: This is the entire feature. Without it there is no
escape hatch at all, and every other scenario is a variation on this one
mechanic.

**Independent Test**: Format a fixture with a `; FMT: OFF`/`; FMT: ON` pair
wrapping deliberately "wrong" indentation and casing; confirm the wrapped
lines are byte-identical before and after, while lines outside the pair are
normalized exactly as they would be without any markers present.

**Acceptance Scenarios**:

1. **Given** a file with one `; FMT: OFF`/`; FMT: ON` pair around several
   misindented, mixed-case lines, **When** `drut format` runs, **Then** the
   wrapped lines are unchanged and all other lines are normalized as usual.
2. **Given** a file with no `; FMT: OFF`/`; FMT: ON` markers at all,
   **When** `drut format` runs, **Then** output is identical to the
   feature not existing (no behavior change for files that don't use it).
3. **Given** a file with multiple, non-overlapping `; FMT: OFF`/`; FMT: ON`
   pairs, **When** `drut format` runs, **Then** each protected range is
   left untouched independently and the regions between and around them are
   normalized as usual.

---

### User Story 2 - An unclosed `; FMT: OFF` protects to end of file (Priority: P2)

A script author adds `; FMT: OFF` but forgets (or doesn't intend) to add a
matching `; FMT: ON` before the file ends. Rather than erroring or silently
ignoring the marker, `drut format` treats the protection as extending from
the marker to the end of the file — matching the long-established
convention of equivalent region-marker features in other formatters (e.g.
Python's Black `# fmt: off`/`# fmt: on`).

**Why this priority**: A real, expected authoring mistake (or deliberate
choice, for a trailing hand-tuned section) that must have predictable,
non-surprising behavior — but it's a variation on US1's core mechanic, not
new machinery.

**Independent Test**: Format a fixture containing a `; FMT: OFF` marker with
no following `; FMT: ON`; confirm every line from the marker to the end of
the file is left untouched, and the file formats without error.

**Acceptance Scenarios**:

1. **Given** a file with `; FMT: OFF` and no subsequent `; FMT: ON`,
   **When** `drut format` runs, **Then** every line from the marker to
   end-of-file is unchanged and no diagnostic or error is produced.

---

### User Story 3 - Markers are recognized consistently everywhere formatting happens (Priority: P2)

The same protection applies identically no matter which surface triggers
formatting — the CLI (`drut format`), the LSP's whole-document and
range-formatting handlers (format-on-save, format-on-paste), and the MCP
`format` tool — since all of them call the same `voyager-core` formatting
entry points.

**Why this priority**: Consistent with this project's single-source-of-truth
architecture (constitution Principle I) — protection is a `voyager-core`
concern, not something any one adapter could implement or forget on its
own. Not independently novel work if US1 is implemented correctly in the
core crate, but it must be independently verified at every integration
point, the same way `009`'s default placement was.

**Independent Test**: Format the same fixture containing a protected range
through the CLI, both LSP formatting handlers, and the MCP tool, each with
no special configuration; confirm all four leave the protected range
untouched identically.

**Acceptance Scenarios**:

1. **Given** a fixture with a protected range, **When** formatted through
   the CLI, both LSP handlers, and the MCP tool independently, **Then** all
   four produce the same untouched protected range.

---

### Edge Cases

- A `; FMT: OFF`/`; FMT: ON` pair straddles a block boundary (opens inside
  an `IF` block, closes outside it, or vice versa) — allowed; protection is
  a pure line-range concern, structurally independent of block/statement
  boundaries, and does not affect parsing or diagnostics in any way.
- A second `; FMT: OFF` appears while already inside a protected range —
  treated as a no-op; only the transition from "on" to "off" and back
  matters, not balanced nesting depth (matches Black's `# fmt: off`
  precedent).
- A `; FMT: ON` appears with no preceding `; FMT: OFF` — a no-op; formatting
  was already active, so there is nothing to turn back on.
- The marker lines themselves (`; FMT: OFF` / `; FMT: ON`) are comment-only
  lines, which the existing formatter already never re-indents or
  re-cases (module scope note in `crates/voyager-core/src/format.rs`:
  leading-whitespace changes only ever apply to a statement/block/closer/
  branch's first line) — no special-casing needed to leave the markers'
  own indentation untouched.
- A file consists entirely of one protected range (`; FMT: OFF` on line 1,
  `; FMT: ON` on the last line, or no closing marker at all) — the whole
  file is left byte-identical by `drut format`.
- Marker recognition is whitespace- and case-flexible around `FMT`/`OFF`/
  `ON` and the colon (matching how Voyager comments and keywords are
  already treated elsewhere in this project), but must be the *entire*
  content of a comment-only line — a trailing `; FMT: OFF` after real
  statement content on the same line is not recognized as a marker (avoids
  ambiguity about what part of that line is "the marker" versus "the
  statement").

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `drut format` MUST recognize a comment-only line whose entire
  trimmed content case-insensitively matches `FMT: OFF` (colon, flexible
  surrounding whitespace) as the start of a protected region.
- **FR-002**: `drut format` MUST recognize a comment-only line whose entire
  trimmed content case-insensitively matches `FMT: ON` as the end of the
  nearest currently-open protected region.
- **FR-003**: Every line from a `; FMT: OFF` marker (inclusive) through its
  matching `; FMT: ON` marker (inclusive), or through end-of-file if
  unmatched, MUST be reproduced byte-for-byte identical to the input —
  leading whitespace, keyword/control-word casing, intra-line spacing,
  trailing whitespace, and line-ending style all left untouched, with no
  exception.
- **FR-004**: Lines outside every protected region MUST continue to be
  normalized exactly as `drut format` already normalizes them today —
  this feature MUST NOT change output for any file that contains no
  `; FMT: OFF`/`; FMT: ON` markers.
- **FR-005**: A `; FMT: OFF` encountered while a region is already open
  MUST be a no-op (does not require a second matching `; FMT: ON`); a
  `; FMT: ON` encountered while no region is open MUST be a no-op.
- **FR-006**: Protected-region recognition MUST be independent of block/
  statement structure — a region may open or close at any line, including
  lines that don't align with any block or statement boundary, without
  producing a diagnostic or affecting `parse`'s structural output in any
  way (this is a formatting-only concern; `tokenize`/`parse` are
  unaffected, per Principle I's `tokenize`/`parse`/format` separation).
- **FR-007**: The same protection behavior MUST be available identically
  through every adapter surface that calls `voyager-core`'s formatting
  entry points — CLI, both LSP formatting handlers (whole-document and
  range), and the MCP `format` tool — implemented once in `voyager-core`,
  not duplicated per adapter.
- **FR-008**: Formatting a file twice in succession MUST produce identical
  output on the second pass (idempotency), including for files containing
  protected regions — trivially satisfied by leaving protected content
  fully untouched, but must be explicitly verified, not assumed.
- **FR-009**: `format`/`format_bytes` MUST NOT panic on any input involving
  `; FMT: OFF`/`; FMT: ON` markers, including malformed or unusual marker
  placement (e.g. only a closing marker, many markers in a row, markers
  inside a block comment) — every case in the Edge Cases section above
  MUST have defined, non-panicking behavior.

### Key Entities

- **Protected region**: A contiguous line range in a source file, delimited
  by a `; FMT: OFF` marker line and either a matching `; FMT: ON` marker
  line or end-of-file, within which `drut format` performs no
  transformation of any kind.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A file with any number of non-overlapping protected regions,
  when formatted, reproduces every line inside those regions byte-for-byte
  identical to the input, in 100% of tested cases (hand-written fixtures
  covering every Edge Case above, plus the real 161-file corpus with
  synthetic marker pairs inserted).
- **SC-002**: A file with no `; FMT: OFF`/`; FMT: ON` markers produces
  byte-identical output to the same file formatted before this feature
  existed — zero regressions against the existing golden-fixture corpus.
- **SC-003**: Formatting any fixture containing protected regions twice in
  a row produces identical output on both passes (idempotency holds).
- **SC-004**: The full 161-file real corpus continues to produce zero new
  diagnostics after this feature ships (matching the zero-false-positive
  bar every prior formatting feature in this project has been held to).

## Assumptions

- Marker syntax is `; FMT: OFF` / `; FMT: ON` exactly as the owner
  specified — case-insensitive on `FMT`/`OFF`/`ON`, flexible on whitespace
  around the colon and leading `;`, but the line's entire trimmed content
  (after the `;`) must match — a marker sharing a line with real statement
  content is not recognized, avoiding ambiguity about where the marker ends
  and the statement begins.
- An unclosed `; FMT: OFF` protects through end-of-file rather than
  producing a diagnostic or being ignored — modeled on Python Black's
  `# fmt: off`/`# fmt: on`, the closest well-known prior art for this exact
  mechanic (a general open-source tooling convention, not Cube Voyager
  vendor documentation, so referencing it doesn't implicate constitution
  Principle II).
- No new `Diagnostic` category is introduced for marker misuse (unclosed
  region, redundant marker, etc.) — this is a formatting-only concern
  layered on top of the existing six diagnostic categories, which stay
  exactly as they are; `tokenize`/`parse` output is unaffected by markers
  entirely.
- This feature is `voyager-core`-only new logic (a marker-scan pass plus a
  gate on the existing indent-plan/casing-edit collection, per
  `crates/voyager-core/src/format.rs`'s existing scope) — no new CLI flag,
  LSP capability, or MCP field is needed, since protection is driven
  entirely by in-file markers, not caller configuration.
- Nesting/duplicate markers resolve by simple on/off state transition, not
  balanced-pair counting (a second `; FMT: OFF` while already off, or a
  stray `; FMT: ON` while already on, are both no-ops) — the simplest
  behavior that has no surprising interaction with itself.
