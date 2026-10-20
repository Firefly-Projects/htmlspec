## 1. Foundation and de-risking spikes

Ordered first because the design's two riskiest assumptions — that `typify` can generate usable types from the contract, and that fatal parse errors are workable in practice — must be settled before rules are written on top of them.

- [x] 1.1 Create the Nix flake with `rust-overlay` and `crane`: pinned Rust toolchain, a `packages.default` building the workspace, a `devShells.default`, and a `checks.default` that runs `cargo test` and `cargo clippy -- -D warnings`. Verify `nix build` and `nix develop` succeed on a clean checkout.
- [x] 1.2 Create the Cargo workspace with the six crates from design — `htmlspec-schema`, `htmlspec-parse`, `htmlspec-format`, `htmlspec-validate`, `htmlspec-assets`, `htmlspec-cli` — each compiling as an empty stub, with a workspace-level `[workspace.dependencies]` block for shared pins. Pin `html5ever` to an exact version, and take `jsonschema` with `default-features = false` so the validator cannot reach the network.
- [x] 1.3 **Spike:** author a minimal JSON Schema covering only the `requirement` unit and generate Rust types from it with `typify` in a build script. Confirm the generated types are usable (constructible, matchable, serde round-trip), and record which schema constraints do not survive generation. Write the outcome to `openspec/changes/html-spec/evidence/typify-spike.md`.
- [x] 1.4 Decide and record the schema-generation approach based on 1.3: generated types, or hand-written types plus an agreement test. If falling back, write the agreement test now so the schema-as-source-of-truth property is preserved by validation. Record that generated types are a convenience and the runtime validator is the enforcement layer.
- [x] 1.5 **Spike:** parse HTML samples with the pinned `html5ever` and enumerate every recovered parse error, including for benign input. Establish this project's own stable diagnostic codes and map the pinned parser messages onto them. Where one message covers two distinct faults, distinguish them by parser state or permit neither. Write the enumeration to `openspec/changes/html-spec/evidence/parse-errors.md` with a justification and a fixture reference per entry.

## 2. Vocabulary and projection draft

The smallest plausible contract, drafted before anything hardens around it. This group produces documents to write against in group 3, not a finished contract.

- [x] 2.1 Draft the smallest plausible vocabulary as JSON Schema: the seven semantic units, required structure, nesting, identifier prefix patterns, and enumerated values for `data-change`, decision status, and task state. Keep to the conservative subset agreed in 1.4. Treat this draft as provisional.
- [x] 2.2 Draft the normative HTML-to-model projection specification: which elements and attributes create a unit, how nesting becomes fields, which content preserves whitespace, how links and reference attributes become references, how figures and captions project, and which presentational markup is discarded.
- [x] 2.3 Wire schema-to-types generation into `htmlspec-schema` per the 1.4 decision, and expose both the schema and the projection specification as published, readable assets rather than build inputs only.

## 3. Dogfood the draft contract

The change's biggest risk is the closed vocabulary, so it gets tested here rather than at the end. Hand-authoring comes before the parser, formatter, and validator harden around the draft.

- [x] 3.1 Hand-author a simple feature proposal as an HTMLSpec document, without tooling.
- [x] 3.2 Hand-author a complex change document exercising cross-references, risks, tasks, and evidence together, using this change's own artifacts as source material.
- [x] 3.3 Hand-author a technical investigation document exercising Mermaid, Vega-Lite, code excerpts, and conclusions, using `README.html` and the foundation-change explainer as source material.
- [x] 3.4 Record every place the draft vocabulary or projection could not represent the content cleanly, at `openspec/changes/html-spec/evidence/vocabulary-dogfood.md`.
- [x] 3.5 Revise the vocabulary and the projection specification from those findings. Changing the contract here is expected and cheap. A schema version makes a later migration identifiable, never inexpensive.
- [x] 3.6 Write the test asserting the schema/Rust split is respected: every schema-expressible rule is expressed in the schema and not duplicated in Rust rule code.
- [x] 3.7 Document the identifier prefix table and change-state semantics in the schema's own descriptions, and cross-reference the projection specification from the schema so neither reads as the whole contract.

## 4. Parse, project, validate

Implements the parseability and rule requirements in `specs/document-validation/spec.md`, against the vocabulary that survived group 3.

- [x] 4.1 Implement HTML parsing in `htmlspec-parse` with the pinned `html5ever` and `kuchikiki`, collecting recovered parse errors rather than discarding them.
- [x] 4.2 Apply the 1.5 enumeration: map each pinned parser message onto this project's stable diagnostic code, and treat any recovery outside the enumeration as fatal, short-circuiting further analysis and reporting that evaluation was curtailed. Add a test that fails when the parser version changes without review of the enumeration.
- [x] 4.3 Implement the document model projection exactly as the projection specification states, and the projection into the JSON shape the schema validates against.
- [x] 4.4 Implement document metadata extraction: schema version, subject, and lifecycle status.
- [x] 4.5 Add mapping fixtures pairing HTML inputs with expected projected models, covering every rule in the projection specification.
- [x] 4.6 Build the rule engine: namespaced stable rule identifiers (`schema/*`, `refs/*`, `safety/*`, `viz/*`, `a11y/*`, `review/*`), a severity model where `safety/*` cannot be downgraded, single-pass accumulation of all diagnostics, and a strict mode promoting warnings to errors.
- [x] 4.7 Implement `schema/*`: validate the projected document against the JSON Schema with the `jsonschema` crate and map its errors onto document locations and identifiers. The runtime validator is the enforcement layer, so rules that vanish in type generation — `minItems`, `const` — must be exercised by fixtures here.
- [x] 4.8 Implement the relational rules Rust owns: document-wide identifier uniqueness, identifier prefix matching unit type, requirements having at least one scenario, scenarios not appearing outside requirements, decisions marked superseded naming a successor, dropped tasks stating a reason.
- [x] 4.9 Implement `refs/*`: every fragment link and every unit cross-reference resolves, and no reference targets a requirement marked `removed`.
- [x] 4.10 Implement `review/*`: report a dependent unit as needing review when the unit it references changes materially. These are warnings that never gate work, and reconciliation clears them.
- [x] 4.11 Implement `safety/*` network rules: reject remote scripts, stylesheets, fonts, images, and iframes; reject prefetch, preconnect, and beacon-style constructs; reject remote data URLs in chart specifications with a rule of our own, because Vega-Lite permits them. Accept relative references into the project's shared assets.
- [x] 4.12 Implement `safety/*` executable-content rules: a code element's children contain text and comment nodes only, with no element descendant, and any executable script not belonging to the pinned runtime is rejected. Add fixtures for an empty example and for text split by a comment, both of which must pass.
- [x] 4.13 Implement `a11y/*`: semantic content must be present in the served markup, and every figure must carry a caption plus visible source or textual description.
- [x] 4.14 Implement `viz/*`: preprocess Vega-Lite's vendored schema so its definition names load, then validate embedded specifications against it. For Mermaid, validate non-emptiness and a recognized leading diagram keyword, and emit the shallow-validation limitation as a documented diagnostic note.
- [x] 4.15 Implement diagnostics output in both human-readable and structured machine-readable forms, each carrying rule identifier, severity, location, and offending identifier where applicable.

## 5. Canonical formatter

Implements `specs/deterministic-formatting/spec.md`.

- [x] 5.1 Implement the canonical serializer in `htmlspec-format`: fixed attribute order, prescribed quoting, one escaping form per character, deterministic block indentation and line breaking. Text is never rewrapped.
- [x] 5.2 Preserve whitespace-sensitive content (`pre`, `code`, `textarea`) byte-for-byte, including internal indentation.
- [x] 5.3 Enforce file-level normalization: UTF-8, single prescribed line ending, exactly one trailing newline, no trailing whitespace.
- [x] 5.4 Implement check mode — determine whether a document is canonical without writing to it.
- [x] 5.5 Define behavior for formatting an invalid document: canonicalize without altering meaning, or refuse with a reason. Never silently repair.
- [x] 5.6 Add a `proptest` generator over the schema vocabulary and assert `format(format(x)) == format(x)` plus semantic preservation (units, identifiers, order, attribute values all survive).
- [x] 5.7 Add no-churn tests: editing one unit, and inserting a unit between two others, each leave every line outside the affected unit untouched by the formatter.
- [x] 5.8 Measure diff minimality as a separate property from formatter churn, and record the result at `openspec/changes/html-spec/evidence/diff-locality.md`. Preserved line breaks mean a one-line paragraph still diffs whole, and that cost belongs to the line structure.

## 6. Template, visual system, and runtime

Implements `specs/document-template/spec.md`.

- [x] 6.1 Vendor and commit the pinned visualization libraries — Mermaid and Vega-Lite (with Vega and Vega-Embed as required) — plus Vega-Lite's JSON Schema. Resolve the exact versions here (design Open Questions).
- [x] 6.2 Create the asset lockfile recording each vendored library's version, source URL, and content hash, and a Nix-driven check that re-fetches from the pinned source and verifies the committed bytes match.
- [x] 6.3 Write the shared stylesheet: flat editorial layout, serif body text, high density, muted color, narrow-width reflow, and change states distinguishable without relying on hue.
- [x] 6.4 Write the print stylesheet: navigation chrome and interactive controls omitted, no clipped content or diagrams, external references identifiable on paper. Settle the typographic specifics here (design Open Questions).
- [x] 6.5 Write the document template: declared schema version, subject, and visible lifecycle status; stable section ordering; navigation that resolves without JavaScript.
- [x] 6.6 Write the progressive-enhancement script in vanilla JavaScript — render figures, collapse their sources, wire navigation state — with the figure's caption-plus-visible-source markup as the default state rather than a `<noscript>` branch. Set and enforce the byte budget in the check suite (design Open Questions).
- [x] 6.7 Render every figure into a sibling element and never over its source. A renderer that replaces the source block destroys it whenever a rendered DOM is saved, so the source must survive a save of the live page. Add a test that saves a rendered document and asserts the diagram source is still present.
- [x] 6.8 Embed the template, stylesheets, script, and vendored assets in `htmlspec-assets` for use by `init`.
- [x] 6.9 Verify the offline and no-JavaScript guarantees by hand in a real browser with networking disabled and scripting disabled, and record the results in `openspec/changes/html-spec/evidence/browser-verification.md`.

## 7. CLI

Implements `specs/toolchain-packaging/spec.md`.

- [x] 7.1 Implement the `htmlspec` binary with `clap`: `validate`, `format`, and `init` subcommands, discoverable help covering every subcommand and option, and a clear error listing available subcommands on an unknown one.
- [x] 7.2 Implement `validate`: whole-project by default, single-file when given a path, with a strict flag.
- [x] 7.3 Implement `format` with default write mode and a non-writing check mode.
- [x] 7.4 Implement `init`: create a conforming document that validates immediately, establishing the shared assets when they are not already present.
- [x] 7.5 Implement the exit-code contract: zero on success, a designated code for document violations, and a distinct code for operational failures such as an unreadable file or invalid invocation.
- [x] 7.6 Implement output routing: structured payload alone on stdout when structured output is requested, all progress and run-level messages on stderr; human-readable terminal output by default.
- [x] 7.7 Implement version reporting that identifies the revision the binary was built from.
- [x] 7.8 Add CLI integration tests covering each exit code path and asserting that structured stdout parses cleanly with progress output present.

## 8. Conformance suite and checks

Implements the conformance requirements in `specs/document-schema/spec.md`.

- [x] 8.1 Build the conformance fixture harness at the workspace root, driving fixtures through the crates' public APIs only, so the suite tests the contract rather than internals.
- [x] 8.2 Author valid and invalid fixtures for every rule. Each invalid fixture declares the rule identifier it violates, and the harness asserts that exact rule is reported.
- [x] 8.3 Add the coverage test that fails when any rule lacks both a valid and an invalid fixture, and when any projection rule lacks a mapping fixture.
- [x] 8.4 Run every fixture through the formatter twice and assert byte-identical output.
- [x] 8.5 Expose the full suite — conformance, unit tests, formatting check, asset lockfile verification, enhancement-script byte budget — as one command, and make the flake check and CI invoke that same command rather than a separate step list.
- [x] 8.6 Package the binary as a standalone artifact and verify it runs on a machine with no source checkout and no Nix at runtime.

## 9. Final self-conformance

Group 3 tested the vocabulary by hand before the tooling hardened. This group tests the finished toolchain end to end.

- [x] 9.1 Convert a real specification — one of this change's own capability specs — into a conforming HTMLSpec document using `init` and the toolchain.
- [x] 9.2 Re-run the three group 3 documents through the finished toolchain, and confirm the contract still represents them after every revision made since.
- [x] 9.3 Review those documents in a browser and fix what the contract gets wrong: missing vocabulary, rules that are too strict, diagnostics that fail to explain themselves. Record findings and resulting contract changes in `openspec/changes/html-spec/evidence/self-conformance.md`.
- [x] 9.4 Update the README: narrow the "Project status" list to the remaining items (agent command lifecycle, OpenSpec migration, single-file export) and document how to build and run the toolchain.
- [x] 9.5 Run `openspec validate html-spec --strict` and check that the change's own artifacts are coherent before archiving.
