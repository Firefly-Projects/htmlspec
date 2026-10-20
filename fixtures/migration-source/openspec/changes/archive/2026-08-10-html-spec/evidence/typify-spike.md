# Evidence: `typify` schema-to-types spike (task 1.3)

**Date:** 2026-08-07
**Pins:** `typify 0.7.0`, `schemars 0.8.22`, Rust `1.90.0`
**Schema under test:** `crates/htmlspec-schema/schema/requirement-spike.schema.json`
**Generated output:** `$OUT_DIR/requirement_spike_schema.rs`, produced by `crates/htmlspec-schema/build.rs`
**Tests pinning every claim below:** `crates/htmlspec-schema/tests/typify_spike.rs`

## Question

The design makes JSON Schema the source of truth and generates Rust types from it. Two things had to be measured before rules are written on top of those types: are the generated types usable, and which schema constraints do not survive generation?

## Result: generation works, and the types are usable

`typify` accepted the whole spike schema — objects, `required`, `enum`, `pattern`, `minLength`, `minItems`, `const`, `additionalProperties: false`, and an internal `$ref` into `definitions`. No schema idiom in the conservative subset blocked generation.

The generated types are usable in all three senses the task asked about.

| Property | Result | Test |
| --- | --- | --- |
| Constructible | Yes. Struct literals work; newtypes are built with `FromStr` / `TryFrom`. | `generated_types_are_constructible` |
| Matchable | Yes. `enum` becomes a real Rust enum with one variant per value. | `generated_enums_are_matchable` |
| Serde round-trip | Yes. `to_string` then `from_str` preserves the value. | `generated_types_round_trip_through_serde` |

Two ergonomic notes. Every constrained string becomes a distinct newtype (`RequirementSpikeId`, `ScenarioTitle`), which is welcome — it is the strong-domain-ID property the base instructions ask for, obtained for free from the schema. Newtypes deref to `String` and hide their field, so construction always runs the validating `FromStr`.

## Constraints that survive generation

- `pattern` — compiled to a `regress::Regex` checked in `FromStr` and in `Deserialize`.
- `minLength` — checked by character count in `FromStr`.
- `enum` of strings — a Rust enum, with `Display`, `FromStr`, and `Default` from the schema's `default`.
- `required` versus optional — required fields are plain, `default` fields get `#[serde(default)]`.
- `additionalProperties: false` — `#[serde(deny_unknown_fields)]`.
- `$ref` into `definitions` — a named type, reused by reference.

## Constraints that do NOT survive generation

Both losses are pinned by tests that assert the loss, so a later `typify` upgrade that closes a gap fails the suite and forces a review rather than passing unnoticed.

1. **`minItems` is dropped.** `"minItems": 1` becomes a plain `Vec<T>`. The generated type deserializes a requirement with an empty `scenarios` array and an empty `statements` array. Test: `loss_min_items_does_not_survive_generation`.

2. **`const` is dropped, and more thoroughly than the design predicted.** The design states that "`const` becomes a plain `String`". My measurement contradicts that: on `typify 0.7.0` a bare `{"const": "requirement"}` with no `"type"` becomes `serde_json::Value`, so the field accepts `42` or an object. Test: `loss_const_does_not_survive_generation`. The design's claim was directionally right and specifically wrong, and the correction matters, because a rule written against `unit` cannot rely on it even being a string.

Neither loss is reachable through the runtime validator, which validates the projected JSON against the schema itself. The gap is only in the generated types.

## Consequence

Generated types are a convenience for writing rule code. The runtime validator is the enforcement layer. Task 1.4 records that decision and the approach it selects.

One follow-on instruction for the vocabulary draft in task 2.1: write discriminators as `{"type": "string", "const": "requirement"}` rather than a bare `const`. That does not restore the constraint, and it does keep the generated field a `String`, which is the weaker of the two failures.
