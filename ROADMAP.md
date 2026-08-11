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

## Later / stretch (explicitly not part of the pre-publish sequence)

Named in the original project framing as hypothetical future phases, out of
scope for every phase shipped so far (most recently restated in
`specs/004-mcp-server/spec.md`'s own Out of Scope list) — listed here only so
they aren't mistaken for part of the sequence above:

- **Phase 5 — per-program-box keyword validation.**
- **Phase 6 — repo-wide/multi-file semantic checking.**
