# Phase 0 Research: Top-Level Indentation Normalization

Everything in this file was verified against a real, working prototype
(built, run, then reverted before committing — the actual implementation
happens cleanly in `/speckit-implement`, not here), not derived from
reading the code alone.

## §1. The exact code change, and the resulting FR-004 resolution for 007

**Decision**: `plan_indentation` gains one line — before calling
`plan_block` for a top-level `Block`, force-plan **every** top-level
node's own line (statement or block alike) to column 0:

```rust
fn plan_indentation(nodes: &[Node], lines: &[Vec<char>], diagnosed_openers: &BTreeSet<Position>, plan: &mut IndentPlan) {
    for node in nodes {
        plan.insert(node.span().start.line, 0);
        if let Node::Block(block) = node {
            plan_block(block, lines, diagnosed_openers, plan);
        }
    }
}
```

No change needed anywhere else. `plan_block`'s existing
`let base = computed_indent(plan, lines, opener_line);` already prefers a
*planned* value over the line's original on-disk indentation
(`computed_indent`'s own doc comment: *"its planned target if one exists,
otherwise its original (untouched) indent"*) — so pre-seeding `plan` with
0 for the top-level line, before `plan_block` reads it, makes every
existing per-nesting-level/closer-alignment/branch-alignment computation
correctly cascade from the new, always-0 anchor with zero changes to that
logic.

**A real, previously-existing gap this closes as a side effect, not a new
concern introduced by this feature**: `plan_indentation` only ever
iterated `Node::Block` entries before — a bare top-level `Node::Statement`
had *no* code path touching it at all, under either the old or (without
this fix) a naively-reversed policy. Verified: a fixture containing only
`    X = 1\n` was untouched by the pre-`008` formatter; the new code
correctly normalizes it to `X = 1\n`. FR-002's "bare statements and every
block-kind opener, with no exceptions" requires exactly this, not just
flipping the block-opener case.

**FR-004's resolution — verified with five prototype scenarios, not
asserted from theory**:

| Scenario | Result |
|---|---|
| Bare top-level statement, non-zero indent | Corrected to 0 (new: previously untouched entirely) |
| Residue pass 1 (`PROCESS` unclosed, `RUN` swallowed, both already at column 0) | No-op — nothing needed changing |
| Residue pass 2 (`ENDPROCESS` added, `RUN` already correct) | No-op |
| **Residue pass 2, `RUN` left at *stale* non-zero indentation with stale-deeper children** | **Fully corrected in this one pass** — `RUN`/`FILEI NETI`/`ENDRUN` all land at 0/4/0 correctly, `changed: true` |
| Still-broken `PROCESS` (never gets `ENDPROCESS`), opener itself indented | Opener forced to 0; its children (both the legitimate `FILEI` body content and the swallowed `RUN`) remain **untouched** — `007`'s skip still applies |

**`007`'s skip is kept, unchanged in code — but its own rationale needs
rewriting, not its behavior.** The prototype's last row proves why: `007`
was never actually protecting a diagnosed block's *opener* line — the
pre-existing (now-reversed) "top-level lines are never touched" policy
already did that, as a completely separate mechanism. `007`'s skip only
ever protected a diagnosed block's *children*, whose relationship to that
block is genuinely uncertain precisely because the block itself is
unmatched — a concern this feature doesn't touch, resolve, or make
redundant. Post-`008`, the two mechanisms divide the same block cleanly:
the opener is unconditionally corrected by the new top-level rule; the
children stay protected by `007`'s unchanged skip. `plan_block`'s and
`diagnosed_block_openers`'s own doc comments need updating to state this
narrowed rationale explicitly (no longer "prevents residue," since `008`'s
unconditional correction now does that far more directly and robustly for
the opener specifically — see the stale-indentation row above, which
`007` alone never would have fixed, since `007` only ever prevented a
*future* write, never corrected a value already sitting in the file).

**Alternatives considered**: Removing `007`'s skip entirely, reasoning
that "top-level is always fixed now, so residue can't accumulate" — 
rejected. That reasoning only applies to the *opener* line; a diagnosed
block's *children* remain exactly as structurally uncertain as they were
under `007`'s own original justification, unrelated to what column the
opener itself sits at. Removing the skip would resume speculatively
reindenting content whose true structural home is unknown — reintroducing
`007`'s original problem for children specifically, even though the
opener-residue problem is now independently solved.

## §2. Adapter impact — confirmed none, not assumed

**Method**: `grep -rn "plan_indentation\|top.level\|top-level" crates/drut-cli crates/drut-lsp crates/drut-mcp`
— zero hits referencing this policy or function in any adapter. Every
adapter's formatting call site (`drut-cli/src/report`, `drut-lsp/src/
formatting.rs`, `drut-lsp/src/range_formatting.rs`) calls the public
`voyager_core::format`/`format_bytes` functions, whose signatures are
unchanged by this feature (same as `006`/`007`'s own confirmed pattern —
the decision logic lives entirely inside `format()`'s own call graph).
**Zero adapter code changes needed.**

## §3. Exact golden-fixture regeneration scope (FR-006) — measured, not estimated

**Method**: Built the §1 change as a real (later-reverted) prototype and
ran `cargo test -p voyager-core --test format_corpus` against the
*existing, committed* golden fixtures without regenerating them —
producing an exact list of every fixture whose output would now differ.

**Result**: `hand_written_fixtures_match_golden_output` — **0 files
affected** (every hand-written `tests/fixtures/valid/*.s`/`*.block`
fixture already sits at column 0 for its own top-level content).
`real_corpus_fixtures_match_golden_output` — **7 of the 9** `real_corpus/`
files affected:

```text
AssignHwy/02_Assign_AM_MD_PM_EV.s
AssignHwy/09_TAZ_Based_Metrics.s
Distribute/3_SumToDistricts_GRAVITY.s
Distribute/4pd_mainbody_distribution.block
InputProcessing/1_InputSetup.s
InputProcessing/2_UrbanizationTermTime.s
ModeChoice/06_HBW_logsums.s
```

The remaining 2 (`AssignHwy` and `InputProcessing` each have one more
file not listed above) are already column-0 at top level and stay
byte-identical. This is the complete, exact regeneration surface FR-006
requires — every file above needs `UPDATE_GOLDEN=1` regeneration
followed by an individual, human-reviewed diff (task-level detail in
tasks.md) confirming *only* top-level indentation shifted, nothing else.

## §4. `contracts/formatting-api.md` (002-cli-check-format) needs a matching amendment

**Finding**: that contract's own prose ("top-level baseline ... left
untouched") is now stale, same as FR-012's own text. Amended alongside
spec.md's FR-012/Assumptions (FR-007) — see
`contracts/top-level-indentation.md` for the exact replacement wording,
kept in sync with spec.md rather than diverging.
