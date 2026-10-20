## Context

See `proposal.md` — Why. The repository is greenfield: a README stating the premise, and nothing else. There are no existing documents, users, or compatibility obligations, so this design is free to choose strictness now rather than negotiate it later.

Three constraints shape everything below.

**The browser is the consumer.** A document must be correct as the browser sees it, not as some stricter grammar wishes it were. That rules out validating a private dialect and hoping the two agree.

**Agents write, humans review.** The format can be more verbose and more explicit than a human would tolerate typing, but the diffs a human reads must stay small. Verbosity is cheap; diff noise is not.

**Nothing may depend on the network.** Documents must render offline in perpetuity, which forces every runtime library to be vendored and pinned, and forces validation to reject anything that reaches outward.

Requirements live in `specs/`. This document explains how they are met.

## Goals / Non-Goals

**Goals:**

- One machine-readable contract that tooling derives from, so the published schema and the validator cannot drift.
- A formatter whose output is a genuine fixed point, verified by property tests rather than asserted.
- Validation strict enough that a document accepted today parses identically in every consumer, including ones not written yet.
- A build that a contributor can reproduce on a clean machine, and that CI runs by invoking the same command a contributor would.

**Non-Goals:**

- No plugin or extension mechanism. The element vocabulary is closed in this change; opening it is a later decision with its own compatibility cost.
- No configuration surface. Behavior is determined by the schema and fixed rules. Every knob added now becomes a compatibility obligation before anyone has asked for one.
- No incremental or watch mode, no language server, no editor integration. Documents are small and the operation is fast.
- No HTML beautifier for arbitrary input. The formatter canonicalizes documents that conform to the contract; it is not a general-purpose tidy tool.

## Decisions

### Rust for the whole toolchain; no ReScript in this change

One language, one build, one binary. The toolchain is a parser, a serializer, and a rule engine over a tree — squarely Rust's strengths, and it produces a standalone artifact with no runtime to install, which the packaging requirements need.

ReScript was considered for the browser-side progressive enhancement. It is not used here. The enhancement layer in this change is small: reveal diagram sources, wire up navigation state, invoke the two vendored rendering libraries. Introducing a second language and its build step for that amount of code contradicts the project's own principle that HTML is the artifact and not the application, and it would put a compile step between an agent's edit and the reviewable page. The enhancement is hand-written vanilla JavaScript held under an explicit size budget enforced by the check suite. If a later change grows real interactive review controls, ReScript becomes the right answer then, and this decision should be revisited rather than defended.

### Parse with the browser's grammar, then treat parser errors as fatal

Documents are parsed with `html5ever`, the same HTML5 tree construction algorithm browsers implement, using a DOM with selector support (`kuchikiki`) over the tree.

The tension is that HTML5 parsing never fails — it recovers. That directly conflicts with the requirement to reject markup whose interpretation depends on error recovery. The resolution is to keep the browser's grammar but discard its forgiveness: `html5ever` reports every parse error it recovers from, and validation treats any such report as a fatal diagnostic. Documents are therefore parsed exactly as a browser would, but only accepted when the browser needed no guesswork.

The alternative was requiring XML well-formedness — parse with an XML parser, serve as polyglot XHTML. Rejected: it validates a grammar no consumer actually uses. A document could be well-formed XML and still be interpreted differently by a browser, which is precisely the failure the requirement exists to prevent.

The weak point is the permitted-recovery list, and it needs more discipline than "collect what benign documents produce". `html5ever` reports a recovery as free text through `TreeSink::parse_error`, not as a typed code, so a list keyed on message wording breaks when the crate rewords a message. Worse, two distinct faults share the string `Unexpected token` on the pinned version, so message text alone cannot distinguish a missing doctype from a stray end tag. The discipline is therefore: zero recovered errors by default, an exact pinned parser version, one fixture per permitted recovery, stable diagnostic codes owned by this project and mapped from the pinned messages, and deliberate review of the enumeration whenever the parser version changes. Where a message covers two faults, the enumeration distinguishes them by parser state or permits neither.

This choice also delivers code-example inertness almost for free. An example that escapes its container does not produce suspicious text — it produces live elements in the tree. The rule is therefore structural: a code element's children contain text and comment nodes only, with no element descendant. The security property is the absence of elements, not a child count. A prototype confirmed why the distinction matters: the safe and dangerous cases both have exactly one child, and "exactly one text node" additionally false-rejects an empty example and text a parser split around a comment. The parser answers the question, so no pattern matching against dangerous strings is involved, and the rule cannot be defeated by an encoding trick a regular expression would resolve differently.

### JSON Schema is the source of truth; Rust types are generated from it

The element contract is authored as JSON Schema and checked into the repository as the published artifact. Rust types are generated from it at build time with `typify`. Validation projects the parsed HTML into a JSON document model and validates that projection against the schema with the `jsonschema` crate.

Generation runs schema-to-types, not types-to-schema. The reverse direction (`schemars`, deriving the schema from Rust structs) is more idiomatic and less friction, but it makes the published contract a build output of this project's source — an external consumer would be reading our implementation's shadow. The requirement is that third parties can determine the vocabulary from the schema alone, so the schema has to be the thing that is authored.

The honest limit: JSON Schema expresses shape well and relations poorly. It handles the vocabulary, required structure, enumerated attribute values, and identifier patterns. It cannot express that identifiers are unique document-wide, that an identifier's prefix must match its element's unit type, that references resolve, or that a reference must not target a removed requirement. Those are relational rules implemented in Rust against the document model. The split is deliberate and drawn on a clear line — the schema owns the vocabulary and shape of a unit, Rust owns relationships between units — so that "which layer is this rule in?" always has an obvious answer. A test asserts every schema-expressible rule is in fact expressed in the schema and not duplicated in Rust.

Two further limits, both measured on a prototype rather than assumed. First, `typify` silently drops constraints Rust cannot express: `minItems: 1` becomes a plain `Vec` and `const` becomes a plain `String`, so generated types accept an empty scenario list and a wrong class name that the runtime validator rejects. Generated types are therefore a convenience for writing rule code, never the enforcement layer, and every document must pass the runtime validator. Second, `typify`'s own documentation notes that some legal schemas do not map cleanly into Rust, which is why the generation spike runs before anything depends on the types.

### The schema is not the whole public contract

JSON Schema describes the *projected model* — the list of semantic units, their fields, and their references. It says nothing about the HTML that produces that model. An outside implementation reading only the schema would not know which elements create a unit, how nesting becomes fields, which whitespace survives, how a link becomes a reference, or which presentational markup to discard.

The public contract is therefore three pieces, not one: the JSON Schema for the projected model, a normative HTML-to-model projection specification, and conformance fixtures pinning the mapping. My earlier claim that a third party could derive the vocabulary "from the schema alone" was too strong, and the projection specification is what makes it true.

### Dependencies mark work for review, they do not gate it

Artifacts depend on each other, and an edit upstream does make dependents suspect. That is not a phase gate. OpenSpec's model is fluid rather than waterfall, and it explicitly supports editing artifacts during implementation and continuing.

So an upstream edit marks dependent units as needing review, validation reports stale or inconsistent references, and an agent reconciles the affected units. Nothing requires wholesale regeneration and nothing blocks work in the meantime. These are `review/*` warnings that reconciliation clears, kept deliberately separate from the `refs/*` errors for references that do not resolve at all.

Thrift and Protobuf were considered and rejected: both describe wire messages for programs, and the artifact here is a schema meant to be read by humans and third-party tools, where JSON Schema is the lingua franca and where Vega-Lite already publishes its own contract in that form.

### The formatter never rewraps text

The canonical serializer normalizes the indentation and line breaking of block structure, fixes attribute order and quoting, and prescribes one escaping form per character. It does not reflow prose. Author line breaks inside text-bearing content are preserved exactly.

This is the load-bearing decision of the formatting work. General HTML formatters that rewrap text are notoriously difficult to make idempotent, because inline whitespace is semantic and every wrap decision interacts with the next. Refusing to rewrap makes the fixed-point property nearly structural.

What it also delivers is the absence of formatter-induced churn, which is narrower than diff minimality. Editing a sentence cannot make the formatter rewrite the surrounding paragraph, because the formatter has no opinion about where that paragraph's lines break. It does not follow that the Git diff is small: a paragraph held on one long line diffs as a whole line when one sentence changes, and that cost belongs to the line structure rather than to the formatter. A prototype measured the difference. On a four-sentence paragraph with one word inserted near the start, a rewrapping formatter produced ten changed diff lines against two — but only for an edit away from the end of a block, where a rewrapper's cascade has room to show. Diff minimality is therefore measured as a separate property, and the specs no longer claim the formatter provides it.

The cost is that prose line breaks are the author's problem, and agents may produce very long lines. That is acceptable — no requirement constrains line length, and a long line still diffs as one line.

Whitespace-sensitive elements (`pre`, `code`, `textarea`) are preserved byte-for-byte inside their content, including indentation, which the specs require for code excerpts.

Idempotence is verified two ways: every conformance fixture is formatted twice and compared byte-for-byte, and a `proptest` generator produces documents from the schema's vocabulary and asserts `format(format(x)) == format(x)`.

`prettier` was the obvious alternative and is not available: it is JavaScript, and taking a Node dependency for the core operation of a Rust toolchain would compromise the reproducible single-artifact packaging.

### Validation projects to a document model, and rules carry stable namespaced identifiers

Parsing produces a document model — the list of semantic units, their attributes, their nesting, and their references — and every rule operates on that model rather than on the raw tree. This keeps rules independent of markup incidentals and lets diagnostics point at units by identifier rather than by byte offset alone.

Every rule has a stable identifier namespaced by concern: `schema/*` for contract conformance, `refs/*` for reference resolution, `safety/*` for network and executable content, `viz/*` for visualization specifications, `a11y/*` for the no-JavaScript and non-color-dependence rules. The namespace makes severity policy legible: `safety/*` rules are errors that cannot be downgraded, which the specs require.

All rules run in one pass and diagnostics accumulate, with one exception: a fatal parse failure short-circuits and reports that analysis was curtailed, because there is no tree to reason about.

### Visualization sources are visible content, not inert script tags

The conventional way to embed a chart specification is a `<script type="application/json">` block. That is rejected here: such a block is invisible without JavaScript, so a document using it would fail the no-JavaScript readability rule while appearing to satisfy it.

Instead, a figure carries its source as ordinary visible content — a `<pre>` inside a `<details>` — plus a mandatory `<figcaption>`. With scripting enabled, the enhancement layer renders the figure and collapses the source. With scripting disabled, the reader sees a caption and can expand the source. The fallback is the default state of the markup rather than a `<noscript>` branch, so there is only one thing to get right.

The renderer must draw into a sibling element and never over the source. Mermaid's default behavior replaces the source block with rendered SVG, which destroys the source the moment anyone saves a rendered DOM — and a saved page is exactly how these artifacts get shared. I confirmed the failure on my own output: the file on disk kept its diagram source, and a copy saved from a live page did not. Sibling rendering makes the source survive that round trip, and a test asserts it.

Vega-Lite specifications are validated against Vega-Lite's own published JSON Schema, vendored and pinned, using the same `jsonschema` crate — the visualization rule is real validation, not a syntax check.

Mermaid is a gap and worth stating plainly: there is no Rust implementation of its grammar, so diagram source cannot be fully validated without running Mermaid in a browser engine. This change validates that the block is non-empty and opens with a recognized diagram keyword. Full validation is deferred; see Risks.

### Vendored assets are committed, with a lockfile Nix verifies

The pinned browser-side libraries are committed to the repository, alongside an asset lockfile recording each library's version, source URL, and content hash.

Committing them means a plain `git clone` yields documents that render offline immediately, with no build step and no Nix — which matters because the whole premise is that a reviewer opens a file in a browser. The obvious objection is that committing third-party build output is unpleasant. It is, and the lockfile is the answer: a Nix-driven check re-fetches each library from its pinned source and verifies the committed bytes match the recorded hash, so the committed copy is auditable rather than trusted.

### Nix flake with `crane` and `rust-overlay`

The flake pins the Rust toolchain via `rust-overlay` and builds with `crane`, which caches dependency compilation separately from the workspace and is actively maintained. It exposes a package (the `htmlspec` binary), a development shell, and a check that runs the full suite.

CI invokes the same flake check a contributor runs locally. This is the point of using Nix here rather than a hand-rolled CI script: the requirement is that CI and local runs verify the same thing, and duplicated step lists drift.

`naersk` was considered; `crane` has better incremental caching behavior and more active maintenance.

### Workspace layout

Five library crates and a binary, split along the seams the specs already draw:

- `htmlspec-schema` — the JSON Schema asset and the types generated from it
- `htmlspec-parse` — HTML to document model, including fatal parse-error handling
- `htmlspec-format` — the canonical serializer
- `htmlspec-validate` — the rule engine and diagnostics
- `htmlspec-assets` — the template, stylesheet, enhancement script, and vendored libraries, embedded for `init`
- `htmlspec-cli` — argument parsing, output formatting, exit codes

Conformance fixtures live at the workspace root and are driven by an integration test that runs against the public APIs, so the suite tests the contract rather than internals and could be pointed at an independent implementation.

## Risks / Trade-offs

**`typify` may not handle the schema idioms the contract needs** → Keep the schema to a conservative subset: objects, enums, patterns, required fields, no exotic composition. If generation blocks on something the contract genuinely needs, fall back to hand-written Rust types plus a test asserting the types and the schema agree, preserving the schema-as-source-of-truth property by verification instead of by construction. Decide this early — task ordering puts the generation spike first, before rules are written on top of the types.

**Fatal parse errors may prove too strict in practice** → `html5ever` reports errors for benign-looking input, and an over-strict validator that rejects reasonable documents will be worked around rather than fixed. Mitigation: triage the full error list during implementation and maintain a documented allowlist of error codes that are recovery-free in every consumer, each with a fixture justifying its presence. The allowlist is part of the contract, not a private escape hatch.

**Mermaid diagrams are only shallowly validated** → An invalid diagram reaches the browser and fails there, which is exactly the silent failure the visualization requirement exists to prevent. Partially mitigated by the visible-source fallback: a broken diagram still shows its caption and source rather than an empty region, so the document remains reviewable. Full validation needs a headless browser check, which is deliberately out of scope; the gap is recorded in the diagnostics as a known limitation rather than papered over.

**The closed vocabulary will prove incomplete** → Seven unit types will not cover everything, and this is the change's largest risk. The schema version declared in every document makes a later migration *identifiable*, which is not the same as making it inexpensive or automatic — my earlier framing overstated what a version buys. The real mitigation is order: group 3 hand-authors three representative documents against the draft vocabulary before the parser, formatter, validator, template, and CLI harden around it. Revising the contract there costs a schema edit. Revising it after group 7 costs a migration. An extension mechanism is still deliberately out of scope, because one designed before the first real document would be designed blind.

**Committed vendored assets bloat the repository and invite staleness** → Held down by the lockfile verification check and by the small approved library set. Repository size grows once per library version, not per document, because the runtime is shared.

**The change is large for one unit of work** → Five capabilities, a new language toolchain, and a packaging story. The sequencing risk is real: rules written before the schema-generation approach is settled would need rewriting. Mitigated by task ordering — the two spikes prove schema generation and the parse strategy, then the draft vocabulary and projection, then hand-authored documents that test them, then parse and validate, then formatting and the template, then the CLI and packaging, then final self-conformance.

**Strictness with no users to complain** → Every rule here is easy to add now and hard to remove later, and there is no real-world document corpus to check the rules against. Mitigated in two places rather than one: group 3 tests the vocabulary against hand-authored documents while changing it is still cheap, and group 9 tests the finished toolchain end to end. The expensive discovery moves to the point where changing direction costs least.

**Vega-Lite's own schema does not load unmodified** → Its `$ref` values contain `<`, `>`, and `|` from TypeScript type names, which are illegal in a URI reference, so draft-07 metaschema validation rejects the schema before any chart is validated. No `jsonschema` option bypasses this, because metaschema validation runs before user options apply. Mitigated by a preprocessing step that slugifies definition names and the refs pointing at them. On the vendored `vega-lite` 6.4.3 the affected count is 126 rather than the 294 measured while writing this design, and the preprocessed schema compiles in 31 ms rather than the estimated 115 ms; the defect is the same and the fix is unchanged. `evidence/vega-lite-schema.md` records the measurement, and a test asserts the *failure* still occurs, so a future Vega-Lite that fixes its own schema forces the preprocessing to be reconsidered rather than carried forever. Separately, Vega-Lite permits remote data URLs by design, so the offline rule needs a rule of our own rather than relying on schema validation to catch it.

## Migration Plan

Nothing to migrate. There are no existing documents, no released artifact, and no consumers. The README's "Project status" list narrows on completion; items 3 and 5 and the single-file export become follow-up changes.

Rollback is deleting the directory.

The final task dogfoods the toolchain by rendering a real specification as a conforming document. That is the acceptance signal for this change: the contract is credible once it has survived contact with content it did not anticipate.

## Open Questions

These can be answered during implementation without changing the specs, the approach, or the task breakdown.

- ~~The exact pinned versions of the two vendored visualization libraries.~~ **Answered:** `mermaid` 11.16.1, `vega-lite` 6.4.3, with `vega` 6.3.1 and `vega-embed` 7.1.0 as the libraries Vega-Lite requires to render. Each is recorded in `assets/vendor.lock.json` with its source URL, archive hash, and extracted-file hash, verified offline by `cargo test -p htmlspec-assets` and against the pinned upstream by the `assetProvenance` flake check.
- ~~Whether the schema version is a date or a semantic version.~~ **Answered: semantic.** A date orders releases and says nothing about whether a document written against an earlier release still validates, which is the only question a version needs to answer. The contract starts at `0.1.0`, because a `0.x` major states that the vocabulary is provisional — groups 4 through 9 still revise it, and revising it there costs a schema edit rather than a migration. `1.0.0` is the release that commits the vocabulary. The match stays exact and stays a schema `const`, so a wrong version fails schema validation with no Rust rule involved. A compatibility range is a decision worth making once a second version exists. `crates/htmlspec-schema/CHANGELOG.md` records the date each version landed and is embedded in the published contract, so the calendar is available without putting a date inside the version.
- The typographic specifics of the print stylesheet — page margins, whether link targets appear as footnotes or inline parentheticals.
- The exact byte budget for the enhancement script. It needs to be small and enforced; the number can be set once the script exists.
