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
8. **Actual publish** (VS Code Marketplace + Open VSX + crates.io) — *not
   started*. Two independent sub-parts: the extension (Marketplace + Open
   VSX, blocked on item 7 per above) and the Rust crates (`crates.io`,
   independent of item 7 — blocked only on adding `version` fields to every
   internal path dependency, which `cargo publish` requires and which are
   currently absent from all five `Cargo.toml`s).
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
12. **`=`/operator spacing normalization** (`PHASE=ILOOP` / `PHASE =ILOOP`
    / `PHASE= ILOOP` / `PHASE = ILOOP` all becoming one canonical form) —
    *queued, not started, needs more research before it's spec-ready*
    (added 2026-08-17). Confirmed genuinely new scope, not an existing
    gap: `format.rs`/`formatting-api.md` today normalize indentation and
    (optionally) casing only — no existing logic touches spacing around
    `=` or any other operator, and the real corpus shows this is
    genuinely inconsistent even within one file (e.g. `ZONES   = 1` three
    lines from `ZONES = 1` in the same fixture). Three shapes discussed,
    none settled yet:
    - An `auto` mode — one space on each side of `=` — but it's not yet
      decided how (or whether) this extends to other operators (`+`,
      `-`, `==`, etc.) inside computed expressions; that's a materially
      bigger, fuzzier scope than `=`-spacing alone (closer to a
      line-reflow feature than a spacing tweak) and needs its own
      research pass before it's folded in, not assumed.
    - Explicit adoption of the Tidyverse (R) style-guide convention —
      always one space around `=`, including in named/keyword-argument
      position, which is where R's own convention deliberately differs
      from e.g. Python's PEP 8 — needs a deeper read of the full
      Tidyverse (and/or Air formatter) rule set before committing to it
      as Voyager's own canonical form, not just the `=` rule in
      isolation.
    - A `preserve` mode (default) — leaves existing spacing exactly as
      written, mirroring this project's own established default-to-
      `Preserve` pattern for every other formatting axis shipped so far.
    - **Related cases identified, not yet scoped** (ranked by how directly
      they follow from `=`-spacing): (1) the same `=` rule applied to
      `Assignment` statements (`MW[1]=mi.1.1` vs `MW[1] = mi.1.1`) — small,
      direct extension; (2) comma spacing between multiple pairs on one
      statement (`MATI=a.mat,MATO=b.mat`, real corpus already shows the
      no-space form) — small, direct extension; (3) arithmetic/comparison
      operator spacing inside expressions (`mi.1.1+mi.2.1`, `I==1`) — the
      "other operators" question above, materially bigger; (4) spacing
      between a control word and its opening paren (`IF(x)` vs `IF (x)`)
      — a separate axis; (5) interior padding inside brackets/parens
      (`MW[ 1 ]` vs `MW[1]`) — another separate axis.

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
