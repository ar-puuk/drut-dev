# Contract: Casing Gains an Explicit `Preserve` Mode

Amends `002-cli-check-format/spec.md`'s FR-015 in place, and corrects
FR-026's now-stale contrast against it (research.md §3) — not a new,
competing contract file.

## `spec.md` FR-015 — amended with a new dated entry

The existing bullet gets a dated entry appended, preserving the original
text:

> **FR-015**: `format` MUST support an opt-in keyword-case normalization
> flag, defaulting to OFF. [...original 002-era text, unchanged...]
> **Amended 2026-08-13 (`014-casing-preserve-mode`)**: the underlying
> representation changes from an optional value (`Option<CasingConvention>`,
> `None` meaning "off") to a three-valued `CasingConvention` enum
> (`Preserve`/`#[default]`, `Upper`, `Lower`) — mirroring
> `TopLevelIndentMode`'s already-shipped shape (FR-026). This is a pure
> representation change: `Preserve` produces byte-identical output to the
> old `None` for every input (research.md §4). `--casing` gains a third
> explicit CLI value, `preserve`, letting a user force casing untouched for
> one run even when a resolved `drut.toml` specifies `upper`/`lower` — the
> existing "no bare `--casing`" rule is unchanged; the flag still requires
> an explicit value whenever given at all.

## `spec.md` FR-026 — corrected, not just left alongside

FR-026's own text currently contrasts itself against `--casing` with a
sentence that becomes inaccurate the moment this feature ships
(research.md §3). The sentence:

> Unlike FR-015's `--casing` flag, this setting has no "off" state —
> omitting the flag resolves to the explicit `preserve` default, not an
> unset/`None` value.

is corrected to:

> As of `014-casing-preserve-mode`, `--casing` shares this same shape —
> both settings resolve to an explicit `preserve`/`Preserve` default at the
> `FormatOptions` layer, with no distinct "off"/`None` state remaining at
> that layer. The two settings differ only in how many *named* states they
> carry (`--casing` has three: `preserve`/`upper`/`lower`; `--top-level-
> indent` has two: `preserve`/`normalize`), not in whether omission is
> distinguishable from an explicit value — for both, that distinction is
> preserved one layer up, in `ExplicitFormatOverride`'s own `Option`-wrapped
> fields (`drut_config::lib.rs`), not in `FormatOptions` itself.

## Algorithm (normative, research.md §1)

```text
render(source, nodes, diagnostics, options):
  ...
  if options.casing != CasingConvention::Preserve:          # was: if let Some(convention) = options.casing
    collect_casing_edits(nodes, char_lines, protected, options.casing, &mut casing_edits)
  ...

edit_for_span(lines, span, convention):
  ...
  replacement = match convention:
    Upper => original.to_ascii_uppercase()
    Lower => original.to_ascii_lowercase()
    Preserve => original.clone()          # NEW -- exhaustiveness only,
                                           # unreachable in practice: render's
                                           # guard above never calls this
                                           # function's call chain at all
                                           # when options.casing == Preserve
  ...
```

`collect_casing_edits`, `collect_block_casing_edits`,
`collect_statement_casing_edits`, `push_if_present` are **not modified** —
all already take a bare `CasingConvention`, unchanged (research.md §1).

## `FormatOptions.casing` construction/read call-site treatment (research.md §2, normative)

| Call site | Required change |
|---|---|
| `voyager-core/src/format.rs` — `CasingConvention` enum, `FormatOptions.casing` field | Add `Preserve` variant/`#[default]`; change field type to bare `CasingConvention`. |
| `voyager-core/src/format.rs` — `render`'s casing-edit gate | `!=` comparison replaces `if let Some`, per Algorithm above. |
| `voyager-core/src/format.rs` — `edit_for_span`'s match | New `Preserve` arm, per Algorithm above. |
| `drut_config/src/lib.rs` — `resolve_format_options`, `default_options` | Both `casing` lines gain `.unwrap_or_default()`. |
| `drut_config/src/parse.rs` — `parse_casing` | New `Some("preserve") => Some(CasingConvention::Preserve)` arm; error messages name all three values. |
| `drut-cli/src/cli.rs` — `CasingArg` | New `Preserve` `ValueEnum` variant; stays `Option<CasingArg>`-wrapped on the `Format` subcommand, unchanged. |
| `drut-cli/src/format_cmd.rs` — `impl From<CasingArg> for CasingConvention` | New `CasingArg::Preserve => CasingConvention::Preserve` arm. |
| `drut-mcp/src/format.rs` — `explicit_override`'s `casing` match | New `Some("preserve") => Some(CasingConvention::Preserve)` arm; error message updated. |
| `drut-lsp/src/formatting.rs`, `range_formatting.rs` | No code change — neither constructs an explicit override; existing test suite passing unmodified is the confirmation. |
| `drut_config::FormatConfig`/`ExplicitFormatOverride` (`drut_config/src/lib.rs`) | **Explicitly unchanged** — both keep `casing: Option<CasingConvention>`. |
| Everywhere else (`FormatOptions::default()`, existing test literals) | Compiler-forced where a struct literal exists (`voyager-core`'s own test module); no behavioral change anywhere, confirmed by the existing suite passing unmodified (research.md §4). |
