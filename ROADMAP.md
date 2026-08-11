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

1. **Format-on-save** — *not started*. The LSP-side prerequisite,
   `textDocument/formatting`, already exists (`crates/drut-lsp/src/formatting.rs`,
   added during 003's manual verification pass) — what's left here is purely the
   client-side decision of whether/how to default `editor.formatOnSave` on for
   this language, e.g. auto-injecting a language-scoped setting the same way
   `extension.ts`'s `ensureVariableColorCustomization` already does for semantic
   token colors.
2. **Format-on-paste** — *not started*. Real new work, not a settings toggle —
   VS Code's `editor.formatOnPaste` is served by `textDocument/rangeFormatting`
   (`DocumentRangeFormattingEditProvider`), which `drut-lsp` doesn't implement
   yet (only whole-document `textDocument/formatting`); needs a new LSP
   capability, not a client-side paste hook (corrected 2026-08-10 — an earlier
   version of this line named the wrong VS Code mechanism).
3. **TOML-based configuration** — *not started*. Let users control settings via
   a TOML file (preferred over `settings.json`).
4. **README/docs overhaul** — *not started*. Features, install steps, usage,
   brought up to date with everything shipped through 004.
5. **CI + release pipeline** — *not started*. Blocking prerequisite for both
   item 6 and item 7 below — no CI exists in this repo yet. Needs to produce
   per-platform (Windows/macOS/Linux) `drut` binaries at minimum.
6. **Extension auto-install/update ("out of the box" binary experience)** —
   *not started, researched only* (2026-08-10). Two real patterns compared:
   rust-analyzer's (binary downloaded from GitHub Releases on first activation —
   small `.vsix`, decoupled from binary rebuilds) vs. ruff's (binary bundled
   directly into the `.vsix`/npm package — no network call needed, but requires
   a full per-platform build matrix upfront). Both are blocked on item 5.
7. **Actual publish** (VS Code Marketplace + Open VSX + crates.io) — *not
   started*. Known gap already flagged and not yet fixed: `vsce` packages
   `editors/vscode/` in isolation, so the repo-root `LICENSE-MIT`/`LICENSE-APACHE`
   files (added in `0ad5500`) will **not** automatically land inside the
   `.vsix` — needs a copy step, or a `files`/`.vscodeignore` adjustment in
   `editors/vscode/`, before this step.

## Queued items (not started, not part of the pre-publish sequence)

Corrections, unspec'd features, and investigations agreed on but deliberately
not started yet, so they don't context-switch away from whatever's actively
being implemented (currently 008). Each needs explicit go-ahead from the
owner once the in-flight work is clear.

1. **Top-level indentation normalization: default reverts to leave-untouched,
   008's forcing becomes an opt-in toggle** — *not started* (corrected
   2026-08-11, owner's explicit preference). `008-top-level-indentation-
   normalization` changed FR-012 so top-level (depth-0) statement indentation
   is *always* normalized to column 0, unconditionally, replacing 007-era
   FR-012 (leave top-level indentation untouched — the corpus showed no
   dominant convention: best single value only 26.9% at column 8, only 20.4%
   at column 0; see `specs/008-.../contracts/top-level-indentation.md`). This
   item reverses that reversal: the **original 007-era leave-untouched
   behavior becomes the default again**, and 008's column-0-forcing becomes
   an **opt-in toggle** for users who want Python-style predictability
   instead. Belongs with pre-publish item 3 (TOML-based configuration) —
   the toggle is a TOML config item, not a new CLI flag. Not yet
   implemented; needs its own amendment pass over FR-012 and the 008
   contract the same way 008 amended 007.
2. **`; FMT: OFF` / `; FMT: ON` region markers** — *not started, new feature,
   not yet spec'd* (added 2026-08-11). Lets users mark a line range to be
   skipped entirely by `drut format`. Reference 007's diagnosed-block-skip
   mechanism (`diagnosed_block_openers`/`plan_block`'s skip-a-diagnosed-
   block's-children logic, `specs/007-.../research.md` §1) as an
   architectural starting point — not a direct reuse, since that mechanism
   skips based on diagnosed block structure, not user-placed markers.
3. **Path-related error for `\n`/`\t`/etc. in file/folder names** — *scope
   against drut still unclear* (added 2026-08-11, clarified same day from
   WF-TDM-Development issue #52, private repo). Turns out this is **not**
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
   pasted issue text). **Open question before this is actionable for
   drut**: this bug lives entirely in a separate Python codebase, not in
   any `.s`/`.block` Voyager script — does it have any analog *inside*
   Voyager control statements themselves (e.g. a `FILEI`/`FILEO` path
   built by concatenating a literal ending in `\` with an `@TOKEN@` that
   could start with `n`/`t`) that would be in drut's tokenizer/lint scope,
   or is this simply out of scope for drut entirely? Owner to weigh in
   before any further action.
4. **`--casing` gains a third mode, `auto`** — *not started, two open
   questions block spec-readiness* (added 2026-08-11, refined same day).
   `--casing` currently supports `upper`/`lower` (002-cli-check-format,
   both apply uniformly to every keyword/control word). The new `auto` mode
   would uppercase all control words/keywords **except** a specific
   exception list that stays lowercase regardless of surrounding casing
   convention — the owner's starting example was `mw`/`mi`/`mo`/`ni`-style
   matrix I/O abbreviations (`MW`=Matrix Write, `MI`=Matrix Input,
   `MO`=Matrix Output), but the exact list isn't confirmed yet. Open
   questions to resolve before this is spec-ready:
   - Is the exception list **owner-defined** (a fixed list the owner
     chooses) or **corpus-derived** (same census methodology already used
     for the RUN/IF families: for every distinct control word and
     `keyword=value` pair name across the WF-TDM-Official-Releases corpus,
     measure what fraction of real occurrences are upper/lower/mixed, and
     flag tokens that are overwhelmingly, e.g. >=90%, lowercase despite most
     other tokens in the same files being uppercase)?
   - Does `auto`'s exception logic apply to control words only, to
     `keyword=value` pair names only, or to both?
   Also clarify, before assuming `ni`'s lowercase convention as a given:
   whether "MW/MI/MO casing" as the owner raised it means (a) hover/
   completion **display** casing, or (b) what `drut format --casing`
   **normalizes to** when casing normalization is enabled — these could
   have different answers. If a corpus-derived answer is chosen, cross-check
   it independently against local vendor documentation (`_archive/Citilabs
   Cube 6.5.1/RG_CUBEVOYAGER.md` and `_archive/OpenPaths Cube/md/`) for the
   casing convention Bentley's own worked examples use for these same
   tokens, and report where the two sources agree vs. disagree rather than
   silently picking one. Findings get reported directly to the owner before
   any `--casing=auto` spec work begins.
5. **Short-form (single-line, self-closing, no explicit closer) syntax for
   other block-opening control words** — *not started, investigate first*
   (added 2026-08-11). Short-`IF` (`IF (...) STATEMENT` on one line, no
   `ENDIF`) is already implemented and correctly produces zero diagnostics
   (FR-007). Investigate whether Voyager has an equivalent shorthand for
   `LOOP`, `JLOOP`, `PROCESS`, `RUN`, or any other block-opening control
   word. Check the local Cube 6.5.1/OpenPaths documentation archive first —
   same method used for the original short-`IF` discovery — before assuming
   `IF` is the only one that has this.
6. **Short-`IF` TextMate syntax-highlighting gap** — *not started* (added
   2026-08-11). `IF (...) STATEMENT` on one line correctly produces zero
   diagnostics (working as designed per FR-007), but the static
   `.tmLanguage.json` grammar has no pattern for this shape, so everything
   after `IF (...)` renders in one generic/undifferentiated highlight color
   instead of proper keyword/string/etc. token coloring. Check whether LSP
   semantic tokens have the same gap or only the static grammar does — if
   only the static grammar is affected, the LSP-connected editor path may
   already be fine and this is VS Code's non-LSP fallback highlighting only.

## Later / stretch (explicitly not part of the pre-publish sequence)

Named in the original project framing as hypothetical future phases, out of
scope for every phase shipped so far (most recently restated in
`specs/004-mcp-server/spec.md`'s own Out of Scope list) — listed here only so
they aren't mistaken for part of the sequence above:

- **Phase 5 — per-program-box keyword validation.**
- **Phase 6 — repo-wide/multi-file semantic checking.**
