# Phase 0 Research: Format-On-Save and Format-On-Paste

## §1. `textDocument/rangeFormatting` is already available in the pinned `lsp-types`

**Decision**: Implement format-on-paste's server side as a genuine
`textDocument/rangeFormatting` LSP capability, using `lsp-types`' existing
`RangeFormatting` request type — no crate upgrade, no new dependency.

**Rationale**: Verified directly against the vendored source for the
version `drut-lsp/Cargo.toml` already pins
(`~/.cargo/registry/src/.../lsp-types-0.97.0/src/request.rs`):

```rust
pub enum RangeFormatting {}
impl Request for RangeFormatting {
    type Params = DocumentRangeFormattingParams;
    type Result = Option<Vec<TextEdit>>;
    const METHOD: &'static str = "textDocument/rangeFormatting";
}
```

`DocumentRangeFormattingParams` (`lsp-types-0.97.0/src/formatting.rs`)
carries `text_document`, `range: Range`, and `options: FormattingOptions` —
the same shape `DocumentFormattingParams` already has, plus the range.
`ServerCapabilities::document_range_formatting_provider` also already
exists (`lsp-types-0.97.0/src/lib.rs:1880`). This whole capability is
additive to `drut-lsp/src/lib.rs`'s existing `server_capabilities()`/
`handle_request` — no change to how `Formatting`/`HoverRequest`/etc. are
already wired.

Separately confirmed (web research, prior to `/speckit-specify`): VS
Code's own `editor.formatOnPaste` setting is served by
`DocumentRangeFormattingEditProvider` — i.e. exactly this LSP capability,
not a bespoke paste-specific provider (`DocumentPasteEditProvider` is a
different, newer VS Code API for transforming pasted *content* — e.g. URL
autolinking — unrelated to reformatting). This is why the earlier
`ROADMAP.md` draft naming `DocumentPasteEditProvider` was wrong and
corrected before this feature's spec was written (spec.md's Input section
already reflects the corrected mechanism).

**Alternatives considered**: A VS Code-proprietary paste-transform
provider was rejected outright — Principle VI (LSP-Standard Mechanisms
Over Editor-Proprietary APIs) directly prefers the standard mechanism here,
and it happens to also be the *only* mechanism `editor.formatOnPaste`
actually consults, so there was no real tradeoff to weigh.

## §2. Range-formatting computation strategy (resolves spec.md FR-003's deferred "how")

**Decision**: On a `textDocument/rangeFormatting` request, run a normal
whole-document `voyager_core::format(&doc.text, FormatOptions::default())`
call (identical to what `formatting.rs` already does), then compute a
**line-by-line diff** between the original and formatted text, and return
`TextEdit`s only for the lines whose 0-based LSP line number falls within
`[range.start.line, range.end.line]` (inclusive).

**Rationale**: `voyager-core`'s formatter has a documented, narrow scope
(`format.rs`'s own module doc): it only ever rewrites a line's *leading*
whitespace (and, opt-in only, keyword casing — never enabled by any LSP
caller, `formatting.rs`'s existing `casing: None`). It never inserts,
removes, reorders, or merges lines. That guarantee makes a line-by-line
diff **exact and cheap** — no generic diffing algorithm (e.g. the
`similar` crate `drut-cli` uses for `--diff` output) is needed, because
line N in the original always corresponds to line N in the formatted
output; the two texts can never disagree on line count. Concretely:

```text
for (line_idx, (original_line, formatted_line))
    in original.lines().zip(formatted.lines()).enumerate()
{
    if original_line != formatted_line
       && line_idx is within the requested range
    {
        emit a TextEdit replacing that line's content
    }
}
```

This directly resolves spec.md's deferred question in favor of
**option (a)**: "run whole-document format and return only the edits
overlapping the requested range." The rejected option (b) — some narrower,
range-local-only computation — was never actually viable: `voyager-core`'s
formatter derives a line's correct indentation from its position in the
*whole* document's block-nesting structure (`format.rs`'s `plan_indentation`
step), so computing a correct answer for even one line already requires
parsing and reasoning about the whole document. There is no cheaper
"local" computation to fall back to — the whole-document pass was always
going to happen; this decision is really just "what to do with its output,"
not "how much of the document to examine."

**Edge case resolution** (spec.md's "paste straddles a block boundary"
case): if the paste changes nesting depth such that a line *outside* the
requested range would also need reindenting (e.g. the block's closer,
arriving later), that correction is **not** applied — it falls outside
`[range.start.line, range.end.line]` and is filtered out, per FR-003's
explicit scope limit. The user still gets a fully correct result by
running "Format Document" (or save, once format-on-save is on) afterward,
which has no such range restriction. This is a deliberate, LSP-convention-
matching choice (range-formatting requests are conventionally answered
within their requested range only), not an oversight.

**Alternatives considered**:
- Returning the *entire* whole-document diff regardless of the requested
  range: rejected — technically simpler, but a range-formatting response
  editing content the client never asked about is surprising behavior and
  not what other language servers do for this request kind.
- A dedicated "format just this range in isolation" parse/render path in
  `voyager-core`: rejected — would require new public API surface and new
  formatting-logic branching in `voyager-core` for a capability whose
  correctness still depends on the whole document's structure anyway
  (see above); the whole-document-then-filter approach gets the identical
  correctness with zero new `voyager-core` surface, which is also strictly
  better for Principle I (no new grammar/formatting logic to justify).

## §3. Format-on-save auto-injection: the correct VS Code configuration API

**Decision**: Use `vscode.workspace.getConfiguration(undefined, { languageId: "drut-voyager" })`
together with `WorkspaceConfiguration.update`'s fourth parameter
(`overrideInLanguage: true`) to write a genuine language-scoped override —
**not** the `"[drut-voyager]"`-object-merge trick `ensureVariableColorCustomization`
uses for `editor.semanticTokenColorCustomizations`.

**Rationale**: These are two different problems that happen to look
similar. `editor.semanticTokenColorCustomizations` is a single *global*
setting whose *value* is an object with a `rules` map keyed by scope name
(`"variable:drut-voyager"`) — writing into it is an ordinary
`config.update("editor.semanticTokenColorCustomizations", {...}, Workspace)`
call on a regular, unscoped setting, which is exactly what 003's existing
code already does. `editor.formatOnSave`, by contrast, is a plain boolean
setting with no such per-language rule-key convention of its own — making
it apply to `.s`/`.block` files *only* (not every language in the
workspace) requires VS Code's actual language-override mechanism, which
writes a `"[drut-voyager]": { "editor.formatOnSave": true }` block into
`.vscode/settings.json`, not a flat top-level key.

Confirmed via a real, working example
([eliostruyf.com, "language-specific settings in a VSCode extension"](https://www.eliostruyf.com/devhack-language-specific-settings-vscode-extension/)):

```typescript
const config = vscode.workspace.getConfiguration("", { languageId: "markdown" });
await config.update("editor.fontSize", 14, vscode.ConfigurationTarget.Workspace, /* overrideInLanguage */ true);
```

The naive alternative — treating `"[drut-voyager]"` itself as a config
*section* name (`getConfiguration("[drut-voyager]")` then
`.update("editor.formatOnSave", true, ...)`) — was checked and rejected:
[`microsoft/vscode#89486`](https://github.com/microsoft/vscode/issues/89486)
documents this approach failing to create the language section at all when
it doesn't already exist in `settings.json`, which is exactly this
feature's first-activation case (nothing exists yet). The
`overrideInLanguage: true` fourth-parameter form is the mechanism VS Code
itself documents and implements this case correctly for.

**Detecting an existing override (for the one-time gate)**: use
`config.inspect("editor.formatOnSave")` and check its
`.workspaceLanguageValue` field specifically, rather than `config.get(...)`
— `get` would return the setting's *effective merged value* (which could
be `true` from an unrelated global default having nothing to do with this
feature), while `inspect().workspaceLanguageValue` reports only whether
*this exact* language-scoped workspace override already exists, which is
the only case that should suppress injection.

**Alternatives considered**:
- Reusing the exact `"[drut-voyager]"`-merge pattern
  `ensureVariableColorCustomization` uses, applied naively to
  `editor.formatOnSave`: rejected per `#89486` above — would silently fail
  to create the override on a workspace with no prior `.vscode/settings.json`
  language section, the majority first-activation case this feature exists
  to handle.
- A global (not workspace-scoped) injection: rejected, same reasoning 003
  already established for the color injection — workspace-scoped is
  visible, inspectable, and trivially removable per-project, and does not
  change the user's behavior for unrelated languages in unrelated
  projects.

## §4. Format-on-paste stays undocumented-in-code, documented-in-README only

**Decision**: No `extension.ts` injection function for
`editor.formatOnPaste` at all (Clarification Q1, Option C) — the
capability exists once `range_formatting.rs` ships, and a user turns it on
themselves via the standard VS Code setting, following a short README
instruction this feature adds.

**Rationale**: Directly follows from the resolved clarification; no
further design decision needed here. Documenting *how* to turn on a
built-in VS Code setting for a specific language needs no extension code
at all — `"[drut-voyager]": { "editor.formatOnPaste": true }` in the user's
own `settings.json` is standard, already-documented VS Code behavior once
the range-formatting *capability* exists server-side.

## §5. No new dependencies anywhere

**Decision confirmed**: `Cargo.toml`s (root and `drut-lsp`) are unchanged;
`editors/vscode/package.json`'s `dependencies`/`devDependencies` are
unchanged. Only `contributes.languages`/config text may gain a short
documentation note (not a new configuration contribution point — see
`contracts/extension-settings.md`).
