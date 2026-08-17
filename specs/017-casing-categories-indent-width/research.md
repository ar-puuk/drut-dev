# Research: Per-Category Casing Configuration and Configurable Indentation Width

Consolidates the design decisions this feature needs, most of which were already settled
during this feature's (unusually long) pre-spec design conversation and `ROADMAP.md` items
9–12 — this file exists to make them traceable from the plan, not to re-derive them from
scratch.

## §1. No lexer/`TokenKind` change is needed — a real simplification vs. the original sizing

`ROADMAP.md`'s resolved-queued item 4 originally sized reaching `OPERAND_PREFIX`/
`ASSIGNMENT_TARGET` tokens as requiring "a lexer-level change to expose the pre-`.` boundary"
— comparable in scope to `008`/`009`. Re-checked directly against the current code rather than
trusting that older estimate:

- **Decision**: No new `TokenKind` variant, no lexer change. `data_reference.rs` is a
  read-only recognition pass over already-parsed data, added entirely within `voyager-core`
  (satisfying Principle I the same way `token_resolution.rs`/`block_resolution.rs` already do
  — neither of those required a lexer change either).
- **Rationale**:
  - `StatementKind::Assignment { target: String, .. }` (`statement.rs`) already stores the full
    target text verbatim, brackets included (confirmed by an existing test asserting
    `target == "MW[1]"` for `MW[1] = ...`). No new parsing is needed to reach an
    assignment-target-shaped data-reference token — only a small helper that strips an
    optional trailing `[...]` to get the base name (`"MW[1]"` → `"MW"`) plus the exact byte
    span of just that base-name portion, so a casing rewrite touches only the name, never the
    subscript.
  - Pair-keyword names with a bracket subscript are stored the same way, already — confirmed
    by `keywords.rs`'s own existing `PAIR_KEYWORDS` entries like `pair_entry("MW[201]",
    &["PATHLOAD"])`, which is only possible because the census that produced them already saw
    `"MW[201]"` as one plain string. The same base-name-stripping helper above covers this
    shape too — one helper, two call sites, not two separate pieces of logic.
  - The one genuinely new piece is the dot-notation read shape (`mi.1.1`, `li.FT`) — today a
    single opaque `TokenKind::Word` whose `.text` is the full string including the dots. This
    needs a new check: does a `Word` token's text start with a recognized data-reference
    prefix immediately followed by `.`? This is a text/span computation over an
    already-tokenized value, not a change to where token boundaries fall — nothing else in the
    parser needs to change, and nothing that currently treats `mi.1.1` as one atomic value
    (pair-keyword value collection, `IF`-condition parsing, etc.) is affected.
- **Alternatives considered**: A true lexer-level split of dot-notation tokens (Path (b) from
  the original `ROADMAP.md` investigation) — rejected. It would ripple through the entire
  tokenizer/parser (anything that currently assumes a dotted value is one `Word` token), with
  a golden-fixture review burden `ROADMAP.md` itself already flagged as larger than either
  `008` or `009`'s, to deliver no additional capability over the read-only recognition-pass
  approach.

## §2. Config-surface additivity — every existing surface keeps its exact current meaning

`ROADMAP.md`'s design conversation for this feature repeatedly established (and this plan
holds to) that no already-shipped configuration surface may change meaning:

- **Decision**: `drut.toml`'s flat `casing` field, `--casing`, and the MCP `casing` param all
  keep working exactly as they do today — interpreted as setting **both** `control_words` and
  `pair_keywords` (the two categories they already reached before this feature), never
  `data_references` (which they structurally never could reach). Three new, independent
  fields/flags/params (`control_words_casing`, `pair_keywords_casing`, `data_references_casing`)
  are added alongside, at every surface, for callers who want finer control or who want to
  reach `data_references` at all.
- **Rationale**: Turning `casing` itself into a nested/structured value (e.g. a `[format.casing]`
  subtable) was considered and rejected specifically because it would break every existing
  `drut.toml` with a flat `casing = "upper"` line — this project has never shipped a breaking
  config-file change (`014`'s own `FormatConfig.casing` stayed `Option`-wrapped for exactly this
  reason), and there's no evidence-driven need to start now. Keeping `casing` flat and adding
  three siblings is the same additive pattern `009`→`012`→`014` already established repeatedly.
- **Precedence** (per category, most-specific wins): explicit new granular flag/param >
  explicit legacy `--casing`/`casing` (for `control_words`/`pair_keywords` only) >
  `drut.toml`'s new granular field > `drut.toml`'s legacy `casing` field (for
  `control_words`/`pair_keywords` only) > built-in `Preserve`. `data_references` skips the two
  legacy tiers entirely — full matrix in data-model.md §3.

## §3. `002-cli-check-format/spec.md` amendment content

FR-015 (defines `--casing`'s shape) and FR-026 (the `top_level_indent`-adjacent requirement
that contrasted itself against `--casing`'s "off state") both need a new dated entry —
following the same amendment discipline `009`'s FR-012 and `014`'s FR-011/FR-012 already used
(append a dated note, never silently rewrite the original text). Content: `--casing` (and
`drut.toml`'s flat `casing` field, and the MCP `casing` param) now sets two of three
independently-configurable casing categories rather than "casing" as a single undifferentiated
concept; the three-category shape and the `data_references` category's own scope are defined
by this feature, not restated in `002`'s own text beyond the pointer.

`001-voyager-script-parser/contracts/public-api.md`'s `formatting-api.md` "casing is the only
configurable axis" exclusion statement also needs correcting — indentation width is now
configurable too (`ROADMAP.md` item 9), following the same pattern `009`'s own contract
amendment already used when `top_level_indent` first became configurable.

## §4. `indent_width` validation — the first numeric (non-closed-enum) configurable value

Every configurable value this project has shipped so far (`casing`, `top_level_indent`) is a
closed string enum — a malformed TOML string either matches a known variant or it doesn't,
with a well-established non-blocking-warn-and-fallback pattern for the "doesn't" case.
`indent_width` is the first configurable value that's a bounded number instead, which is a
genuinely new validation shape, not a copy of an existing one:

- **Decision**: `indent_width` accepts any TOML integer, validated at the `drut-config`
  resolve layer (not inside `voyager-core`, which accepts any `u8` its caller passes — the
  bound is a policy choice, not a grammar fact) against a sane range. Carrying forward
  `ROADMAP.md` item 9's own recommendation: **1–16**. Outside that range (including `0` and
  negative values, which TOML permits syntactically), falls back to the built-in default (`4`)
  with the same non-blocking notice every other malformed `[format]` value already produces.
- **`FormatOptions.indent_width` default**: `FormatOptions` currently derives `Default` purely
  from each field's own `Default` impl (`CasingConvention`/`TopLevelIndentMode`'s `#[default]`
  enum variants). `u8::default()` is `0`, not the desired `4`, so `FormatOptions` needs a
  small manual `impl Default` once `indent_width` is added — the first field on this struct
  whose correct default isn't just its type's own `Default::default()`. Considered a newtype
  (`IndentWidth(u8)` with its own `Default` returning `4`, keeping the derive pattern
  everywhere else) instead — rejected as more ceremony than a four-line manual impl warrants
  for one field.

## §5. `keywords.rs` corrections

- **`NUMREC`/`CNT`/`ITER`/`LP`/`RECNUM` removal**: the vendor-documented `LOOP
  <name>=start,end[,increment]` syntax takes a free-form, user-chosen loop-variable name in
  that position — Bentley's own reference-guide examples use `iter`/`INDEX`/`L3`/`_K`/`_L`
  interchangeably in the identical slot. These five were never reserved keywords; the original
  2026-08-10 corpus census (`keywords.rs` module docs) miscategorized them as fixed
  `PairKeyword` entries because it couldn't structurally distinguish "a fixed keyword=value
  pair" from "a user declaring a loop counter" — both produce the same `word=value` shape.
  Removing them also removes them from `completion_candidates`/`did_you_mean` suggestions for
  the `LOOP=` position, which is the actual user-facing correctness fix (suggesting a fixed
  "correct" name for a position that has none was actively misleading).
- **`ZONES` addition**: vendor-confirmed real (`RUN PGM=MATRIX ZONES=3 ...`), and genuinely
  dual-role — a pair-keyword under `RUN`/`RUN PGM=MATRIX` and, separately, an ordinary
  `ZONES = 1`-shaped plain assignment (extremely common in the real corpus). `keywords.rs`
  (completion dictionary) gains only the pair-keyword-shaped entry, `observed_with: ["RUN"]` —
  its assignment-shaped usage is `data_reference.rs`'s concern (FR-008), not this dictionary's,
  matching the existing split of concerns between "what's offered as completion" and "what's
  reachable by casing."

## §6. `data_references` family — finalized recognized-name list

Directly from this feature's own pre-spec vendor-doc research (`_archive/Citilabs Cube 6.5.1/
RG_CUBEVOYAGER.md`, cross-checked against `_archive/OpenPaths Cube/html/`), paraphrased in this
project's own words per constitution Principle II:

| Family | Members | Shape(s) confirmed |
|---|---|---|
| Matrix | `MI`, `MO`, `MW` | `MI`: dot-notation read only. `MO`: pair-keyword parameter only (`MATO=file,MO=1`). `MW`: pair-keyword-shaped (`PATHLOAD ... MW[201]=`), assignment target (`MW[1] = ...`), and bracket-notation read-back (`mw[3]*mw[99]`). |
| Line | `LI`, `LW` | Both dot-notation only; `LW` is written and read back via dot-notation (no bracket form). |
| Node | `NI`, `NW` | Same shape as Line. |
| Zone | `ZI`, `ZONES`, `Z` | `ZI`: dot-notation read only. `ZONES`: pair-keyword-shaped and plain-assignment-shaped (§5). `Z`: a bare field reference (TAZID shorthand in zone data). |
| Database | `DBI`, `DBA` | `DBI`: a `FILEI DBI=` pair-keyword (loader) — that specific shape stays in `keywords.rs`'s own dictionary too, same split as `ZONES`. `DBA`: dot-notation read only, indexed into what `DBI` loaded. |
| Record | `RO` | Dot-notation, write-only (flushed via `WRITE RECO=`). |
| Link-endpoint | `A`, `B` | Bare field reference (from-node/to-node), not part of the I/O-suffix scheme. |
| Implicit loop index | `I`, `J` | Reserved, always-same-name identifiers (the outer zone loop and `JLOOP`'s inner loop respectively) — referenced inline like any variable, never user-renamed (distinct from `LOOP`'s own free-form variable-name slot, §5). |

Matching is case-insensitive (mirrors every other keyword-matching rule in this project); the
canonical uppercase spelling above is what a `KeywordEntry`-style table would store.
Deliberately excluded (per this feature's own design conversation, `ROADMAP.md` item 11): any
split between `MW`/`LW`/`NW`'s own multiple roles — they get one casing value across all their
shapes in this feature, not one-per-role.
