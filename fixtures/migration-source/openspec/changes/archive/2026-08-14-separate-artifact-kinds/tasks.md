## 1. Declare the kind in the contract

- [x] 1.1 Add `kind` to `crates/htmlspec-schema/schema/document.schema.json` as a required property with a closed enumeration of `proposal`, `design`, `tasks`, `specification`, and remove `status` from both the properties and the `required` list
- [x] 1.2 Rewrite section 9.3 of `crates/htmlspec-schema/doc/projection.md` to project `kind` from `<meta name="htmlspec-kind">`, stating that an absent kind is invalid and a kind outside the enumeration is invalid, and remove the `status` clause
- [x] 1.3 Add a test asserting the schema enumerates exactly four kinds, so a fifth cannot be added without a decision — the set being closed is what makes per-kind rules possible

## 2. Project the kind

- [x] 2.1 Replace `Model.status` with `Model.kind` in `crates/htmlspec-parse/src/model.rs`, keeping the field required rather than `Option`, so an absent kind is a projection fault rather than a silent `None`
- [x] 2.2 Update `crates/htmlspec-parse/src/project.rs:100` to read `htmlspec-kind`, and confirm `meta_content` needs no change because the mechanism is the same
- [x] 2.3 Expose the kind vocabulary through `schema_facts` so rule code reads the four values from the schema, the way `refs.rs` reads the `changeState` enumeration. A Rust `const` listing them fails `rule_code_does_not_restate_schema_expressible_values`
- [x] 2.4 Update the 10 `status` assertions in `crates/htmlspec-parse/tests/projection.rs`, and confirm each still names the projection-specification section it exercises

## 3. The three rules

- [x] 3.1 Add `crates/htmlspec-validate/src/rules/kinds.rs` with a `kind/*` namespace, reading `Projection.model` rather than the tree because every input the rules need is already projected
- [x] 3.2 Add the placement rule: a `specification` carrying a task unit is an error naming the task, and stating that a specification records the obligation a task satisfied rather than the work that satisfied it
- [x] 3.3 Add the completeness rule: a `tasks` document carrying no task unit is an error. Assert separately that a tasks document carrying other units alongside its tasks reports nothing, because the rule constrains what must be present rather than what is permitted
- [x] 3.4 Add the advisory rule as a `review/*` warning rather than a `kind/*` one: a `specification` requirement that no verification names. `review/*` is the namespace strict mode refuses to promote, and in `kind/*` this would let `--strict` fail a specification for being in progress
- [x] 3.5 Scope the advisory rule to the `specification` kind alone, so a requirement unit in a proposal is never checked for verification — it is not part of the contract there
- [x] 3.6 Register the rules in `rules::ALL` and extend `every_namespace_has_a_live_rule` for the `kind` namespace
- [x] 3.7 Add a test asserting the `kind/*` namespace has exactly two rules and no more, and that no rule reports on unit placement in the `proposal` or `design` kinds. A rule added without a decision fails here

## 4. Reshape the conformance fixtures

- [x] 4.1 Remove the task unit from `base()` in `conformance/src/bin/generate_fixtures.rs` and declare `kind=specification`, so the 40 fixtures that pin non-task rules become valid specifications
- [x] 4.2 Give `invalid-schema-missing-reason` an `edit` that adds back a dropped task with no reason, declared `kind=tasks`. It is the only fixture pinning a rule a task alone can violate, because `schema/missing-reason` reads `task.state == dropped` and no other unit has a state
- [x] 4.3 Repoint the reference and review fixtures at the base's decision unit, which already carries `data-refs`. `refs/unresolved`, `refs/removed-target`, and `review/stale-reference` all fire from a decision — verified by building such a document and running the binary — so removing the task does not weaken them
- [x] 4.4 Repoint `warning-review-stale-verification` at the base's verification unit, which fires the rule when it names an added requirement and carries its `method` field
- [x] 4.5 Confirm `invalid-schema-prefix-mismatch` still fires. The rule iterates every unit rather than tasks alone, and the fixture works by giving a unit an identifier whose prefix names a different unit type
- [x] 4.6 Regenerate the fixtures and confirm all 41 still declare and report exactly the rules they did before, with `kind` substituted for `status` and the task present only where `invalid-schema-missing-reason` adds it back

## 5. Migrate the projection fixtures

- [x] 5.1 Substitute `status` for `kind` in each of the 23 `fixtures/projection/*/expected.json` files. These hold the projected model itself, so the field name appears in committed test data rather than in markup, and there is no generator to re-run
- [x] 5.2 Rewrite `fixtures/projection/metadata` against the new section 9.3. Its `rules` array names `9.3` and its whole purpose is the field being removed, so it is a rewrite rather than a substitution
- [x] 5.3 Assign a kind to each of the 6 projection fixtures that carry a task unit — `field-name-unknown`, `metadata`, `metadata-absent`, `references`, `no-primary-field`, `unjudged` — choosing `tasks` where the task is the construct under test and removing the task where it is incidental
- [x] 5.4 Confirm `cargo test -p htmlspec-parse` passes, since these fixtures pin the projection specification rather than the rules and nothing else covers that mapping

## 6. Migrate the remaining documents

- [x] 6.1 Migrate `assets/template.html` to declare a kind, and decide which kind `init` produces by default
- [x] 6.2 Reshape `fixtures/dogfood/complex-change.html` and `simple-proposal.html`, which carry requirements and tasks together. They are the project's shipped examples of the vocabulary, and the conflated shape is what this change exists to separate
- [x] 6.3 Confirm `fixtures/dogfood/technical-investigation.html` needs only a kind declaration, since it carries requirements without tasks and already has a specification's shape
- [x] 6.4 Substitute the status literal in the 25 Rust source files carrying an inline test document. The compiler finds none of these, so a missed one fails at run time as an absent kind
- [x] 6.5 Migrate the archived `openspec/changes/archive/2026-08-10-html-spec/documents/deterministic-formatting.html`, and confirm the presentation checks still discover it under the archived location
- [x] 6.6 Confirm the 10 `fixtures/parse-errors` documents need no change, because none declares a status and malformed markup never reaches the metadata rules
- [x] 6.7 Confirm `scripts/check` passes with steps 1 through 6 in one commit, since a half-migrated tree fails it and there is no useful intermediate state

## 7. Pin the rules and the freedom

- [x] 7.1 Add a fixture for the placement rule: a `specification` carrying a task unit, declaring the one rule identifier it violates and no other. The harness asserts the declared rules are the reported rules, so a fixture claiming more than it trips is a fixture that passes for the wrong reason
- [x] 7.2 Add a fixture for the completeness rule: a `tasks` document with no task unit
- [x] 7.3 Add a fixture for the advisory rule: a `specification` requirement that no verification names, declaring `valid: true` because the rule is a warning that never gates
- [x] 7.4 Add a **positive** fixture for the proposal kind: a `proposal` carrying a requirement unit, declaring `valid: true` and no rules. It fails if anyone later adds a placement matrix, which is how a guarantee about absence stays pinned
- [x] 7.5 Add a **positive** fixture for the design kind: a `design` carrying a task unit, declaring `valid: true` and no rules
- [x] 7.6 Add a fixture proving the contract rules are not relaxed by kind: a `proposal` loading a remote script, which fails exactly as any other kind would
- [x] 7.7 Extend the conformance harness's `known_rules` and add `kind/*` coverage, so a fixture cannot declare a rule the validator lacks

## 8. Initialization and the change directory

- [x] 8.1 Extend `init` to take a kind, and to produce a document shaped for that kind
- [x] 8.2 Have `init` produce a `proposal` as a page of prose with no unit scaffolding at all, since scaffolding would push an author toward stating a contract at the stage meant for arguing about one
- [x] 8.3 Have `init` produce a `specification` with the units its kind expects, because its shape is not the author's to invent
- [x] 8.4 Add the ability to initialize a whole change directory, with the documents that stage needs, each declaring its kind and each valid immediately
- [x] 8.5 Add a test that every kind `init` can produce validates immediately, so the promise holds for four kinds rather than one

## 9. Archival

- [x] 9.1 Write the archival skill: the instructions an agent follows to create or update a change's specification artifact from its proposal, design, tasks, and implemented result
- [x] 9.2 State in those instructions that the merge applies partial updates and preserves content it does not mention, following OpenSpec's sync guidance that the delta represents intent rather than a wholesale replacement
- [x] 9.3 Wire the checking half into the archival operation: validate the produced specification, and refuse to proceed while it reports an error
- [x] 9.4 Have archival apply the specification's contract changes to the project's accepted specifications after it validates, and archive the change directory intact with all four documents preserved
- [x] 9.5 Add a test that the archival operation ships no parser reading a proposal or a design and emitting requirements, since those kinds carry no imposed structure and there is nothing to parse reliably

## 10. Verify the whole property

- [x] 10.1 Confirm `scripts/check` passes with every step landed
- [x] 10.2 Confirm a proposal carrying every one of the seven units validates with no diagnostic, which is the freedom this change is for
- [x] 10.3 Reintroduce each of the three rules' defects and confirm the suite fails, since a check that has never failed is a check nobody has tested
- [x] 10.4 Confirm `--strict` does not fail a specification whose requirements are not yet verified, which is the `review/*` placement decision holding
- [x] 10.5 Author one real change in the project's own format, using `init` and the four kinds, and record what the vocabulary made awkward. Forty-one generated fixtures cannot perform this test
- [x] 10.6 Record evidence for the measured claims: the document population before and after, which rules fire from which units, and the archival round trip
