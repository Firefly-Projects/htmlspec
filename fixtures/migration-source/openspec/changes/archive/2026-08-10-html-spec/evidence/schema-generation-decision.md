# Decision: schema-generation approach (task 1.4)

**Date:** 2026-08-07
**Input:** `typify-spike.md` (task 1.3)
**Decision:** Keep generated types. Do not fall back to hand-written types.

## Why

The spike answered the blocking question directly. `typify 0.7.0` generated usable types from every schema idiom in the conservative subset, so the fallback in the design's risk register — hand-written Rust types plus an agreement test — is not needed. Generation is retained, and `crates/htmlspec-schema/build.rs` is the only place that turns the schema into types.

## What generated types are, and are not

Generated types are a **convenience for writing rule code**. They give rule authors named fields, real enums, and strong newtypes for identifiers, all derived from the authored schema, so rule code cannot invent a field the contract does not have.

The **runtime validator is the enforcement layer**. Every document is validated against the JSON Schema itself, with the `jsonschema` crate, before or independently of any conversion into the generated types. This is not belt-and-braces. It is required, because generation is lossy in ways the spike measured: `minItems` becomes a plain `Vec`, and a bare `const` becomes `serde_json::Value`. A document that only satisfied the generated types would satisfy strictly less than the contract.

Two rules follow, and they are enforced by tests rather than by memory:

1. No code path may accept a document on the strength of successful deserialization into a generated type alone.
2. No rule may restate in Rust a constraint the schema already expresses. Task 3.6 writes the test for this split.

## Fallback status

The fallback is not implemented, and the agreement test the task offered as its companion is not written, because there are no hand-written types for it to compare against. The schema-as-source-of-truth property is preserved **by construction** here rather than by verification: the types are a build output of the schema, so they cannot disagree with it about the vocabulary.

If a later contract change needs an idiom `typify` cannot generate, the fallback is still available and its cost is understood. The condition for taking it is a generation failure in `build.rs`, not inconvenience.

## Verification

- `cargo test -p htmlspec-schema` — 8 spike tests pass, including the two that pin the measured constraint losses.
- The loss-pinning tests fail if a `typify` upgrade closes a gap, which forces this decision to be re-read rather than silently outgrown.
