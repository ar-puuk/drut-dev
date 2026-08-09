# Phase 0 Research: Voyager Script Tokenizer & Structural Parser

All items below were either resolved during `/speckit-specify` (the `.block`-vs-`.s`
grammar question, and the three additional statement forms — see spec.md
Assumptions) or are resolved here. No `NEEDS CLARIFICATION` markers remain in the
Technical Context.

## 1. Runtime dependency policy

- **Decision**: The `voyager-core` crate depends only on `std`. No lexer-generator
  (e.g. `logos`), parser-combinator (e.g. `nom`, `pest`), or error-derive crate (e.g.
  `thiserror`) is used.
- **Rationale**: This crate is the constitution's single source of truth for grammar
  and parsing logic (Principle I). Its grammar is simple and line-oriented (comments,
  continuation characters, three block types, four statement forms) — well within
  what a hand-written scanner handles clearly, without a generator's abstraction tax
  or a combinator library's error-type opinions leaking into the one error type every
  downstream adapter (CLI/LSP/MCP/formatter) will match on. Zero dependencies also
  means zero supply-chain surface for the crate every other crate in the project
  depends on.
- **Alternatives considered**:
  - `logos` (lexer generator): fast, but its derive-macro token definitions are a poor
    fit for context-sensitive rules like "continuation depends on the last
    non-comment character" and "`.block` files don't require a top-level `RUN`
    wrapper" — those are easier to express directly in code than to encode in a
    token-regex table.
  - `nom`/`pest` (parser combinators/PEG): would work, but add a non-trivial
    learning-curve and error-reporting model on top of a grammar simple enough not to
    need one; revisit only if hand-written matching becomes unwieldy in practice.
  - `thiserror`: convenient for `Display`/`Error` boilerplate, but the diagnostic
    type here is a plain data record (category + message + location), not a Rust
    `std::error::Error` chain being propagated — a hand-written `Display` impl is a
    few lines and keeps the zero-dependency property.

## 2. Diagnostic representation

- **Decision**: `Diagnostic` is a struct with a `DiagnosticKind` enum (one variant per
  required category: `UnmatchedIf`, `UnmatchedLoop`, `UnclosedBlockComment`,
  `InvalidContinuation`, `UnmatchedRun`, plus room to grow), a `Span`, and a `String`
  message composed in the project's own words at construction time (never a copied
  vendor string).
- **Rationale**: FR-017 requires a category, message, and location at minimum;
  structured categories (not just message strings) let downstream tools (linter, LSP)
  branch on `DiagnosticKind` without parsing text, and keep the "own words" requirement
  (Principle II, FR-024) enforceable by construction — the message is built once,
  in one place, per kind.
- **Alternatives considered**: A single opaque `String` per diagnostic — rejected
  because it forces every downstream consumer to pattern-match on message text, which
  is exactly the brittleness structured diagnostics exist to avoid.

## 3. Fixture corpus sourcing and licensing

- **Decision**: This plan does **not** assume real files from
  `WF-TDM-Official-Releases` (or any other third-party production project) may be
  copied verbatim into this repository's `tests/fixtures/`. That repository was used
  only to *research* real-world grammar shapes during `/speckit-specify` (per its own
  spec.md Assumptions) — no content from it has been copied into this project.
  Before populating `tests/fixtures/valid/`, confirm the actual rights/license under
  which any candidate source scripts may be redistributed in this (public, per
  constitution Principle VIII) repository. Where rights are unclear or absent, use
  hand-written or hand-adapted fixtures that reproduce the *structural shape*
  (nesting patterns, continuation styles, nested `IF`/`LOOP`/`RUN` combinations, the
  four statement forms) observed during research, without reproducing an original
  author's specific business logic, variable names, or comments verbatim.
- **Rationale**: The same "don't redistribute what you don't have the right to"
  concern that motivates Principle II (vendor docs) applies to arbitrary third-party
  script content, even though Principle II's letter is scoped to Bentley/Citilabs
  documentation. Treating it identically avoids introducing a licensing problem while
  building the tool meant to avoid exactly that problem elsewhere.
- **Open follow-up**: Confirm sourcing/licensing for the actual fixture set with the
  project owner before or during implementation; this is a prerequisite for closing
  out FR-025/SC-001/SC-002, not a blocker for designing the parser itself.
- **Resolved, 2026-08-09 (T049)**: The project owner directed copying a curated,
  redaction-checked 9-file subset (~5,200 lines) of `WF-TDM-Official-Releases` into
  `crates/voyager-core/tests/fixtures/valid/real_corpus/` — see that directory's own
  `README.md` for per-file provenance and the redaction check performed before
  copying. This resolves the open follow-up above for that subset specifically.
- **Full-corpus validation (2026-08-09, beyond the committed subset)**: Separately,
  and without copying any additional files into the repository, `voyager-core`'s
  `parse_bytes()` was run read-only against all 161 `.s`/`.block` files in
  `WF-TDM-Official-Releases` (not just the 9-file committed subset) via a throwaway,
  uncommitted example script. Result: **161/161 files parsed with zero diagnostics**,
  and **zero panics** — every call was wrapped in `std::panic::catch_unwind` and
  confirmed rather than assumed not to panic. This is materially stronger evidence for
  SC-001 (zero false positives) than the 9-file subset alone provides, though it isn't
  itself fixture-corpus evidence (research is read-only; T049's committed subset
  remains the actual `tests/fixtures/` corpus SC-001/SC-002 are graded against). The
  same full-corpus pass also surfaced the subscript-assignment-target classification
  gap now tracked as FR-023's amendment (see spec.md; single-subscript targets alone
  appear 6,000+ times in one file, `08_TripTablesByPeriod.s`) and the
  `DistributeINTRASTEP` finding (see spec.md Assumptions) — this validation run is
  what found both.

## 4. RUN PGM nesting — structural vs. semantic scope

- **Decision**: The parser balances `RUN PGM=.../ENDRUN` the same way it balances
  `IF`/`ENDIF` and `LOOP`/`ENDLOOP` — as a nestable block type — without enforcing any
  Cube-specific rule about whether nested `RUN PGM` boxes are semantically legal.
- **Rationale**: Whether Cube Voyager itself permits a `RUN PGM` box to open while
  another is already open is a semantic/domain rule, not a structural balancing rule,
  and semantic checking is explicitly out of scope for this phase (FR-019). If real
  fixtures later show nested `RUN PGM` is always invalid in practice, that becomes a
  candidate lint rule for a later phase (subject to constitution Principle IV: ships
  as a warning until validated against the corpus with zero false positives).
- **Alternatives considered**: Emitting a diagnostic on nested `RUN PGM` now — rejected
  as scope creep into semantic validation this phase explicitly excludes.

## 5. Workspace layout

- **Decision**: Introduce a Cargo workspace at the repo root now, with
  `crates/voyager-core` as its first and only member for this phase.
- **Rationale**: Constitution Technology & Architecture Constraints names CLI, LSP
  server, MCP server, formatter, and extension client as future adapters over this
  same core crate. Starting a workspace now costs nothing and avoids a `Cargo.toml`
  restructure (and the churn that implies for this crate's path/import references)
  when those adapters arrive.
- **Alternatives considered**: A bare single-crate repo (`Cargo.toml` at root, no
  `crates/` folder) — simpler today, but would require moving this crate into a
  `crates/` subdirectory later, which is unnecessary churn given the adapters are
  already planned, not speculative.
