//! Per-rule grammar notes: for every functional requirement this crate
//! implements, a short, plain-English restatement of the rule and which
//! Cube Voyager version baseline it was validated against (FR-024).
//!
//! Every note here is written in this project's own words — constitution
//! Principle II forbids copying phrasing from Bentley/Citilabs vendor
//! documentation, even for rules that were originally confirmed against it.
//! This module has no compile-time dependency on the rest of the crate; it
//! exists purely so grammar rules and their rationale live in one inspectable
//! place (see also `specs/001-voyager-script-parser/spec.md`'s Functional
//! Requirements, which is the authoritative source this module restates).

/// One documented grammar rule: which functional requirement it backs, the
/// Voyager baseline it was validated against, and a plain description.
#[derive(Debug, Clone, Copy)]
pub struct GrammarNote {
    pub fr: &'static str,
    pub baseline: &'static str,
    pub note: &'static str,
}

/// Every grammar rule this crate implements, FR-003 through FR-033.
pub const NOTES: &[GrammarNote] = &[
    GrammarNote {
        fr: "FR-003",
        baseline: "Voyager 6.5",
        note: "A control statement opens with a bare control word (not itself \
               part of a keyword=value pair), followed by zero or more \
               space-separated keyword=value pairs.",
    },
    GrammarNote {
        fr: "FR-004",
        baseline: "Voyager 6.5",
        note: "A semicolon starts a line comment that runs to the end of its \
               physical line, no matter where on the line it appears.",
    },
    GrammarNote {
        fr: "FR-005",
        baseline: "Voyager 6.5",
        note: "`/*` and `*/` delimit a block comment that may span several \
               lines. A second `/*` encountered while one is already open \
               starts its own nested comment; the outer comment only finishes \
               once every comment nested inside it has closed. An unclosed \
               comment is reported at whichever `/*` — outer or inner — never \
               found its own `*/`.",
    },
    GrammarNote {
        fr: "FR-006",
        baseline: "Voyager 6.5",
        note: "A statement continues onto the next physical line when the \
               last non-comment, non-blank character on the line is one of \
               `, + - / * ^ & | =`. Any run of completely blank lines between \
               that line and the one that resumes the statement is skipped \
               without breaking the continuation. Separately, a control \
               statement may instead be continued with a `{` placed right \
               after the control word: everything up to the next `}` belongs \
               to the statement (no per-line continuation character needed), \
               and that next `}` always ends it even if another `{` appears \
               first inside — brace bodies do not nest, unlike block \
               comments.",
    },
    GrammarNote {
        fr: "FR-007",
        baseline: "Voyager 6.5",
        note: "An `IF (...)` statement followed, on the same physical line, \
               by exactly one further statement is a complete, self-closing \
               block on its own — that statement is the entire body and no \
               `ENDIF` is expected or consumed for it. A statement instead \
               trailing `ELSEIF`/`ELSE`/`ENDIF` on the same line is its own \
               ordinary statement, not folded into a block.",
    },
    GrammarNote {
        fr: "FR-008",
        baseline: "Voyager 6.5",
        note: "`LOOP`/`ENDLOOP` blocks nest to any depth; `BREAK` is a valid \
               statement anywhere inside one.",
    },
    GrammarNote {
        fr: "FR-009",
        baseline: "Voyager 6.5",
        note: "`RUN PGM=...` opens a block that an explicit `ENDRUN` closes, \
               or that closes implicitly — by a sibling statement at the same \
               nesting depth — at whichever comes first of the next \
               `RUN`/`!RUN` statement or a shell-escape statement. The \
               disabled form, `!RUN`, does not get this implicit treatment: \
               it always needs its own explicit `ENDRUN`.",
    },
    GrammarNote {
        fr: "FR-010",
        baseline: "Voyager 6.5",
        note: "`@name@` is tokenized as one substitution-reference token, \
               whether it appears bare in a keyword's value or inside a \
               quoted string literal, with no evaluation or substitution \
               performed by this crate.",
    },
    GrammarNote {
        fr: "FR-011",
        baseline: "Voyager 6.5",
        note: "Control words and keywords are matched without regard to \
               case; the source text's original casing is preserved in the \
               returned tokens/statements.",
    },
    GrammarNote {
        fr: "FR-012",
        baseline: "Voyager 6.5",
        note: "An `IF` with no matching `ENDIF` before end-of-input, or an \
               `ENDIF`/`ELSEIF`/`ELSE` with no open `IF` to belong to \
               (including one that follows an already self-closed \
               short-`IF`), is reported as `UnmatchedIf`.",
    },
    GrammarNote {
        fr: "FR-013",
        baseline: "Voyager 6.5",
        note: "A `LOOP` with no matching `ENDLOOP` before end-of-input, or a \
               dangling `ENDLOOP`, is reported as `UnmatchedLoop`.",
    },
    GrammarNote {
        fr: "FR-014",
        baseline: "Voyager 6.5",
        note: "A block comment with no matching `*/` before end-of-input is \
               reported as `UnclosedBlockComment`, anchored at whichever `/*` \
               never found its match.",
    },
    GrammarNote {
        fr: "FR-015",
        baseline: "Voyager 6.5",
        note: "A continuation character with no valid following content — no \
               further line at all, or no line ever produces content before \
               end-of-input — is reported as `InvalidContinuation`. Blank \
               lines in between do not themselves count as the failure.",
    },
    GrammarNote {
        fr: "FR-016",
        baseline: "Voyager 6.5",
        note: "A non-disabled `RUN` with neither an explicit `ENDRUN` nor an \
               implicit closer, a disabled `!RUN` missing its required \
               explicit `ENDRUN`, or a dangling `ENDRUN`, is reported as \
               `UnmatchedRun`.",
    },
    GrammarNote {
        fr: "FR-017",
        baseline: "Voyager 6.5",
        note: "Every diagnostic always carries a kind, a location, and an \
               original-wording message — never a free-floating string with \
               no location.",
    },
    GrammarNote {
        fr: "FR-018",
        baseline: "Voyager 6.5",
        note: "Parsing continues past a recorded defect wherever structurally \
               feasible, rather than aborting at the first one found.",
    },
    GrammarNote {
        fr: "FR-019",
        baseline: "Voyager 6.5",
        note: "This crate performs no per-program-box keyword validation and \
               no semantic/reference checking — only structural recognition.",
    },
    GrammarNote {
        fr: "FR-020",
        baseline: "Voyager 6.5",
        note: "A file's top level may contain zero or more blocks/statements; \
               it is not required to be wrapped in a single enclosing block.",
    },
    GrammarNote {
        fr: "FR-021",
        baseline: "Voyager 6.5",
        note: "A `:identifier` line is a label statement, valid at the top \
               level or nested inside a block.",
    },
    GrammarNote {
        fr: "FR-022",
        baseline: "Voyager 6.5",
        note: "A line starting with `*` (optionally doubled, `**`) is a \
               shell-escape statement; the command text that follows — \
               parenthesized or not — is stored as-is and never parsed as \
               Voyager grammar.",
    },
    GrammarNote {
        fr: "FR-023",
        baseline: "Voyager 6.5",
        note: "An identifier immediately followed by `=` and a value, with no \
               preceding control word and no further keyword=value pairs \
               afterward, is a plain assignment statement rather than a \
               control statement.",
    },
    GrammarNote {
        fr: "FR-024",
        baseline: "Voyager 6.5",
        note: "Every grammar rule records which Voyager version baseline it \
               was validated against, in this project's own wording.",
    },
    GrammarNote {
        fr: "FR-025",
        baseline: "Voyager 6.5",
        note: "The fixture corpus must correctly flag a deliberately-broken \
               example of every diagnostic category this crate defines.",
    },
    GrammarNote {
        fr: "FR-026",
        baseline: "Voyager 6.5",
        note: "A `BREAK` statement with no enclosing block of any kind is \
               reported as `MisplacedBreak`. Nested inside any block kind — \
               including `Process`/`PHASE`, where several Voyager programs \
               accept it — it is left alone, since this crate has no \
               per-program knowledge to judge it further.",
    },
    GrammarNote {
        fr: "FR-027",
        baseline: "Voyager 6.5",
        note: "This crate has no third-party runtime dependencies.",
    },
    GrammarNote {
        fr: "FR-028",
        baseline: "Voyager 6.5",
        note: "`PROCESS ... ENDPROCESS` is the underlying block type, with \
               `PHASE=value` accepted as an opener shortcut and `ENDPHASE` as \
               an interchangeable closer spelling. An explicit closer is \
               optional: the block also closes implicitly — by a sibling \
               statement at the same nesting depth, mirroring `RUN` — at the \
               next `PROCESS`/`PHASE=` statement.",
    },
    GrammarNote {
        fr: "FR-029",
        baseline: "Voyager 6.5",
        note: "`JLOOP ... ENDJLOOP` is a loop-block kind distinct from \
               `LOOP`/`ENDLOOP`. Confirmed against the real fixture corpus: \
               it may nest inside `If`, `Loop`, `Run`, or `Process` blocks, \
               but not inside another `JLoop` — this differs from what the \
               vendor reference documentation states, which the fixture \
               evidence is treated as overriding (see spec.md Assumptions).",
    },
    GrammarNote {
        fr: "FR-030",
        baseline: "Voyager 6.5",
        note: "`DistributeMULTISTEP ... EndDistributeMULTISTEP` is a \
               parallel-processing sub-block, observed always sequential and \
               never nested.",
    },
    GrammarNote {
        fr: "FR-033",
        baseline: "Voyager 6.5",
        note: "`LINKLOOP ... ENDLINKLOOP` is a bare, argument-less loop-block \
               shorthand for looping over a network's link records. It may \
               nest inside `If`, `Loop`, `Run`, or `Process` blocks, but not \
               inside another `LinkLoop`.",
    },
    GrammarNote {
        fr: "FR-034",
        baseline: "n/a — general byte-decoding behavior, not Voyager-version-specific",
        note: "The byte-oriented entry points decode raw input as UTF-8 first; \
               any individual byte that isn't valid UTF-8 falls back to its \
               Windows-1252 interpretation instead of rejecting the whole \
               file, since real production scripts have been observed with a \
               stray non-UTF-8 byte. A byte with no defined Windows-1252 \
               interpretation either is replaced with the Unicode replacement \
               character and reported as `InvalidEncoding`; a byte that \
               resolves successfully under either encoding is not reported at \
               all — recovering from an encoding quirk is not a defect.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// T050/T051: every FR this crate implements (FR-003 through FR-034 —
    /// FR-001/FR-002 are architectural, not grammar, and documented
    /// elsewhere; FR-031/FR-032 were never adopted, see spec.md FR-033's
    /// note) must have exactly one grammar note.
    #[test]
    fn covers_every_implemented_grammar_fr_exactly_once() {
        let expected: Vec<u32> = (3..=30).chain(33..=34).collect();
        let mut present: Vec<u32> = NOTES
            .iter()
            .map(|n| {
                n.fr.trim_start_matches("FR-")
                    .parse::<u32>()
                    .expect("FR id should parse")
            })
            .collect();
        present.sort_unstable();
        assert_eq!(
            present, expected,
            "grammar_notes.rs is missing or duplicating an FR entry"
        );
    }

    #[test]
    fn every_note_has_a_baseline_and_non_empty_text() {
        for n in NOTES {
            assert!(!n.baseline.is_empty(), "{} has no baseline", n.fr);
            assert!(!n.note.is_empty(), "{} has no note text", n.fr);
        }
    }
}
