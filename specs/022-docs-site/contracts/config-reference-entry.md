# Contract: Configuration Reference Entry

Defines what every one of the 10 `[format]` field entries in
`docs-site/src/configuration-reference.md` MUST contain (spec.md FR-003–FR-006).
This is also the contract `scripts/check-docs-coverage.ps1` partially enforces
(field-name presence only — the rest is a human-review concern, not automatable).

## Required shape per entry

Each field gets its own heading (`### <drut.toml key>`, e.g. `### blank_lines`) —
the heading text is what the coverage script matches against `FormatConfig`'s
field names, so it MUST be the exact `drut.toml` key, not a prettified label.

Under that heading, in this order:

1. **One-sentence summary** of what the field controls.
2. **Values** — every accepted value, each with a one-clause plain-language
   meaning (not just the bare token — e.g. not just "`auto`" but "`auto` — contracts
   runs of blank lines down to the configured cap").
3. **Default** — the value used when the field is unset at every tier.
4. **Also known as** — the field's CLI flag (e.g. `--blank-lines`) and MCP tool
   parameter name (e.g. `blank_lines`), when either differs in spelling from the
   `drut.toml` key or a reader might reasonably search for it under either name
   (FR-004). Every field in the current set uses the same snake_case spelling
   across all three surfaces except the CLI's kebab-case flag rendering — still
   state both explicitly rather than assuming the reader infers the kebab-case
   conversion.
5. **Example** — a minimal `drut.toml` `[format]` snippet showing the field set to
   a non-default value.
6. **Precedence** — a one-line pointer to the shared precedence-chain explanation
   (data-model.md's Precedence Chain entity), not a restated copy of it (FR-005).
   A field with a legacy/granular relationship (the four casing fields) MUST also
   state that relationship inline here, in both directions (FR-006) — e.g.
   `control_words_casing`'s entry states it overrides `casing` for this category;
   `casing`'s entry states the three granular fields exist and take priority when
   set.

## Acceptance check

- [ ] All 10 field headings present, spelled exactly as the `drut.toml` key
      (`casing`, `control_words_casing`, `pair_keywords_casing`,
      `data_references_casing`, `top_level_indent`, `indent_width`,
      `operator_spacing`, `blank_lines`, `top_level_blank_line_cap`,
      `nested_blank_line_cap`).
- [ ] Every entry has all 6 required parts, in order.
- [ ] The precedence chain is documented exactly once on the page, not repeated
      per field.
- [ ] Both directions of the legacy/granular relationship are stated (on
      `casing`'s entry AND on each of the three granular fields' entries).
