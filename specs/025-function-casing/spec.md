# Feature Specification: Function-Call Casing Normalization

**Feature Branch**: `025-function-casing`

**Created**: 2026-08-18

**Status**: Draft

**Input**: User description: "Should we add casing formatting for built-in functions as a
separate rule, the same way `024-function-call-highlighting` added highlighting for them?
Reuse that feature's 138-name research as the source of truth. Formatter behavior change,
so it needs its own `[format]` config field (following the existing
`casing_control_words`/`casing_pair_keywords`/`casing_data_references` three-way
`Preserve`/`Upper`/`Lower` convention), full CLI/MCP/editor-setting surface, and
Constitution Principle III's idempotence/behavior-preservation/golden-fixture discipline —
unlike `024`, which was cosmetic-only and added no new surface."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A script author normalizes built-in function-name casing across a file (Priority: P1)

A script author runs `drut format` (or saves a file with format-on-save enabled) on a
script that calls Cube Voyager built-in functions in inconsistent casing —
`replacestr(...)` in one place, `REPLACESTR(...)` in another, `RightStr(...)` elsewhere.
With a casing convention configured for function calls, every recognized function name
call renders in the same chosen casing, the same consistency `casing_control_words`
already gives control words like `IF`/`ENDIF`.

**Why this priority**: This is the entire content of the request — without it, there is no
feature.

**Independent Test**: Format a fixture containing `replacestr(...)`, `REPLACESTR(...)`, and
`RightStr(...)` with `casing_function_calls = "upper"`. Confirm all three render
`REPLACESTR`/`RIGHTSTR` (uppercase), and re-running format on the output makes no further
change (idempotence).

**Acceptance Scenarios**:

1. **Given** `casing_function_calls = "upper"` and a line reading
   `RouteName = replacestr(RouteName,'-','',0)`, **When** the file is formatted, **Then**
   `replacestr` renders as `REPLACESTR`; the string arguments (`'-'`, `''`) are untouched.
2. **Given** `casing_function_calls = "lower"` and a line reading
   `if (RIGHTSTR(TRIM(RouteName),1)='-')`, **When** the file is formatted, **Then** both
   `RIGHTSTR` and `TRIM` render lowercase (`rightstr`, `trim`).
3. **Given** `casing_function_calls` unset (defaults to `preserve`, matching every other
   casing category's own default), **When** the file is formatted, **Then** every function
   call's casing is byte-identical to the input — no change at all.
4. **Given** a file already formatted under `casing_function_calls = "upper"`, **When** it
   is formatted again, **Then** the second pass produces zero edits (idempotence,
   Constitution Principle III).

---

### User Story 2 - A coincidentally-named pair-keyword or control word keeps its own casing category (Priority: P2)

Two real Cube Voyager names are dual-purpose: `FORMAT` is both a `FILEO` pair-keyword
(`FORMAT=CSV`) and a built-in function (`FORMAT(volume,8,2,',')`); `LOG` is both a control
statement word (`LOG VAR=...`) and a built-in function (`LOG(x)`, natural logarithm). A
script author configuring different casing conventions for different categories (e.g.
`casing_pair_keywords = "upper"` but `casing_function_calls = "lower"`) needs each
occurrence's casing governed by which role it's actually playing in that specific
statement, not by its spelling alone.

**Why this priority**: Protects against a regression the position-gated design must get
right for the feature to be trustworthy — secondary to Story 1, but a real, evidenced case
(not hypothetical), found by cross-checking the 138-name function list against
`voyager-core`'s existing `PAIR_KEYWORDS`/control-word vocabulary.

**Independent Test**: Format a fixture containing `FILEO FORMAT=CSV` (a pair-keyword
occurrence) and `X = FORMAT(volume,8,2,',')` (a function-call occurrence) with
`casing_pair_keywords`/`casing_function_calls` set to different conventions from each
other. Confirm each occurrence's casing follows its own category's convention.

**Acceptance Scenarios**:

1. **Given** `casing_pair_keywords = "upper"` and `casing_function_calls = "lower"`, and a
   line reading `FILEO format=csv`, **When** the file is formatted, **Then** `format`
   (the pair-keyword name) renders `FORMAT`; `csv` (its value, `#pair-values`/
   `data_references` territory, not this feature's concern) is unaffected by this
   feature's own rule either way.
2. **Given** the same settings, and a line reading `X = format(volume,8,2,',')`,
   **When** the file is formatted, **Then** `format` (the function call) renders `format`
   (already lowercase, `casing_function_calls = "lower"` keeps it so).

---

### Edge Cases

- What happens to a function-shaped substring inside a quoted string, e.g.
  `PRINT LIST='calling replacestr(x) here'`? It is never rewritten — the same
  quote-safety guarantee `data_references`' own casing pass already documents for itself
  (`data_reference.rs`'s module docs).
- What happens to a recognized function name with no following `(` — e.g. a bareword, or a
  `keyword=value` pair name that happens to spell a recognized function name (`MAX=100`)?
  It is untouched by this feature's rule; casing for that occurrence (if any) is owned by
  whichever category actually recognizes that position (`casing_pair_keywords` for a
  pair-keyword name, or no casing rule at all for a bare, unrecognized-position word) —
  mirrors `024`'s own FR-006/User Story 2 exactly.
- What happens to a function name written with intervening whitespace before its `(`, e.g.
  `replacestr (x)`? Real vendor/corpus usage never writes it that way (`024`'s
  `research.md` §6); this feature does not rewrite the spacing (that is
  `operator_spacing`'s territory, unaffected here) and does not treat this as a function
  occurrence for casing purposes, matching `024`'s own position-gating exactly.
- What happens to a name recognized by more than one category by coincidence (`FORMAT`,
  `LOG` — confirmed real, not hypothetical: cross-checked directly against
  `voyager-core::keywords.rs`'s `PAIR_KEYWORDS`/`CONTROL_WORDS`)? Each occurrence's own
  structural position (`(` immediately follows → function call; `=` immediately follows →
  pair-keyword; leading word of a statement → control word) decides which category's
  casing rule applies to that specific occurrence — never both, never neither, the same
  single-ownership discipline `data_references`' own module docs already establish for its
  overlap with `pair_keywords`/`control_words`.
- What happens to a real Cube Voyager built-in function not yet in the 138-name list? Its
  casing is left untouched, exactly as before this feature exists — matches `024`'s own
  FR-004 non-exhaustive stance; not a claim the function is unsupported.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `voyager-core` MUST expose a fourth, independently-configurable casing
  category, `function_calls`, alongside the three `017-casing-categories-indent-width`
  already established (`control_words`, `pair_keywords`, `data_references`) — same
  `CasingConvention` enum (`Preserve`/`Upper`/`Lower`), same `Preserve`-is-default shape.
- **FR-002**: A token MUST be treated as a `function_calls`-category occurrence only when
  its text case-insensitively matches one of the 138 recognized function names (ported
  from `specs/024-function-call-highlighting/research.md` §2 into `voyager-core` as the
  single source of truth — Constitution Principle I) AND it is immediately followed by `(`
  with zero intervening whitespace — the same position-gating `024` already established
  for highlighting, required here for correctness, not merely consistency (see spec Edge
  Cases: `FORMAT`/`LOG` are real, evidenced dual-category names).
- **FR-003**: Matching MUST skip any occurrence inside a quoted string, mirroring
  `data_reference.rs`'s existing quote-tracking (FR-003's own Edge Cases).
- **FR-004**: A token already claimed by `control_words`, `pair_keywords`, or
  `data_references` for a given occurrence (by that occurrence's own structural position)
  MUST NOT also be queued for `function_calls` casing at the same occurrence — single
  ownership per occurrence, the same discipline `data_references`' overlap with
  `pair_keywords`/`control_words` already establishes; this is naturally satisfied by
  FR-002's `(`-immediately-follows requirement (a pair-keyword name is followed by `=`, a
  control word by whitespace, never simultaneously by `(`), and MUST be verified, not just
  assumed.
- **FR-005**: The formatter MUST remain idempotent with `function_calls` casing applied
  (`format(format(x)) == format(x)`, Constitution Principle III) and MUST NOT alter
  program meaning — only the casing of the function-name token itself changes; arguments,
  spacing, and surrounding tokens are unaffected by this rule.
- **FR-006**: This category MUST be reachable through the same four surfaces every other
  casing category already is: a `drut.toml` `[format]` field (`casing_function_calls`), a
  CLI flag (`--casing-function-calls`), the MCP `format` tool's parameters
  (`casing_function_calls`), and a VS Code client setting
  (`drut.format.casingFunctionCalls`) — naming pattern matches the existing three
  (group-word-leads convention, per `CHANGELOG.md`'s `[Unreleased]` rename precedent).
- **FR-007**: This feature MUST NOT change `voyager-core`'s tokenizer, parser, grammar
  model (`Statement`/`Block` kinds), or any `Diagnostic` category — a `function_calls`
  casing occurrence is recognized by a read-only pass over already-parsed data, the same
  architectural shape `data_reference.rs`'s module docs describe for itself, not a new
  structural AST concept.
- **FR-008**: Every formatter change to this category MUST be verified against the
  fixture corpus with a golden-file diff before merge (Constitution Principle III;
  `crates/voyager-core/tests/format_corpus.rs`'s existing harness).
- **FR-009**: `editors/vscode/syntaxes/drut.tmLanguage.json` (the `024`-shipped
  `#function-calls` highlighting pattern) is NOT modified by this feature — highlighting
  and casing are independent concerns already correctly separated by `024`'s own scope
  decision; this feature only makes the *list itself* (currently duplicated as a
  manually-synced JSON copy) canonical inside `voyager-core`, the same relationship
  `#control-words` already has with `statement.rs`'s `FIXED_KEYWORDS`.

### Key Entities

- **Recognized function name list**: the 138-name, category-grouped list from
  `024-function-call-highlighting/research.md` §2, ported into `voyager-core` (exact
  module/location decided in `plan.md`/`research.md`) as the canonical source; `editors/
  vscode`'s grammar JSON becomes a documented, manually-synced mirror of it (matching the
  existing `#control-words`/`FIXED_KEYWORDS` relationship), not the other way around.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: With `casing_function_calls` set to `upper` or `lower`, every one of the 138
  recognized function names, when called (`NAME(...)`), renders in the configured casing
  after formatting — verified by an automated test over the complete list, not a sample
  (mirrors `024`'s own SC-001 remediation).
- **SC-002**: `casing_function_calls` unset or `preserve` produces byte-identical output to
  today's formatter for every real fixture in the corpus — zero behavior change for any
  project that doesn't opt in.
- **SC-003**: `format(format(x))` produces zero additional edits for every real corpus
  fixture under every non-`preserve` `casing_function_calls` value (idempotence,
  end-to-end, not just per-token).
- **SC-004**: The two real dual-category names (`FORMAT`, `LOG`) each render under the
  correct casing category for both of their real occurrence shapes, confirmed against
  fixtures exercising both shapes of each name.

## Assumptions

- The 138-name list itself is not re-researched here — `024`'s `research.md` §2 (built
  from a complete reading of two vendor documentation editions, Cube Voyager 6.5.1 and
  OpenPaths Cube/CUBE CONNECT Edition, cross-validated against each other) is reused
  as-is. A function found missing from that list later is fixed there and mirrors forward
  into this feature's casing list the same way (`024` research.md §5's own amendment
  path), not re-litigated per feature.
- Scope is `voyager-core` (the new recognition/casing-edit logic) plus its adapters that
  already expose the other three casing categories: `drut-config`, `drut-cli`, `drut-mcp`,
  and `editors/vscode`'s client-settings passthrough (NOT its highlighting grammar, per
  FR-009). `drut-lsp`'s format-on-save/format-on-paste paths pick this up for free, the
  same way they already do for the other three categories (no LSP-specific work needed
  beyond whatever `drut-config`/`drut-cli` already wire through).
- No new diagnostic category, no new grammar/parsing behavior — this is a rewrite rule
  layered on already-recognized text, the same category of change `data_references`'
  casing already is.
