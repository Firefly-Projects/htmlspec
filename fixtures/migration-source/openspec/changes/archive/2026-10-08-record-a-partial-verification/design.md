## Context

See proposal.md for why. The shape of the current system that the approach depends on:

- **The outcome is a string the schema enumerates.** `data-outcome` projects into
  `Verification.outcome: Option<String>` (`crates/htmlspec-parse/src/model.rs`). The four values
  live only in `document.schema.json`, lines 613-621. The model holds whatever the attribute says,
  because it is the projection of documents that may be invalid, and the validator judges them.
- **A conditional field is a validator rule.** The schema "keeps to a conservative subset with no
  conditional composition, which is what makes type generation work" (the header of
  `crates/htmlspec-validate/src/rules/relations.rs`). So "a `dropped` task states a reason" is
  `dropped_tasks_state_a_reason` in that file, reported as `schema/missing-reason`. It finds the
  `dropped` value through `status_named("dropp", …)`, a stem read against the schema, because
  `tests/schema_rust_split.rs` forbids enumerated literals in rule code.
  `every_conditional_rule_finds_its_state` fails if a stem stops matching.
- **Every reader of the outcome takes its values from the schema.** There are two, and neither
  names a value:
  - `review.rs` `has_run` decides between `review/stale-verification` and
    `review/stale-reference`. It counts every value except the "not run" one as having run.
  - `runs.rs` `has_run` decides whether a run's record recorded an outcome. It counts a value as
    recorded when the schema enumerates it and it is not the "not run" one.

  `requirements_are_verified` does not read the outcome at all.
- **Presentation labels each outcome by value.** `assets/sha256-ebf35a36fe4dc1a7/htmlspec.css`,
  the revision `own` records since `mark-the-words-a-modification-changed`, has one `content` rule
  per outcome and deliberately no `attr()`.
  `conformance/tests/presentation.rs` `every_enumerated_state_value_has_a_label` fails for any
  enumerated value without a rule.
- **A second single-valued paragraph is dropped silently.** Measured in section 6 of the
  evidence: two `method` paragraphs validate clean, and the model keeps the first.

## Goals / Non-Goals

**Goals:**

- A check that reached part of what it names can be recorded as what it was, with the part it
  missed held in the model rather than in prose.
- A partial record that does not name what it missed is rejected.
- `partial` behaves as a verification that ran, everywhere a verification is read, with no new
  branch in any code that reads outcomes.

**Non-Goals:**

- **Addressing part of a scenario.** A reference still names a unit. The `unchecked` paragraph
  says in words which clause went unchecked.
- **Rejecting a contradiction such as `passed` with an `unchecked` paragraph.** See decision 4.
- **Making the review warning read outcomes.** The contributor settled that `partial` clears
  `review/unverified-requirement` like every other outcome.
- **Fixing the two pre-existing faults the measurement found**: `review.rs`'s `has_run` treating an
  unenumerated value as run, and the silent drop of a repeated single-valued paragraph. Each is
  recorded under "Not in this change".

## Decisions

### 1. `partial` is a fifth outcome value

The value means that the check ran against the requirement or scenario the verification names,
reached part of it, and found that part to hold. If the part it reached did not hold, the outcome is
`failed`, and the verification may still say what the check never reached (decision 4).

**Considered: `passed`, with the gap stated in the method paragraph.** This is what the migration
did for `verify-reference-names-a-fabricated-address` and `verify-a-second-copy-of-one-library`.
Rejected, because the model says `passed`, and nothing that reads the model can see the prose.

**Considered: a second attribute beside the outcome**, such as `data-coverage="part"`. Rejected. It
combines with `not-run` into a statement that means nothing. Every measured case is a check whose
reached part held. And a failed check that reached part of its scenario is already expressible, as
`failed` with an `unchecked` paragraph.

**Considered: one verification per clause.** Rejected, because a reference addresses a unit, and
the smallest unit is a scenario. A verification naming the scenario as `passed` claims its whole
Then, whatever its title says.

### 2. The paragraph is `data-field="unchecked"`

The field names its content directly: what the check did not check. It pairs with `method`, which
says what the check did. The word already carries this meaning in the project's own prose. For
example, `crates/htmlspec-cli/src/preflight.rs` line 923 says "reported as unchecked rather than
guessed at".

**Considered: `unverified`.** Rejected. It collides with `review/unverified-requirement`, which means
that no verification names a requirement, and a partial verification does name it.

**Considered: `gap`.** Rejected, because it is ambiguous. This project already uses the word
for a hole in the contract (contract-authority's "a measured gap rather than a hypothetical one")
and for what a record lacks (this change's own proposal). A field should not need the reader to
guess which.

**Considered: `uncovered`.** Rejected. "Covered" already means fixture coverage here
(`conformance/tests/conformance.rs` calls a rule without fixtures `uncovered`) and appears in
project-migration's coverage check.

**Considered: `scope`**, stating what the check did cover. Rejected. The risk that remains is the
part not checked, and a reader should not have to subtract the scope from the scenario to find it.

### 3. `unchecked` is repeatable

Every `data-field="unchecked"` paragraph projects, in document order, into an array, as a decision's
`rationale` does.

**Considered: single-valued, like `reason` and `method`.** Rejected, on the measurement. The
projection keeps the first single-valued paragraph and drops the rest, and nothing reports it.
`Paragraphs::single` says "the caller reports the repetition", and no caller does. For this field a
dropped paragraph would understate what a check missed, which is the overstatement this change
exists to remove. A check can miss more than one clause, and an author will write one paragraph
for each.

### 4. The rule is an error beside the dropped-task rule

`relations.rs` gains `MISSING_UNCHECKED`, `schema/missing-unchecked`, and
`partial_verifications_state_what_they_did_not_check`, which finds `partial` with
`status_named("parti", "definitions/verification/properties/outcome")`. A `partial` verification
whose `unchecked` field is empty is reported against the verification.
`every_conditional_rule_finds_its_state` gains the stem. The rule reads the field, so an unmarked
paragraph does not satisfy it. Section 4.0.2 makes an unmarked paragraph in a verification a note,
and a consumer should find the gap without parsing notes.

**Considered: a conditional in the schema** (`if` outcome is `partial`, `then` require
`unchecked`). Rejected, because the schema keeps to a subset with no conditional composition. That
is why the dropped task's reason is a validator rule too.

**Considered: a `review/*` warning.** Rejected. `review/unverified-requirement` is advisory because
a specification in progress legitimately lacks verifications. A `partial` with no stated gap is not
work in progress. The author who wrote `partial` knew at that moment what was left, and
document-validation's "Violations are classified by severity" makes structural conformance an error.

**Considered: rejecting `unchecked` on a `passed` verification.** That combination contradicts
itself, and it is not added. The overstatements the measurement found carried no `unchecked`
paragraph: their authors wrote the gap into the method instead. So the rule would not have caught
the case that exists. A reason on a task that is not dropped is likewise accepted today. A later
change can add it against a real case.

**`unchecked` is accepted on every outcome.** On `failed` or `blocked` it says what the check
never reached. On `not-run` it is redundant and harmless.

### 5. The requirement lives in document-schema and does not restate the outcome set

The new requirement, "A partial verification states what it did not check", sits beside "A task
declares completion state" and "A decision records its outcome and rationale". Each of those states
its unit's conditional field.

**Considered: stating all five outcomes in the requirement**, as "A task declares completion state"
states its four states. Rejected. "The element contract is published as a machine-readable schema"
makes the schema "the single source of truth for … permitted attribute values". A prose list would
be a second copy that every added value must also edit, and adding a value is this change. The
requirement names `partial` alone, because it attaches an obligation to that value. The task
requirement's list is left alone. Changing it is not this change's subject.

**Considered: document-validation.** Rejected. That capability states how validation reports, and
what a unit must contain is document-schema's. The dropped-task and superseded-decision conditions
are both stated there.

### 6. What a partial verification counts as

**It clears `review/unverified-requirement`.** The contributor settled this. `failed` and `not-run`
clear it today, measured in evidence section 5, because `requirements_are_verified` never reads the
outcome. No code changes. What changes is document-validation's text, which now says so. No test
asserts it today. The review module's own tests declare a `tasks` document, where the rule does not
run. The conformance harness checks that a fixture's declared rules fire, and never that others
stay silent. So a test is added that iterates every outcome the schema enumerates.

**It draws `review/stale-verification`, not `review/stale-reference`, when its target moves. This
is intended:** a partial check did run, and a recorded result against an obligation that has since
moved is the stronger case that warning exists for. `has_run` needs no change, because it
excludes only the "not run" value. `the_not_run_outcome_is_found_in_the_schema` needs no edit
either. It asserts `has_run` for every enumerated value other than `not-run`, so it covers `partial`
the moment the schema lists it, and it catches a mutation that excludes `partial`. On HEAD the
warning already fires for `partial`, but for the wrong reason: the function counts any unenumerated
value as run.

**In a run's record it is a recorded outcome.** `runs.rs` `has_run` counts an enumerated value that
is not "not run", so `partial` clears `review/unrecorded-requirement` once the schema lists it. On
HEAD it does not, because the value is outside the enumeration. runbook-execution's principle already
covers this: the obligation is that the run recorded an outcome. Its text needs no edit.
runbook-portability needs none either: preflight writes `passed` or `not-run`
(`runs.rs` lines 1597-1615), and a machine check has no part to miss.

### 7. The screen stylesheet labels `partial`, in a new revision

`htmlspec.css` gains `[data-outcome="partial"] > h2::after` (and `h3`, `h4`), with `content:
"partial"`, colored `--hs-state-modified` as `blocked` is: a result that asks for attention. The
change is a new content-addressed revision, added through `assets/vendor.lock.json`'s procedure.
The `own` digest moves to it, and `ebf35a36fe4dc1a7`, the revision
`mark-the-words-a-modification-changed` added, joins `retained` beside `aac8967609f80d9f`, which
that change retired. The new revision is `ebf35a36fe4dc1a7`'s bytes with the one rule added, so it
carries that change's marks for `ins` and `del`. The print stylesheet needs nothing, because it
recolors every `[data-outcome]` label generically.

**Considered: `attr(data-outcome)`.** Rejected for the reason the stylesheet gives: it "would
render an invalid value as though the vocabulary contained it".

**Considered: a generated label on the `unchecked` paragraph**, as `renamed-from` has. Rejected.
`renamed-from` holds a bare heading, which a reader could take for a claim, so it needs a label.
`unchecked` holds a sentence the author writes, like `method` and `reason`, and neither of those is
labelled.

### 8. The instructions say when, and when not

`archive.md`'s Verification paragraph gains the rule for a check that reached part of what it
names: record `partial`, and name each part it did not check in an `unchecked` paragraph. It also
states the boundary: a check whose condition or subject differs from the scenario's is not a
partial check of it, and gets no verification. The measurement found eight declines of that kind,
and they should stay declined.

## Risks / Trade-offs

- **[`partial` becomes the easy answer for a check of something nearby.]** → The requirement
  defines `partial` as a check of the right thing that stopped short, and the instructions say a
  nearby check gets no verification. Eight of the thirteen measured declines were of that kind, so
  this boundary is the one authors will meet.
- **[An author records `partial` to silence the unverified warning.]** → Any verification already
  silences it, including `not-run`, which is what the specification template writes. `partial` adds
  no new route, and it costs the author a stated gap.
- **[A document on an older stylesheet revision shows no label for `partial`.]** → The outcome is in
  the markup and the model either way, and each `unchecked` paragraph is ordinary text that reads
  under any stylesheet. Only the badge is missing.
- **[The contract version moves in several changes of one set.]** → Task 1.3 reads
  `CONTRACT_VERSION` at implementation time, and this change moves it one minor version from
  whatever it then is.
- **[`partial` in a document read by an older toolchain.]** → It fails `schema/invalid`, as measured
  on HEAD. A downgrade is loud.

## Migration Plan

This is additive for documents. No existing document carries `partial` or `unchecked`, and no
existing outcome changes meaning.

**The document contract moves one minor version**, because a value and a field are added. When
this was written it was `0.2.0`. `reject-change-markup-inside-a-unit` and
`mark-the-words-a-modification-changed` moved it to `0.4.0`, and `record-a-task-done-differently`,
the change below this one in the stack, moves it to `0.5.0`. So this change moves it from `0.5.0`
to `0.6.0`. The new version joins the
`schemaVersion` enumeration, and every earlier version stays, as the CHANGELOG's bump table requires
of a minor bump.

**The release is expected to be `0.26.0`**, the next minor after the change before it in this set.
The contributor confirms the increment at the release task.

Rollback is a revert. A document that adopted `partial` then fails `schema/invalid`, and one that
adopted `unchecked` fails `schema/projection`, so a rollback is loud, never silent.
