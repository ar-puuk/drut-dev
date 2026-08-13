---

description: "Task list for Casing Gains an Explicit Preserve Mode"
---

# Tasks: Casing Gains an Explicit `Preserve` Mode

**Input**: Design documents from `/specs/014-casing-preserve-mode/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/,
quickstart.md (all present)

**Tests**: Included — a public-API/CLI/config-surface shape change touching
three adapters requires real coverage: `voyager-core`'s own unit tests for
the new default, `drut-config`'s TOML/resolution coverage, the CLI/MCP
explicit-override tests each user story names, and an explicit confirmation
that nothing else changed (this feature's own defining constraint, FR-003).

**Organization**: Foundational phase carries the shared core-type change
every story needs (unlike `009`, no single story *is* the core change here
— all three stories consume it equally). Then three user stories matching
spec.md exactly: US1 (P1, the explicit override capability), US2 (P1, zero
output change for everyone who doesn't opt in), US3 (P2, the same default
resolves identically at every integration point). US1 and US3 depend only
on Foundational; US2 is a verification-only story (nothing to build, only
to confirm) and also depends only on Foundational.

**Everything in this file's scope was measured against the real, current
codebase during planning (research.md §1-§5), not estimated**:

- **Exactly two `voyager-core` call sites** read `options.casing`/match on
  `CasingConvention` at all: `render`'s casing-edit gate and
  `edit_for_span`'s match (research.md §1) — both must change, one for
  behavior (the gate) and one purely for compile-exhaustiveness
  (`edit_for_span`, practically unreachable with `Preserve`).
- **`drut_config::FormatConfig`/`ExplicitFormatOverride` do NOT change** —
  only `resolve_format_options`'s two internal lines gain
  `.unwrap_or_default()` (research.md §2, FR-004).
- **`drut-lsp` needs zero source changes** — neither `formatting.rs` nor
  `range_formatting.rs` constructs an explicit casing override; the
  existing suite passing unmodified after the type change compiles through
  is the confirmation (research.md §2, FR-008).
- **A pre-existing spec sentence becomes factually stale the moment this
  ships**: `002-cli-check-format/spec.md`'s FR-026 currently contrasts
  itself against `--casing` with "this setting has no 'off' state,
  unlike FR-015's `--casing` flag" — that contrast is no longer true once
  `--casing` shares `top_level_indent`'s non-optional resolved-value shape
  (research.md §3). This gets its **own dedicated task (T018)**, confirmed
  with the owner before this file was generated — not folded silently into
  the FR-015 amendment task or any code task.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an
  incomplete sibling task)
- **[Story]**: US1/US2/US3 — omitted for Setup/Foundational/Polish tasks
- Every task names its exact file path

## Path Conventions

- `crates/voyager-core/src/format.rs` — the new `Preserve` variant, the
  `FormatOptions.casing` type change, `render`'s gate, `edit_for_span`'s
  match, and this crate's own test module.
- `crates/drut-config/src/lib.rs` — `resolve_format_options`/
  `default_options`'s two `.unwrap_or_default()` additions.
- `crates/drut-config/src/parse.rs` — `parse_casing`'s new `"preserve"` arm.
- `crates/drut-config/tests/parse.rs`, `tests/resolve.rs` — new Foundational
  coverage.
- `crates/drut-cli/src/cli.rs`, `format_cmd.rs` — `CasingArg`'s new variant
  + `From` impl arm (US1).
- `crates/drut-cli/tests/format_flags.rs` — new `--casing=preserve` override
  test (US1).
- `crates/drut-mcp/src/format.rs` — `explicit_override`'s new `"preserve"`
  arm + doc comment (US1); own test module gains the override test (US1)
  and the default-confirmation test (US3).
- `crates/drut-lsp/` — **no source changes anywhere** (US3's confirmation
  is running the existing suite, not adding one).
- `specs/002-cli-check-format/spec.md` — FR-015 amended (T017), FR-026
  corrected (T018) — two separate tasks, per the owner's explicit
  confirmation before this file was generated.

---

## Phase 1: Setup

- [x] T001 Confirm baseline: `cargo build --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings` both clean,
      on this fresh branch before any change.

**Checkpoint**: Baseline confirmed clean.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The core representation change every story depends on —
`Preserve` existing as a real, default `CasingConvention` variant, and
`FormatOptions.casing` no longer being `Option`-wrapped. No user story
work can meaningfully start until this compiles and its own tests pass.

- [x] T002 In `crates/voyager-core/src/format.rs`: add `Preserve` as
      `CasingConvention`'s third variant and `#[default]` (add `Default` to
      the derive list, per data-model.md); change `FormatOptions.casing`
      from `Option<CasingConvention>` to bare `CasingConvention`. Correct
      both stale doc comments in the same edit (FR-010, code-level only —
      spec-document-level fixes are T017/T018): `CasingConvention`'s own
      comment ("no hardcoded default; `FormatOptions.casing` being `None`
      is how 'off' is represented, not a third variant here") and
      `FormatOptions.casing`'s field comment ("`None` (default) leaves all
      keyword/control-word casing untouched").
- [x] T003 In `crates/voyager-core/src/format.rs`: change `render`'s
      casing-edit gate from `if let Some(convention) = options.casing {
      collect_casing_edits(..., convention, ...) }` to `if options.casing
      != CasingConvention::Preserve { collect_casing_edits(...,
      options.casing, ...) }`; add a third, practically-unreachable arm to
      `edit_for_span`'s match (`CasingConvention::Preserve =>
      original.clone()`), per contracts/casing-preserve-mode.md's exact
      Algorithm. Depends on T002.
- [x] T004 In `crates/voyager-core/src/format.rs`'s own test module: update
      the two compiler-forced struct literals (`upper()`'s `casing:
      Some(CasingConvention::Upper)` → `casing: CasingConvention::Upper`;
      `normalize()`'s `casing: None` → `casing: CasingConvention::
      Preserve`) and the one inline literal in the `casing_lower_...` test.
      Add two new direct unit tests: `CasingConvention::default() ==
      CasingConvention::Preserve`, and a representative fixture confirming
      `CasingConvention::Preserve` produces byte-identical output to what
      the old `None`-based path produced (FR-003). Depends on T003.
- [x] T005 [P] In `crates/drut-config/src/lib.rs`: add `.unwrap_or_default()`
      to `resolve_format_options`'s `let casing = explicit.casing.or
      (config.format.casing);` line and to `default_options`'s `casing:
      explicit.casing` line — matching `top_level_indent`'s existing lines
      in both functions exactly (FR-004). `FormatConfig`/
      `ExplicitFormatOverride`'s `casing` fields are explicitly **not**
      touched. Depends on T002.
- [x] T006 [P] In `crates/drut-config/src/parse.rs`: add a
      `Some("preserve") => Some(voyager_core::CasingConvention::Preserve)`
      arm to `parse_casing`, alongside the existing `"upper"`/`"lower"`
      arms; update both `InvalidValue` error messages to name all three
      valid values (FR-005). Depends on T002.
- [x] T007 [P] Add new tests: `crates/drut-config/tests/parse.rs` —
      `casing = "preserve"` in a `[format]` table parses to
      `Some(CasingConvention::Preserve)`, producing zero warnings (SC-005);
      `crates/drut-config/tests/resolve.rs` — with no `drut.toml` and no
      explicit override, `resolve_format_options` yields
      `CasingConvention::Preserve`. Depends on T005, T006.

**Checkpoint**: `Preserve` is a real, compiling, tested part of
`voyager-core` and `drut-config`. `cargo build --workspace` succeeds
(CLI/MCP's own `CasingArg`/string matches still only cover `upper`/`lower`
at this point — unaffected, since neither is exhaustive-over-3-variants
yet). `voyager-core`'s and `drut-config`'s own suites pass with zero
fixture change beyond T004's compiler-forced literal updates.

---

## Phase 3: User Story 1 - Force casing untouched for one run, overriding a project's config (Priority: P1)

**Goal**: `--casing=preserve` (CLI) and `casing: "preserve"` (MCP) let a
user force casing untouched for one run even when a resolved `drut.toml`
specifies `upper`/`lower`.

**Independent Test**: With a resolved `drut.toml` specifying
`casing = "upper"`, format a script with lowercase control words two ways
— no flag (picks up `upper`) vs. `--casing=preserve` (stays untouched) —
and confirm the two outputs differ exactly as expected.

### Implementation for User Story 1

- [x] T008 [US1] In `crates/drut-cli/src/cli.rs`: add `Preserve` as
      `CasingArg`'s third `ValueEnum` variant (stays `Option<CasingArg>`
      on `Command::Format`, unchanged). In `crates/drut-cli/src/
      format_cmd.rs`: add `CasingArg::Preserve => CasingConvention::
      Preserve` to `impl From<CasingArg> for CasingConvention`. Depends on
      T002.
- [x] T009 [US1] In `crates/drut-mcp/src/format.rs`: add
      `Some("preserve") => Some(voyager_core::CasingConvention::Preserve)`
      to `explicit_override`'s `casing` match; update the `FormatInput.
      casing` doc comment to name all three values; update the error
      message for an unrecognized string. Depends on T002.

### Tests for User Story 1

- [x] T010 [P] [US1] Add a new test to `crates/drut-cli/tests/
      format_flags.rs`: with a `drut.toml` setting `casing = "upper"`,
      `--casing=preserve` leaves lowercase control words untouched —
      mirroring the existing `explicit_casing_flag_overrides_drut_toml_
      for_one_run_only` test's shape (which already proves the reverse
      direction, `upper` overriding `lower`). Confirm the existing
      bare-`--casing`/invalid-value usage-error tests still pass
      unmodified. Depends on T008.
- [x] T011 [P] [US1] Add a new test to `crates/drut-mcp/src/format.rs`'s
      own test module: with a `drut.toml` setting `casing = "upper"`,
      `casing: Some("preserve".to_string())` leaves lowercase control
      words untouched — mirroring `explicit_casing_param_overrides_a_
      present_drut_toml`'s shape. Depends on T009.

**Checkpoint**: User Story 1 independently proven — an explicit `preserve`
override works identically at the CLI and MCP surfaces.

---

## Phase 4: User Story 2 - Nothing about existing formatting output changes (Priority: P1)

**Goal**: Confirm, not build — every script that formatted with casing
untouched before this feature continues to produce byte-identical output.

**Independent Test**: Run the full 161-file real corpus through `drut
format` with no casing flag, before and after this change, confirm zero
output differs.

### Verification for User Story 2

- [x] T012 [US2] Run `cargo test -p voyager-core --lib format::`, `cargo
      test -p voyager-core --test format_sequence`, and `cargo test -p
      voyager-core --test format_corpus` — confirm all green with **zero
      fixture/golden regeneration and zero assertion changes** beyond
      T004's compiler-forced literal updates. Report this explicitly: this
      is the actual proof of FR-003, not an inspection substitute for it.
      Depends on Phase 2 (T002-T007).
- [x] T013 [US2] Real-corpus revalidation: with `$DRUT_CORPUS_PATH` set,
      run `cargo test --release -p drut-cli --test fixture_corpus_e2e --
      --ignored` and confirm both the check-clean and the idempotent-
      format-write sub-tests still pass — i.e. still 161/161 clean *and*
      the actual formatted bytes are unchanged from the pre-feature
      baseline, not merely diagnostic-free. Depends on T012.

**Checkpoint**: User Story 2 independently proven — zero output change,
confirmed at both unit-test and real-corpus scale.

---

## Phase 5: User Story 3 - The default is the same everywhere a format request can originate (Priority: P2)

**Goal**: CLI, LSP, and MCP all resolve an unset casing preference to the
identical `Preserve` behavior.

**Independent Test**: Call `voyager_core::FormatOptions::default()`
directly, then every LSP handler that formats a document, then the MCP
`format` tool — confirm all three leave casing untouched.

### Verification for User Story 3

- [x] T014 [P] [US3] Add a direct, minimal test to `crates/voyager-core/
      src/format.rs`'s test module asserting `FormatOptions::default()
      .casing == CasingConvention::Preserve` — the single most direct
      confirmation of this story, distinct from the behavioral tests
      around it. Depends on T002.
- [x] T015 [P] [US3] Run `cargo test -p drut-lsp --lib` and confirm all
      green with **zero test added or modified, zero source file
      touched** — this is intentional (research.md §2): `drut-lsp` never
      constructs an explicit casing override, so the existing suite
      passing unmodified after the type change compiles through *is* the
      confirmation. Report explicitly that no `drut-lsp` file appears in
      this feature's diff. Depends on Phase 2.
- [x] T016 [P] [US3] Add a new test to `crates/drut-mcp/src/format.rs`'s
      own test module: with no `casing` parameter and no governing
      `drut.toml`, a script with lowercase control words is left
      untouched — mirroring `top_level_indentation_defaults_to_preserve_
      not_normalize`'s existing shape for the sibling setting. This
      exercises only Foundational's default-resolution path
      (`.unwrap_or_default()`, T005), not T009's `"preserve"`-string
      parsing — **depends on Phase 2 (Foundational) only**, so User Story
      3 stays independently testable without waiting on User Story 1.

**Checkpoint**: User Story 3 independently proven — CLI, LSP, and MCP all
confirmed to agree on the `Preserve` default.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Spec-document amendments and whole-workspace/full-corpus
re-proof, once all three stories are done.

- [x] T017 Amend `specs/002-cli-check-format/spec.md`'s **FR-015** with a
      new dated entry (`contracts/casing-preserve-mode.md`'s exact
      replacement text) describing the `Option`-to-three-variant-enum
      representation change and the new explicit `--casing=preserve` CLI
      value. Original FR-015 text preserved, not replaced.
- [x] T018 **Dedicated task, separate from T017** — correct
      `specs/002-cli-check-format/spec.md`'s **FR-026**, whose existing
      text ("Unlike FR-015's `--casing` flag, this setting has no 'off'
      state...") becomes factually stale once T017 lands (research.md §3).
      Replace with `contracts/casing-preserve-mode.md`'s exact replacement
      text. **Verification**: re-read the corrected FR-026 bullet and
      confirm (a) the old contrastive sentence no longer appears verbatim
      anywhere in the file (`grep`-confirmed) and (b) the new text
      accurately describes the shape both settings now share, cross-
      checked against T002/T003's actual shipped code, not just against
      the contract document. Depends on T017 (the correction references
      FR-015's amended text).
- [x] T019 `cargo test --release --workspace` and `cargo clippy --workspace
      --all-targets -- -D warnings`, both clean.
- [x] T020 [P] Full 161-file corpus revalidation across all three adapter
      surfaces, each reported individually:
      ```powershell
      $env:DRUT_CORPUS_PATH = "path\to\WF-TDM-Official-Releases"
      cargo test --release -p drut-cli --test fixture_corpus_e2e -- --ignored
      cargo test --release -p drut-lsp --test diagnostics_corpus -- --ignored
      cargo test --release -p drut-mcp --test diagnostics_corpus -- --ignored
      ```
      Expected: still 161/161 clean everywhere — zero output change at
      full scale (SC-001), zero new diagnostics anywhere.

**Checkpoint**: Feature-complete against spec.md; FR-015 amended, FR-026
corrected as its own verified task; full workspace and full corpus
re-proven clean.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational only.
- **User Story 2 (Phase 4)**: Depends on Foundational only — independent
  of US1 (it verifies the *unset* path; US1 exercises the *explicit*
  path).
- **User Story 3 (Phase 5)**: Depends on Foundational only (T014/T015/T016
  all) — independently testable without User Story 1, per spec.md's own
  story structure. T016 shares a file (`crates/drut-mcp/src/format.rs`)
  with T009/T011, so in practice run it after whichever of those lands
  first to avoid an edit conflict, but there is no *requirement*
  dependency on either.
- **Polish (Phase 6)**: T017/T018 are independent of the code phases
  (documentation only); T019/T020 depend on all three stories being
  complete.

### Parallel Opportunities

- T005, T006 can run in parallel once T002 lands; T007 depends on both.
- T010, T011 can run in parallel once T008/T009 (respectively) land.
- T012 and T014/T015 can run in parallel once Foundational completes —
  different files, non-conflicting.
- T020's three corpus-validation commands are independent of each other.

---

## Parallel Example: Once Foundational (T002-T007) Lands

```bash
Task: "T008: CasingArg gains Preserve + From impl arm"
Task: "T009: drut-mcp explicit_override gains preserve arm"
Task: "T012: confirm voyager-core's full suite passes unmodified"
Task: "T014: FormatOptions::default().casing direct assertion"
Task: "T015: confirm drut-lsp's full suite passes with zero files touched"
```

---

## Implementation Strategy

### Single Pass (all three stories are small and share one Foundational change)

1. Setup → baseline confirmed clean.
2. Foundational → `Preserve` exists, compiles, is tested at the
   `voyager-core`/`drut-config` layer.
3. User Story 1 → the explicit-override capability, CLI and MCP.
4. User Story 2 → confirmation, not construction — zero output change at
   both unit and real-corpus scale.
5. User Story 3 → the same default confirmed identical at every surface,
   including the LSP surface's *absence* of any change as its own proof.
6. Polish → FR-015 amended, FR-026 corrected (its own task), full
   workspace/corpus re-proof.

---

## Notes

- T017/T018's split is the one sequencing detail in this feature that
  actually matters to get right per the owner's explicit instruction: the
  FR-026 correction must remain independently identifiable and
  individually verifiable, never silently absorbed into the FR-015
  amendment or any code task's diff.
- Commit after each task or logical group.
