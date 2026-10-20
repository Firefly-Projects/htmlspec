## Why

**A verification cannot record a check that covered part of what it names.** Its outcome is one
of `passed`, `failed`, `blocked`, or `not-run`, and none of them means "this ran, reached part of
the scenario, and that part held". A verification names units, and the smallest unit it can name
is a scenario, so it cannot narrow itself to the clause its check reached. Its author has two
choices, and both are wrong:

- `passed`, which claims the clauses the check never reached;
- no verification at all, which loses the record of the clause it did reach.

`evidence/the-measurement-2026-09-30.md` measured this in the one real case the project has, its
own migration, staged in `~/projects/html-spec-migration/`:

- **The review removed two units** for covering part of their scenario:
  `verify-a-second-store-planted-inside-the-project` ("the record lacks 'names both locations'")
  and `verify-mutation-19` ("it checks only the collision check, not what migration produces").
  Each check ran, reached one clause of its Then, and found it to hold.
- **Two others were kept as `passed`** although their draft says they "partly cover their THEN":
  `verify-reference-names-a-fabricated-address` and `verify-a-second-copy-of-one-library`. Each
  method paragraph admits the gap in prose ("The record gives the rule identifier and the exit
  code"), and the model says `passed`.
- **The staged tree holds 94 verification units**: 92 `passed`, 1 `not-run`, 1 `blocked`. The
  drafting brief offered the four values and no other.
- **13 notes, in 9 drafts, decline a unit because the check "only resembles the scenario".** On
  my reading of each against its Then, 5 describe a check that reached part of it, and 8 a check
  of something else.

Today the contract rejects both halves of a truthful record. `data-outcome="partial"` fails
`schema/invalid` ("is not one of \"passed\", \"failed\" or 2 other candidates"), and a paragraph
marked `data-field="unchecked"` fails `schema/projection` on any outcome.

**Why now.** The project's migration out of OpenSpec is staged and waiting. Without this value,
the migrated documents carry these checks as `passed`, drop them, or record the gap as an open
question beside them.

## What Changes

- **A verification outcome `partial`.** It means the check ran against what the verification
  names, reached part of it, and found that part to hold. A check whose covered part did not hold
  is still `failed`.
- **A partial verification states what it did not check**, in one or more paragraphs marked
  `data-field="unchecked"`, projected into the verification's `unchecked` field. A `partial`
  verification without one is an error, `schema/missing-unchecked`, the way a dropped task without
  a reason is `schema/missing-reason`.
- **`unchecked` is required on one outcome and accepted on all.** A check that failed or was
  blocked may say what it never reached.
- **A partial verification counts wherever a verification counts.** It clears
  `review/unverified-requirement`, as `failed` and `not-run` already do. It draws
  `review/stale-verification` rather than `review/stale-reference` when its target moves, because
  a partial check did run. In a run's record it is a recorded outcome, so it clears
  `review/unrecorded-requirement`. None of this needs a code branch: every reader of the outcome
  already takes its values from the schema.
- **The screen stylesheet labels it.** The stylesheet writes each outcome as a word, value by value,
  so `partial` needs a rule. That makes a new content-addressed revision of `htmlspec.css`.
- **The contract moves one minor version**, because this adds a value and a field. Every document
  that validates today still validates.
- **The archive instructions say when to use it**, and that a check of something nearby is not a
  partial check of the scenario.
- **Unchanged:** the four existing outcomes and what each means; the policy that the review
  warning asks whether a verification names a requirement, not what it found; what preflight
  writes into a new run's record, which is `passed` or `not-run`; and every existing document.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-schema`: a new requirement, "A partial verification states what it did not check".
  It sits beside "A task declares completion state" and "A decision records its outcome and
  rationale", which each state their unit's conditional field. No requirement states anything
  about the verification unit today, and folding this into "The element contract is published as
  a machine-readable schema" would bury one unit's rule inside a rule about publication. **It does
  not restate the outcome set.** The schema is "the single source of truth for … permitted
  attribute values", and a prose copy would be a second list to update each time a value is
  added, which is this change's own edit. The requirement names `partial` alone, because it
  attaches an obligation to that value.
- `document-validation`: "A specification requirement without verification is reported as
  incomplete" states that a verification names a requirement whatever its outcome, `partial`
  included, and gains two scenarios. It is modified rather than added to, because what clears the
  warning is that requirement's own rule, and every other word stays.

## Impact

- `crates/htmlspec-schema/schema/document.schema.json`: `partial` in the verification outcome's
  enumeration, a description of the outcome, and an `unchecked` array property. The
  `schemaVersion` enumeration, the `$id`, `CONTRACT_VERSION` in `src/lib.rs`, and `CHANGELOG.md`
  move one minor version.
- `crates/htmlspec-schema/doc/projection.md`: the verification row of the section 4.0 field table,
  section 4.8, the normalized-text list in 6.1, the validator rules in 10.1, and the version line.
- `crates/htmlspec-parse`: `Verification` in `model.rs` gains `unchecked`. `project_verification`
  in `project.rs` accepts the field.
- `crates/htmlspec-validate/src/rules/relations.rs`: `MISSING_UNCHECKED` and the rule, beside
  `MISSING_REASON`. `tests/schema_rust_split.rs` lists `partial` as a schema-owned literal.
- `crates/htmlspec-validate/src/rules/review.rs` and `crates/htmlspec-cli/src/runs.rs`: no code
  change. Each gains a test that pins how `partial` is treated.
- Conformance: a valid and an invalid fixture through `conformance/src/bin/generate_fixtures.rs`,
  and the rule in `known_rules()`. Projection: the `named-fields` and `unjudged` mapping fixtures.
- `assets/`: a new revision of `htmlspec.css` carrying a `partial` label. `assets/vendor.lock.json`
  moves `own` to it and keeps the old one in `retained`. The four templates declare the new
  contract version, so their pins move too. The specification template lists no outcomes, and its
  example verification stays `not-run`.
- `crates/htmlspec-assets/instructions/archive.md`: its Verification paragraph, the only place an
  instruction explains recording a verification. No instruction lists the outcome values. The
  installed copies are regenerated.
- **Not in this change**, and recorded so it is not assumed to follow:
  - Carrying a source's partial checks through migration. That is
    `migrate-into-the-extended-vocabulary`, and the four staged units above stay as they are until
    it lands.
  - Re-auditing the migration's other 90 `passed` units.
  - A rule rejecting `unchecked` on a `passed` verification.
  - runbook-execution and runbook-portability. "A run's conformance to its runbook's contract is
    reported and never gated" already states that "the obligation is that the run recorded an
    outcome, not that the outcome was good", and `partial` is a recorded outcome. runbook-portability's
    "a verification of each passed machine requirement" is written by preflight, which records a
    machine check as `passed` or leaves it `not-run`, and a machine check has no part to miss.
  - `review.rs`'s `has_run` counting a value outside the schema as a check that ran. The
    measurement found this, and it affects only which warning accompanies a `schema/invalid` error.
  - A repeated `method`, `reason`, or `mitigation` paragraph, which the projection drops without a
    diagnostic although `Paragraphs::single` says the caller reports it. The measurement found
    this, and it is why `unchecked` is repeatable.
