---

description: "Task list for 022-docs-site: Published Documentation Site"
---

# Tasks: Published Documentation Site

**Input**: Design documents from `/specs/022-docs-site/`

**Prerequisites**: plan.md (required), spec.md (required for user stories),
research.md, data-model.md, contracts/, quickstart.md — all present, see
`specs/022-docs-site/`. Revised 2026-08-17 after direct owner correction: GitHub
Pages "Deploy from a branch" only serves repo-root `/` or `/docs`, and the owner
wants GitHub Actions usage minimized — deployment is now an Actions-free committed
`docs/` folder, not a `deploy-pages` workflow (see research.md §2/§2a/§6).

**Tests**: No dedicated test framework applies (this is content + light CI
plumbing, not application code) — `scripts/check-docs-coverage.ps1` and the
freshness check (both Phase 2) plus `mdbook build` itself serve as the automated
correctness gate; quickstart.md is the manual/CI validation script. No separate
`tests/` tasks are generated.

**Organization**: Tasks are grouped by user story (spec.md's P1/P2/P3) to enable
independent implementation and testing of each story, per this project's standing
task-generation convention.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: Which user story this task belongs to (US1/US2/US3)
- Every task states its exact file path

## Path Conventions

Per plan.md's Project Structure: `docs-site/src/*.md` for chapter content,
`docs-site/book.toml` for book config, `docs/` for the committed, published
build output, `dev-notes/` for the relocated internal troubleshooting log,
`.github/workflows/docs.yml` for the build-check-only CI job, `scripts/` for the
two small automation scripts, `README.md`/`CONTRIBUTING.md` at repository root
for migration.

---

## Phase 1: Setup (mdBook Skeleton + Directory Reshuffle)

**Purpose**: Get an empty-but-buildable book in place, with the `docs/`/
`dev-notes/` split settled, before any real content or CI exists.

- [X] T001 Create `docs-site/book.toml` (title "Drut", `site-url` set to the
      default GitHub Pages path per research.md §4, `git-repository-url`
      pointing at this repo, description one sentence, and `[build] build-dir =
      "../docs"` so output lands at repo-root `docs/`, research.md §2)
- [X] T002 [P] `git mv docs/known-environment-quirks.md
      dev-notes/known-environment-quirks.md` — `docs/` is now reserved for
      published build output, so this pre-existing, unrelated file relocates
      first, before `docs/` starts accumulating generated content (research.md
      §5). Historical references to the old path in already-shipped specs
      (`002-cli-check-format/research.md`, `003-lsp-vscode-extension/tasks.md`)
      are deliberately left as-is (dated historical record, not living docs).
- [X] T003 Create `docs-site/src/SUMMARY.md` listing all 8 chapters in the order
      from contracts/site-structure.md, each entry pointing at its chapter file
- [X] T004 [P] Create the 8 stub chapter files under `docs-site/src/`
      (`introduction.md`, `install.md`, `getting-started.md`,
      `cli-reference.md`, `editor-guide.md`, `mcp-guide.md`,
      `formatter-guide.md`, `configuration-reference.md`), each with just its
      chapter's H1 heading, so `mdbook build` (from `docs-site/`) succeeds
      immediately with placeholder content
- [X] T005 [P] Create `scripts/build-docs.ps1`: runs `mdbook build` from
      `docs-site/` (writing to `../docs` per T001's `build-dir`), then
      (re)creates an empty `docs/.nojekyll` — mdBook clears its build directory
      on every build, so `.nojekyll` must be recreated after, not committed once
      and expected to survive (research.md §2). One script, used identically for
      a local publish (quickstart.md step 5) and CI's freshness check (T007).

**Checkpoint**: `.\scripts\build-docs.ps1` succeeds; `docs/` contains a built,
navigable 8-chapter site plus `.nojekyll`; `mdbook serve --open` (from
`docs-site/`) previews it live.

---

## Phase 2: Foundational (Build-Check CI — Blocking Prerequisite)

**Purpose**: The one automated gate every user story's acceptance depends on: a
build/coverage/freshness check with **no deploy step** (research.md §2a, per the
owner's direct instruction to minimize GitHub Actions usage). Publishing itself
(Phase 6, T027) is a plain commit — no workflow required for it at all.

**⚠️ CRITICAL**: No user story content work should be considered complete until
this phase's gate is wired in and green (even though T007's coverage check will
legitimately fail until Phase 3/US1 fills in real field headings — that's
correct, not a bug).

- [X] T006 Create `scripts/check-docs-coverage.ps1`: extract every field name from
      `FormatConfig` in `crates/drut-config/src/lib.rs` (regex on `pub <name>:`
      lines within the struct body), assert each appears as a `### <name>`
      heading in `docs-site/src/configuration-reference.md`, exit non-zero and
      print the specific missing field name(s) on failure, exit 0 with a
      confirmation message on success — per contracts/config-reference-entry.md.
      Expected to fail at this point in the task list (Phase 1's stub
      `configuration-reference.md` has no field headings yet) — that's correct;
      T014 is where it starts passing.
- [X] T007 Create `.github/workflows/docs.yml` with exactly **one** job (`docs`):
      (1) `.\scripts\build-docs.ps1`, (2) `.\scripts\check-docs-coverage.ps1`,
      (3) a freshness check — re-run `build-docs.ps1` into a clean checkout and
      `git status --porcelain -- docs` to confirm the committed `docs/` matches
      what a fresh build produces, failing the job if it doesn't (research.md
      §6) — triggered on push/PR to `main` and any `[0-9][0-9][0-9]-*` feature
      branch (mirroring `ci.yml`'s existing trigger shape). **No** second job,
      **no** `pages: write`/`id-token: write` permission, **no** secrets, **no**
      `actions/deploy-pages`/`upload-pages-artifact` step anywhere in the file
      (depends on T001–T005)

**Checkpoint**: CI runs the build, the coverage check, and the freshness check on
every push/PR — all three currently red except the freshness check (which is
vacuously green: an empty `docs/` still matches a fresh empty-content build).
Content work can now begin.

---

## Phase 3: User Story 1 - Find out what a config field does (Priority: P1) 🎯 MVP

**Goal**: Every one of the 10 `[format]` fields has a complete, accurate,
findable entry (name, values, default, effect, example, precedence) — the
specific, named pain point this feature exists to fix.

**Independent Test**: Open the built/served site and, for every field in
data-model.md's table, locate a complete entry per contracts/
config-reference-entry.md without reading source code or spec-kit artifacts;
`scripts/check-docs-coverage.ps1` exits 0. Verified end-to-end via
quickstart.md step 7.

### Implementation for User Story 1

- [X] T008 [US1] Write the shared precedence-chain explanation (data-model.md's
      Precedence Chain entity — the 4-tier list) as its own subsection at the top
      of `docs-site/src/configuration-reference.md`, before any field entry
      (depends on T004)
- [X] T009 [US1] Write the 4 casing-field entries (`casing`,
      `control_words_casing`, `pair_keywords_casing`, `data_references_casing`)
      in `docs-site/src/configuration-reference.md` per contracts/
      config-reference-entry.md's required shape, including the two-directional
      legacy/granular relationship callout from data-model.md's "Precedence
      source note" entity (depends on T008)
- [X] T010 [US1] Write the `top_level_indent` and `indent_width` entries in
      `docs-site/src/configuration-reference.md` (depends on T008)
- [X] T011 [US1] Write the `operator_spacing` entry in
      `docs-site/src/configuration-reference.md` (depends on T008)
- [X] T012 [US1] Write the `blank_lines`, `top_level_blank_line_cap`, and
      `nested_blank_line_cap` entries in `docs-site/src/configuration-
      reference.md`, stating that the two caps only matter when `blank_lines =
      auto` (depends on T008)
- [X] T013 [US1] Run `.\scripts\build-docs.ps1` to regenerate `docs/` with the
      real content so far (keeps the freshness check green as content lands)
      (depends on T009–T012)
- [X] T014 [US1] Run `scripts/check-docs-coverage.ps1` (quickstart.md step 3);
      fix any missing/misspelled field heading in `docs-site/src/configuration-
      reference.md` until it exits 0 (depends on T009–T012)

**Checkpoint**: User Story 1 fully functional and independently testable/
demoable — the configuration reference is complete and coverage-enforced. This
alone is a shippable MVP.

---

## Phase 4: User Story 2 - Get a new project working end to end (Priority: P2)

**Goal**: A newcomer can go from nothing installed to seeing a real diagnostic
or formatted result, using only the site.

**Independent Test**: Follow the site's own Install → Getting Started path with
no outside help; identify all four MCP tools from the MCP guide alone. Verified
end-to-end via quickstart.md step 8.

### Implementation for User Story 2

- [X] T015 [P] [US2] Write `docs-site/src/introduction.md` (what Drut is, who
      it's for, why it exists, explicit non-goals — no per-program-box semantic
      validation) per contracts/site-structure.md row 1 (depends on T004)
- [X] T016 [P] [US2] Write `docs-site/src/install.md` (CLI via `cargo install
      drut-cli` / build-from-source; VS Code/Open VSX extension; note that the
      extension self-installs its own `drut` binary) per contracts/
      site-structure.md row 2 (depends on T004)
- [X] T017 [US2] Write `docs-site/src/getting-started.md`: a runnable
      walkthrough (install, then `drut check` and `drut format --diff` against a
      small sample script) with real expected output shown (depends on T016)
- [X] T018 [P] [US2] Write `docs-site/src/cli-reference.md` documenting
      `check`/`format`/`server`/`mcp` and every `format` flag, sourced from
      data-model.md's field table (depends on T014 for accurate cross-references
      into the now-complete configuration reference)
- [X] T019 [P] [US2] Write `docs-site/src/editor-guide.md`: diagnostics, hover,
      completion/spell-check, folding, format-on-save (auto-on) and
      format-on-paste (opt-in, with the exact `.vscode/settings.json` snippet),
      the undefined-`@token@` hint stated with its documented blind spots
      (constitution Principle VII), and the 10 `drut.format.*` editor client
      settings from `021-editor-settings-config` (depends on T014)
- [X] T020 [P] [US2] Write `docs-site/src/mcp-guide.md` documenting all four MCP
      tools (`diagnose`, `format`, `query_structure`, `lookup_keyword`) — what
      each returns and when an AI-assistant integrator would reach for it
      (depends on T014)
- [X] T021 [US2] Cross-link `cli-reference.md`'s flags and `editor-guide.md`'s
      settings back to their matching `configuration-reference.md` entries
      (relative mdBook links) so a reader arriving by either name lands on the
      same entry, per spec.md FR-004 (depends on T018, T019)
- [X] T022 [US2] Run `.\scripts\build-docs.ps1` to regenerate `docs/` (depends
      on T015–T021)

**Checkpoint**: User Stories 1 AND 2 both work independently — a newcomer has a
complete, working path from install to first real result.

---

## Phase 5: User Story 3 - Understand formatter behavior before running it (Priority: P3)

**Goal**: A reader can correctly predict what the formatter will and won't
change before trusting it against real work.

**Independent Test**: Read the Formatter guide alone; correctly predict output
for representative before/after examples across casing, indentation, operator
spacing, and blank-line normalization. Verified end-to-end via quickstart.md
step 9.

### Implementation for User Story 3

- [X] T023 [US3] Write `docs-site/src/formatter-guide.md`'s opening guarantee
      statement (idempotent, never reorders statements or changes program
      meaning — Principle III, stated accurately) plus a casing-categories
      before/after example (depends on T004)
- [X] T024 [US3] Add an operator-spacing before/after example (`preserve` vs.
      `fixed` vs. `auto`, including the vertical-alignment behavior) to
      `docs-site/src/formatter-guide.md` (depends on T023)
- [X] T025 [US3] Add a blank-line-normalization before/after example and the
      `; FMT: OFF`/`; FMT: ON` region explanation to `docs-site/src/formatter-
      guide.md` (depends on T023)
- [X] T026 [US3] Run `.\scripts\build-docs.ps1` to regenerate `docs/` (depends
      on T023–T025)

**Checkpoint**: All three user stories independently functional; full site
content complete.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Publish for real, retire the scattered docs this site replaces, and
record the feature in this project's own standing tracking files.

- [X] T027 Publish: run `.\scripts\build-docs.ps1` one final time, then `git add
      docs/ && git commit`, per quickstart.md step 5 — the actual, Actions-free
      "deploy." (One-time, separately: set the repository's Settings → Pages
      source to "Deploy from a branch" → `main` → `/docs`, if not already done —
      not a code task, called out here so it isn't missed.) (depends on Phases
      3–5)
- [X] T028 [P] Trim `README.md`'s Documentation section to link to the published
      site as the documentation home (FR-007/FR-008) — no net growth in
      `README.md`'s length (SC-003) (depends on Phases 3–5)
- [X] T029 [P] Replace `CONTRIBUTING.md`'s "Configuration" section with a short
      pointer to `configuration-reference.md` (FR-009) (depends on T014)
- [X] T030 [P] Replace `CONTRIBUTING.md`'s "Editor behavior" section with a
      short pointer to `editor-guide.md` (FR-009) (depends on T019)
- [X] T031 Add a sentence to `CONTRIBUTING.md`'s Workflow section stating FR-011's
      obligation explicitly: a feature that adds/changes a `[format]` config
      field, a CLI flag, an MCP tool, or LSP-visible behavior updates the
      corresponding `docs-site/` page as part of that feature's own change —
      making the obligation a visible review-time expectation, not only spec
      prose (analyze finding E2) (depends on T029, T030)
- [X] T032 Run quickstart.md steps 1–4a and 6–9 end to end locally (build via
      `build-docs.ps1`, preview/search spot-check, coverage script, the
      freshness-check break/fix drill, the `README.md`/`CONTRIBUTING.md` diff
      review, and the three User Story walkthroughs) and fix any failures found
      (depends on T027–T031). Step 5's actual live-Pages confirmation is
      necessarily post-push verification, not part of this task's local scope.
- [X] T033 [P] Add a `CHANGELOG.md` "Added" entry for the published
      documentation site, matching this project's existing per-feature entry
      convention
- [X] T034 [P] Update `ROADMAP.md`, marking this pre-publish sequence item done
      with a dated note pointing at `specs/022-docs-site/`, per this project's
      standing convention of recording every shipped feature there

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup (needs the 8 stub files,
  `SUMMARY.md`, and `build-docs.ps1` to exist) — BLOCKS all user stories.
- **User Stories (Phase 3–5)**: All depend on Foundational completion.
  - US1 (Phase 3) has no dependency on US2/US3.
  - US2 (Phase 4) is content-independent of US1 but T018/T019/T020 deliberately
    wait on US1 (T014) so their cross-references into the configuration
    reference are accurate on first write, not written against a moving target.
  - US3 (Phase 5) has no dependency on US1/US2 content — could run in parallel
    with either once Foundational is done.
- **Polish (Phase 6)**: Depends on all three user stories being complete (the
  publish step and the README/CONTRIBUTING migration both need the real content
  they're pointing to).

### Within Each User Story

- US1: T008 (shared precedence text) before any field entry (T009–T012, all edit
  the same file so not marked [P]); T013 (rebuild) then T014 (coverage check)
  last.
- US2: T015/T016/T018/T019/T020 touch different files, marked [P]; T017 depends
  on T016 (same file, sequential); T021 depends on T018+T019 both existing; T022
  (rebuild) last.
- US3: T023 (opening guarantee) before T024/T025 (same file, sequential); T026
  (rebuild) last.

### Parallel Opportunities

- Setup: T002, T004, T005 touch different files/directories from each other and
  from T001/T003 — parallelizable once T003 exists (T004's stub files are what
  `SUMMARY.md` points at).
- Foundational: T006 and T007 touch different files — T007 depends on T005
  existing (the workflow calls `build-docs.ps1`) but not on T006 being finished.
- Once Foundational completes, US1 and US3 can be worked fully in parallel (no
  shared files); US2's cross-reference tasks (T018–T021) are the only reason US2
  is sequenced after US1 above — if that accuracy risk is accepted, US2's
  content-only tasks (T015, T016, T017, T019, T020) could also start in
  parallel with US1.
- Polish: T028, T029, T030, T033, T034 touch different files — all
  parallelizable once their content dependencies land; T027 (publish) and T032
  (validation run) are last, in that order (T032 wants the real published state
  to check against, though it can also run against the local build).

---

## Parallel Example: User Story 2

```text
# Once US1 (Phase 3) is complete, launch together:
Task: "Write docs-site/src/introduction.md"
Task: "Write docs-site/src/install.md"
Task: "Write docs-site/src/cli-reference.md"
Task: "Write docs-site/src/editor-guide.md"
Task: "Write docs-site/src/mcp-guide.md"
# Then sequentially: getting-started.md (needs install.md's content settled),
# then the cross-link pass (T021), then a rebuild (T022).
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: run `scripts/check-docs-coverage.ps1` and manually
   confirm every field entry against contracts/config-reference-entry.md's
   acceptance check
5. This alone resolves the specific complaint that started this feature
   ("even I struggle to find what the options are for each toml item") and is a
   legitimate stopping point if time-boxed — publish it (T027-shaped) even before
   US2/US3 land, if desired.

### Incremental Delivery

1. Setup + Foundational → an empty, build-check-gated (not deploy-gated) book
   shell.
2. Add US1 → the configuration reference, the centerpiece → validate → this is
   the MVP.
3. Add US2 → a complete newcomer path → validate.
4. Add US3 → formatter trust/predictability → validate.
5. Polish → publish for real (commit `docs/`), retire the scattered old docs,
   record the feature in `CHANGELOG.md`/`ROADMAP.md`.

---

## Notes

- [P] tasks touch different files with no completed-task dependency between them.
- No test-framework tasks were generated — `mdbook build` (via
  `scripts/build-docs.ps1`), `scripts/check-docs-coverage.ps1`, and the freshness
  check are this feature's automated correctness gate, all created in Phase 2
  ahead of the content that must satisfy them (the same "write the check before
  the thing it checks" shape a TDD test suite would follow, adapted to a content
  feature).
- Constitution Principle II (no verbatim vendor documentation) and Principle VII
  (naming honesty) apply to every content-writing task (T008–T025) — every
  chapter is written in the project's own words, describing actual shipped
  behavior, never overclaiming past what a feature's own spec documents.
- Deployment is deliberately **not** a task-list item beyond T027's plain commit
  — there is no deploy workflow to author, per the owner's direct instruction to
  minimize this feature's GitHub Actions footprint (spec.md's 2026-08-17
  Clarification; research.md §2).
