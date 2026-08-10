# Contract: `voyager-core::keywords` Public API

A new addition to `voyager-core`'s existing public contract
(`001-voyager-script-parser/contracts/public-api.md`) — this section supplements
that file rather than replacing it. Like the rest of `voyager-core`, this module
adds no runtime dependency (research.md §4/§5) and never panics.

## Entry points

```text
fn completion_candidates(ctx: CompletionContext) -> Vec<&'static KeywordEntry>
fn did_you_mean(token: &str) -> Option<&'static KeywordEntry>
```

See `data-model.md` §1 for `KeywordEntry`/`KeywordRole`/`CompletionContext`'s
exact shape.

- **Input**: `completion_candidates` takes a `CompletionContext` describing only
  *where* the request happened structurally (an optional enclosing control word),
  never a document, URI, or LSP type — keeping this module protocol-agnostic
  exactly as `001-voyager-script-parser/contracts/public-api.md` requires of the
  rest of the crate. `did_you_mean` takes a single already-extracted token string
  (the caller — `drut-lsp` — is responsible for deciding which token to check).
- **Determinism**: Both functions are pure and side-effect-free; the same input
  always produces the same output (mirrors `tokenize`/`parse`'s determinism
  guarantee) — required for `drut-lsp`'s own tests to be meaningful and safe to
  call on every keystroke.
- **No panics**: Neither function panics on any input, including an empty
  string, a token containing non-ASCII/non-UTF-8-adjacent content, or a
  `CompletionContext` naming a control word the dictionary has never heard of
  (returns the general fallback list in that case, per data-model.md §1's
  `completion_candidates` validation rule — never an error).
- **Case sensitivity**: Both functions compare case-insensitively against
  dictionary entries (mirrors `001-voyager-script-parser` FR-011), consistent
  with how `voyager-core` already treats control-word/keyword matching
  elsewhere.

## Dictionary provenance (constitution Principle II)

The dictionary itself (the concrete list of `KeywordEntry` values) is a
hand-written artifact derived from the FR-012 corpus census — structural-position
classification against the fixture corpus, the same methodology
`001-voyager-script-parser`'s own control-word evidence trail used (see that
spec's Assumptions on FR-003/CHK008) — never copied from vendor documentation.
Each entry's `observed_with` list is a direct output of that census (which
control words a given `keyword=value` pair name was actually seen paired with in
real scripts), not a guess.

## What this contract does *not* promise (by design, this phase)

- **No exhaustiveness guarantee.** `001-voyager-script-parser`'s own settled
  finding (CHK008) is that there is no closed keyword vocabulary in the grammar
  itself — this dictionary is a best-effort, real-usage-grounded completion/
  spell-check aid, not a validation source. A keyword absent from the dictionary
  is not thereby invalid Voyager script, and `drut-lsp` MUST NOT (and does not)
  raise a diagnostic for it.
- **No per-program-box keyword validation** (e.g. it does not know that `RUN
  PGM=MATRIX` specifically takes a `ZONES=` keyword) — spec.md's explicit
  out-of-scope item, deferred to a hypothetical later phase.
- **No dynamic/runtime dictionary updates** — the dictionary is a compile-time
  constant; there is no API to add or override entries at runtime this phase.

## Stability expectations for adapters

Any adapter wanting completion or spell-check candidates (`drut-lsp` today; a
future CLI or MCP tool, if one is ever built) depends on these two entry points
and the `KeywordEntry`/`KeywordRole`/`CompletionContext` shapes in data-model.md
§1 — the same "single source of truth" stability guarantee
`001-voyager-script-parser/contracts/public-api.md` already states for
`tokenize`/`parse`.
