//! Keyword completion/spell-check dictionary (data-model.md §1,
//! `contracts/keyword-dictionary-api.md`; FR-012, FR-014). Hand-written,
//! never sourced from vendor documentation (constitution Principle II).
//!
//! **Provenance note**: `ControlWord` entries are drawn from `statement.rs`'s
//! `FIXED_KEYWORDS` — the closed, already corpus-evidenced set of
//! block-structural control words established during
//! `001-voyager-script-parser`'s own census work. `PairKeyword` entries and
//! their `observed_with` scoping were populated 2026-08-10 by walking every
//! `.s`/`.block` file (161/161, zero diagnostics) in the real
//! WF-TDM-Official-Releases corpus with `voyager_core::parse` (recursively
//! over `ParseResult.nodes` — real parser output, not a regex
//! approximation), recording every `keyword=value` pair name observed
//! alongside its enclosing control word — reusing `001`'s own
//! structural-position census methodology. Not committed to this repo (the
//! throwaway survey scripts were run and discarded, matching this project's
//! established corpus-survey precedent, e.g. `002-cli-check-format/
//! research.md`'s indentation-width survey).
//!
//! **Filtering applied** (raw extraction produced 2,689 candidate
//! `(control_word, keyword)` pairs; two filters, chosen from the data's own
//! distribution rather than an arbitrary a priori number, brought that down
//! to 198 distinct keyword names in the first, 2026-08-10 pass):
//! - **`distinct_files >= 3`**: 2,272 of 2,689 raw entries (84.5%) occurred
//!   in exactly one file — overwhelmingly per-program data (e.g. `ARRAY`'s
//!   own array-name position produced 1,025 "keywords" that are really
//!   user-chosen array names, not general syntax). The corpus's own
//!   distribution shows a sharp drop after one file (2,272 → 417 at ≥2 →
//!   212 at ≥3), so "recurs across at least 3 independently-authored files"
//!   was adopted as the "common enough to be general" bar — the same
//!   dominant-signal-decides principle `002-cli-check-format/spec.md`
//!   FR-012 already applied to indentation width.
//! - **Identifier shape** (`^[A-Z_][A-Z0-9_]*(\[[0-9]+\])?$` after
//!   uppercasing): in the first, 2026-08-10 pass, this filter's real job was
//!   masking a **`voyager-core` parsing defect** this same survey surfaced —
//!   `statement.rs`'s `pair_keyword_boundaries` had no quote-awareness, so a
//!   `word = value`-shaped substring *inside a quoted string* (e.g. `PRINT
//!   LIST='\nScenarioDir = r"..."',`, a Python script being written out by a
//!   real `PRINT` statement) was misclassified as a genuine second
//!   keyword=value pair. 479 raw entries (all under `PRINT`, e.g.
//!   `\NSCENARIODIR`) were this artifact, and — because the same boilerplate
//!   template recurs across a family of near-identical setup scripts —
//!   several survived even the `distinct_files >= 3` threshold (e.g.
//!   `\NSCENARIODIR` at 11 files), so the numeric filter alone didn't fully
//!   separate this specific noise from signal; the identifier-shape filter
//!   (a `\`-prefixed token can never be valid Cube Voyager keyword syntax
//!   under any reading of the grammar) caught what the numeric filter
//!   didn't. **This defect is now fixed** (`pair_keyword_boundaries` gained
//!   the same naive, non-escape-aware per-quote-character toggle
//!   `lexer.rs`'s own `;`/`/*`-in-quotes handling already used — see
//!   `specs/001-voyager-script-parser/spec.md`'s FR-003 amendment and
//!   Assumptions entry, dated 2026-08-10, for the full fix/evidence trail),
//!   and the census was re-run against the fix: the raw `PRINT`-under-quote
//!   artifact entries no longer appear as candidates at all (0 removed by
//!   this filter in the re-run, versus 479 raw / several file-threshold
//!   survivors before), and the **only** resulting change to the 198-entry
//!   set was the loss of one single entry, `COST` under `PRINT` — itself
//!   confirmed to be the same bug's artifact from a different real file
//!   (`04_Create_drive_access_links.s`'s `PRINT ... LIST='NT LEG=1-1,
//!   MODE=40, COST=2.40, DIST=1.00, ONEWAY=F, SPEED=25.0'`, an NT network
//!   card string being printed whose own embedded content coincidentally
//!   also spelled `COST=...`; its sibling fields `LEG`/`MODE`/`DIST`/
//!   `ONEWAY`/`SPEED` never crossed the `distinct_files >= 3` threshold
//!   either before or after the fix, so `COST` alone had been the one
//!   survivor). Every other entry — including `PROCESS`/`FILEI` and
//!   `RUN`/`PGM`/`MSG`/`PRNFILE`, both independently spot-checked before
//!   and after — is byte-for-byte identical across both passes. The
//!   identifier-shape filter is kept in the pipeline regardless (a
//!   structural certainty, not an editorial judgment call, unlike the
//!   `distinct_files` threshold, which still needs it) as a defense-in-depth
//!   check against any future defect of the same shape, not because it is
//!   still masking a live one.
//!
//! No control-word-specific exclusions were applied (e.g. `ARRAY` was not
//! hand-excluded despite the above) — the same uniform threshold that
//! reduced `ARRAY`'s 1,025 raw entries to 8 survivors is trusted equally
//! for every control word, consistent with letting real signal decide
//! rather than injecting per-control-word editorial judgment.

/// One dictionary entry (FR-012), part of a static, compile-time-constant
/// dictionary — never derived at runtime from any single document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeywordEntry {
    /// Canonical spelling. Matching against document text is
    /// case-insensitive (mirroring FR-011 in `001-voyager-script-parser`);
    /// this is the casing shown in a completion/spell-check suggestion.
    pub name: &'static str,
    /// Which completion position this entry is valid for.
    pub role: KeywordRole,
    /// For `PairKeyword` entries: the control word(s) this keyword name was
    /// observed paired with during the census (see module docs for the
    /// 2026-08-10 corpus survey and its filtering). Empty for `ControlWord`
    /// entries.
    pub observed_with: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordRole {
    /// Valid as the first word of a new statement.
    ControlWord,
    /// Valid as a `keyword=value` pair's keyword name, scoped by
    /// `observed_with`.
    PairKeyword,
}

/// The caller-supplied description of *where* in a document completion was
/// requested — deliberately narrow, so this module never needs to know
/// about documents, URIs, or LSP types (Principle I: `voyager-core` has no
/// protocol dependency).
#[derive(Debug, Clone, Copy)]
pub struct CompletionContext<'a> {
    /// `Some(word)` when the cursor falls inside a `Statement` whose `kind`
    /// is `Control { word, .. }`. `None` when no enclosing `Control`
    /// statement was found.
    pub enclosing_control_word: Option<&'a str>,
}

/// The general-syntax control words (data-model.md §1) — mirrors
/// `statement.rs`'s `FIXED_KEYWORDS`, the closed, corpus-evidenced
/// block-structural vocabulary. `!RUN` is represented as `RUN` here plus a
/// leading-`!` note in the caller's own rendering, matching how
/// `statement.rs`/`block.rs` already treat `!RUN` as a `Run` variant rather
/// than a wholly separate control word.
const CONTROL_WORDS: &[KeywordEntry] = &[
    entry("IF", KeywordRole::ControlWord),
    entry("ELSEIF", KeywordRole::ControlWord),
    entry("ELSE", KeywordRole::ControlWord),
    entry("ENDIF", KeywordRole::ControlWord),
    entry("LOOP", KeywordRole::ControlWord),
    entry("ENDLOOP", KeywordRole::ControlWord),
    entry("BREAK", KeywordRole::ControlWord),
    entry("RUN", KeywordRole::ControlWord),
    entry("ENDRUN", KeywordRole::ControlWord),
    entry("PROCESS", KeywordRole::ControlWord),
    entry("PHASE", KeywordRole::ControlWord),
    entry("ENDPROCESS", KeywordRole::ControlWord),
    entry("ENDPHASE", KeywordRole::ControlWord),
    entry("JLOOP", KeywordRole::ControlWord),
    entry("ENDJLOOP", KeywordRole::ControlWord),
    entry("LINKLOOP", KeywordRole::ControlWord),
    entry("ENDLINKLOOP", KeywordRole::ControlWord),
    entry("DISTRIBUTEMULTISTEP", KeywordRole::ControlWord),
    entry("ENDDISTRIBUTEMULTISTEP", KeywordRole::ControlWord),
];

/// `PairKeyword` entries (module docs: 2026-08-10 corpus census, re-run
/// against the `pair_keyword_boundaries` fix the same day — 197 distinct
/// keyword names surviving the `distinct_files >= 3` + identifier-shape
/// filters, one fewer than the first pass's 198 after dropping `COST`/
/// `PRINT`, itself the same bug's artifact; see module docs). Sorted
/// alphabetically by keyword name; a keyword observed under multiple
/// control words (e.g. `FILE` under both `PRINT` and `READ`) has all of
/// them in one entry's `observed_with`.
const PAIR_KEYWORDS: &[KeywordEntry] = &[
    pair_entry("ANSWER", &["PROMPT"]),
    pair_entry("APPEND", &["PRINT"]),
    pair_entry("AUTOARRAY", &["FILEI"]),
    pair_entry("CAPARRAY", &["ARRAY"]),
    pair_entry("CHECKRETURNCODE", &["WAIT4FILES"]),
    pair_entry("CONSOLIDATE", &["PATHLOAD"]),
    pair_entry("CSV", &["PRINT"]),
    pair_entry("DBI[1]", &["FILEI"]),
    pair_entry("DBI[2]", &["FILEI"]),
    pair_entry("DBI[3]", &["FILEI"]),
    pair_entry("DBI[4]", &["FILEI"]),
    pair_entry("DEC", &["PATHLOAD"]),
    pair_entry("DELIMITER", &["FILEI"]),
    pair_entry("EXCLUDE", &["FILEO"]),
    pair_entry("EXCLUDEGROUP", &["PATHLOAD"]),
    pair_entry("FACTORI", &["FILEI"]),
    pair_entry("FAREI", &["FILEI"]),
    pair_entry("FIELDS", &["FILEO"]),
    pair_entry("FILE", &["PRINT", "READ"]),
    pair_entry("FILEI", &["PROCESS"]),
    pair_entry("FILES", &["WAIT4FILES"]),
    pair_entry("FORM", &["FILEO", "PRINT"]),
    pair_entry("FORMAT", &["FILEO"]),
    pair_entry("GEOMI[1]", &["FILEI"]),
    pair_entry("GPID_LINKNO", &["ARRAY"]),
    pair_entry("INCLUDE", &["FILEO"]),
    pair_entry("INTERPOLATE", &["LOOKUP"]),
    pair_entry("INTRASTEP", &["DISTRIBUTE"]),
    pair_entry("ITER", &["LOOP"]),
    pair_entry("LINEI[1]", &["FILEI"]),
    pair_entry("LINEI[2]", &["FILEI"]),
    pair_entry("LINEI[3]", &["FILEI"]),
    pair_entry("LINEI[4]", &["FILEI"]),
    pair_entry("LINEI[5]", &["FILEI"]),
    pair_entry("LINEI[6]", &["FILEI"]),
    pair_entry("LINEI[7]", &["FILEI"]),
    pair_entry("LINEI[8]", &["FILEI"]),
    pair_entry("LINKI", &["FILEI"]),
    pair_entry("LINKI[1]", &["FILEI"]),
    pair_entry("LINKI[2]", &["FILEI"]),
    pair_entry("LINKO", &["FILEO"]),
    pair_entry("LIST", &["LOOKUP", "PRINT"]),
    pair_entry("LOOKUPI", &["LOOKUP"]),
    pair_entry("LOOKUPI[1]", &["FILEI"]),
    pair_entry("LOOKUPI[2]", &["FILEI"]),
    pair_entry("LOOKUPI[3]", &["FILEI"]),
    pair_entry("LOOKUPI[4]", &["FILEI"]),
    pair_entry("LOOKUP[01]", &["LOOKUP"]),
    pair_entry("LOOKUP[02]", &["LOOKUP"]),
    pair_entry("LOOKUP[03]", &["LOOKUP"]),
    pair_entry("LOOKUP[04]", &["LOOKUP"]),
    pair_entry("LOOKUP[05]", &["LOOKUP"]),
    pair_entry("LOOKUP[06]", &["LOOKUP"]),
    pair_entry("LOOKUP[07]", &["LOOKUP"]),
    pair_entry("LOOKUP[08]", &["LOOKUP"]),
    pair_entry("LOOKUP[09]", &["LOOKUP"]),
    pair_entry("LOOKUP[10]", &["LOOKUP"]),
    pair_entry("LOOKUP[11]", &["LOOKUP"]),
    pair_entry("LOOKUP[12]", &["LOOKUP"]),
    pair_entry("LOOKUP[13]", &["LOOKUP"]),
    pair_entry("LOOKUP[14]", &["LOOKUP"]),
    pair_entry("LOOKUP[15]", &["LOOKUP"]),
    pair_entry("LOOKUP[16]", &["LOOKUP"]),
    pair_entry("LOOKUP[17]", &["LOOKUP"]),
    pair_entry("LOOKUP[18]", &["LOOKUP"]),
    pair_entry("LOOKUP[19]", &["LOOKUP"]),
    pair_entry("LOOKUP[1]", &["LOOKUP"]),
    pair_entry("LOOKUP[20]", &["LOOKUP"]),
    pair_entry("LOOKUP[21]", &["LOOKUP"]),
    pair_entry("LOOKUP[2]", &["LOOKUP"]),
    pair_entry("LOOKUP[3]", &["LOOKUP"]),
    pair_entry("LOOKUP[4]", &["LOOKUP"]),
    pair_entry("LOOKUP[5]", &["LOOKUP"]),
    pair_entry("LOOKUP[6]", &["LOOKUP"]),
    pair_entry("LP", &["LOOP"]),
    pair_entry("MATI[01]", &["FILEI"]),
    pair_entry("MATI[02]", &["FILEI"]),
    pair_entry("MATI[03]", &["FILEI"]),
    pair_entry("MATI[04]", &["FILEI"]),
    pair_entry("MATI[05]", &["FILEI"]),
    pair_entry("MATI[06]", &["FILEI"]),
    pair_entry("MATI[07]", &["FILEI"]),
    pair_entry("MATI[08]", &["FILEI"]),
    pair_entry("MATI[09]", &["FILEI"]),
    pair_entry("MATI[10]", &["FILEI"]),
    pair_entry("MATI[11]", &["FILEI"]),
    pair_entry("MATI[12]", &["FILEI"]),
    pair_entry("MATI[13]", &["FILEI"]),
    pair_entry("MATI[14]", &["FILEI"]),
    pair_entry("MATI[15]", &["FILEI"]),
    pair_entry("MATI[1]", &["FILEI"]),
    pair_entry("MATI[2]", &["FILEI"]),
    pair_entry("MATI[3]", &["FILEI"]),
    pair_entry("MATI[4]", &["FILEI"]),
    pair_entry("MATI[5]", &["FILEI"]),
    pair_entry("MATI[6]", &["FILEI"]),
    pair_entry("MATI[7]", &["FILEI"]),
    pair_entry("MATI[8]", &["FILEI"]),
    pair_entry("MATI[9]", &["FILEI"]),
    pair_entry("MATO", &["FILEO"]),
    pair_entry("MATO[1]", &["FILEO"]),
    pair_entry("MATO[2]", &["FILEO"]),
    pair_entry("MATO[3]", &["FILEO"]),
    pair_entry("MATO[4]", &["FILEO"]),
    pair_entry("MATO[5]", &["FILEO"]),
    pair_entry("MATO[6]", &["FILEO"]),
    pair_entry("MATO[7]", &["FILEO"]),
    pair_entry("MO", &["FILEO"]),
    pair_entry("MSG", &["RUN"]),
    pair_entry("MULTISTEP", &["DISTRIBUTE"]),
    pair_entry("MW[201]", &["PATHLOAD"]),
    pair_entry("MW[202]", &["PATHLOAD"]),
    pair_entry("MW[203]", &["PATHLOAD"]),
    pair_entry("MW[204]", &["PATHLOAD"]),
    pair_entry("MW[301]", &["PATHLOAD"]),
    pair_entry("MW[302]", &["PATHLOAD"]),
    pair_entry("MW[303]", &["PATHLOAD"]),
    pair_entry("MW[304]", &["PATHLOAD"]),
    pair_entry("MW[401]", &["PATHLOAD"]),
    pair_entry("MW[402]", &["PATHLOAD"]),
    pair_entry("MW[403]", &["PATHLOAD"]),
    pair_entry("MW[404]", &["PATHLOAD"]),
    pair_entry("NAME", &["FILEO", "LOOKUP"]),
    pair_entry("NEAREST", &["LOOKUP"]),
    pair_entry("NETI", &["FILEI"]),
    pair_entry("NETI[1]", &["FILEI"]),
    pair_entry("NETI[2]", &["FILEI"]),
    pair_entry("NETO", &["FILEO"]),
    pair_entry("NOACCESS", &["PATHLOAD"]),
    pair_entry("NODEI", &["FILEI"]),
    pair_entry("NODEI[1]", &["FILEI"]),
    pair_entry("NODEI[2]", &["FILEI"]),
    pair_entry("NODEO", &["FILEO"]),
    pair_entry("NTLEGI[1]", &["FILEI"]),
    pair_entry("NTLEGI[2]", &["FILEI"]),
    pair_entry("NTLEGI[3]", &["FILEI"]),
    pair_entry("NTLEGI[4]", &["FILEI"]),
    pair_entry("NUMLINKS", &["ARRAY"]),
    pair_entry("NUMREC", &["LOOP"]),
    pair_entry("ONOFFS", &["FILEO"]),
    pair_entry("PATH", &["PATHLOAD"]),
    pair_entry("PENI", &["PATHLOAD"]),
    pair_entry("PERIOD", &["LOOP"]),
    pair_entry("PGM", &["RUN"]),
    pair_entry("PHASE", &["PROCESS"]),
    pair_entry("PRINTO", &["FILEO", "PRINT"]),
    pair_entry("PRINTO[1]", &["FILEO"]),
    pair_entry("PRINTO[2]", &["FILEO"]),
    pair_entry("PRNFILE", &["RUN"]),
    pair_entry("PROCESSID", &["DISTRIBUTEINTRASTEP", "DISTRIBUTEMULTISTEP"]),
    pair_entry("PROCESSLIST", &["DISTRIBUTEINTRASTEP"]),
    pair_entry("PROCESSNUM", &["DISTRIBUTEMULTISTEP"]),
    pair_entry("QUESTION", &["PROMPT"]),
    pair_entry("READNTLEGI", &["GENERATE"]),
    pair_entry("RECNUM", &["LOOP"]),
    pair_entry("RECO", &["WRITE"]),
    pair_entry("RECORD", &["MERGE"]),
    pair_entry("RECO[1]", &["FILEO"]),
    pair_entry("RECO[2]", &["FILEO"]),
    pair_entry("RECO[3]", &["FILEO"]),
    pair_entry("RECO[4]", &["FILEO"]),
    pair_entry("REPORTO", &["FILEO"]),
    pair_entry("RESULT", &["LOOKUP"]),
    pair_entry("SORT", &["FILEI"]),
    pair_entry("SYSTEMI", &["FILEI"]),
    pair_entry("TAZID", &["FILEI"]),
    pair_entry("TRANTIME", &["PARAMETERS"]),
    pair_entry("TYPE", &["ARRAY"]),
    pair_entry("VAR", &["FILEI", "LOG"]),
    pair_entry("VOL[01]", &["PATHLOAD"]),
    pair_entry("VOL[02]", &["PATHLOAD"]),
    pair_entry("VOL[03]", &["PATHLOAD"]),
    pair_entry("VOL[04]", &["PATHLOAD"]),
    pair_entry("VOL[05]", &["PATHLOAD"]),
    pair_entry("VOL[06]", &["PATHLOAD"]),
    pair_entry("VOL[07]", &["PATHLOAD"]),
    pair_entry("VOL[08]", &["PATHLOAD"]),
    pair_entry("VOL[09]", &["PATHLOAD"]),
    pair_entry("VOL[10]", &["PATHLOAD"]),
    pair_entry("VOL[11]", &["PATHLOAD"]),
    pair_entry("VOL[12]", &["PATHLOAD"]),
    pair_entry("VOL[13]", &["PATHLOAD"]),
    pair_entry("VOL[14]", &["PATHLOAD"]),
    pair_entry("VOL[15]", &["PATHLOAD"]),
    pair_entry("VOL[16]", &["PATHLOAD"]),
    pair_entry("VOL[23]", &["PATHLOAD"]),
    pair_entry("Z", &["FILEI"]),
    pair_entry("ZDATI[1]", &["FILEI"]),
    pair_entry("ZDATI[2]", &["FILEI"]),
    pair_entry("ZDATI[3]", &["FILEI"]),
    pair_entry("ZDATI[4]", &["FILEI"]),
    pair_entry("ZDATI[5]", &["FILEI"]),
    pair_entry("ZDATI[6]", &["FILEI"]),
    pair_entry("_HOTZONE_DIST", &["ARRAY"]),
    pair_entry("_HOTZONE_TOLL", &["ARRAY"]),
    pair_entry("_HOTZONE_VMT", &["ARRAY"]),
    pair_entry("_HOTZONE_VOL", &["ARRAY"]),
];

const fn entry(name: &'static str, role: KeywordRole) -> KeywordEntry {
    KeywordEntry {
        name,
        role,
        observed_with: &[],
    }
}

const fn pair_entry(name: &'static str, observed_with: &'static [&'static str]) -> KeywordEntry {
    KeywordEntry {
        name,
        role: KeywordRole::PairKeyword,
        observed_with,
    }
}

fn all_entries() -> impl Iterator<Item = &'static KeywordEntry> {
    CONTROL_WORDS.iter().chain(PAIR_KEYWORDS.iter())
}

/// Returns the completion candidates for `ctx` (`contracts/
/// keyword-dictionary-api.md`).
///
/// - `ctx.enclosing_control_word == None`: every `ControlWord` entry (the
///   general-syntax fallback list).
/// - `ctx.enclosing_control_word == Some(word)`: every `PairKeyword` entry
///   whose `observed_with` contains `word` (case-insensitive); if that set
///   is empty, falls back to every `PairKeyword` entry regardless of
///   `observed_with` — never an empty suggestion list by construction,
///   though see module docs: `PAIR_KEYWORDS` is itself empty in this pass,
///   so this fallback currently always yields an empty `Vec` too, honestly
///   reflecting the not-yet-populated census rather than fabricating
///   entries.
pub fn completion_candidates(ctx: CompletionContext<'_>) -> Vec<&'static KeywordEntry> {
    match ctx.enclosing_control_word {
        None => CONTROL_WORDS.iter().collect(),
        Some(word) => {
            let scoped: Vec<&'static KeywordEntry> = PAIR_KEYWORDS
                .iter()
                .filter(|e| e.observed_with.iter().any(|w| w.eq_ignore_ascii_case(word)))
                .collect();
            if scoped.is_empty() {
                PAIR_KEYWORDS.iter().collect()
            } else {
                scoped
            }
        }
    }
}

/// Damerau-Levenshtein edit distance (research.md §5) — insertions,
/// deletions, substitutions, and adjacent-character transpositions, all
/// cost 1. Case-insensitive (compares uppercased chars).
fn damerau_levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.to_ascii_uppercase().chars().collect();
    let b: Vec<char> = b.to_ascii_uppercase().chars().collect();
    let (la, lb) = (a.len(), b.len());

    // d[i][j] = distance between a[..i] and b[..j].
    let mut d = vec![vec![0usize; lb + 1]; la + 1];
    for (i, row) in d.iter_mut().enumerate().take(la + 1) {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate().take(lb + 1) {
        *cell = j;
    }

    for i in 1..=la {
        for j in 1..=lb {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            let mut best = (d[i - 1][j] + 1) // deletion
                .min(d[i][j - 1] + 1) // insertion
                .min(d[i - 1][j - 1] + cost); // substitution
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(d[i - 2][j - 2] + 1); // transposition
            }
            d[i][j] = best;
        }
    }
    d[la][lb]
}

/// The core tie-aware nearest-match selection (research.md §5), factored out
/// of [`did_you_mean`] so it's directly unit-testable against a synthetic
/// dictionary, independent of what real entries happen to be close to what.
/// Returns `None` for an exact match, no match within distance 2, or a tie
/// between two or more equally-close entries.
fn nearest_within_threshold<'a>(
    token: &str,
    candidates: impl Iterator<Item = &'a KeywordEntry>,
) -> Option<&'a KeywordEntry> {
    let mut best_distance = usize::MAX;
    let mut best: Option<&'a KeywordEntry> = None;
    let mut tied = false;

    for candidate in candidates {
        if candidate.name.eq_ignore_ascii_case(token) {
            return None; // Exact match — nothing to suggest.
        }
        let dist = damerau_levenshtein(token, candidate.name);
        match dist.cmp(&best_distance) {
            std::cmp::Ordering::Less => {
                best_distance = dist;
                best = Some(candidate);
                tied = false;
            }
            std::cmp::Ordering::Equal => {
                tied = true;
            }
            std::cmp::Ordering::Greater => {}
        }
    }

    if tied || best_distance > 2 {
        None
    } else {
        best
    }
}

/// Fuzzy "did you mean" lookup (FR-014, research.md §5). Returns the unique
/// dictionary entry within Damerau-Levenshtein distance 2, when exactly one
/// exists. Returns `None` for an exact match (case-insensitive), no
/// sufficiently close match, or a tie between two or more equally-close
/// entries (spec Story 5 AS2/AS3, Edge Cases).
pub fn did_you_mean(token: &str) -> Option<&'static KeywordEntry> {
    if token.is_empty() {
        return None;
    }
    nearest_within_threshold(token, all_entries())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_candidates_general_fallback_when_no_control_word() {
        let candidates = completion_candidates(CompletionContext {
            enclosing_control_word: None,
        });
        assert!(candidates.iter().any(|e| e.name == "IF"));
        assert!(candidates.iter().any(|e| e.name == "RUN"));
        assert_eq!(candidates.len(), CONTROL_WORDS.len());
    }

    #[test]
    fn completion_candidates_scoped_returns_real_census_data_for_run() {
        // RUN's real 2026-08-10 census data (module docs): PGM, MSG, PRNFILE.
        let candidates = completion_candidates(CompletionContext {
            enclosing_control_word: Some("RUN"),
        });
        let names: Vec<&str> = candidates.iter().map(|e| e.name).collect();
        assert!(names.contains(&"PGM"), "names were: {names:?}");
        assert!(names.contains(&"MSG"), "names were: {names:?}");
        assert!(names.contains(&"PRNFILE"), "names were: {names:?}");
        // Scoped, not the general ControlWord list leaking through.
        assert!(!names.contains(&"IF"));
    }

    #[test]
    fn completion_candidates_scoped_falls_back_to_full_pair_list_when_control_word_has_no_data() {
        // "ENDIF" is a real ControlWord entry that never takes keyword=value
        // pairs in the census (block closers don't) — exercises the
        // documented fallback (data-model.md §1): every PairKeyword entry,
        // not an empty list, since PAIR_KEYWORDS itself is non-empty now.
        let candidates = completion_candidates(CompletionContext {
            enclosing_control_word: Some("ENDIF"),
        });
        assert_eq!(candidates.len(), PAIR_KEYWORDS.len());
    }

    #[test]
    fn did_you_mean_finds_unique_close_match_including_transposition() {
        // "FI" vs "IF" is a single transposition under Damerau-Levenshtein.
        let result = did_you_mean("FI");
        assert_eq!(result.map(|e| e.name), Some("IF"));
    }

    #[test]
    fn did_you_mean_no_match_within_threshold_returns_none() {
        assert!(did_you_mean("XYZZY123").is_none());
    }

    #[test]
    fn did_you_mean_exact_match_returns_none() {
        assert!(did_you_mean("if").is_none());
        assert!(did_you_mean("IF").is_none());
    }

    #[test]
    fn did_you_mean_empty_string_does_not_panic() {
        assert!(did_you_mean("").is_none());
    }

    #[test]
    fn did_you_mean_close_but_not_exact_finds_it() {
        // "ENDLOP" is one deletion away from "ENDLOOP" and nothing else in
        // the dictionary is remotely close, so this exercises the
        // non-tied, within-threshold path against the real dictionary.
        let result = did_you_mean("ENDLOP");
        assert_eq!(result.map(|e| e.name), Some("ENDLOOP"));
    }

    #[test]
    fn nearest_within_threshold_returns_none_on_a_genuine_tie() {
        // Synthetic dictionary, independent of real entries: "CAT" and
        // "COT" are both distance 1 from "COT"... use "CAT"/"DOT" instead,
        // both distance 1 from "COT" (one substitution each), so "COT"
        // ties between them and must resolve to None.
        let a = entry("CAT", KeywordRole::ControlWord);
        let b = entry("DOT", KeywordRole::ControlWord);
        let dict = [a, b];
        assert!(nearest_within_threshold("COT", dict.iter()).is_none());
    }

    #[test]
    fn nearest_within_threshold_returns_the_unique_closest() {
        let a = entry("CAT", KeywordRole::ControlWord);
        let b = entry("ELEPHANT", KeywordRole::ControlWord);
        let dict = [a, b];
        let result = nearest_within_threshold("COT", dict.iter());
        assert_eq!(result.map(|e| e.name), Some("CAT"));
    }
}
