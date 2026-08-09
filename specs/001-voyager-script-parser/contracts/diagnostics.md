# Contract: Diagnostic Taxonomy

Downstream tools (linter output, LSP diagnostics, CLI exit-code/report logic) match on
`DiagnosticKind`, not on message text. This table is the contract for what kinds exist
and what each one means; message wording itself is free to evolve (it's prose, not a
matched value) as long as it stays original wording (constitution Principle II,
FR-024) and stays anchored at the span described here.

| `DiagnosticKind` | Anchored at | Triggering condition | Source FR |
|---|---|---|---|
| `UnmatchedIf` | The `IF` (or dangling `ENDIF`/`ELSEIF`/`ELSE`) statement's location | An `IF` has no matching `ENDIF` before end-of-input, **or** an `ENDIF`/`ELSEIF`/`ELSE` appears with no corresponding open `IF` — including a stray `ENDIF` that follows a short-`IF` (FR-007), which already self-closed and has nothing left open to match | FR-012 |
| `UnmatchedLoop` | The `LOOP` (or dangling `ENDLOOP`) statement's location | A `LOOP` has no matching `ENDLOOP` before end-of-input, **or** an `ENDLOOP` appears with no corresponding open `LOOP` | FR-013 |
| `UnclosedBlockComment` | The `/*` that opened the comment | A block comment has no matching `*/` before end-of-input — for nested block comments (FR-005), this is whichever `/*`, inner or outer, never found its own match | FR-014 |
| `InvalidContinuation` | The continuation character (or the line that follows it) | A statement's last non-comment character is a continuation character but there is no following line, or the following line is not a valid continuation — blank lines in between don't count as breaking it (FR-006) | FR-015 |
| `UnmatchedRun` | The `RUN`/`!RUN` (or dangling `ENDRUN`) statement's location | A non-`disabled` `RUN PGM=...` has no matching `ENDRUN` **and** no implicit closer (the next `RUN`/`!RUN`, or a shell-escape statement) before end-of-input; a `disabled` (`!RUN`) block gets no implicit-closer exception and is diagnosed on a missing `ENDRUN` alone, the same as `IF`/`LOOP`; **or** an `ENDRUN` appears with no corresponding open `RUN`/`!RUN` | FR-016 |
| `MisplacedBreak` | The `BREAK` statement's location | A `BREAK` statement appears with no enclosing block of any kind — not nested inside an `IF`, `LOOP`, `RUN`, `PROCESS`/`PHASE`, `JLOOP`, or `LINKLOOP` | FR-026 |

**Note on `MisplacedBreak`**: The feature description's own list of "at minimum"
diagnostic categories named five. `MisplacedBreak` started as a spec Assumptions-
section note and has since been promoted to a full requirement, **FR-026**, with the
same binding force as FR-012–FR-016. Its triggering condition was later narrowed from
"no enclosing `LOOP`" to "no enclosing block of any kind": vendor reference
documentation showed `BREAK` is legitimate, program-dependent syntax inside a
`PROCESS`/`PHASE` stack in several Voyager programs, and this parser has no
per-program knowledge (FR-019) to tell that case apart from a genuine misuse — so it
only flags the case with no structural cover at all. Consumers MUST NOT assume the
diagnostic kind set is closed at six — new structural-defect kinds may be added
within this phase's scope (still "structural," not semantic) without that being a
breaking change to this contract, provided existing kinds keep their meaning.

**Note on block kinds without a diagnostic category**: `Process`/`PHASE` (FR-028),
`JLoop` (FR-029), `LinkLoop` (FR-033), and `DistributeMultistep` (FR-030) are matched
structurally when well-formed, the same as the six kinds above, but none of them has
its own `DiagnosticKind` — an unmatched opener of one of these kinds is accepted
silently (its span just runs to end-of-input) rather than producing a diagnostic.
FR-025 only requires fixture coverage for the six kinds in the table; extending
diagnostic coverage to these four is explicitly left to a later phase.

## Required fields (every diagnostic, every kind)

Per FR-017, every `Diagnostic` value carries all three, always:
1. `kind: DiagnosticKind` — one of the table above.
2. `span: Span` — see data-model.md; a single location, not a free-floating message.
3. `message: String` — human-readable, original wording, describing the specific
   defect (e.g. naming which opener has no matching closer).

## Non-goals for this contract

- No severity levels (error vs. warning) are defined at this layer. All six-plus
  kinds here are structural syntax defects, not heuristic lint findings — the
  constitution's "ship new rules as warnings until validated" policy (Principle IV)
  governs future *semantic/lint* rule categories built in later phases on top of this
  crate, not this phase's structural diagnostics.
- No fix-it/suggested-edit data. That's a formatter/LSP-code-action concern for a
  later phase.
