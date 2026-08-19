# Roadmap

Tracks the release-readiness sequence agreed on 2026-08-10 — a set of small,
independent follow-ups that don't need their own spec-kit cycle (unlike
001–004), plus explicitly-later phases that are *not* part of getting to a
first publish. Not a spec-kit artifact itself; just a place to not lose track
of this list.

## Pre-publish sequence (in order)

Each item's status is tracked here so this doesn't have to be re-derived from
scratch next time. "Not started" doesn't mean unexamined — some of these have
already been researched or partially unblocked; see the note per item.

1. **Format-on-save** — *done* (`005-format-on-save-paste`, 2026-08-11).
   `extension.ts`'s `ensureFormatOnSaveEnabled` auto-enables
   `editor.formatOnSave` for `.s`/`.block` files on first activation,
   one-time and removal-respecting (same pattern as
   `ensureVariableColorCustomization`, verified via the extracted
   `shouldInjectFormatOnSave` predicate's own unit tests). Code complete,
   automated-tested, and the real-VS-Code smoke test
   (`specs/005-format-on-save-paste/quickstart.md` step 5) manually
   confirmed.
2. **Format-on-paste** — *done* (`005-format-on-save-paste`, 2026-08-11).
   `drut-lsp` now serves `textDocument/rangeFormatting`
   (`crates/drut-lsp/src/range_formatting.rs`) — whole-document format
   internally, line-diffed, filtered to the requested range; verified
   against real block-boundary fixtures (a paste that opens or closes a
   block, shifting indentation outside its own range), not just simple
   cases. Ships opt-in — see CONTRIBUTING.md's "Editor behavior" section for
   the setting. Code complete, automated-tested, and the real-VS-Code smoke
   test (quickstart.md step 6) manually confirmed.
3. **TOML-based configuration** — ✅ *done, merged 2026-08-13 as
   `012-toml-configuration`*. `drut.toml` (`casing`/`top_level_indent` under a
   `[format]` table) is discovered via per-file upward directory walk-up
   (stopping at a `.git` boundary or filesystem root), respected identically
   by the CLI, LSP, and MCP adapters (`crates/drut-config`), with
   `defaults < drut.toml < explicit CLI-flag/MCP-param` precedence, a
   `--isolated` CLI escape hatch, and non-fatal per-field fallback on a
   malformed value (never hard-fails — constitution Principle IV). See
   `specs/012-toml-configuration/`.
   - **Same-day follow-on, folded in here rather than getting its own
     sequence slot**: ✅ *done, merged 2026-08-13 as
     `013-lsp-config-file-watch`*. Fixes a real bug found during 012's own
     manual verification — an open `.s`/`.block` document's config-warning
     diagnostic went stale when `drut.toml` was edited directly, without the
     document itself being closed/reopened. `drut-lsp` now registers a
     `workspace/didChangeWatchedFiles` watcher for `**/drut.toml` (gated on
     the client advertising dynamic-registration support; graceful,
     no-crash fallback to prior behavior otherwise) and re-publishes
     diagnostics for every open document on a matching change. See
     `specs/013-lsp-config-file-watch/`.
4. **`casing` gains an explicit third `Preserve` mode** — ✅ *done, merged
   2026-08-13 as `014-casing-preserve-mode`*. Mirrors `TopLevelIndentMode`'s
   shape (`Preserve`/`#[default]`, `Upper`, `Lower`) instead of the previous
   `Option<CasingConvention>` (`None` standing in for "leave untouched") — a
   pure representation change, byte-identical output to before for every
   existing input. Not to be confused with the still-deferred `--casing auto`
   mode below (resolved queued item 4). See `specs/014-casing-preserve-mode/`.
5. **README/docs overhaul** — ✅ *done, 2026-08-13*. Split into a short,
   visitor/Marketplace-facing `README.md` (what Drut is, why it exists, quick
   install, feature list, links out) and `CONTRIBUTING.md` (architecture,
   per-crate status, configuration design, build/test commands, versioning,
   credits — the prior README's full content, relocated largely as-is). A
   second pass to add real install-from-Marketplace/crates.io instructions to
   `README.md`'s Install section is still needed once item 8 is live.
6. **CI + release pipeline** — ✅ *done, 2026-08-13*. `.github/workflows/
   ci.yml`: build/test/clippy gate on push/PR to `main` and any `NNN-*`
   feature branch, two jobs (Rust, VS Code extension), no publishing, no
   secrets — live and green on `main`. `.github/workflows/release.yml`:
   tag-triggered (`v*.*.*`) cross-platform `drut-cli` release pipeline
   (Windows x64, macOS x64+arm64, Linux x64 — native GitHub-hosted runners
   per OS, no cross-compilation; Linux/Windows arm64 deliberately deferred,
   low real demand for this project's Windows-centric domain), asset naming
   (`drut-<target-triple>.<ext>` + `.sha256`, no version in the filename)
   mirrors rust-analyzer's own release assets, release notes sourced from
   `CHANGELOG.md`'s matching version section (not `--generate-notes`, to
   avoid two independently-drifting descriptions of the same release). A
   `preflight` job fails fast (before any of the 4 builds start) if the
   pushed tag doesn't match every workspace crate's lockstep version, or if
   `CHANGELOG.md` has no section for it. Verified live via a disposable test
   tag end-to-end (all jobs green, one asset downloaded/checksum-verified/
   confirmed a real executable), then fully cleaned up (no trace in the real
   release history).
7. **Extension auto-install/update ("out of the box" binary experience)** —
   *not started, researched only* (2026-08-10; unblocked as of item 6 above).
   Two real patterns compared: rust-analyzer's (binary downloaded from GitHub
   Releases on first activation — small `.vsix`, decoupled from binary
   rebuilds) vs. ruff's (binary bundled directly into the `.vsix`/npm package
   — no network call needed, but requires a full per-platform build matrix
   upfront, which item 6 now provides regardless). This item must land
   **before** item 8's extension publish goes live, not after — publishing an
   extension that isn't yet "batteries included" would ship a v1 that fails
   the project's own stated bar the moment someone installs it.
8. **Actual publish** (VS Code Marketplace + Open VSX + crates.io) — ✅ *live*.
   First published 2026-08-18 (v0.2.0, then v0.2.1 same day) ahead of item 7
   landing — the "batteries included" ordering note below was not honored in
   practice, so item 7 remains a real, tracked gap for the versions already
   live, not a hard blocker that held. v0.3.0 published 2026-08-19 via the
   same `publish.yml` dispatch (extension + all 5 crates, `voyager-core` →
   `drut-config` → `drut-lsp`/`drut-mcp` → `drut-cli`). v0.3.1 published
   2026-08-19 the same way, same day (the `028-identifier-highlighting`
   merge plus its accompanying `voyager-core` casing fix). v0.3.2 published
   2026-08-19 the same way, same day (a `#pair-values` grammar follow-up fix
   plus a README refresh, both found/requested immediately after the 0.3.1
   publish). v0.3.3 published 2026-08-19 the same way, same day (a real
   language-server-won't-start bug report against the freshly published
   0.3.2 extension — `isOnPath`'s Tier 1 pre-flight check treated any spawn
   error other than `ENOENT` as "found," so a blocked PATH entry was
   confidently used instead of falling through to Tier 2/3).
   - ✅ *Fixed 2026-08-13*: `vsce` packaging `editors/vscode/` in isolation
     meant the repo-root `LICENSE-MIT`/`LICENSE-APACHE` files never made it
     into the `.vsix`. Both are now copied into `editors/vscode/` directly,
     plus a short `LICENSE` pointer file (vsce's own expected filename,
     explaining the dual-license split) — verified by unzipping the rebuilt
     `.vsix` and confirming byte-identical content for all three files, not
     just that vsce's own warning went away.
   - ✅ *Fixed 2026-08-13*: extension icon. `editors/vscode/icon.png`
     (1600×1600, square — well above the 128×128 minimum both platforms
     require) wired via `package.json`'s `"icon"` field; `icon.svg` kept
     alongside as a source file for regenerating other sizes later. Verified
     bundled and correctly declared in the rebuilt `.vsix`'s own manifest.
   - **Screenshots/GIFs — still open, real gap, not something to generate
     synthetically.** Added 2026-08-13. The Marketplace/Open VSX listing has
     no visual content at all yet (highlighting, hover, live diagnostics,
     folding in action) — this matters a lot for Marketplace conversion and
     needs real captures from the actual running extension, not a mockup.
     Owner-scoped, same as the icon was.
9. **`indent_width` becomes a configurable `[format]` setting** — ✅ *done,
   implemented 2026-08-17 as part of `017-casing-categories-indent-width`*
   (added 2026-08-13). **Explicitly a deliberate owner decision,
   not an evidence-driven one** — recorded honestly as such, the same way
   `008`'s default-reversal and `009`'s later correction of it were both
   recorded rather than smoothed over. Immediately before this was queued, a
   findings-only investigation (no code changed) re-confirmed the corpus
   evidence actually argues *against* this: 4-space-per-nesting-level
   indentation is dominant at 82.4% of 30,652 real body-indent occurrences
   corpus-wide (87.5% of files with nested content have 4 as their own
   per-file dominant value; 67% are internally ≥90% consistent on one
   value) — a materially stronger signal than either `casing` or
   `top_level_indent` had when *they* were made configurable, and
   `002-cli-check-format/contracts/formatting-api.md` had already, from the
   very first formatter contract, explicitly and deliberately excluded
   indentation width from configurability on exactly that evidence ("casing
   is the only configurable axis"). No evidence of anyone ever requesting a
   different width was found anywhere in this project's specs, docs, or git
   history. The owner reviewed these findings and chose to proceed anyway —
   overriding the recommendation is the owner's prerogative and this item
   exists to build it, not to relitigate the decision.
   - **Scope**: `indent_width` added to `drut.toml`'s `[format]` table
     (integer, default `4` — the corpus-confirmed value, now overridable
     rather than fixed); threaded through the same shape `009` built for
     `top_level_indent` — `FormatOptions` gains an `indent_width` field,
     `drut-cli` gains `--indent-width=<N>`, `drut-mcp`'s `format` tool gains
     a matching parameter, `drut-config`'s discovery/parse/resolve logic
     covers it with the same per-field precedence already established
     (`explicit flag > drut.toml > built-in default`).
   - **Contract update required, not optional**: `formatting-api.md`'s "No
     configurable indentation width/style beyond the one canonical form..."
     exclusion statement must be corrected to reflect the reversal when this
     ships — not left standing while the code contradicts it — with the
     update stating plainly that this was a deliberate decision made against
     the corpus evidence, not a response to a data gap or a user request.
   - **Validation question, to be decided during the spec-kit cycle, not
     deferred past it**: does an invalid value (`0`, or something absurd
     like `500`) get rejected, or is any positive integer accepted as-is?
     Current recommendation carried into that cycle: a sane bound (e.g.
     1–16), using the same non-blocking-warning-and-fallback pattern already
     established for every other malformed `drut.toml` value, not a hard
     failure.
   - **Sequencing**: full spec-kit cycle when started, same rigor as
     `009`/`012`/`014` (touches `drut-config`, all three adapters, and
     `format.rs`'s core indentation math). **Update 2026-08-17**: the prior
     hold is lifted — owner decision, bundled into the same spec-kit cycle
     as item 10 below (both add fields to the same `[format]` table, same
     adapters touched).
10. **`--casing` reframed as a 3-category `drut.toml` setting, `auto`
    dropped entirely** — ✅ *done, implemented 2026-08-17 as
    `017-casing-categories-indent-width`, bundled with item 9*. Supersedes
    resolved-queued item 4
    below — real corpus/vendor-doc evidence plus direct stakeholder input
    (`casing-convention-decision.csv`, Bill and Chris's per-token opinions,
    GitHub issue #3) settled what item 4 left as an open philosophy
    question. Two changes from item 4's original framing:
    - **No `auto`/preset mode.** Item 4's blocking "real-usage-fidelity vs.
      vendor-canon-fidelity" question (see "Open questions" below — now
      resolved as moot) doesn't need answering, because drut ships no
      opinionated house style at all. Each project states its own
      preference in its own `drut.toml`; the tool imposes nothing beyond
      the existing `Preserve` default.
    - **Three independently-configurable categories**: `control_words`
      (the existing `CONTROL` role), `pair_keywords` (the existing
      `PAIRKEYWORD` role), and `data_references` (a new role covering
      `MI`/`MO`/`MW`, `LI`/`LW`, `NI`/`NW`, `ZI`/`ZONES`/`Z`, `DBI`/`DBA`,
      `RO`, `A`/`B`, `I`/`J` — merging item 4's originally-separate
      `OPERAND_PREFIX` and `ASSIGNMENT_TARGET` roles into one user-facing
      knob, since nothing in the evidence supports splitting them *yet* —
      see item 11 below for the one case that does). Each independently
      `upper`/`lower`/`preserve`, default `preserve`.
    - **`keywords.rs` correction found along the way**: `NUMREC`/`CNT`/
      `ITER`/`LP`/`RECNUM` were miscategorized as `PairKeyword` dictionary
      entries by the original census — the real `LOOP <name>=start,end
      [,inc]` syntax takes a free-form, user-chosen loop-variable name in
      that slot (vendor doc confirms `iter`/`INDEX`/`L3`/`_K` used
      interchangeably across examples), so these were never real keywords.
      Removed from the dictionary, not cased either way. `ZONES` added
      instead — a real, previously-missing entry, confirmed dual-role
      (pair-keyword under `RUN PGM=MATRIX`, and a plain assignment).
    - Reaching `data_references` still needs the lexer change item 4
      already scoped (the `.` boundary in `mi.1.1`-shaped tokens isn't a
      token delimiter today) — unchanged core-crate scope and golden-
      fixture review burden from item 4's original sizing.
11. **`data_references` may need to split into a 4th category** (an
    `assignment_targets` category, separating `MW`/`LW`/`NW`'s
    assignment-target role — `MW[1] = ...` — from their operand-prefix-
    read role — `mw[3]*mw[99]` — and from the pure-read tokens like `MI`/
    `LI`) — *queued, not started, evidence captured 2026-08-17*. Real
    evidence, not speculative: Bill's own recorded vote differs by role
    for the same token (`UC` for `MW`'s `PATHLOAD MW[201]=` pair-keyword-
    shaped usage, `lc` for `MW[1] = ...`'s assignment-target usage), and
    the corpus backs the split (99.7% lowercase for the pair-keyword form,
    n=360, near-unanimous; 85.4% lowercase for the assignment-target form,
    n=6071, noticeably softer — real authors write these two roles
    differently in practice). Deliberately **not** built into item 10 —
    additive-only design: a `data_references`-default with a later,
    optional `assignment_targets` override that falls back to
    `data_references` when unset is a pure additive change, non-breaking
    for anyone's existing `drut.toml`; building it speculatively now,
    before item 10 even ships, would be exactly the premature-abstraction
    item 10 itself was trimmed down to avoid. Revisit once item 10 has
    real usage, or immediately if the owner (or Bill specifically) wants
    it sooner.
12. **Operator spacing normalization** (`PHASE=ILOOP` / `PHASE =ILOOP`
    / `PHASE= ILOOP` / `PHASE = ILOOP` all becoming one canonical form) —
    ✅ *done, implemented 2026-08-17 as `018-operator-spacing`*
    (added 2026-08-17). Confirmed genuinely new scope, not an existing
    gap: `format.rs`/`formatting-api.md` today normalize indentation and
    (optionally) casing only — no existing logic touches spacing around
    `=` or any other operator, and the real corpus shows this is
    genuinely inconsistent even within one file (e.g. `ZONES   = 1` three
    lines from `ZONES = 1` in the same fixture).

    **Update 2026-08-17**: scope settled (owner decision) as three modes,
    default `preserve`, replacing the `auto`/Tidyverse framing sketched
    above:
    - `preserve` (default) — leaves existing spacing exactly as written,
      mirroring every other formatting axis shipped so far.
    - `fixed` — normalizes every operator occurrence (assignment `=`,
      comparison `==`/`<>`/`>=`/`<=`/`<`/`>`, and arithmetic `+`/`-`/`*`/
      `/` inside computed expressions) to exactly one space on each side,
      independent of neighboring lines. Also normalizes comma spacing
      between multiple pairs on one statement (`MATI=a.mat,MATO=b.mat` →
      `MATI = a.mat, MATO = b.mat`) — folding in related case (2) below.
    - `auto` — `fixed`, plus: consecutive `Assignment` statements (only
      `Assignment`, not pair-keyword `Control` lines) at the same block
      nesting depth have their `=` vertically aligned to the column of
      the longest left-hand side in the run. A run breaks (and
      realigns fresh) on a blank line, a comment-only line, or an
      indentation-depth change — closest real precedent is `gofmt`'s
      automatic alignment of consecutive `const`/`var`/struct-literal
      lines, which breaks the same way. Note this is the opposite
      tradeoff Prettier/R's `styler`/Tidyverse guide take (they refuse to
      align at all, since edits to one line force whitespace-only diffs
      on unrelated sibling lines) — accepted knowingly, not overlooked,
      because Drut applies it automatically rather than asking anyone to
      hand-maintain it.
    - Explicit Tidyverse-style-guide adoption was considered and dropped
      in favor of the shape above, which is closer to `gofmt` than to any
      R-ecosystem formatter.
    - **Related cases, resolved into the modes above**: (1) the `=` rule
      applied to `Assignment` statements — folded into `fixed`/`auto`
      directly; (2) comma spacing between multiple pairs — folded into
      `fixed`/`auto` per above; (3) arithmetic/comparison operator
      spacing inside expressions — folded into `fixed`/`auto` (the "all
      operators" scope decision); (4) spacing between a control word and
      its opening paren (`IF(x)` vs `IF (x)`) and (5) interior padding
      inside brackets/parens (`MW[ 1 ]` vs `MW[1]`) — both settled
      (owner decision) as always no-space-inside under `fixed`/`auto`,
      folded in rather than kept as separate configurable axes; `preserve`
      stays untouched, so an unconfigured project sees zero behavior
      change, matching every other axis shipped so far.

13. **Blank-line-run normalization** — ✅ *done, implemented 2026-08-17 as
    `019-blank-line-normalization`* (added 2026-08-17, alongside
    `018-operator-spacing`'s own completion; see
    `specs/019-blank-line-normalization/` for the full spec/plan/tasks).
    Well-precedented shape (Python's `black`/JS's
    `prettier` both cap consecutive blank lines, fewer nested than at
    module level) — not a novel idea needing its own research pass the way
    casing/operator-spacing did. Scope settled through direct conversation:
    - Two independently-configurable positive-integer caps, not one — a
      top-level cap (default `2`) and a nested cap covering *any* line
      inside *any* block regardless of depth, uniformly, not scaling
      further per nesting level (default `1`) — mirroring
      `top_level_indent`'s existing top-level-vs-everything-else split,
      not a per-depth-level scheme.
    - Two modes only, `preserve` (default)/`auto` — no third `fixed` tier
      the way `operator_spacing` has, since there's only one real
      non-preserve behavior here (cap at the configured value), not two
      meaningfully different ones.
    - `auto` only *contracts* a run of consecutive blank lines down to the
      applicable cap when it exceeds that cap — never pads a shorter run
      up. A "blank" line includes a whitespace-only line (spaces/tabs, no
      visible content), not just a strictly zero-length one.
    - Both caps validate the same way `indent_width` already does (a
      sane range, out-of-range degrades to the default with a
      non-blocking notice, never a hard failure) — exact range TBD at
      planning time, not fixed by this conversation.
    - `; FMT: OFF`/`; FMT: ON` regions are left untouched, same as every
      other formatting axis already shipped.
    - Exact `drut.toml`/CLI/MCP field names are a planning-phase decision,
      not fixed here — same additive-only, never-breaking-an-existing-
      surface discipline every prior formatting axis in this project has
      followed.
14. **Undefined `@token@` diagnostic** — ✅ *done, implemented 2026-08-17 as
    `020-undefined-token-diagnostic`* (added 2026-08-17). Originally
    requested as "a red squiggly" for any variable used without being
    defined; scoped down
    through direct conversation once grounded against `token_resolution.rs`
    (the `016-token-hover-value` resolver — the only existing resolution
    logic remotely relevant here):
    - **Scope: `@token@` substitution references only.** Plain assignment
      identifiers (`X` used with no prior `X = value`) and data-reference
      tokens (`MI`/`MW`/etc., bound by a `FILEI`/`FILEO` pair-keyword
      statement rather than a plain assignment) are explicitly out of
      scope — neither has any existing resolution logic, and the latter's
      binding mechanism is structurally different from `@token@`'s.
    - **Confidence bar: never claim non-existence beyond hover's own
      reach.** `token_resolution.rs` has deliberate, accepted gaps (a
      `@token@` on a block-opener line, more than one level of `READ FILE`
      inclusion, a token-built `READ FILE` path) that are harmless for
      hover (silent fallback to nothing) but would become false-positive
      sources if "resolution failed" were treated as "doesn't exist." This
      diagnostic only fires when it sits within the *same* resolution
      boundary hover already covers — a resolver blind spot is never
      itself treated as evidence a token is undefined.
    - **Severity: Hint or Information, not Error** — a deliberate downgrade
      from the original "red squiggly" ask, decided directly in
      conversation. Constitution Principle IV states false positives are
      worse than false negatives ("an unflagged bug is forgivable; a false
      flag on working code is not") — Error severity would overstate the
      confidence this check can actually have, given real cross-file
      Voyager scripts routinely exceed the resolver's one-level-of-
      inclusion reach.
    - **Update 2026-08-17**: checked against the actual precedent rather
      than assumed — `drut-lsp/src/diagnostics.rs` already has this exact
      shape *twice* (the unclosed `; FMT: OFF` marker, `010-fmt-region-
      markers`; a malformed `drut.toml` warning, `012-toml-configuration`),
      each a standalone function outside `voyager-core::Diagnostic`/
      `DiagnosticKind` entirely, published at HINT severity with its own
      distinct `source` string, chained alongside the six real
      `DiagnosticKind`-based diagnostics. Neither prior feature amended
      `001-voyager-script-parser`'s spec, because neither is a
      `DiagnosticKind` value — this feature follows the identical shape, a
      third such stream, so it needs no amendment there either. (Original
      framing below, superseded by this finding.)
    - **Surface reach, decided 2026-08-17**: LSP-only, matching both prior
      Hint-severity streams exactly — never reaches CLI `check` or MCP
      `diagnose`, which stay strictly `DiagnosticKind`-only ("never a
      narrowed subset," `002-cli-check-format` FR-003) unchanged.
15. **Editor-settings exposure for every `[format]` config field** — ✅ *done,
    implemented 2026-08-17 as `021-editor-settings-config`* (added
    2026-08-17; see `specs/021-editor-settings-config/` for the full
    spec/plan/tasks). All 10 current `drut-config::FormatConfig`/`ExplicitFormatOverride`
    fields (`casing`, `control_words_casing`, `pair_keywords_casing`,
    `data_references_casing`, `top_level_indent`, `indent_width`,
    `operator_spacing`, `blank_lines`, `top_level_blank_line_cap`,
    `nested_blank_line_cap`) become settable as editor client settings, not
    just `drut.toml`. Two real decisions made through direct conversation:
    - **Precedence**: `drut.toml` wins over a client setting — precedence
      becomes `explicit CLI flag/MCP param > drut.toml > client setting >
      built-in default` (no separate "explicit" tier exists for LSP
      requests specifically, since none exists today either — a client
      setting slots in between `drut.toml` and the built-in default only).
      A personal editor preference deliberately never overrides a project's
      own committed formatting config, matching how Prettier/ESLint-style
      tools with a project config file already behave.
    - **Mechanism**: the standard LSP `workspace/configuration`/
      `workspace/didChangeConfiguration` capability (constitution Principle
      VI), not something VS Code-specific — confirmed via direct grep that
      `drut-lsp` has **zero** existing handling of either today (every
      match in the repo is inside `node_modules`, the unused client
      library) — this is a genuinely new server capability, not a thin
      settings-declaration task. Benefits any conforming LSP client, not
      only VS Code.
    - Exact settings namespace/naming (e.g. `drut.format.controlWordsCasing`
      mapped from `control_words_casing`, VS Code's camelCase convention)
      is a planning-phase decision, not fixed here.
16. **Published documentation site** — ✅ *done, implemented 2026-08-17 as
    `022-docs-site`* (added 2026-08-17; see `specs/022-docs-site/` for the
    full spec/plan/tasks/research). Prompted directly: "even I struggle to
    find what the options are for each toml item" — `CONTRIBUTING.md`'s old
    "Configuration" section had drifted to documenting only 2 of the
    eventual 10 real `[format]` fields (items 9/10/12/13/15 above each added
    a field without anyone circling back to that section). Built with
    [mdBook](https://rust-lang.github.io/mdBook/), covering an introduction,
    install, a getting-started walkthrough, a CLI reference, an editor (LSP)
    guide, an MCP guide, a formatter guide, and a complete field-by-field
    configuration reference (every field's values/default/effect/example
    plus the shared precedence chain). `README.md` now links to it as the
    documentation home; `CONTRIBUTING.md`'s "Configuration" and "Editor
    behavior" sections were replaced with pointers.
    - **Deployment reversed mid-cycle, owner correction**: the original plan
      used an Actions-native `deploy-pages` workflow. Corrected directly:
      classic GitHub Pages ("Deploy from a branch") only serves a branch's
      repo-root `/` or `/docs`, and the owner wants GitHub Actions usage
      minimized. Shipped instead: mdBook's build output is redirected
      (`book.toml`'s `build-dir`) to a committed repo-root `docs/`, which
      Pages serves directly — **zero deploy Actions, zero secrets, zero
      Pages permissions**. One CI job survives (`docs.yml`): build + a
      config-field coverage check + a freshness check (diffs a fresh build
      against the committed `docs/`, catching a forgotten rebuild before
      merge) — no deploy job. The pre-existing `docs/known-environment-
      quirks.md` moved to a new `dev-notes/` directory, since `docs/` is now
      reserved for published output.
    - **Process fix, not just a one-time write-up**: `CONTRIBUTING.md`'s
      Workflow section now states explicitly that a feature changing a
      `[format]` field, CLI flag, MCP tool, or LSP-visible behavior updates
      `docs-site/` as part of that same change — a direct fix for the exact
      staleness mechanism that prompted this item, surfaced during this
      feature's own `/speckit-analyze` pass rather than left as spec prose
      nobody but this feature's author would read.

## Resolved queued items (historical log, not part of the pre-publish sequence)

Corrections, features, and investigations raised alongside the `008`→`010`
work, deliberately queued rather than context-switched into mid-stream at
the time. All six below are now closed — done, deferred, or explicitly
decided — kept here as a record rather than deleted, since several took
real investigation to resolve. (Item 7, the one genuinely open question
this queue surfaced, is **not** resolved — see "Open questions" below,
after this log.)

1. **Top-level indentation normalization: default reverts to leave-untouched,
   008's forcing becomes an opt-in toggle** — ✅ *done, merged 2026-08-12 as
   `009-top-level-indent-toggle`* (corrected 2026-08-11, owner's explicit
   preference; shipped the next day). `008-top-level-indentation-
   normalization` had changed FR-012 so top-level (depth-0) statement
   indentation was *always* normalized to column 0, unconditionally,
   replacing 007-era FR-012 (leave top-level indentation untouched — the
   corpus showed no dominant convention: best single value only 26.9% at
   column 8, only 20.4% at column 0). `009` reversed that reversal: the
   **original 007-era leave-untouched (`Preserve`) behavior is the default
   again**, and 008's column-0-forcing is now an **opt-in**
   `--top-level-indent=normalize` CLI flag (still a CLI flag, not the TOML
   config item originally guessed here — TOML-based configuration, pre-
   publish item 3 above, remains unbuilt). All 20 tasks complete and
   independently verified: workspace build/clippy/test clean, full 161-file
   corpus clean across CLI/LSP/MCP, and the 7 golden-fixture reverts
   confirmed whitespace-only. See `specs/009-top-level-indent-toggle/`.
2. **`; FMT: OFF` / `; FMT: ON` region markers** — ✅ *done, merged
   2026-08-12 as `010-fmt-region-markers`* (added 2026-08-11, shipped the
   next day). Marker recognition reuses the existing tokenizer's
   `LineComment` — no new lexer/parser/grammar shape; protection is gated
   at collection time (`plan_indentation`/`plan_block`/`plan_children`'s 4
   `plan.insert` call sites, `push_if_present`'s single casing-edit
   funnel), proven correct via three interaction tests against every
   sibling "don't touch this range" mechanism already in `format.rs`: the
   opener-residue case (a protected block opener's out-of-region children
   anchor to its *true* on-disk column, not a discarded planned value —
   the specific bug `007`'s own diagnosed-block-skip mechanism exists to
   avoid), the `009` interaction (`TopLevelIndentMode::Normalize`), and
   the `007` interaction (a protected region overlapping a diagnosed/
   unmatched block). An unclosed `; FMT: OFF` protects through
   end-of-file and is surfaced via a dedicated, non-`Diagnostic` signal at
   every adapter (CLI stderr notice, MCP response field, LSP
   `HINT`-severity/`"drut-fmt"`-sourced diagnostic) — reconsidered from a
   fully-silent default during spec review, per this project's own
   recurring finding that silent unbounded-scope behavior is a real bug
   source. All 26 tasks complete and independently verified: workspace
   build/clippy/test clean, full 161-file corpus clean across CLI/LSP/MCP,
   2 new golden fixtures (1 hand-written, 1 derived from real
   WF-TDM-Official-Releases shapes) manually verified with zero existing
   fixture changed. Amends `002-cli-check-format/spec.md` (FR-027). See
   `specs/010-fmt-region-markers/`.
3. **Path-related error for `\n`/`\t`/etc. in file/folder names** —
   *deferred/out of scope, owner declined to pursue* (added 2026-08-11,
   clarified same day from WF-TDM-Development issue #52, private repo; set
   aside 2026-08-11 — already decided when the queue was planned, not
   awaiting a decision). Turns out this is **not**
   a Cube Voyager engine bug — it's a bug in WF-TDM-Development's own
   Python automation scripts, which build scenario-data paths via manual
   string concatenation like
   `r"7_PostProcessing\\vizTool_Backup\scenario-data\\" + ScenarioCode + "\\"`.
   `r""` raw-string-literal-ness only covers the static portion of the
   string; concatenating a variable directly after a trailing `\` can form
   a real escape sequence in the *result* string once Python evaluates it —
   so a `ScenarioCode` starting with `n` or `t` silently becomes `\n`
   (newline) or `\t` (tab) instead of a literal backslash-n/t, corrupting
   the path. Issue reporter's own recommended fix is `os.path.join(...)`
   instead of manual concatenation; the assignee (`ar-puuk`, issue comment
   2026-03-12) is instead moving the codebase to `pathlib.Path` more
   broadly, and as of that comment still couldn't locate the exact
   file/line where the reported `r"...\\" + ScenarioCode` concatenation
   happens (asked the reporter to point to it — unresolved as of the
   pasted issue text). This bug lives entirely in a separate Python
   codebase, not in any `.s`/`.block` Voyager script. **Decided**: out of
   scope for drut — set aside, not pursued. No open question remains here;
   don't resurface unless new evidence of a Voyager-script-side analog
   turns up.
4. **`--casing` gains a third mode, `auto`** — *investigated, deliberately
   deferred* (added 2026-08-11, investigated and closed 2026-08-12; not a
   rejection of the idea, a scope call for the current pre-publish push).
   `--casing` currently supports `upper`/`lower` (002-cli-check-format,
   both apply uniformly to every recognized control-word/keyword-name
   token). The owner's starting example was `mw`/`mi`/`mo`/`ni`-style
   matrix I/O abbreviations staying lowercase; investigation ran the
   corpus-census methodology (real tokenizer/parser, not regex) across all
   161 WF-TDM-Official-Releases files and cross-checked independently
   against both `_archive/Citilabs Cube 6.5.1/RG_CUBEVOYAGER.md` and
   `_archive/OpenPaths Cube/md/`.

   **Key finding that reframes the whole question**: the grammar has (at
   least) four distinct token roles relevant to casing — `CONTROL` words,
   `PAIRKEYWORD` names, `ASSIGNMENT_TARGET`s, and (not something the
   codebase already names) **`OPERAND_PREFIX`** — the substring before the
   first `.` in dotted tokens like `mi.1.1`/`li.FT`/`lw.TPen` (`.` isn't a
   lexer delimiter today, so these are single opaque `Word` tokens).
   `MI`/`LI`/`LW`/`NI`/`ZI` — the tokens with the cleanest lowercase
   signal — live in `OPERAND_PREFIX`; `MW` lives in `ASSIGNMENT_TARGET`
   (`MW[1] = ...`); `MO` lives in `PAIRKEYWORD` but is genuinely mixed
   (55.4% upper / 44.6% lower — the owner's own example doesn't hold up
   here). **`--casing` in its current design can only ever reach
   `CONTROL` and `PAIRKEYWORD` tokens** — it structurally cannot touch
   `OPERAND_PREFIX` or `ASSIGNMENT_TARGET` without a real scope expansion.

   **Exception-list evidence (real thresholds, not the owner's original
   guess)**:

   | Category | Confident lowercase exceptions | Notes |
   |---|---|---|
   | Pair-keywords (reachable today) | `NUMREC`, `CNT`, `ITER`, `LP`, `RECNUM`, `MISSINGZO`, `MISSINGZI`, `PERIOD` | Clean, ≥90%+, low volume (2–7 files each) |
   | Control words (philosophy-dependent, see "Open questions" below) | `IF`, `ENDIF`, `ELSE`, `ELSEIF`, `LOOP`, `ENDLOOP` | Corpus 81–95% lowercase; vendor docs 71–94% uppercase — disagree |
   | Operand prefixes (unreachable today) | `LW` 99.9%, `LI` 98.4%, `NI` 97.1%, `MI` 92.0%, `DBA` 95.2% lowercase | `ZI` borderline (89.8%); `DBI` too mixed (60.2%); `RO` is the *opposite* — 100% uppercase |
   | Not supported by evidence at all | `MO` (mixed 55/45), `MW` (assignment-target only, a category neither path below reaches) | Directly contradicts the owner's original example |

   **Two paths sketched, both rejected for now**:
   - **Path (a) — scope `auto` to what's reachable today** (control words +
     the 8-item pair-keyword exception list above). Small, cheap: a static
     lowercase-list check ahead of the existing uppercase transform, no new
     token category, no lexer change. **Doesn't deliver what was asked** —
     `MW`/`MI`/`NI` stay completely untouched by `--casing` in any mode,
     which is the specific thing that motivated the request. The
     pair-keyword set alone is a real but minor consistency nicety, not
     what "matrix I/O casing" implies.
   - **Path (b) — extend the formatter with a genuine new concept**,
     operand-prefix casing. Requires: (1) its own targeted grammar-research
     pass to nail the full closed prefix set with confidence; (2) a
     lexer-level change to expose the pre-`.` boundary without colliding
     with anything else that legitimately contains a `.` (numeric
     literals, etc.) — and per Principle I this has to live in
     `voyager-core`, not a formatter-only regex hack; (3) a genuinely new,
     independent `FormatOptions` axis (not a value inside the existing
     `CasingConvention` — the three categories behave too differently for
     one enum); (4) full CLI/LSP/MCP threading, same shape as 009's
     `TopLevelIndentMode`; (5) a golden-fixture review burden **larger**
     than either 008 or 009's — `LI` appears in 37 of 161 corpus files,
     `MI` in 39, so this would likely touch most of `real_corpus/`, not a
     contained 7-file set. **This is 008/009-scale core-crate scope on its
     own**, and it *still* doesn't fully solve the case that motivated it
     — `MW` only appears as an `ASSIGNMENT_TARGET` in this corpus, a fourth
     token category neither path reaches, so even Path (b) as scoped
     leaves the owner's own `mw` example unhandled unless assignment-target
     casing is added as a further, fifth dimension on top of everything
     above.

   **Decision**: not worth pursuing in the current pre-publish push. Revisit
   once real usage after publish confirms this is actually wanted, or take
   it on as a dedicated future phase with its own grammar-research spike
   budgeted up front (see "Open questions" below for a related, independent
   question that would need answering either way). Full findings (methodology, full
   corpus distribution tables, per-archive vendor-doc results) are not
   preserved elsewhere — re-run the census (throwaway `voyager-core`
   example calling `tokenize_bytes`/`build_statements` directly, same
   approach as the RUN/IF census) if this is picked back up rather than
   trusting memory of the numbers above.

   **Update 2026-08-17**: revisited — real usage did confirm this was
   wanted (GitHub issue #3). Superseded by pre-publish sequence item 10,
   which resolves this differently than either path sketched above: no
   `auto` mode at all, a 3-category `drut.toml` setting instead, and the
   `OPERAND_PREFIX`/`ASSIGNMENT_TARGET` split flagged as unreachable here
   is deliberately deferred again as item 11, now with real evidence
   (Bill's own split vote) rather than left as a philosophy question.
5. **Short-form (single-line, self-closing, no explicit closer) syntax for
   other block-opening control words** — *closed, IF-only confirmed*
   (investigated 2026-08-11). Checked both archive sources' dedicated
   sections for `LOOP`, `JLOOP`, `LINKLOOP`, `RUN`, `PROCESS`/`PHASE`, and
   `DISTRIBUTEMULTISTEP` (Citilabs Cube 6.5.1 Reference Guide; cross-checked
   independently against OpenPaths Cube's own control-statements docs).
   Only `IF` is ever documented as having two forms (a single-statement
   form and a block form) — both sources agree, no disagreement to
   surface. `RUN`'s optional-`ENDRUN`/implicit-close-by-next-`RUN`-or-
   shell-escape and `PROCESS`'s `PHASE=`-shorthand/implicit-close-by-next-
   `PROCESS` are already fully implemented (matches what both sources
   document) and are a different kind of shorthand than short-`IF`'s
   self-closing single-line form, not a gap. No implementation work
   surfaced; no further action needed.
6. **Short-`IF` TextMate syntax-highlighting gap** — *done* (fixed
   2026-08-11). Root cause was not the static grammar (its patterns are
   token-level and line-position-agnostic, so they'd have colored a
   short-IF's body statement correctly on their own) — it was
   `crates/drut-lsp/src/semantic_tokens.rs` emitting one `shortIf` semantic
   token spanning the *entire* short-IF line (header + body statement),
   which overrides the static grammar's normal coloring for everything in
   that span (LSP semantic tokens take priority over TextMate scopes).
   Narrowed the token to just the header (`IF` through the condition's
   closing paren); the body statement now renders under the static
   grammar's own coloring again. LSP-only fix, no `voyager-core` change.
7. **Short-`IF` condition swallowed into one flat color** — *done* (fixed
   2026-08-18). Follow-up to item 6: narrowing the `shortIf` token to "the
   header through the condition's closing paren" fixed the *body* but left
   a second, distinct bug in that same header span — the condition's own
   tokens (`@token@` references, operators, numbers) rendered as one flat
   `shortIf` color instead of their normal distinct static-grammar colors,
   since the one semantic token now covered them too (real-world report:
   `IF (@MODE@ = 1) PRINT ...` showed `@MODE@ = 1` uniformly in the `IF`
   color, where the block-style `IF (@MODE@ = 1)\n...\nENDIF` form colors
   `@MODE@`/`=`/`1` distinctly). This also meant the `shortIf` token
   overlapped the separately-emitted `@token@` variable-ref token whenever
   one appeared in the condition — invalid per the LSP semantic-tokens spec
   (tokens on a line must be non-overlapping), and undefined-behavior
   territory for any client. Narrowed the `shortIf` token further, to just
   the `IF`/`ELSEIF` keyword itself (ending at the condition's opening
   `(`); the condition and body now both render entirely under the static
   grammar's own coloring, matching the block-style form, with no
   overlapping tokens. LSP-only fix (`crates/drut-lsp/src/
   semantic_tokens.rs`), no `voyager-core` change.
8. **Range-dash spacing exemption for `operator_spacing`** — *done*, merged
   2026-08-18 as `023-range-dash-spacing`. `fixed`/`auto` treated every `-`
   inside a `Control` statement's pair-keyword value as arithmetic
   subtraction, spacing it apart (`1 - 50`) — but a `-` joining two bare
   integer literals there is Cube Voyager's own inclusive-range list
   notation (`SELECTLINK=1-50,75,90-100`, `NODES=200-300`), not
   subtraction, and spacing it apart obscured the convention. Confirmed
   against the real fixture corpus, not just reasoned about: four real
   files (`AssignHwy/02_Assign_AM_MD_PM_EV.s`,
   `Distribute/3_SumToDistricts_GRAVITY.s`,
   `Distribute/4pd_mainbody_distribution.block`,
   `ModeChoice/06_HBW_logsums.s`) contain exactly this shape (`mo=31-60`,
   `EXCLUDEGROUP=1-2,7`), and their golden fixtures under
   `operator_spacing=fixed`/`auto` needed updating once this shipped — real
   evidence the bug was live, not hypothetical. A binary `-` inside a
   pair-keyword value with a bare integer literal directly adjacent on both
   sides now renders with zero surrounding whitespace instead of one space
   each side, regardless of how it was originally spaced; every `-`
   elsewhere (an `Assignment` RHS, an `IF`/short-`IF` condition, a `LOOP`
   bound) keeps its existing spacing unchanged, and `preserve` mode is
   unaffected either way. No new `[format]` field, CLI flag, MCP parameter,
   or editor setting — reachable through the existing `operator_spacing`
   setting alone. `crates/voyager-core/src/operator_spacing.rs` only, no
   adapter-crate change.
9. **Function-call syntax highlighting** — *done*, merged 2026-08-18 as
   `024-function-call-highlighting`. The VS Code grammar colored a Cube
   Voyager built-in function call (e.g. `REPLACESTR(...)`) only when it
   happened to sit immediately after `=` and got caught by the unrelated
   `#pair-values` rule — the identical function nested one token deeper
   (`RIGHTSTR(TRIM(RouteName),1)` inside an `IF` condition) rendered
   unstyled. A dedicated `#function-calls` pattern now colors a recognized
   function name every time `(` immediately follows, regardless of
   position. The recognized-name list started as a 21-function census of
   this project's own real corpus, but was deliberately rebuilt to be
   organization-agnostic: a complete reading of every function-related
   chapter in two local vendor documentation mirrors (Cube Voyager 6.5.1
   and OpenPaths Cube/CUBE CONNECT Edition), cross-validated against each
   other (both editions agree completely) — 138 functions across Numeric/
   Trig/Character-String (the general Control Language core), Highway/
   Matrix-program, Public Transport skim, CONVERGE-phase
   iteration-statistics, and CUBE Cluster utility categories, plus one
   real-corpus-confirmed function (`PRINTPROGRESS`) absent from both
   editions. Deliberately excludes a separate camelCase object-model/
   scripting-API surface found in the same OpenPaths docs (e.g.
   `addNonTransitLeg()`) — out of scope for the Voyager control-statement
   language this grammar targets. Verified with a data-driven grammar test
   iterating all 138 names, not just a hand-picked sample. No
   `voyager-core`/parser change, no new `[format]` field, CLI flag, MCP
   parameter, or editor setting — `editors/vscode/syntaxes/
   drut.tmLanguage.json` and its `grammar.test.ts` coverage only.
10. **Function-call casing normalization** — *done*, merged 2026-08-18 as
    `025-function-casing`. A follow-up to item 9 above: reuses that
    feature's 138-name function list — now ported into `voyager-core`
    (`function_call.rs`) as the canonical source, `editors/vscode`'s
    grammar a documented mirror of it, not the other way around — to add a
    fourth independently-configurable casing category,
    `casing_function_calls` (`Preserve`/`Upper`/`Lower`, same shape as
    `casing_control_words`/`casing_pair_keywords`/`casing_data_references`
    from `017-casing-categories-indent-width`). Unlike item 9, a real
    formatter-behavior change: recognized via a new, `data_reference.rs`-
    shaped read-only token scan (a function call routinely appears on an
    `Assignment`'s right-hand side, which `format.rs`'s `control_words`/
    `pair_keywords` AST walk never reaches), gated on the name being
    immediately followed by `(` with zero intervening whitespace — required,
    not just consistent with item 9's own design, since two real names
    collide with existing `voyager-core` vocabulary by coincidence
    (`FORMAT`, a `FILEO` pair-keyword; `LOG`, a control word) and only that
    position check keeps their two roles from ever colliding. Reachable via
    `drut.toml` (`casing_function_calls`), the CLI
    (`--casing-function-calls`), the MCP `format` tool, and the VS Code
    `drut.format.casingFunctionCalls` setting — the same four surfaces
    `017`'s own three categories already use. Golden-fixture verified
    against the real corpus (`golden_casing_function_calls/`, `Upper`
    variant — e.g. `currenttime()` → `CURRENTTIME()`, `formatdatetime(...)`
    → `FORMATDATETIME(...)`, `max(...)` → `MAX(...)`, confirmed live in real
    files, nothing else in each diff); `Lower`'s real-corpus idempotence
    additionally checked with zero extra fixture files (`check_idempotent`
    needs no golden file, only a format-twice self-diff). Defaults to
    `preserve` — zero behavior change for any project that doesn't opt in.
11. **Editor highlight color customization** — *done*, merged 2026-08-18 as
    `026-highlight-customization`. 9 new personal VS Code settings,
    `drut.highlight.<category>` (`controlWords`, `statementWords`,
    `functionCalls`, `pairKeywords`, `values`, `numbers`, `operators`,
    `comments`, `strings`), kept in sync with VS Code's own native
    `editor.tokenColorCustomizations` (User/Global scope) rather than a new
    coloring engine or an LSP semantic-tokens rebuild — reuses the TextMate
    scopes `024`/`025` already ship. Unlike `[format]`/`drut.format.*`,
    deliberately **no** `drut.toml [highlight]` section, CLI flag, or MCP
    parameter: color is a personal/accessibility preference (theme,
    colorblindness, monitor), not a shared file-content convention the way
    casing or indentation is — a decision made explicitly, not defaulted
    into, after discussion. Prerequisite grammar fix: `#statement-words`
    and `#function-calls` shared one scope (`support.function.drut`) since
    `024`, so `statementWords`/`functionCalls` couldn't be colored
    independently; split into `support.function.statement.drut`/
    `support.function.builtin.drut` (pure rename, no visible change for
    anyone not using the new settings). `@name@` substitution
    (`variables`) is deliberately excluded — a real, already-shipped
    mechanism (`ensureVariableColorCustomization`, semantic-token-based,
    workspace-scoped, one-time, added because some themes render that scope
    invisibly) already owns its color, and this feature's TextMate-scope
    mechanism would not visibly win against it; folding it in would mean
    changing that mechanism's tested lifecycle, not just adding a setting.
    Never touches a rule this extension didn't add (exact scope-set match,
    not substring), and skips the write entirely when nothing would
    actually change (found via `/speckit-analyze`: an unconditional write
    on every activation would be harmless in value but still an
    unnecessary `settings.json` touch for a user who never configures this
    at all). Constitution Principle VI (prefer LSP-standard mechanisms) was
    explicitly weighed and set aside — the LSP-standard alternative
    (semantic tokens) would mean substantially expanding `drut-lsp`'s
    narrow, 3-type semantic-tokens implementation to duplicate the
    grammar's own classification logic, for a feature with no cross-editor
    portability goal to begin with.
12. **`@name@` variable highlight color customization** — *done*, merged
    2026-08-18 as `027-named-variable-highlight`. The one category item 11
    deliberately deferred, now added: `drut.highlight.namedVariables`, the
    10th `drut.highlight.*` setting. Reconciles two guarantees that had to
    coexist without regressing either: the pre-existing
    `ensureVariableColorCustomization`'s "a manual deletion of the seeded
    `variable:drut` rule sticks forever, for a workspace that never touches
    this new setting" (fully preserved, verified by dedicated regression
    tests, for anyone who doesn't use it) and this feature's own "live,
    reactive, cleanly-reverting" behavior once a user does set it. Written
    at **Workspace** scope specifically — a deliberate, documented exception
    to item 11's Global-only rule for its other 9 categories, required
    because VS Code resolves an object-valued setting like
    `editor.semanticTokenColorCustomizations` per-scope (not a cross-scope
    deep merge), and the pre-existing default already lives at Workspace
    scope in any workspace this extension has ever activated in — a
    Global-scope write would be silently masked there. Unsetting after a
    custom color was configured reverts to the documented default
    (`#4EC9B0`) rather than removing the override outright, since a fully
    theme-driven state would reintroduce the invisible-under-some-themes
    problem the original mechanism exists to fix. All new logic
    (`decideVariableColorSync`) is a pure, unit-tested function with zero
    `vscode` dependency, same testability discipline item 11 established.
13. **Data-reference & user-variable highlighting** — *done*, merged
    2026-08-19 as `028-identifier-highlighting`. Found by real-world testing
    against a production script: two identifier classes had no genuine
    highlighting mechanism, only accidental coloring from the unrelated
    `pairKeywords`/`pairValues` position-based rules. Two more
    `drut.highlight.*` settings (the 11th/12th), same item-11 personal-
    setting mechanism, no new `vscode` capability. `dataReferences`: the
    Matrix/Line/Node/Zone/Database family (`MI`, `MW`, `DBA`, `ZONES`, ...,
    `casing_data_references`'s own 17-name list) now highlighted by name,
    not position — `DBA` inside `ROUND(DBA.2.VOL[numrec])` renders the same
    as `DBA` in `X = DBA.2.field`. Wins precedence over `pairKeywords`/
    `pairValues` for the same name (`ZONES` in `RUN PGM=MATRIX ZONES=5`) via
    grammar array order, mirroring `data_reference.rs`'s own one-name-one-
    owner rule. `userVariables`: a catch-all for any bareword identifier not
    already claimed by a more specific category, placed last in the
    grammar's pattern list so array position alone is the filtering
    mechanism — fixes `_BNode` rendering unstyled while `_ANode` (purely by
    accident of sitting immediately after `=`) rendered, in the same
    expression. Both skip `Label`/`ShellEscape` lines entirely (two new
    small grammar patterns, `#label`/`#shell-escape`) — neither is real
    Voyager syntax to highlight as a "variable." A bareword immediately
    adjacent to `=` deliberately keeps its existing `pairKeywords`/
    `pairValues` color rather than switching to `userVariables` — this
    grammar has no real parse tree to distinguish a keyword-pair's enum-like
    value from an ordinary assignment's variable reference, and reassigning
    that position would silently change item 11's already-shipped
    behavior. Same investigation also found and fixed an unrelated
    `voyager-core` casing bug (not part of this feature; landed directly on
    `main`): a data-reference name used as a block opener's own value (a
    `LOOP` bound, `LOOP NUMREC = counter, DBI.2.NUMRECORDS`) was invisible
    to `casing_data_references`'s rewrite, since only the opener's
    *keyword*-pair spans were ever scanned, never its value tokens.
14. **Unused `@token@` diagnostic** — *done*, merged 2026-08-19 as
    `029-unused-token-diagnostic`. A fifth Hint-severity, LSP-only diagnostic
    stream (source `drut-token`, code `UnusedToken`) — the exact inverse of
    item-020's `UndefinedToken`: flags an `Assignment` statement whose target
    name is never referenced via `@name@` anywhere in scope. Required one
    real new piece of `voyager-core` logic, not just wiring:
    `all_variable_refs_including_openers`, a new function (not a
    modification — `all_variable_refs` and its `020` consumer are untouched)
    that also scans `Block::opener_tokens` (already added this session for
    the item-13 casing fix) so a token used only on a block-opener line
    (`RUN PGM=@Prog@`) counts as a genuine use — reusing the *unmodified*
    `all_variable_refs` here would have made that position a false positive
    instead of `020`'s own acceptable false negative for the same gap. Two
    scope decisions resolved via direct clarification rather than assumed:
    every dead assignment site is flagged independently when reassigned
    with zero reads (not deduplicated to one-per-name), and the check
    applies unconditionally regardless of whether a file participates in
    any `READ FILE` relationship — a deliberately accepted, documented
    false-positive risk for the shared-parameters-file authoring pattern
    (a name used only by a file that includes this one is invisible to any
    existing resolution logic, in either direction), permanently mitigated
    by staying at Hint severity, never CLI/MCP reach. Surfaced and fixed a
    latent bug in several pre-existing tests' fixtures along the way (bare
    `Y = 1`-shaped assignments incidental to what those tests actually
    checked, now genuinely flagged) and one shared corpus fixture
    (`undecodable_byte.s`) — fixed by converting the incidental assignments
    to non-`Assignment` statements, verified byte-for-byte that the fixture's
    actual invalid-UTF-8 byte was untouched.

## Open questions (not part of the pre-publish sequence)

Unlike the log above, this is **not** resolved — a genuine open question,
surfaced during the resolved item 4's investigation, still awaiting an
answer:

- **Casing philosophy: real-usage-fidelity vs. vendor-canon-fidelity**
  (added 2026-08-12). The `IF`/`ENDIF`/`ELSE`/`ELSEIF`/`LOOP`/`ENDLOOP`
  family sits on a genuine, high-volume fork: the real
  WF-TDM-Official-Releases corpus is 81–95% lowercase for these
  (hundreds to thousands of occurrences each); both vendor doc archives
  (Citilabs 6.5.1 and OpenPaths — which gave near-identical counts on
  every token checked, suggesting one underlying Bentley-authored source
  lineage rather than two independent ones) are 71–94% uppercase. More
  evidence won't resolve this — it's a values call: **should any future
  `auto`-style casing mode match what real WFRC analysts actually write**
  (minimizing reformatting diff on real scripts), **or Bentley's
  documented canonical style** (the "textbook-correct" look)? Not urgent
  on its own — nothing currently depends on it — but flagged here because
  it would need an answer before the resolved log's item 4 (`--casing
  auto`) or any future casing-related work can proceed, and the answer
  likely isn't token-specific: whichever philosophy is picked probably
  applies consistently across future casing decisions, not just this one
  family.

  **Resolved as moot, 2026-08-17**: pre-publish sequence item 10 decided
  not to ship any opinionated `auto`/preset mode at all, so this question
  no longer needs an answer from drut itself — each project's own
  `drut.toml` states its own preference, informed by whichever philosophy
  its own authors care about, without the tool having to pick a side.
  Separately worth recording, since it surfaced during the vendor-doc
  research for item 10: the FORTRAN/COBOL-era all-caps convention that
  vendor-canon uppercase likely traces back to was itself a 1950s-60s
  hardware artifact (punch cards/line printers had no lowercase glyphs),
  not a principled design choice — so "vendor-canon" was never actually
  the more-authoritative side of this fork, just the older habit. Left
  here as a closed record, not deleted, since the investigation itself
  remains useful context.

## Later / stretch (explicitly not part of the pre-publish sequence)

Named in the original project framing as hypothetical future phases, out of
scope for every phase shipped so far (most recently restated in
`specs/004-mcp-server/spec.md`'s own Out of Scope list) — listed here only so
they aren't mistaken for part of the sequence above:

- **Phase 5 — per-program-box keyword validation.**
- **Phase 6 — repo-wide/multi-file semantic checking.**
