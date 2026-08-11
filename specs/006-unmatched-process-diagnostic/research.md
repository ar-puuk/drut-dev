# Phase 0 Research: UnmatchedProcess Diagnostic

## §1. Full adapter-impact inventory (resolves FR-007's correction)

**Decision**: Treat this as a four-crate-touching feature, not a
voyager-core-only one. Every file below was opened and read directly — not
inferred from a pattern — specifically because the feature description
asked this to be confirmed, not assumed.

**Method**: `grep -rn "DiagnosticKind::" crates/` across the whole
workspace, then each hit's containing function was read in full to
determine whether it's exhaustive (no `_ =>` wildcard).

**Findings, file by file**:

| File | Construct | Exhaustive? | Change needed |
|---|---|---|---|
| `crates/voyager-core/src/diagnostic.rs` | `DiagnosticKind` enum itself | — | Add the `UnmatchedProcess` variant (the actual feature) |
| `crates/voyager-core/src/block.rs` | `parse_process` | — | Add the firing logic (the actual feature) |
| `crates/voyager-core/tests/fixture_corpus.rs` | `parse_diagnostic_kind` (string → kind, for the `; EXPECT:` fixture marker) | **Yes** | Add `"UnmatchedProcess" => Some(DiagnosticKind::UnmatchedProcess)` |
| `crates/voyager-core/tests/fixture_corpus.rs` | `every_diagnostic_category_has_at_least_one_broken_fixture`'s hardcoded 7-element array | **Yes** | Add `DiagnosticKind::UnmatchedProcess` to the array — this test will otherwise fail once the new fixture (§6) exists and declares it, or pass vacuously if the array isn't updated (worse: it would silently stop proving FR-025-equivalent coverage for the new kind) |
| `crates/drut-cli/src/report/sarif.rs` | `ALL_KINDS: [DiagnosticKind; 7]` | **Yes** (array length is checked) | Add the variant, bump to `8` |
| `crates/drut-cli/src/report/sarif.rs` | `rule_id` | **Yes** | Add `"unmatched-process"` (kebab-case, matching `contracts/sarif-mapping.md`'s existing convention) |
| `crates/drut-cli/src/report/sarif.rs` | `short_description` | **Yes** | Add one original-wording sentence (§4) |
| `crates/drut-cli/src/report/text.rs` | `format!("{:?}", diag.kind, ...)` | N/A — Debug-formats the kind directly | **No change needed** — the one adapter surface that was already correctly kind-agnostic |
| `crates/drut-lsp/src/diagnostics.rs` | `kind_name` | **Yes** | Add `UnmatchedProcess => "UnmatchedProcess"`; module doc's "six of `voyager-core`'s seven `DiagnosticKind` values" count needs updating to seven of eight |
| `crates/drut-mcp/src/diagnose.rs` | `category_name` | **Yes** | Add `UnmatchedProcess => "UnmatchedProcess"`; `DiagnosticDto.category`'s doc comment count needs updating |
| `crates/drut-lsp/src/semantic_tokens.rs` | `d.kind == DiagnosticKind::UnmatchedIf` / `MisplacedBreak` (equality checks, not exhaustive matches) | No | **No change needed** — these check for one specific existing kind each, unrelated to `UnmatchedProcess` |
| `editors/vscode/src/*` | — | — | **No change needed** — confirmed via repo-wide grep, zero hand-listed diagnostic kinds; renders whatever `drut-lsp` sends |

**Why this matters beyond "these files need edits"**: every one of the
exhaustive matches above will fail to *compile* the moment
`DiagnosticKind::UnmatchedProcess` exists, in the crate that owns each
match. This is actually a safety net, not a risk — Rust's own exhaustiveness
checking makes it structurally impossible to add the variant and forget an
adapter, unlike a silently-incomplete `if`/`match` with a wildcard arm
would allow. `crates/drut-lsp/src/semantic_tokens.rs`'s two equality checks
and `text.rs`'s `Debug` format are exactly the kind of *non*-exhaustive
usage that would compile fine either way — confirmed both exist and don't
need touching, rather than assumed safe.

## §2. `PROCESS` has no disabled/skip variant (resolves spec.md's Edge Cases open question)

**Decision**: No special-casing needed — `PROCESS` has no `!PROCESS`
equivalent to `RUN`'s `!RUN`.

**Method**: `grep -n "BangProcess\|!PROCESS\|disabled" crates/voyager-core/src/block.rs`.

**Finding**: every hit for `disabled`/`BangRun` is `Run`-specific
(`BlockKind::Run { pgm, disabled }`, `Role::BangRun`, the `disabled` field
on `parse_run`'s signature). `parse_process`/`BlockKind::Process` has no
equivalent field or role at all. `parse_run`'s own `disabled` branch (which
skips the implicit-closer exception entirely for `!RUN`) has no analog to
port — the new `parse_process` logic is simpler than `parse_run`'s, not a
straight copy of every branch, just the two relevant ones (explicit close,
implicit close) plus the new diagnostic fallback.

## §3. Exact firing-condition design (mirrors `parse_run`, not reinvented)

**Decision**: `parse_process` gains exactly the shape `parse_run` already
has, minus the `disabled` handling (§2) it has no analog for:

```rust
fn parse_process(
    statements: &[Statement],
    i: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Block, usize) {
    let opener_span = statements[i].span;
    let opener_pairs = opener_pair_spans(&statements[i]);
    let name = pair_value_text(&statements[i], "PHASE");
    let (children, mut idx) = parse_sequence(
        statements, i + 1, BodyContext::InsideProcessBody, true, diagnostics,
    );

    if idx < statements.len() && role_of(&statements[idx]) == Role::EndProcess {
        // unchanged: explicit close (closer: Some)
    }

    if idx < statements.len() && role_of(&statements[idx]) == Role::Process {
        // unchanged in effect, now its own explicit branch rather than
        // folded into the single "either way, no diagnostic" fallback:
        // implicit close (closer: None, no diagnostic, FR-004)
    }

    // NEW: everything else -- true EOF, or the enclosing scope's own
    // closer forced an early stop (role_of(&statements[idx]) is neither
    // EndProcess nor Process) -- genuinely unmatched.
    diagnostics.push(Diagnostic::new(
        DiagnosticKind::UnmatchedProcess,
        opener_span,
        "this PROCESS/PHASE= has no matching ENDPROCESS/ENDPHASE and no \
         following PROCESS/PHASE= statement before the end of the file",
    ));
    // closer: None, same span-fallback as before
}
```

**Why this correctly handles the nested-early-stop case (spec.md
Acceptance Scenario 4) with no special-casing**: `parse_sequence`'s generic
closer-stopping set (every universal closer role — `ENDIF`/`ENDLOOP`/
`ENDRUN`/etc.) already stops body-parsing *before* `BodyContext::
InsideProcessBody`'s own `Role::Process`-specific stop check runs, for any
context, including inside a `Process` body — this is existing,
unmodified `parse_sequence` behavior `parse_run`'s own `UnmatchedRun`
already relies on for the identical case. When a `PROCESS` is nested
inside an `IF` whose `ENDIF` arrives first, `idx` lands on that `ENDIF`
statement — `role_of(&statements[idx])` is neither `EndProcess` nor
`Process`, so the new code falls through to the diagnostic exactly as
intended, with zero new logic required to detect this sub-case
specifically.

**Alternatives considered**: A bespoke condition written from scratch for
`Process` was rejected — `parse_run`'s shape is already proven correct
(including this exact nested-early-stop sub-case) and re-deriving the same
logic independently would risk a subtle divergence for no benefit
(Principle I's spirit: one proven pattern, not two independently-maintained
near-duplicates within the same crate).

## §4. Message and SARIF wording (constitution Principle II)

**Decision**: `"this PROCESS/PHASE= has no matching ENDPROCESS/ENDPHASE and
no following PROCESS/PHASE= statement before the end of the file"` for the
`Diagnostic.message`, directly parallel to `UnmatchedRun`'s existing
`"this RUN has no matching ENDRUN and no following RUN/!RUN/shell-escape
statement before the end of the file"` — same sentence shape, substituting
`PROCESS`/`ENDPROCESS`'s own real keyword names and closer list (no
`!RUN`/shell-escape analog, per §2). SARIF `rule_id`:
`"unmatched-process"` (kebab-case, matching every existing `rule_id`'s
convention exactly). SARIF `short_description`:
`"A PROCESS/PHASE= has no matching ENDPROCESS/ENDPHASE and no implicit
closer."`, mirroring `UnmatchedRun`'s own short-description sentence shape
(`"A RUN has no matching ENDRUN and no implicit closer, or an ENDRUN has no
open RUN."`) — dropping the "or an ENDRUN has no open RUN" clause, since
`Process` (unlike `If`/`Loop`/`Run`) doesn't diagnose a dangling
`ENDPROCESS`/`ENDPHASE` with no open opener (spec.md doesn't ask for that,
and `UnmatchedRun`'s own dangling-`ENDRUN` case is a distinct, separately-
motivated rule not part of this feature's scope).

All original wording — no vendor documentation consulted (constitution
Principle II).

## §5. `contracts/diagnostics.md` amendment (the authoritative diagnostics reference)

**Decision**: Amend `specs/001-voyager-script-parser/contracts/
diagnostics.md` in place — add an `UnmatchedProcess` row to its table
(directly under `UnmatchedRun`, matching the table's existing kind-grouping
order), and rewrite the "Note on block kinds without a diagnostic
category" paragraph so it names `Process` as resolved by this feature
while `JLoop`/`LinkLoop`/`DistributeMultistep` remain deferred — not left
describing all four as equally undecided (FR-010).

**Rationale**: This file is the single authoritative diagnostics contract
every adapter's own doc comments already point back to
(`crates/drut-lsp/src/diagnostics.rs`'s module doc literally cites it by
name). Creating a second, competing diagnostics reference specific to this
feature would fragment that single source of truth for no benefit — the
existing file already has the exact right shape (one row per kind, one
shared "kinds without a category" note) for this addition.

## §6. Fixture-corpus test-machinery updates + the new broken fixture (FR-009)

**Decision**: `crates/voyager-core/tests/fixture_corpus.rs` gains two
one-line additions (§1's table) plus a new fixture,
`crates/voyager-core/tests/fixtures/broken/
unmatched_process_with_trailing_content.s`, reproducing the exact
real-world shape that motivated this feature — not the minimal
`process_block_reports_unconditional_counterpart` unit-test shape
(`PROCESS PHASE=INPUT\nFILEI=ni.1\n`, a single trailing line), but a
larger, more realistic fixture with multiple real statements following the
unclosed `PROCESS` (matching the actual paste-triggered scenario that
surfaced this gap during 005's manual verification work), declaring
`; EXPECT: UnmatchedProcess` on its first line per the existing marker
convention.

**Rationale**: FR-009 explicitly requires more than a minimal synthetic
case. The existing `process_block_reports_unconditional_counterpart` test
(`crates/voyager-core/tests/block_resolution.rs`) stays exactly as it
is — it tests `block_at`'s counterpart resolution, a different concern
from this feature's diagnostic — but its fixture shape is deliberately
*not* reused verbatim for the new broken fixture, precisely because
FR-009 asks for the real, larger shape instead.
