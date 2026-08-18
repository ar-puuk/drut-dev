# Contract: Site Structure

Defines the required chapter list (`docs-site/src/SUMMARY.md`) and the minimum
content each chapter must contain to satisfy spec.md FR-002. `mdbook build` fails
if `SUMMARY.md` references a file that doesn't exist, which mechanically enforces
the chapter list once `SUMMARY.md` is authored — this contract governs what belongs
*in* each file, which `mdbook build` cannot check for itself.

| # | Chapter (file) | MUST contain | Source material to draw from (rewrite, don't copy) |
|---|---|---|---|
| 1 | `introduction.md` | What Drut is, who it's for, why it exists, what it does NOT do (no per-program-box semantic validation — matches `README.md`'s existing scope framing) | `README.md`'s existing pitch paragraph |
| 2 | `install.md` | CLI install (via `cargo install drut-cli` and building from source); VS Code/Open VSX extension install; a note that the extension self-installs its own `drut` binary (no separate CLI install needed for editor-only use) | `README.md` Install section; `ROADMAP.md` item 7 (extension auto-install) |
| 3 | `getting-started.md` | A runnable walkthrough: install, then `drut check` and `drut format --diff` against a small sample script, with the actual expected output shown | `CONTRIBUTING.md`'s "Try the CLI" snippet, expanded with real sample output |
| 4 | `cli-reference.md` | Every subcommand (`check`, `format`, `server`, `mcp`) and every `format` flag, sourced from data-model.md's field table plus `check`'s `--format`/`format`'s `--write`/`--check`/`--diff`/`--isolated` | `crates/drut-cli/src/cli.rs` doc comments (rewritten, not copied verbatim) |
| 5 | `editor-guide.md` | Diagnostics (all categories), hover, completion/spell-check, folding, format-on-save (auto-on) and format-on-paste (opt-in, with the exact `.vscode/settings.json` snippet), the undefined-`@token@` hint (`020`, with its documented blind spots stated honestly per constitution Principle VII), and editor client settings (`021`, the 10 `drut.format.*` settings and their precedence position) | `CONTRIBUTING.md`'s "Editor behavior" section; `specs/020-undefined-token-diagnostic/`, `specs/021-editor-settings-config/` |
| 6 | `mcp-guide.md` | All four tools (`diagnose`, `format`, `query_structure`, `lookup_keyword`) — what each returns and when an AI-assistant integrator would use it | `specs/004-mcp-server/` |
| 7 | `formatter-guide.md` | What the formatter guarantees (idempotent, never reorders statements or changes program meaning — Principle III, stated accurately) and what each axis does (casing categories, indentation, operator spacing, blank-line normalization) with a before/after example per axis; `; FMT: OFF`/`; FMT: ON` regions | `specs/018-operator-spacing/`, `specs/019-blank-line-normalization/`, `specs/010-fmt-region-markers/`, `CHANGELOG.md` |
| 8 | `configuration-reference.md` | One entry per field per contracts/config-reference-entry.md, plus the shared precedence-chain explanation (data-model.md) stated once | data-model.md (authoritative for this feature) |

**Navigation**: `SUMMARY.md` lists all 8 chapters in the order above; mdBook
renders this as the sidebar automatically — no separate navigation file needed.
