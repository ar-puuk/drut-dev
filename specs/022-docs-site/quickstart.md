# Quickstart: Validating the Published Documentation Site

Manual/CI validation steps proving this feature works end-to-end. No new Rust
tests — validation is build-success plus direct content/link checks, per
plan.md's Testing strategy.

## Prerequisites

- mdBook installed locally for authoring/preview and for publishing
  (`cargo install mdbook` — a one-time local tool install, not a project
  dependency; CI installs its own pinned copy independently for the build-check
  job, see `.github/workflows/docs.yml`).
- PowerShell (already the project's primary shell — CLAUDE.md).

## 1. Build the site locally

```powershell
.\scripts\build-docs.ps1
```

**Expected**: exits 0. Internally this runs `mdbook build` from `docs-site/`
(output redirected to repo-root `docs/` per `book.toml`'s `build-dir =
"../docs"`, research.md §2 — `docs/` is the committed, published output, not a
gitignored `docs-site/book/`) and then (re)creates `docs/.nojekyll` — a single
wrapper script so a local publish and CI's freshness check (step 4a) always build
the same way (see tasks.md Setup phase). No "file not found" errors for any
`SUMMARY.md` entry.

## 2. Preview it

```powershell
cd docs-site
mdbook serve --open
```

**Expected**: opens a local browser tab; sidebar lists all 8 chapters in
contracts/site-structure.md's order; the search box (top-left, magnifying glass)
returns a result for typing "blank_lines".

## 3. Run the configuration-reference coverage check

```powershell
.\scripts\check-docs-coverage.ps1
```

**Expected**: exits 0 and prints confirmation that all 10 `FormatConfig` field
names (data-model.md's table) have a matching heading in
`docs-site/src/configuration-reference.md`. Deliberately break it once during
implementation (temporarily delete one field's heading, re-run, confirm a non-zero
exit and a message naming the missing field) to prove the check actually catches
the failure mode it exists for — then restore the heading.

## 4. Confirm the build-check CI job runs — and that it never deploys

Open a PR against this branch (or push to it, since `ci.yml`'s existing trigger
already covers `[0-9][0-9][0-9]-*` branches) and confirm in the Checks tab:

**Expected**: a single `docs` job (from `.github/workflows/docs.yml`) runs
`mdbook build` + the coverage check + the freshness check (research.md §6) and
passes. Confirm by reading `docs.yml` directly that there is no second job, no
`pages: write`/`id-token: write` permission, and no `actions/deploy-pages`/
`upload-pages-artifact` step anywhere in the file — publishing is intentionally
not this workflow's job (research.md §2/§2a).

## 4a. Prove the freshness check actually catches a forgotten rebuild

During implementation, once real content exists: edit any line in
`docs-site/src/introduction.md` without rebuilding, commit just that source
change, and push.

**Expected**: the `docs` job's freshness-check step fails, naming `docs/` as out
of sync with its source. Then run `.\scripts\build-docs.ps1` and commit the
regenerated `docs/`, push again, and confirm the job now passes — this is the
mechanism that stands in for a deploy step's own "did this actually take effect"
signal (spec.md Edge Cases).

## 5. Publish for real (after merge to `main`)

This is the actual, Actions-free "deploy" — a normal commit, not a separate
pipeline. Not part of this spec-kit cycle's own local validation (it requires a
real push reaching `main`), documented here so a maintainer has the exact steps:

1. On `main`, after this feature (and any later content change) is merged:
   `.\scripts\build-docs.ps1`.
2. `git add docs/ && git commit -m "..." && git push`.
3. One-time only, the first time this feature ships: in the repository's Settings
   → Pages, set Source to "Deploy from a branch," Branch to `main`, Folder to
   `/docs`.

**Expected**: the site is reachable at the default GitHub Pages URL (research.md
§4) within about a minute of the push — no separate build/deploy latency, since
the served content is the pushed commit itself (SC-005).

## 6. Confirm `README.md`/`CONTRIBUTING.md` migration (SC-003/SC-004)

```powershell
git diff main -- README.md CONTRIBUTING.md
```

**Expected**: `README.md`'s Documentation section now links to the published site
as the documentation home rather than only `CONTRIBUTING.md`/`CHANGELOG.md`/
`ROADMAP.md`; no net growth in `README.md`'s length. `CONTRIBUTING.md`'s
"Configuration" and "Editor behavior" sections are replaced with short pointers to
the corresponding site chapters — grep for the field names `top_level_indent`/
`casing` in `CONTRIBUTING.md` post-change and confirm they no longer appear
alongside full prose explanations there (only, at most, in a one-line pointer).
Also confirm `CONTRIBUTING.md`'s Workflow section now states FR-011's
update-the-site-as-part-of-your-own-change obligation explicitly (analyze finding
E2), not only in `spec.md`.

## 7. End-to-end User Story 1 walkthrough (the named pain point)

As a fresh reader with no prior context, open the published (or locally-served)
site and locate the entry for `blank_lines` using only the search box or sidebar
navigation — no grepping source, no reading `spec.md`.

**Expected**: reachable in under 3 clicks/one search query; the entry states its
values, default, effect, example, and precedence per contracts/
config-reference-entry.md, matching spec.md User Story 1's acceptance scenarios.

## 8. End-to-end User Story 2 walkthrough (newcomer install-to-result)

As a fresh reader with no prior Drut familiarity, follow only
`docs-site/src/install.md` then `getting-started.md` — no other page, no outside
help — through to running `drut check` and `drut format --diff` against the
walkthrough's own sample script.

**Expected**: both commands run successfully and produce output matching what
`getting-started.md` shows; separately, confirm `mcp-guide.md` alone lets a
reader name all four MCP tools (`diagnose`/`format`/`query_structure`/
`lookup_keyword`) and what each is for, per spec.md User Story 2's Independent
Test and SC-002.

## 9. End-to-end User Story 3 walkthrough (formatter predictability)

Read only `docs-site/src/formatter-guide.md`, then, without running `drut`,
write down the predicted output for: (a) a `casing = upper` example, (b) an
`operator_spacing` example under `preserve` vs. `fixed` vs. `auto`, (c) a
`blank_lines = auto` example. Then actually run `drut format` against each and
compare.

**Expected**: all three predictions match the real output, per spec.md User
Story 3's Independent Test and Acceptance Scenarios.
