# The Vega-Lite schema does not load unmodified

**Task:** 4.14 (`viz/*` rules), with the vendoring from 6.1 and 6.2 pulled forward
because the rule cannot be written without the schema.

**Measured on:** 2026-08-08, `vega-lite` 6.4.3, `jsonschema` 0.49.6.

## What the design predicted, and what is actually true

The design's Risks section states that "294 of its `$ref` values contain `<`, `>`,
and `|` from TypeScript type names". On the version vendored here the count is
**126**, not 294 — the same defect at a different magnitude, presumably because
the design measured a Vega-Lite 5 release and the pinned version is 6.4.3. The
shape of the problem, and the fix, are unchanged.

Everything else the design predicted holds exactly:

- The schema declares draft-07 and has 458 definitions.
- 126 definition names, and every `$ref` pointing at one, embed characters that
  are illegal in a URI reference.
- Compilation fails before any chart is validated, with
  `"#/definitions/MarkPropDef<(Gradient|string|null)>" is not a "uri-reference"`.
- No `jsonschema` option bypasses it, because metaschema validation runs before
  user options apply.

## The fix, and its cost

Slugify every definition name and every local `$ref` with the same function, then
compile. Measured on a release build:

| Step | Time |
| --- | --- |
| Preprocess (rename 126 definitions and rewrite their refs) | 3.0 ms |
| Compile the preprocessed schema | 31.0 ms |

The design estimated roughly 115 ms to compile; 31 ms is the measurement on this
version and this machine. Both are one-off per process — the compiled validator is
cached in a `OnceLock` — so the cost does not scale with the number of charts.

After preprocessing, a valid chart is accepted and a chart with an unknown mark is
rejected, so the rule is real validation rather than a syntax check.

## Why the rename is safe

It is a pure rename: `slugify` is applied to the definition names and to the refs
pointing at them, so the schema means exactly what it meant.

Two properties are asserted rather than assumed, in
`crates/htmlspec-validate/src/rules/viz.rs`:

- `the_vendored_schema_is_rewritten_consistently` checks that the definition count
  is unchanged (no two names collapsed into one) and that no `$ref` is left
  dangling.
- `the_vendored_schema_does_not_compile_unmodified` asserts the *failure* still
  happens. If a future Vega-Lite fixes its own schema, that test fails and tells
  us to reconsider the preprocessing rather than carrying it forever.

## The other half: Vega-Lite permits remote data by design

Confirmed. A specification with `{"data": {"url": "https://..."}}` is valid
Vega-Lite, so schema validation reports it as correct. This project rejects it
with a rule of its own, `safety/remote-chart-data`, which searches the whole
specification rather than just the top level — `data` also appears inside `layer`,
`hconcat`, and a lookup transform.

`$schema` is deliberately not treated as a data URL: it is a remote *identifier*
and the vendored copy is what validates against it. Rejecting it would make every
conforming chart invalid.

## Versions vendored

Resolved to what was current when the assets task ran, closing that design Open
Question:

| Library | Version |
| --- | --- |
| mermaid | 11.16.1 |
| vega | 6.3.1 |
| vega-lite | 6.4.3 |
| vega-embed | 7.1.0 |

`assets/vendor.lock.json` records each one's source URL, archive hash, and file
hash. Two checks verify them: `cargo test -p htmlspec-assets` compares the
committed bytes against the recorded hashes offline, and
`nix build .#checks.<system>.assetProvenance` re-fetches each archive from its
pinned URL, extracts the recorded member, and compares that. The second is what
proves the committed bytes came from where the lockfile says; it ran clean for all
five files.
