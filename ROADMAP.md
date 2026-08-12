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
   cases. Ships opt-in — see README's "Editor behavior" section for the
   setting. Code complete, automated-tested, and the real-VS-Code smoke
   test (quickstart.md step 6) manually confirmed.
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
2. **`; FMT: OFF` / `; FMT: ON` region markers** — *not started, new feature,
   not yet spec'd* (added 2026-08-11). Lets users mark a line range to be
   skipped entirely by `drut format`. Reference 007's diagnosed-block-skip
   mechanism (`diagnosed_block_openers`/`plan_block`'s skip-a-diagnosed-
   block's-children logic, `specs/007-.../research.md` §1) as an
   architectural starting point — not a direct reuse, since that mechanism
   skips based on diagnosed block structure, not user-placed markers.
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
   | Control words (philosophy-dependent, item 7 below) | `IF`, `ENDIF`, `ELSE`, `ELSEIF`, `LOOP`, `ENDLOOP` | Corpus 81–95% lowercase; vendor docs 71–94% uppercase — disagree |
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
   budgeted up front (see item 7 below for a related, independent question
   that would need answering either way). Full findings (methodology, full
   corpus distribution tables, per-archive vendor-doc results) are not
   preserved elsewhere — re-run the census (throwaway `voyager-core`
   example calling `tokenize_bytes`/`build_statements` directly, same
   approach as the RUN/IF census) if this is picked back up rather than
   trusting memory of the numbers above.
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

7. **Casing philosophy question: real-usage-fidelity vs. vendor-canon-
   fidelity** — *open, independent of item 4* (added 2026-08-12, surfaced
   during item 4's investigation). The `IF`/`ENDIF`/`ELSE`/`ELSEIF`/
   `LOOP`/`ENDLOOP` family sits on a genuine, high-volume fork: the real
   WF-TDM-Official-Releases corpus is 81–95% lowercase for these
   (hundreds to thousands of occurrences each); both vendor doc archives
   (Citilabs 6.5.1 and OpenPaths — which gave near-identical counts on
   every token checked, suggesting one underlying Bentley-authored source
   lineage rather than two independent ones) are 71–94% uppercase. More
   evidence won't resolve this — it's a values call: **should any future
   `auto`-style casing mode match what real WFRC analysts actually write**
   (minimizing reformatting diff on real scripts), **or Bentley's
   documented canonical style** (the "textbook-correct" look)? Not urgent
   on its own — nothing currently depends on it — but flagged as its own
   entry because it would need an answer before item 4 (or any future
   casing-related work) can proceed, and the answer likely isn't
   token-specific: whichever philosophy is picked probably applies
   consistently across future casing decisions, not just this one family.

## Later / stretch (explicitly not part of the pre-publish sequence)

Named in the original project framing as hypothetical future phases, out of
scope for every phase shipped so far (most recently restated in
`specs/004-mcp-server/spec.md`'s own Out of Scope list) — listed here only so
they aren't mistaken for part of the sequence above:

- **Phase 5 — per-program-box keyword validation.**
- **Phase 6 — repo-wide/multi-file semantic checking.**
