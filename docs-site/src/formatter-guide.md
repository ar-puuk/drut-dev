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

The four independent casing fields —
[`casing_control_words`](configuration-reference.md#casing_control_words),
[`casing_pair_keywords`](configuration-reference.md#casing_pair_keywords),
[`casing_data_references`](configuration-reference.md#casing_data_references),
and [`casing_function_calls`](configuration-reference.md#casing_function_calls) —
only ever touch keyword *names* — never a value, and never a category they
aren't scoped to. This example makes several boundaries visible at once, with
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
own scope, also left unset here) all stay exactly as written. Each field only
ever affects its own category — set the one(s) you want independently.

### Function-call casing

`casing_function_calls` normalizes a recognized Cube Voyager built-in function
name's casing, but only where it's immediately followed by `(` with no
intervening whitespace — the unambiguous call position, since Voyager has no
user-definable functions:

```diff
-RouteName = replacestr(RouteName,'-','',0)
+RouteName = REPLACESTR(RouteName,'-','',0)
```

The recognized list (138 names) spans the general Control Language core
(`ABS`, `TRIM`, `REPLACESTR`, `ROUND`, ...), Highway/Matrix-program functions
(`ROWSUM`, `PATHTRACE`, ...), Public Transport skim functions (`TIMEA`,
`BRDINGS`, `GCOST`, ...), the CONVERGE-phase iteration-statistics family
(`GAPCHANGE`, `RGAPMIN`, ...), and CUBE Cluster utility functions — the same
list the VS Code extension's syntax highlighting uses (see the
[Editor Guide](editor-guide.md#syntax-highlighting)).

Two real names exist as more than one thing in Cube Voyager: `FORMAT` is also
a `FILEO` pair-keyword, and `LOG` is also a control word. Each occurrence's
own position decides which field governs it — never both, never neither:

```diff
 [format]
 casing_pair_keywords = "upper"
 casing_function_calls = "lower"
```

```diff
-FILEO format=csv
+FILEO FORMAT=csv
-X = FORMAT(volume,8,2,',')
+X = format(volume,8,2,',')
```

`format` on the first line is a pair-keyword name (followed by `=`), governed
by `casing_pair_keywords` alone. `FORMAT` on the second line is a function
call (followed by `(`), governed by `casing_function_calls` alone.

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

## Line wrapping

`preserve` (the default) leaves every line exactly as written, however long.
`auto` wraps an over-width `Control` statement's `keyword=value` pair list
across multiple physical lines once it exceeds
[`line_wrap_width`](configuration-reference.md#line_wrap_width) (default
`120`) — using Cube Voyager's own existing line-continuation syntax, the same
trailing comma that already makes the next physical line a continuation of the
same statement. Only `Control` statements are eligible; an `Assignment`
statement's arithmetic/string expression is never touched by this feature.

[`line_wrap_style`](configuration-reference.md#line_wrap_style) decides how
pairs are distributed across the new continuation lines. `fill` (the default)
packs as many pairs as fit per line:

```diff
-RUN PGM=MATRIX, ZONES=5, PRINT=1, COMBINE=T
+RUN PGM=MATRIX, ZONES=5, PRINT=1,
+    COMBINE=T
```

`one_per_line` places exactly one pair per continuation line instead, however
much width is left over on any given line:

```diff
-RUN PGM=MATRIX, ZONES=5, PRINT=1, COMBINE=T
+RUN PGM=MATRIX,
+    ZONES=5,
+    PRINT=1,
+    COMBINE=T
```

(Both examples above use a narrowed `line_wrap_width = 40` so the wrap is
visible at doc-page width — the real default is `120`.)

A statement that already contains a continuation character anywhere — i.e. you
already hand-wrapped it — is left completely untouched, regardless of width:

```
RUN PGM=MATRIX,
    ZONES=5, PRINT=1, COMBINE=T
```

stays exactly as written under `line_wrap = "auto"`, even though the combined
statement is well over 40 characters. This is deliberate, not a missed case:
it's the safest boundary for an output-modifying transform (no fighting
hand-formatted content) and it's also what makes the feature idempotent by
construction — once `auto` wraps a statement, the wrapped result itself
contains a continuation character, so a second format pass sees "already
continued" and leaves it alone. `format(format(x)) == format(x)` holds without
needing to re-derive it from scratch.

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
