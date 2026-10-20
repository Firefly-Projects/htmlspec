## Why

HTMLSpec currently exists only as a README argument: HTML should be the canonical format for agent-authored specs because the browser is the real review surface. The agent commands, the OpenSpec migration, and the single-file export all rest on decisions nobody has made yet: what markup carries a requirement, what the page looks like, how it serializes byte-for-byte, and what makes it invalid.

This change establishes that foundation. It defines the canonical document, its machine-readable schema, its deterministic serialization, and a validator that rejects unsafe or unparseable specs.

## What Changes

- **Canonical HTML schema.** A normative element contract for the semantic units of a spec: requirement, scenario, decision, task, evidence, risk, and verification. Each has a fixed structure, a stable ID convention, and a `data-change` lifecycle state so agents and Git diffs can address individual units without a second data format.
- **Machine-readable schema source of truth.** The element contract is expressed as JSON Schema, not prose. Rust types are derived from it so the validator, the formatter, and any future generator cannot drift from the spec. A conformance suite of valid and invalid fixtures pins the contract.
- **Document template and pinned offline runtime.** A single page template plus a shared `assets/` directory holding the restrained visual system (flat editorial, serif body, muted color, print styles) and version-pinned Mermaid and Vega-Lite. No network fetches, no remote fonts, no tracking. Core content stays readable with JavaScript disabled; MathML is native.
- **Deterministic formatter.** A canonical serializer producing stable attribute order, predictable indentation, and idempotent output, so rich documents still yield minimal, meaningful diffs. Formatting is a fixed point: formatting twice equals formatting once.
- **Validator.** Structural and safety checks — unique resolvable IDs, cross-references that point at real requirements, well-formed markup, non-executable code examples, valid Vega-Lite specs, and rejection of remote scripts or hidden network dependencies.
- **Rust toolchain packaged with Nix.** One `htmlspec` binary exposing `validate`, `format`, and `init`, built and distributed through a flake so contributors and CI get the same pinned toolchain. ReScript is reserved for later, only if the review surface needs behavior beyond progressive enhancement.

Non-goals for this change, deferred deliberately: the `/htmlspec:*` agent command lifecycle, migration from an existing OpenSpec project, and the portable single-file export. No **BREAKING** changes — the project has no released surface yet.

## Capabilities

### New Capabilities

- `document-schema`: The normative element contract for requirements, scenarios, decisions, tasks, evidence, risks, and verification, including stable ID conventions, `data-change` states, and cross-reference semantics — plus the JSON Schema that expresses it as the machine-readable source of truth.
- `document-template`: The canonical page structure, the restrained visual system, print behavior, and the shared version-pinned offline runtime for diagrams, charts, and equations, including the no-JavaScript readability guarantee.
- `deterministic-formatting`: Canonical serialization rules that make generated HTML stable across runs and agents — attribute ordering, whitespace, element ordering, idempotence, and diff-minimality guarantees.
- `document-validation`: The structural and safety rules a document must satisfy to be accepted, the severity model, and the machine-readable diagnostic output the validator emits.
- `toolchain-packaging`: The `htmlspec` command surface (`validate`, `format`, `init`), its exit-code and output contract, and reproducible Nix flake packaging for development, distribution, and CI.

### Modified Capabilities

None. `openspec/specs/` is empty; this is the project's first change.

## Impact

- **New code.** A Rust workspace: schema types generated from JSON Schema, a canonical HTML serializer, a validator, and a thin CLI. No existing code to modify.
- **New assets.** `htmlspec/assets/` with pinned CSS and vendored Mermaid and Vega-Lite builds, checked into the repository so the artifact works offline and in perpetuity.
- **Dependencies.** Rust crates for HTML parsing and serialization, JSON Schema validation, and CLI argument handling — all pinned. Vendored JavaScript libraries are pinned by version and integrity hash.
- **Build and CI.** A `flake.nix` becomes the entry point for the dev shell, the release build, and CI checks. Contributors need Nix, or a compatible pinned Rust toolchain.
- **Documentation.** The README's "Project status" list narrows: items 1, 2, 4, and 6's shared-asset half are answered here. Items 3 and 5, and the single-file export, remain open and become follow-up changes.
- **Downstream.** This schema is the contract every later change depends on. Getting the element vocabulary and ID conventions wrong is expensive to reverse once documents exist in the wild, so the conformance fixtures matter more than usual.
