# Contract: Function-Call Casing Normalization (amends `017-casing-categories-indent-width`)

Extends `voyager-core::format::CasingSettings` and its four adapter surfaces
(`drut-config`, `drut-cli`, `drut-mcp`, `editors/vscode` client settings). A conceptual
signature contract, not final Rust/TOML/JSON source — same convention every prior
contract doc in this repo follows.

## Public API change (additive only)

- `voyager_core::CasingSettings` gains one new field, `function_calls:
  CasingConvention` — every existing field, every existing function signature
  (`format`/`format_bytes`), and every other public type is unchanged. A caller
  constructing `CasingSettings` with struct-update syntax (`..CasingSettings::default()`)
  needs no code change; a caller naming every field explicitly gains one new field to
  set (or omit, defaulting to `Preserve`).
- `voyager_core::function_call_entries()`, `function_call_occurrences()`,
  `FunctionCallEntry`, `FunctionCallOccurrence` are new public exports, mirroring
  `data_reference_entries()`/`data_reference_occurrences()`/`DataReferenceEntry`/
  `DataReferenceOccurrence` exactly in shape and visibility.
- `drut-config`, `drut-cli`, `drut-mcp` each gain one new field/flag/parameter
  (`casing_function_calls`), following the exact `Option<CasingConvention>` /
  `Option<CasingArg>` shape their existing three casing fields already use — additive,
  no existing field renamed or removed.

## Behavior contract

- **Recognition scope**: a `Word` token matches the `function_calls` category only when
  (a) its text case-insensitively equals one of the 138 names in
  `data-model.md` §1, (b) it is not inside a single-/double-quoted string, and (c) it is
  immediately followed by `(` with zero intervening whitespace. All three conditions are
  required simultaneously — relaxing any one reintroduces a real false positive
  (`FORMAT=CSV`/bare `LOG` without (c); a quoted `'...REPLACESTR(...)...'` without (b); an
  unrecognized `MYCALC(x)` without (a)).
- **`Preserve` (default) is a strict no-op**: `casing_function_calls` unset or
  `"preserve"` produces byte-identical output to today's formatter, for every input —
  the new recognition/edit code path is never reached at all when this field is
  `Preserve` (`data-model.md` §4's `!= Preserve` gate).
- **`Upper`/`Lower` rewrite exactly the matched span**: only the function-name token's
  own casing changes; the arguments, the surrounding statement, and every other token
  are byte-identical before and after.
- **Single ownership, never double-claimed**: a token claimed by `control_words`,
  `pair_keywords`, or `data_references` for a given occurrence is never also claimed by
  `function_calls` for that same occurrence, and vice versa — structurally guaranteed by
  the disjoint trigger conditions (`research.md` §3/§5), not by an explicit skip list.
- **Idempotent**: `format(format(x)) == format(x)` for every `casing_function_calls`
  value, on every input, including inputs already in the target casing (no edit
  produced when nothing would change).
- **No diagnostic, no parser, no grammar change**: `voyager_core::parse`/`tokenize`'s
  output, and every `Diagnostic` category, are entirely unaffected by this feature.
- **`editors/vscode`'s highlighting grammar is untouched**: `024`'s `#function-calls`
  TextMate pattern continues to render identically regardless of this feature's
  configuration — highlighting and casing are independent, non-interacting concerns.

## Illustrative examples

| Input (`casing_function_calls = "upper"`) | Output | Why |
|---|---|---|
| `RouteName = replacestr(RouteName,'-','',0)` | `RouteName = REPLACESTR(RouteName,'-','',0)` | Function call on an assignment RHS |
| `if (rightstr(trim(RouteName),1)='-')` | `if (RIGHTSTR(TRIM(RouteName),1)='-')` | Nested function calls both rewritten |
| `FILEO format=csv` (`casing_pair_keywords = "upper"`, `casing_function_calls = "lower"`) | `FILEO FORMAT=csv` | `format` here is a pair-keyword name (followed by `=`), not a function call — governed by `casing_pair_keywords`, untouched by `casing_function_calls` |
| `X = FORMAT(volume,8,2,',')` (same settings as above) | `X = format(volume,8,2,',')` | `FORMAT` here is a function call (followed by `(`) — governed by `casing_function_calls`, untouched by `casing_pair_keywords` |
| `PRINT LIST='calling replacestr(x) here'` | unchanged | Function-shaped text inside a quoted string is never a real occurrence |
| `MAX = 100` | unchanged | `MAX` here is not followed by `(` — not a function-call occurrence (governed by whichever category recognizes this position, if any; not this one) |
