# Phase 0 Research: Drut CLI — `check` and `format` Subcommands

The feature description and spec.md's Assumptions section already resolved this
feature's three explicit open questions (exit codes, casing default, SARIF-vs-text
default) before this phase started. What remains here is technical: where formatting
logic lives, and which crates to depend on for traversal/argument-parsing/SARIF/diff.
No `NEEDS CLARIFICATION` markers remain in the Technical Context.

## 1. Where formatting-decision logic lives

- **Decision**: Add `format`/`format_bytes` as new public entry points on
  `voyager-core` itself (mirroring `parse`/`parse_bytes`'s shape), in a new
  `crates/voyager-core/src/format.rs` module. The CLI's `format` subcommand calls
  this and only handles stdout/`--write`/`--check`/`--diff` disposition of the
  result.
- **Rationale**: Constitution Principle I scopes "grammar/parsing logic" to
  `voyager-core` and forbids any adapter from re-implementing it. Whitespace
  normalization isn't itself a grammar *rule*, but computing it correctly requires
  exactly the same structural knowledge parsing already produces — block-nesting
  depth (for indentation), where a statement's continuation actually ends (FR-013's
  "must not change which lines are continuations"), and comment/token boundaries.
  Implementing that in the CLI would mean either (a) re-deriving this structural
  knowledge independently of `voyager-core` — a second, adapter-local structural
  analysis exactly of the kind Principle I forbids and that risks drifting from the
  parser's own notion of block nesting — or (b) having the CLI walk
  `voyager-core`'s already-returned `ParseResult`/`Token` data and apply rendering
  rules to it outside the crate that defines what that data means. Keeping the
  renderer inside `voyager-core`, next to the types it operates on, avoids both.
  It also means the golden-file/idempotency/behavior-preservation test suite
  (constitution Principle III) can run at the `voyager-core` test layer — faster,
  and testing the actual decision logic directly rather than through a spawned CLI
  process — with the CLI layer only needing to test that it wires flags to the
  right disposition of the string `voyager-core` hands back.
- **Alternatives considered**:
  - Formatting logic lives entirely in the CLI crate, treating `voyager-core`'s
    token/statement stream as a generic input. Rejected: this is the "adapter
    re-implements structural logic" case Principle I exists to prevent, and would
    force the CLI to independently track nesting depth and continuation boundaries
    that `voyager-core`'s `Block`/`Statement` types already carry.
  - A brand-new third crate (`crates/drut-fmt`), matching the constitution's
    Technology & Architecture Constraints list which names "a formatter" as its own
    adapter alongside CLI/LSP/MCP. Rejected *for this phase*: the feature
    description frames `format` as a CLI subcommand, not a standalone tool with its
    own invocation surface, and splitting the crate now would add a workspace
    member with no independent consumer yet. If a non-CLI consumer of formatting
    (e.g. the future LSP server's format-on-save) needs it later, the logic already
    lives in `voyager-core` and is trivially reachable from a new crate too — this
    decision doesn't foreclose that, it just doesn't build the extra crate
    speculatively today.

## 2. Argument/subcommand parsing

- **Decision**: `clap`, derive API, for the `check`/`format` subcommands and their
  flags (`--format`, `--write`, `--check`, `--diff`, `--casing`).
- **Rationale**: This is ordinary CLI plumbing with zero overlap with Voyager
  grammar/parsing — exactly the kind of dependency the spec's Assumptions section
  already calls acceptable for this crate (unlike `voyager-core`'s hard
  zero-dependency rule, which is scoped to the core crate specifically). `clap`'s
  derive API keeps the flag surface declared once, in one place, and its built-in
  usage-error handling covers the "`--casing` given without/with an invalid value"
  edge case (spec Edge Cases) for free.
- **Alternatives considered**: Hand-rolled `std::env::args()` parsing — rejected as
  pure reinvention with worse error messages and no help-text generation, for a
  problem with no grammar-adjacent complexity to justify hand-writing it.

## 3. Directory traversal and `.gitignore` handling

- **Decision**: The `ignore` crate (the same one `ripgrep` is built on) for
  recursive, `.gitignore`-aware directory walking (FR-002), filtering to `.s`/
  `.block` extensions in the walker's own filter step (FR-003).
- **Rationale**: Correctly implementing `.gitignore` semantics (nested `.gitignore`
  files, negation patterns, precedence rules) by hand is a substantial, well-solved
  problem with no relationship to Voyager grammar — a textbook case of "duplicated
  effort with no grammar/parsing content" the spec's Assumptions section already
  flags as acceptable to depend on rather than hand-write. Confirmed current and
  actively maintained on crates.io during this research pass (millions of monthly
  downloads via `ripgrep`'s own usage).
- **Alternatives considered**: `walkdir` + hand-rolled `.gitignore` parsing —
  rejected because the `.gitignore` matching semantics are the hard, error-prone
  part `ignore` already solves; `walkdir` alone doesn't help with FR-002's actual
  requirement.
- **Confirmed during T005 implementation**: `WalkBuilder`'s `require_git` option
  defaults to `true` — a `.gitignore` file only takes effect when a `.git` (or
  `.jj`) directory is found somewhere in its parent chain, exactly mirroring real
  `git`'s own behavior (a bare `.gitignore` with no repository has no effect on
  `git status` either). Left at its default rather than overridden to `false`,
  since FR-002's own wording ("the same way `git` itself would decide") asks for
  this, not a looser "any `.gitignore` file anywhere, repo or not" interpretation.
  `tests/traversal.rs`'s `.gitignore` case creates a bare `.git` directory (no
  real repo needed) to exercise this.

## 4. SARIF output

- **Original decision (superseded during implementation, see below)**: `serde` +
  `serde-sarif` for typed SARIF 2.1.0 structures.
- **Superseded 2026-08-09, during T001-T003**: `serde-sarif`'s build script
  (code-generated types via `schemafy` from a bundled JSON Schema) was blocked by
  an OS-level Application Control policy on the implementation machine
  (`os error 4551`, confirmed reproducible and specific to this one crate — every
  other pinned dependency, including the `jsonschema` dev-dependency used for
  SC-003 validation, builds cleanly with no build script involved).
- **Revised decision**: Hand-write a minimal `#[derive(Serialize)]` struct set in
  `drut-cli` covering exactly the SARIF 2.1.0 shape `contracts/sarif-mapping.md`
  specifies (`run.tool.driver.{name,rules}`, `results[].{ruleId,level,message,
  locations}`), serialized via plain `serde_json`. No `serde-sarif` dependency.
- **Rationale**: The reason `serde-sarif` was originally chosen — avoid
  hand-assembling JSON that could silently drift from the schema — is still
  honored: a hand-written `#[derive(Serialize)]` struct is exactly as typed as a
  generated one for the narrow subset of the schema this feature actually emits,
  and SC-003's real guarantee was always going to come from independently
  validating the *emitted* JSON against the official schema via `jsonschema` in
  tests, not from trusting either crate's types unchecked. The build-script
  blocker removes a nice-to-have (not having to hand-write the struct shape) but
  doesn't weaken the actual correctness guarantee, since that guarantee was never
  "trust the crate," it was always "verify the output."
- **Alternatives considered**: Hand-built JSON via `serde_json::json!` (still
  rejected, same reasoning as the original decision — untyped assembly is the
  easy-to-drift risk a struct removes); investigating a fix/workaround for the
  Application Control block itself — out of scope for this feature and not
  something a spec-level decision can resolve, since it's a property of the
  specific build machine, not the crate or the code. See
  `docs/known-environment-quirks.md` for the general pattern (it recurred with an
  unrelated crate during later `.gitattributes` verification work) — that's the
  durable home for this machine-property note going forward, not this file.

## 5. Diff generation for `format --diff`

- **Decision**: The `similar` crate for unified-diff output (FR-019).
- **Rationale**: A correct, readable unified diff needs a real diff algorithm
  (Myers or similar) to produce minimal, human-readable hunks — not something worth
  hand-writing for a CLI flag with no grammar-adjacent complexity. `similar` is
  itself dependency-free and widely used (confirmed current on crates.io during
  this research pass), so it doesn't drag in its own dependency tree.
- **Alternatives considered**: A naive line-by-line comparison — rejected; on a
  multi-hundred-line real script (see `001-voyager-script-parser`'s plan.md scale
  note), a non-minimal diff would bury the actual whitespace changes in noise,
  undermining `--diff`'s whole purpose of letting a user see exactly what would
  change before trusting `--write`.

## 6. Dependency versions and security posture (confirmed 2026-08-09)

- **Decision**: Pin the following as of this research pass — re-confirm exact pins
  at implementation time via `cargo add`/`cargo update`, since `Cargo.lock` doesn't
  exist yet for `drut-cli`:

  | Crate | Confirmed latest | Role |
  |---|---|---|
  | `clap` | 4.6.6 (derive feature) | §2 |
  | `ignore` | 0.4.33 (requires Rust ≥ 1.88) | §3 |
  | `serde` | 1.0.229 (derive feature) | §4 |
  | `serde_json` | 1.0.151 | §4 |
  | `serde-sarif` | 0.8.0 | §4 |
  | `similar` | 3.1.2 (requires Rust ≥ 1.85) | §5 |
  | `jsonschema` (dev-dependency) | 0.49.8 (requires Rust ≥ 1.85) | §4/§7 |

- **Security advisory check**: Queried the RustSec advisory database
  (`rustsec/advisory-db`, per-crate directory listing) for all seven crates above
  on 2026-08-09. None has a `crates/<name>/` directory in the database, meaning
  **zero open (or historical) RUSTSEC advisories are on record for any of them** as
  of this date. This is a point-in-time result, not a standing guarantee — it
  covers these crates directly, not their own transitive dependency trees (which
  don't exist to inspect yet, since no `Cargo.lock` has been generated for
  `drut-cli`).
- **MSRV note**: `ignore` (≥1.88), `similar` (≥1.85), and `jsonschema` (≥1.85) each
  impose a minimum Rust version above `voyager-core`'s current "no nightly, no
  stated floor" baseline. Since the project already tracks current stable and
  states no explicit MSRV commitment (`001-voyager-script-parser/plan.md`
  Technical Context), this isn't a conflict — just worth recording in case an MSRV
  policy is adopted later.
- **Standing recommendation, not a one-time check**: Add `cargo audit` (or
  `cargo deny check advisories`) as a CI step once `drut-cli`'s `Cargo.lock` exists,
  so advisories filed *after* this research pass — against these crates or their
  transitive dependencies — surface automatically rather than requiring another
  manual pass like this one.

## 7. CLI-level test strategy (avoiding duplicate coverage)

- **Decision**: `voyager-core`'s new `tests/format_corpus.rs` is the sole place
  formatting *correctness* (idempotency, structural equivalence, golden-file diffs)
  is asserted. `drut-cli`'s own tests assert only traversal/filtering, exit-code
  selection, SARIF schema validity, and flag-driven file-I/O behavior (default/
  `--write`/`--check`/`--diff`), plus one full-corpus end-to-end smoke test
  (`tests/fixture_corpus_e2e.rs`) that runs the actual built `drut` binary to
  reproduce SC-001 (161/161 clean) through the CLI specifically, per the spec's
  Definition of Done ("must reproduce that same result end-to-end through the CLI,
  not regress it").
- **Rationale**: Re-asserting formatting/parsing correctness at the CLI layer (by
  spawning the binary against the full corpus for every property FR-013/FR-014
  already require) would be slow (process-spawn overhead × 161 files × N
  properties) and redundant with what `voyager-core`'s own suite already proves —
  it would also blur which crate is actually responsible for a failure when a test
  breaks. One full-corpus CLI-level smoke test is kept specifically because the
  Definition of Done calls out CLI-level reproduction of SC-001 as its own
  requirement (traversal + byte-reading + `parse_bytes` call + exit-code wiring
  could each independently break even if `voyager-core` itself is correct).
- **Alternatives considered**: Running the full golden-file/idempotency suite again
  at the CLI level — rejected as redundant per the rationale above; a real defect
  in `format_bytes` would already be caught at the `voyager-core` layer, and a CLI-
  level-only failure would necessarily be a traversal/I/O/flag-wiring bug, which
  the narrower CLI test set already targets directly.

## 8. `Block.closer` — a necessary, additive amendment to `001`'s data model (discovered during T020)

- **Decision**: Add `pub closer: Option<Span>` to `voyager-core`'s `Block` struct
  (`001-voyager-script-parser/data-model.md` § Block, amended in place — see that
  file's entry), populated wherever `block.rs` matches an explicit closing
  statement, `None` for an implicit close (`Run`/`Process`) or a genuinely
  unmatched block.
- **Rationale**: FR-012's closer-alignment rule (explicit closers align to their
  opener's column, 99.2% real-world agreement) needs to know, per block, whether
  an explicit closer statement actually exists and where — but `Block.span.end`
  conflates two different cases that look identical from the outside: a real
  closer statement's span, *or* a fallback to the last child's span when no real
  closer exists (implicit `Run`/`Process` closing, or a genuinely unmatched
  block — see `block.rs`'s `end_span_or` helper, used in exactly the fallback
  case). Getting this wrong would have meant the formatter either double-indents
  a `Run`/`Process` block's last body line (once as an ordinary body statement,
  again as a misidentified "closer") or silently corrupts indentation for two of
  the most common block kinds in real Voyager scripts. The only two ways to get
  this information were (a) re-derive block-matching/implicit-closing logic
  independently in the CLI or formatter — exactly the grammar-logic duplication
  constitution Principle I forbids — or (b) have `voyager-core` itself expose the
  fact it already computes internally while matching blocks. (b) is the only
  option consistent with Principle I.
- **Verified non-breaking**: grep-confirmed no code outside `block.rs` constructs
  a `Block` struct literal, and no code anywhere destructure-pattern-matches all
  of `Block`'s fields exhaustively — both would be compile errors from adding a
  field, neither occurs. `cargo test -p voyager-core` (60 unit + 8 fixture-corpus
  tests) and `cargo clippy -p voyager-core --all-targets` both stayed green
  after the change, with zero other lines touched.
- **Alternatives considered**: Computing "was this closer explicit" via a
  separate, parallel pass over the token stream in the CLI/formatter (matching
  `ENDIF`/`ENDLOOP`/etc. keywords again) — rejected as the exact re-implementation
  Principle I exists to prevent, and strictly worse (two independent
  implementations of the same fact, which can drift) than exposing what
  `voyager-core` already knows.
