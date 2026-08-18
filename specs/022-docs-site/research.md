# Phase 0 Research: Published Documentation Site

All open questions from the feature description were resolved with the owner
directly (via `AskUserQuestion`) before `/speckit-specify` ran; this document
formalizes those decisions plus the smaller implementation-level questions that
came up while turning them into a plan.

## 1. Doc-site generator: mdBook

**Decision**: mdBook.

**Rationale**: Decided directly with the owner. Markdown-native (every existing
doc in this repo — `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `ROADMAP.md`,
every `specs/*/spec.md` — is already Markdown, so no content-format translation is
needed going in). It's the Rust ecosystem's own standard tool (`rustc`, `Cargo`,
`rust-analyzer`, and most `rustup`-adjacent projects all publish docs this way),
which fits a project whose core crate has a zero-runtime-dependency ethos
(CLAUDE.md, constitution Principle I context) — mdBook itself never becomes a
`Cargo.toml` dependency, it's a build-time CLI only. Built-in full-text search
satisfies FR-012 with no plugin. Deploys to GitHub Pages with a small, well-trodden
GitHub Actions recipe.

**Alternatives considered**:
- **Docusaurus** — more visual polish and built-in versioning UI, but pulls in a
  full npm/React toolchain this project doesn't otherwise need (only
  `editors/vscode` currently touches npm, and only for the extension itself, not
  documentation). Rejected: adds a second JS toolchain surface for no capability
  this feature's scope actually needs (spec.md explicitly puts versioned docs and
  custom visual branding out of scope).
- **Plain GitHub Pages (Jekyll default)** — zero build tooling, but weak
  navigation/search without hand-rolled Jekyll config/layouts, and introduces a
  Ruby-ecosystem convention this project has no other use for. Rejected: the
  "excellent, clear" bar the feature was requested against (searchable, navigable,
  a real information architecture) needs more structure than Jekyll's bare default
  gives for free.

## 2. GitHub Pages deployment mechanism: "Deploy from a branch" (`main`/`docs`), committed output, no deploy Actions — **superseded, see below**

**Original decision (superseded 2026-08-17)**: `actions/configure-pages` +
`actions/upload-pages-artifact` + `actions/deploy-pages`, Pages source set to
"GitHub Actions." Rejected after direct owner correction: (1) the owner wants
GitHub Actions usage minimized, not just secret-free; (2) classic "Deploy from a
branch" mode — the Actions-free alternative — only serves a source branch's
repo-root `/` or `/docs` folder, which the original decision hadn't accounted for.

**Revised decision**: Pages source is set (one-time, manually, in repository
Settings → Pages) to "Deploy from a branch" → `main` → `/docs`. mdBook's
`book.toml` redirects its build output to repo-root `docs/` (`[build] build-dir =
"../docs"`, since `docs-site/` is the book source) via `mdbook build`, run locally
by whoever is publishing a content change, and committed as an ordinary part of
that change — GitHub Pages serves whatever is at `main`'s `docs/` directly, with
**no GitHub Actions workflow involved in deployment at all**. A `docs/.nojekyll`
file (empty, committed) disables GitHub's default Jekyll processing of the
`/docs` folder, which would otherwise mangle or ignore some of mdBook's generated
output (Jekyll's own convention — needed any time a non-Jekyll static site is
served this way, not specific to mdBook).

**Rationale**: Directly satisfies the owner's instruction to avoid GitHub Actions
"as much as possible" — deployment becomes a plain `git commit`/`git push`, the
same mechanism every other change in this repository already uses, with zero new
workflow permissions, zero OIDC/token configuration, and no separate deploy
artifact/environment to reason about. The trade-off (a maintainer must remember to
rebuild before committing) is closed by §6 below's automated freshness check,
recovering the same "never silently stale" guarantee FR-013 wants, without an
Actions-based deploy step.

**Alternatives considered**:
- **Actions-native `deploy-pages`** (the original decision) — rejected per above;
  its zero-secrets property doesn't offset the owner's explicit preference to keep
  this feature's Actions footprint minimal, and it can't target `/docs` from a
  branch-deploy Pages source anyway (that source mode's whole point is *not*
  invoking Actions for deployment).
- **`peaceiris/actions-gh-pages` to a `gh-pages` branch** — still an Actions-driven
  deploy step, same objection as above; also still not the `/docs`-on-`main`
  convention the owner specifically flagged. Rejected.

## 2a. CI scope: one build-check job only, no deploy job — resolved directly with the owner

**Decision**: `.github/workflows/docs.yml` contains exactly one job — `mdbook
build` (from `docs-site/`, output redirected to `../docs` per §2) plus
`scripts/check-docs-coverage.ps1` plus §6's freshness check — triggered on
push/PR to `main` and any `[0-9][0-9][0-9]-*` feature branch, mirroring `ci.yml`'s
existing trigger shape. No deploy job, no `pages: write`/`id-token: write`
permissions, no secrets — asked and confirmed directly with the owner (over a
fully-manual, zero-automation alternative) specifically to keep FR-013's
"failure must be visible" guarantee intact without adding deploy-related Actions
surface.

## 3. Doc/reality drift prevention: a coverage check, not just review discipline

**Decision**: A small PowerShell script (`scripts/check-docs-coverage.ps1`,
matching this project's existing PowerShell-first tooling convention — see
CLAUDE.md's Commands section) that extracts every field name from
`drut-config::FormatConfig`'s struct definition and asserts each one appears as a
heading/anchor in `docs-site/src/configuration-reference.md`, run as its own step
in `docs.yml` on every push/PR (not just `main`).

**Rationale**: The gap that prompted this whole feature — `CONTRIBUTING.md`'s
"Configuration" section documenting only 2 of the 10 real `[format]` fields — arose
from exactly this kind of silent drift: a section written once, then not
mechanically re-checked as more fields shipped (`017`, `018`, `019`). FR-011 makes
staying current a process requirement, but a process requirement with no automated
check is exactly the failure mode that already happened once. A field-name
existence check is cheap (a regex scan, no new crate, no new language toolchain)
and catches the single highest-value failure mode (a shipped field with zero
documentation) without needing to verify prose *quality*, which stays a human
review concern.

**Alternatives considered**:
- **Review discipline alone** (a PR-template checklist item) — already how this
  gap happened once; rejected as insufficient on its own, though still worth
  keeping informally.
- **A `cargo test`-based check inside `drut-config`** — would require either a
  `syn`-based parse of the struct (real complexity for a text-matching problem) or
  a hand-maintained field-name constant list duplicating the struct definition
  (exactly the drift risk being solved for, one level removed). Rejected: a direct
  source-text regex scan in a short script is simpler and just as effective for
  this specific "does the name appear at all" check.

## 4. Site URL: default GitHub Pages URL, no custom domain

**Decision**: `https://<owner>.github.io/<repo>/` (mdBook's `book.toml`
`[output.html] site-url` set to the repository's default Pages path), no `CNAME`/
custom domain.

**Rationale**: Matches spec.md's Assumptions (GitHub Pages accepted as-is, no
separate hosting account expected) and this project's current pre-1.0,
not-yet-widely-distributed state — a custom domain can be added later without
restructuring the site if ever wanted, and isn't blocking any of this feature's
success criteria.

## 5. `docs-site/` (source) vs. `docs/` (published, committed output) vs. `dev-notes/` (relocated internal log) — revised 2026-08-17

**Original decision**: mdBook book source lives in a new `docs-site/` directory,
kept distinct from the existing `docs/` (which held only
`known-environment-quirks.md`, a contributor troubleshooting log).

**Revised, still distinct but for a different reason**: §2's revision means
`docs/` is no longer available as "some other directory" — it's now *required* to
be the Pages-served output folder (classic "Deploy from a branch" only offers
repo-root `/` or `/docs`), so the separation from `docs-site/` (the mdBook
*source*) is load-bearing, not just tidiness: `docs-site/` is never served
directly, `docs/` is never hand-edited directly (it's regenerated output).

**`known-environment-quirks.md` relocated to `dev-notes/`**: since `docs/` is now
reserved for committed, published build output, the pre-existing
`docs/known-environment-quirks.md` — confirmed by direct read to be a
contributor/dev-machine troubleshooting log, explicitly *not* "a place for project
decisions or rationale" per its own opening line, and out of scope for the
user-facing site — can no longer live there without becoming technically
fetchable as part of the deployed site (GitHub Pages serves every file under the
configured folder, not just mdBook's own output). Moved to a new top-level
`dev-notes/` directory instead: purely internal, unambiguously unrelated to
anything Pages-related, and available to hold other non-crate engineering notes
later the same way `docs/`'s original framing intended. Historical references to
the old `docs/known-environment-quirks.md` path in already-shipped specs
(`002-cli-check-format/research.md`, `003-lsp-vscode-extension/tasks.md`) are left
as-is — those are dated historical records of decisions made at the time, not
living documentation, matching this project's standing practice of amending
forward (ROADMAP.md-style "Update" notes) rather than rewriting history.

## 6. Freshness check: CI verifies the committed `docs/` isn't stale, without deploying anything

**Decision**: the single `docs.yml` job (research.md §2a) also runs `mdbook
build` into a clean temporary location and diffs it against the committed
`docs/` (`git status --porcelain -- docs` after overwriting `docs/` with a fresh
build, inside the checkout — fails the job if non-empty).

**Rationale**: This is what recovers FR-013's "never silently stale" guarantee
without any deploy Actions: a maintainer who edits `docs-site/src/*.md` but
forgets to run `mdbook build`/commit the regenerated `docs/` gets a failing,
visible CI check on their PR, exactly as if their code had a failing test —
instead of the mismatch only surfacing later by someone comparing the live site
against the source by hand. Cheap to implement (no new tool, just the mdBook CLI
already needed for the build-check itself, plus `git status`) and requires no
Pages-specific permission at all.

**Alternatives considered**:
- **No freshness check, trust the author** — rejected: this is exactly the
  unguarded failure mode `CONTRIBUTING.md`'s two-of-ten-field config section
  already demonstrated once for hand-maintained content; a mechanical check is
  cheap enough here that there's no real reason to accept that risk again.
