## Context

See proposal.md — Why for the motivation. Requirements are in `specs/artifact-kinds/spec.md`, `specs/document-template/spec.md`, `specs/document-validation/spec.md`, and `specs/toolchain-packaging/spec.md`.

The constraints that shape the approach are existing project facts, and three of them are load-bearing:

- **`status` is projected, not merely declared.** `crates/htmlspec-parse/src/project.rs:100` reads `<meta name="htmlspec-status">` into `Model.status`, and `crates/htmlspec-schema/doc/projection.md` section 9.3 makes that projection normative: "An absent status is invalid. A status outside the schema's enumeration is invalid." So replacing the field is a change to the projection specification, the JSON Schema, the Rust model, and the projection code — not a metadata rename.
- **The projection specification is a fourth public artifact.** The archived foundation design settled that "the public contract is therefore three pieces, not one: the JSON Schema for the projected model, a normative HTML-to-model projection specification, and conformance fixtures pinning the mapping." A new metadata field has to appear in all three or the contract is incomplete.
- **The schema/Rust split is enforced by a test.** `rule_code_does_not_restate_schema_expressible_values` in `crates/htmlspec-validate/tests/schema_rust_split.rs` fails if rule code restates a value the schema could express. The kind vocabulary is an enumeration, so it belongs in the schema, and the three downstream rules are relational, so they belong in Rust. That line is drawn for us.
- **Sixty-nine of the project's 82 HTML documents declare a status**, and they fall into four groups that differ in kind rather than only in size. 41 conformance fixtures, which a generator writes. **23 projection fixtures, which are hand-written and whose `expected.json` holds the projected model itself** — so the field name appears in committed test data 23 times, and `fixtures/projection/metadata` exists to pin section 9.3, the clause defining the field being removed. 3 dogfood documents, which are authored prose. The template, and one archived document. The 10 parse-error fixtures declare no status, because malformed markup never reaches the metadata rules, and the 3 root HTML files are not html-spec documents at all.
- **25 Rust source files carry a status literal in an inline test document** — 12 in `htmlspec-validate`, 6 in `htmlspec-format`, 3 in `htmlspec-parse`, 2 each in `htmlspec-cli` and `htmlspec-assets`. The compiler finds none of them: a stale literal fails at run time rather than at build time.
- **Only one existing rule takes a task as its subject.** `schema/missing-reason` reads `task.state == dropped`, and no other unit has a state. I checked the others by building documents and running the binary: `refs/unresolved`, `refs/removed-target`, `review/stale-reference` all fire from a decision, `review/stale-verification` fires from a verification, and `schema/prefix-mismatch` fires on any unit. My earlier claim that six rules needed a task was wrong, and it matters because it decides how many fixtures must become `tasks` documents.
- **`scripts/check` is the single definition of a healthy revision.** Any new rule has to pass it, and the migration has to land in one commit that keeps it green — unlike the previous change, there is no deliberate red phase here.

## Goals / Non-Goals

**Goals:**

- A declared kind that carries the document's role, replacing `status`, expressed once in the schema and derived everywhere else.
- Exactly the three enforced rules the proposal states, and demonstrably no others.
- Unconstrained proposal and design kinds, with that freedom pinned by fixtures rather than asserted in prose.
- An archival operation whose synthesis is agent-driven and whose checking is code.
- A migration that leaves `scripts/check` green in the same commit.

**Non-Goals:**

- The `/htmlspec:*` command lifecycle beyond archival.
- The Markdown-to-HTML converter.
- Any change to the seven units' own definitions or to `document-schema`'s unit shapes.
- A parser that reads a proposal or a design and emits requirements. This is a non-goal in the strong sense: the specs forbid it.

## Decisions

### Replace `status` rather than add `kind` beside it

Both fields could coexist — `kind` for role, `status` for stage — and that is the more conservative move. It is rejected because the two would immediately disagree. A document's kind is fixed for its life and its change's stage is not, so `status` would drift from reality on every document that outlives one stage, and nothing reads it to notice. The field records the wrong subject: a stage describes a change as it moves, and a change's stage is derivable from which of its documents exist.

Keeping a field that no rule reads, that describes something other than its own document, and that a reader would reasonably trust, is worse than removing it. Removal is breaking and the breakage is mechanical.

Alternative considered: keep `status` and deprecate it silently. Rejected because a required field cannot be deprecated without either making it optional — which changes the schema anyway — or leaving every document carrying a value nothing believes.

### The kind vocabulary lives in the schema; the three rules live in Rust

`kind` is a closed enumeration of four values on a required property, which is exactly what JSON Schema expresses well, and `rule_code_does_not_restate_schema_expressible_values` would fail a Rust `const` listing the same four strings. So the schema owns the vocabulary and rule code reads it through `schema_facts`, the way `refs.rs` already reads the `changeState` enumeration to find the removed state.

The three enforced rules are relational — a unit's kind against its document's kind, a count of tasks, a requirement against the verifications naming it — and JSON Schema cannot express any of them. They go in Rust. This is the same split the foundation change drew, and drawing it again here costs nothing because the line was already argued.

### One rule module, and the placement check reads the model rather than the tree

The three rules share one question — what kind is this document? — so they belong in one module, `rules/kinds.rs`, in a new `kind/*` namespace. A `kind/*` namespace rather than extending `schema/*`: these are not contract-shape faults, and a reader triaging a `schema/invalid` diagnostic is looking for a malformed unit.

They read `Projection.model` rather than the tree. Every input they need — the document's kind, its task list, its requirements, its verifications — is already projected, and a rule that walked the tree for units the model already holds would be a second projection that could disagree with the first. `safety/*` reads the tree because remote scripts never reach the model; that reasoning does not apply here.

Severity follows the proposal exactly: placement and completeness are errors in a namespace that is downgradable, and the advisory rule is a `review/*` warning rather than a `kind/*` one. That last choice matters — `review/*` is the namespace strict mode refuses to promote, and an unverified requirement must not gate the work that produces the verification. Putting it in `kind/*` would let `--strict` fail a specification for being in progress, which is the phase gate the dependency model exists to avoid.

### Unconstrained means *no rule fires*, and fixtures prove it

"Validation reports nothing about which units a proposal or design carries" is a claim about absence, and absence is the hardest thing to pin. A rule that quietly stopped firing would pass every test written to assert silence.

So the proposal and design fixtures are **positive** fixtures carrying units that would be violations if any placement matrix existed: a `proposal` with a requirement unit and a `design` with a task unit, each declaring `valid: true` and no rules. If someone later adds a placement rule for those combinations, those fixtures fail. The suite's existing `every_fixture_is_accepted_exactly_when_it_declares_it_is` does the work with no new harness concept.

A second guard states the boundary from the other side: a test asserting the `kind/*` namespace has exactly two rules and `review/*` gained exactly one. A count is crude, and it is the thing that fails when a fourth rule appears without a decision.

### `kind` is required, with no default

An optional `kind` with a default would make every unmigrated document silently become whatever the default is, and the most convenient default — `specification` — is the strictest kind. A document that was a proposal would acquire the one placement error the vocabulary has, and its author would be told their proposal must not carry tasks.

Required and undefaulted means the migration is visible: an unmigrated document fails with "no kind declared", which names the actual problem. This is the same argument the foundation change made for the schema version.

### Archival synthesis is a skill; the program only checks

The proposal's fourth bullet forbids a parser, and the reason is worth restating as a design fact rather than a rule: the upstream kinds have no imposed structure, so there is no grammar to parse. An extractor would have to guess which paragraph of a design states an obligation, and a wrong guess produces a specification that passes every check while misstating the contract — worse than producing nothing, because it looks authoritative.

The division is that an agent writes and `htmlspec validate` checks. html-spec can hold that division in a way OpenSpec's Markdown sync cannot, because it has a validator: the agent's judgment is unconstrained and its output is not.

What the program contributes to archival is therefore small and worth naming precisely: it validates the specification, and it refuses to proceed while the specification reports an error. Everything else in the sequence is instructions.

CAUTION: this makes archival non-reproducible in the strict sense. Two agents synthesizing the same change can produce differently-worded specifications that both validate. The specs require idempotence "in effect" — running twice must not alter the specification's *meaning* — which is a weaker property than byte-identity and the strongest one available when the operation is judgment. A reviewer reads the specification at archival, which is the same review a human-authored spec gets.

### The archived change keeps all four documents

Archival could keep only the specification, since that is what survives as contract. It keeps everything. The whole argument for separating stages is that a proposal's argument and a design's reasoning are worth having later, and discarding them at archival would make the separation a filing convention after all — which is the thing this change exists to fix.

The cost is directory size, and it is negligible: four HTML documents against the 6 MB shared asset store the previous change just deduplicated.

### Migration rewrites every document in one commit

The kind for each existing document is decidable by inspection rather than by rule, but it is not a substitution. Every conformance fixture, the template, and two dogfood documents carry a **task** alongside their requirements, because `base()` returns the rendered template and the template demonstrates all seven units at once. Declaring those documents `specification` trips the one placement rule, so the base loses its task and exactly one fixture — `invalid-schema-missing-reason`, the only one pinning a rule a task alone can violate — gains it back through its own `edit` function. That is a variant within one base rather than a second base.

The projection fixtures are the harder half and they are not documents in the same sense. Their `expected.json` **is** the projected model, so the field appears in committed test data rather than in markup a person reads, and one of them exists to pin the clause being removed.

One commit, because `scripts/check` must stay green and a half-migrated tree fails it. Unlike the asset change, there is no useful intermediate state here: a schema that accepts both `status` and `kind` during a transition would be a third shape nobody wants to specify.

The fixture generator does most of the work from one edit to its base. The 25 Rust files carrying an inline status literal are a mechanical substitution in test source, and the compiler finds none of them — a stale literal produces a document that fails validation at run time with "no kind declared", which is why the required-and-undefaulted decision matters more here than it looks.

## Risks / Trade-offs

- **A "reports nothing" guarantee decays silently** → Positive fixtures carrying would-be violations, plus a rule-count test. Both fail when a rule is added rather than passing quietly.
- **The advisory warning becomes a gate under `--strict`** → It lives in `review/*`, the one namespace strict mode refuses to promote. A `kind/*` warning would be promotable and would fail an in-progress specification.
- **Agent-synthesized specifications vary between runs** → Accepted deliberately, and bounded by validation plus review at archival. Byte-reproducibility is unavailable for a judgment operation, and the alternative is the parser the specs forbid.
- **A document declares a kind whose intended content it does not match** → Not a defect. Intended content is guidance and the specs say so explicitly. The risk is that a reader mistakes guidance for a rule, which is why the normative table has an "enforced structure" column reading `None` twice.
- **The migration misses an inline test document and the suite fails obscurely** → It fails as "no kind declared" on a named document, which is the clearest failure this change can produce. The required-and-undefaulted decision buys that.
- **`kind` and the projection specification drift** → Section 9.3 is rewritten in the same change, and the fixtures pin the mapping. The foundation change's three-piece contract argument applies unchanged.
- **Four kinds prove to be the wrong number** → The set is closed and adding a fifth is a schema change with its own compatibility cost. That is the same bet the seven-unit vocabulary made, and it is worth making again only because the four kinds come from OpenSpec's own working practice rather than from invention.

## Migration Plan

1. Add `kind` to the JSON Schema as a required, closed enumeration, and rewrite section 9.3 of the projection specification to project it. Remove `status` from both.
2. Update the Rust model and `project.rs` to read `htmlspec-kind`, and update `schema_facts` so rule code reads the vocabulary from the schema.
3. Add `rules/kinds.rs` with the two `kind/*` rules and the one `review/*` rule, registered in `rules::ALL`.
4. Remove the task unit from the fixture generator's base and declare `kind=specification`. Give `invalid-schema-missing-reason` an `edit` that adds back the dropped task it pins, declared `kind=tasks`. Regenerate, and confirm the reference and review fixtures still fire from the base's decision unit rather than from the task that left.
5. Migrate the 23 projection fixtures: substitute the field in each `expected.json`, and rewrite `fixtures/projection/metadata` — the fixture that pins section 9.3 — against the new clause. Update the 10 `status` assertions in `projection.rs`.
6. Migrate the template, the 3 dogfood documents, and the 25 Rust files carrying an inline status literal. Reshape `complex-change.html` and `simple-proposal.html`, which carry requirements and tasks together and are the project's shipped examples of the conflated shape.
7. Add the fixtures for the three rules, plus the two positive fixtures that pin the upstream freedom.
8. Extend `init` to take a kind and to create a whole change directory.
9. Write the archival skill and the instructions it follows, and wire the validation half into the archival operation.

Steps 4 through 6 are one commit with steps 1 through 3, because a half-migrated tree fails `scripts/check`. Steps 7 through 9 are separable and land after.

Rollback: steps 1 through 6 are not independently useful, so the revert unit is the whole migration commit. After it lands there is nothing to roll back to that a document conforms to, which is what BREAKING means and why the migration is one commit rather than several.
