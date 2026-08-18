//! Blank-line-run recognition (`019-blank-line-normalization`; spec.md
//! FR-001–FR-008, data-model.md §1). A read-only recognition pass over
//! already-parsed `Node`/line data, no lexer/parser change — mirrors
//! `data_reference.rs`/`operator_spacing.rs`'s established self-contained-
//! module shape.

use std::collections::BTreeSet;

use crate::Node;

/// Every line number (1-based) that falls strictly inside some *top-level*
/// block's own span — no recursion into `children`/branches needed at all
/// (research.md §4): a nested block's `span` is always entirely contained
/// within its parent's (guaranteed by `block.rs`'s own matching logic), so
/// marking only the top-level slice's own block ranges already correctly
/// classifies every line at every depth. A line not covered by any
/// top-level block's range is top-level by elimination.
pub(crate) fn nested_lines(nodes: &[Node]) -> BTreeSet<u32> {
    let mut nested = BTreeSet::new();
    for node in nodes {
        if let Node::Block(block) = node {
            let start = block.span.start.line + 1;
            let end = block.span.end.line;
            let mut line = start;
            while line <= end {
                nested.insert(line);
                line += 1;
            }
        }
    }
    nested
}

/// A maximal run of consecutive blank lines (research.md §2's whitespace
/// convention: a line is blank when every character on it is `' '` or
/// `'\t'`, vacuously true for a zero-length line). `is_nested`/
/// `is_protected` are each computed once per run from its first line
/// (research.md §3: a run can never straddle a block boundary or a
/// protected-region boundary, since both are bounded by non-blank lines, so
/// this is exact, not an approximation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlankRun {
    pub first_line: u32,
    pub len: u32,
    pub is_nested: bool,
    pub is_protected: bool,
}

fn is_blank_line(line: &[char]) -> bool {
    line.iter().all(|c| *c == ' ' || *c == '\t')
}

pub(crate) fn find_blank_runs(lines: &[Vec<char>], nested: &BTreeSet<u32>, protected: &BTreeSet<u32>) -> Vec<BlankRun> {
    let mut runs = Vec::new();
    let mut idx = 0usize;
    while idx < lines.len() {
        if is_blank_line(&lines[idx]) {
            let first_line = (idx + 1) as u32;
            let mut end_idx = idx;
            while end_idx + 1 < lines.len() && is_blank_line(&lines[end_idx + 1]) {
                end_idx += 1;
            }
            let len = (end_idx - idx + 1) as u32;
            runs.push(BlankRun {
                first_line,
                len,
                is_nested: nested.contains(&first_line),
                is_protected: protected.contains(&first_line),
            });
            idx = end_idx + 1;
        } else {
            idx += 1;
        }
    }
    runs
}

/// For each non-protected run whose length exceeds its applicable cap
/// (`nested_cap` if `is_nested` else `top_level_cap`), the line numbers to
/// delete — the run's own trailing `len - cap` lines (research.md §5: the
/// run's first `cap` lines always survive untouched, FR-006). A protected
/// run contributes nothing.
pub(crate) fn lines_to_delete(
    nodes: &[Node],
    lines: &[Vec<char>],
    protected: &BTreeSet<u32>,
    top_level_cap: u8,
    nested_cap: u8,
) -> BTreeSet<u32> {
    let nested = nested_lines(nodes);
    let runs = find_blank_runs(lines, &nested, protected);

    let mut to_delete = BTreeSet::new();
    for run in runs {
        if run.is_protected {
            continue;
        }
        let cap = if run.is_nested { nested_cap } else { top_level_cap } as u32;
        if run.len > cap {
            let mut line = run.first_line + cap;
            let end = run.first_line + run.len - 1;
            while line <= end {
                to_delete.insert(line);
                line += 1;
            }
        }
    }
    to_delete
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn char_lines(source: &str) -> Vec<Vec<char>> {
        // Mirrors format.rs's own split_lines exactly on the point that
        // matters here: a source ending in '\n' does NOT get a spurious
        // trailing empty "line" after the last real one (format.rs's
        // `while !rest.is_empty()` loop stops the moment the final '\n' is
        // consumed) -- naively using `source.split('\n')` instead would
        // fabricate an extra blank line at end-of-file that the real
        // renderer never sees, silently changing run boundaries in a test
        // fixture ending with "...\n".
        let mut result = Vec::new();
        let mut rest = source;
        while !rest.is_empty() {
            if let Some(pos) = rest.find('\n') {
                let (line, after) = rest.split_at(pos);
                let line = line.strip_suffix('\r').unwrap_or(line);
                result.push(line.chars().collect());
                rest = &after[1..];
            } else {
                result.push(rest.chars().collect());
                rest = "";
            }
        }
        result
    }

    fn nodes_of(source: &str) -> Vec<Node> {
        parse(source).nodes
    }

    // -- nested_lines ------------------------------------------------------

    #[test]
    fn nested_lines_marks_top_level_blocks_interior_and_leaves_top_level_only_lines_unmarked() {
        // T008: a top-level block's interior is marked; content strictly
        // between top-level nodes (never inside any block) is not.
        let src = "X = 1\n\nIF (A=1)\nY = 2\nENDIF\n\nZ = 3\n";
        let nodes = nodes_of(src);
        let nested = nested_lines(&nodes);
        // Line 1: X = 1 (top-level statement) -- not nested.
        assert!(!nested.contains(&1));
        // Line 2: blank line between top-level statement and IF block -- not nested.
        assert!(!nested.contains(&2));
        // Line 3: IF (opener line itself) -- not counted as "nested" (the
        // range starts at start.line + 1).
        assert!(!nested.contains(&3));
        // Line 4: Y = 2, inside the IF's body -- nested.
        assert!(nested.contains(&4));
        // Line 5: ENDIF (closer line) -- IS included, since span.end.line is
        // the closer's own line and the range is inclusive of it.
        assert!(nested.contains(&5));
        // Line 6: blank line after the block, before the next top-level
        // statement -- not nested.
        assert!(!nested.contains(&6));
        // Line 7: Z = 3, top-level -- not nested.
        assert!(!nested.contains(&7));
    }

    #[test]
    fn nested_lines_covers_a_doubly_nested_block_for_free_without_recursion() {
        // T008 (load-bearing per research.md §4 / tasks.md notes): a LOOP
        // nested inside a RUN block, with a blank-line run inside the LOOP's
        // own body -- the *top-level* RUN's own span already covers every
        // line at every depth, with zero recursion into RUN's children.
        let src = "RUN PGM=MATRIX\nLOOP i=1,5\nY = 2\n\n\n\n\nZ = 3\nENDLOOP\nENDRUN\n";
        let nodes = nodes_of(src);
        let nested = nested_lines(&nodes);
        // Lines 2-9 (LOOP opener through ENDLOOP) are all inside RUN's span
        // (RUN starts line 1, ends line 10) -- so lines 4-7 (the blank run
        // inside LOOP's own body, doubly nested) must be marked nested,
        // despite nested_lines() never looking at RUN's children at all.
        for line in 4..=7 {
            assert!(nested.contains(&line), "line {line} (doubly nested) must be marked nested");
        }
        // Sanity: the doubly-nested LOOP's own span really is inside RUN's.
        let Node::Block(run_block) = &nodes[0] else { panic!("expected a Run block") };
        assert_eq!(run_block.children.len(), 1);
        let Node::Block(loop_block) = &run_block.children[0] else { panic!("expected a nested Loop block") };
        assert!(loop_block.span.start.line >= run_block.span.start.line);
        assert!(loop_block.span.end.line <= run_block.span.end.line);
    }

    // -- find_blank_runs -----------------------------------------------------

    #[test]
    fn find_blank_runs_treats_whitespace_only_line_as_blank_and_groups_with_zero_length_neighbor() {
        let src = "X = 1\n\n   \n\t\nY = 2\n";
        let lines = char_lines(src);
        let empty = BTreeSet::new();
        let runs = find_blank_runs(&lines, &empty, &empty);
        assert_eq!(runs.len(), 1, "a zero-length line, a spaces-only line, and a tab-only line form ONE run");
        assert_eq!(runs[0].first_line, 2);
        assert_eq!(runs[0].len, 3);
    }

    #[test]
    fn find_blank_runs_computes_is_nested_and_is_protected_once_per_run_from_first_line() {
        let src = "X = 1\n\n\n\nY = 2\n";
        let lines = char_lines(src);
        let mut nested = BTreeSet::new();
        nested.insert(2);
        nested.insert(3);
        nested.insert(4);
        let mut protected = BTreeSet::new();
        protected.insert(2);
        let runs = find_blank_runs(&lines, &nested, &protected);
        assert_eq!(runs.len(), 1);
        assert!(runs[0].is_nested);
        assert!(runs[0].is_protected);
    }

    // -- lines_to_delete -------------------------------------------------

    #[test]
    fn lines_to_delete_keeps_exactly_the_first_cap_lines_of_an_over_cap_run() {
        // Top-level run of 5 blank lines, default cap 2 -- lines 4-7 deleted
        // (the trailing 3), lines 2-3 survive.
        let src = "X = 1\n\n\n\n\n\nY = 2\n";
        let lines = char_lines(src);
        let nodes = nodes_of(src);
        let protected = BTreeSet::new();
        let to_delete = lines_to_delete(&nodes, &lines, &protected, 2, 1);
        assert_eq!(to_delete, [4u32, 5, 6].into_iter().collect::<BTreeSet<_>>());
    }

    #[test]
    fn lines_to_delete_leaves_an_at_or_under_cap_run_alone() {
        let src = "X = 1\n\n\nY = 2\n";
        let lines = char_lines(src);
        let nodes = nodes_of(src);
        let protected = BTreeSet::new();
        let to_delete = lines_to_delete(&nodes, &lines, &protected, 2, 1);
        assert!(to_delete.is_empty());
    }

    #[test]
    fn lines_to_delete_applies_nested_cap_uniformly_regardless_of_depth() {
        // A doubly-nested over-cap run (nested cap 1) gets the SAME cap as a
        // singly-nested one -- not a further-reduced value at deeper nesting
        // (FR-008).
        let src = "RUN PGM=MATRIX\nLOOP i=1,5\nY = 2\n\n\n\n\nZ = 3\nENDLOOP\nENDRUN\n";
        let lines = char_lines(src);
        let nodes = nodes_of(src);
        let protected = BTreeSet::new();
        let to_delete = lines_to_delete(&nodes, &lines, &protected, 2, 1);
        // Blank run is lines 4-7 (len 4), nested cap 1 -- first line (4)
        // survives, lines 5-7 deleted.
        assert_eq!(to_delete, [5u32, 6, 7].into_iter().collect::<BTreeSet<_>>());
    }

    #[test]
    fn lines_to_delete_contributes_nothing_for_a_run_inside_a_protected_region() {
        let src = "X = 1\n\n\n\n\n\nY = 2\n";
        let lines = char_lines(src);
        let nodes = nodes_of(src);
        let mut protected = BTreeSet::new();
        for line in 2..=6 {
            protected.insert(line);
        }
        let to_delete = lines_to_delete(&nodes, &lines, &protected, 2, 1);
        assert!(to_delete.is_empty());
    }
}
