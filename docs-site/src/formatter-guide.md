# Formatter Guide

## What formatting guarantees

The formatter is **idempotent** — running it twice produces the same result as
running it once (`format(format(x)) == format(x)`). It is **strictly
behavior-preserving**: it never reorders statements, never changes which lines
are continuations of a prior statement, and never alters program meaning. It
only ever changes whitespace and, opt-in, keyword casing. If a script is
structurally broken (see the [Editor Guide](editor-guide.md#diagnostics)'s
diagnostic list), the formatter still does its best on the parts it understands
rather than refusing outright — but a diagnosed/unmatched block's own children
are left with their original indentation rather than guessed at.

Every field below is documented in full in the
[Configuration Reference](configuration-reference.md) — this page focuses on
*what changes*, with real examples.

## Casing

The three independent casing fields —
[`casing_control_words`](configuration-reference.md#casing_control_words),
[`casing_pair_keywords`](configuration-reference.md#casing_pair_keywords), and
[`casing_data_references`](configuration-reference.md#casing_data_references) —
only ever touch keyword *names* — never a value, and never a category they
aren't scoped to. This example makes both boundaries visible at once, with
`casing_control_words = "upper"`:

```diff
-run pgm=matrix
+RUN pgm=matrix
     mati=base.mat,mo=out.mat
-endrun
+ENDRUN
```

`run`/`endrun` (control words) are uppercased — but `pgm` (a pair-keyword name,
`casing_pair_keywords`'s own scope, left unset here), `matrix` (a value, never
touched), and `mati`/`mo` (data-reference tokens,
[`casing_data_references`](configuration-reference.md#casing_data_references)'s
own scope, also left unset here) all stay exactly as written. Each of the
three fields only ever affects its own category — set the one(s) you want
independently.

## Indentation

[`indent_top_level`](configuration-reference.md#indent_top_level) controls
depth-0 statements; [`indent_width`](configuration-reference.md#indent_width)
controls spacing per nesting level inside a block, relative to the block's own
opening-statement column. See [Getting Started](getting-started.md#3-see-what-formatting-would-change)
for a full before/after example.

## Operator spacing

`preserve` (the default) leaves spacing exactly as written. `fixed` normalizes
every operator to exactly one space on each side and removes interior padding
inside brackets/parens:

```diff
-IF(ZONES==1)
-    ZONES=1
-    CNT=2
-    ITER=333
+IF(ZONES == 1)
+    ZONES = 1
+    CNT = 2
+    ITER = 333
 ENDIF
```

`auto` does everything `fixed` does, plus vertically aligns the `=` of
consecutive `Assignment` statements at the same nesting depth to the longest
left-hand side in the run:

```diff
-IF(ZONES==1)
-    ZONES=1
-    CNT=2
-    ITER=333
+IF(ZONES == 1)
+    ZONES = 1
+    CNT   = 2
+    ITER  = 333
 ENDIF
```

A run resets at a blank line, a comment-only line, a nesting-depth change, or a
non-`Assignment` statement — so alignment never reaches across unrelated
sections of a script.

## Blank-line normalization

`preserve` (the default) leaves every run of consecutive blank lines exactly as
written, however long. `auto` contracts a run down to the applicable cap
([`blank_lines_top_cap`](configuration-reference.md#blank_lines_top_cap)
between top-level statements/blocks, default `2`;
[`blank_lines_nested_cap`](configuration-reference.md#blank_lines_nested_cap)
inside any block's body, default `1`) — only when a run *exceeds* the cap, never
padding a shorter run up:

```diff
 RUN PGM=MATRIX
-
 
     MATI=a.mat
 
-
     MATO=b.mat
-
-
 
 ENDRUN
```

## `; FMT: OFF` / `; FMT: ON` regions

Wrap a range in `; FMT: OFF` / `; FMT: ON` to exclude it from formatting
entirely — useful for a block whose hand-tuned spacing carries meaning to a
reviewer that automatic formatting would otherwise flatten:

```diff
-RUN PGM=MATRIX
+RUN PGM = MATRIX
 ; FMT: OFF
     ZONES=1
 ; FMT: ON
-    MATI=a.mat
+    MATI = a.mat
 ENDRUN
```

Everything between the markers (`ZONES=1` above) is untouched, while the lines
outside them still get `operator_spacing = "fixed"` applied. An **unclosed**
`; FMT: OFF` (no matching `; FMT: ON` before end of file) protects through the
rest of the file and is always surfaced — never silently unbounded with no
indication — as a Hint diagnostic (source `drut-fmt`, see the
[Editor Guide](editor-guide.md#diagnostics)), a CLI stderr notice, or an MCP
`format` response field, depending on which surface you're using.
