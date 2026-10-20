# What the review of PR #20 found, and what each finding turned out to be

**Date:** 2026-09-05
**Subject:** the seven findings in `code-review-pr-20-2026-09-05.md`
**Method:** each finding verified against the code before any fix, and each fix verified by
breaking it deliberately and watching the intended test fail

Every one of the seven confirmed. Two were majors and both were real defects in what this change
shipped rather than in what it inherited. One minor's measurement did not reproduce, and that is
recorded below rather than repeated.

## The two majors

### 1. The rule shipped with no fixture, and the gate that exists to catch that passed

`review/example-content` was absent from `known_rules()` in `conformance/tests/conformance.rs`.
That list is hand-maintained and its own comment states the trade: "a rule absent from this list
is a rule with no coverage requirement." So `every_rule_has_both_a_valid_and_an_invalid_fixture`
never asked for a fixture, and the rule shipped pinned by unit tests only.

Reproduced by adding the constant and nothing else:

```
these rules have no fixture that pins them: review/example-content
```

**A detail the review did not reach, and it sharpens the finding.** Every one of the 49
conformance fixtures already carried `id="req-example"`, because the generator's base *is* the
specification template. The rule therefore fired on 100% of the corpus while no fixture declared
it. The suite was not silent about this rule for want of a document that violates it — it was
silent while every document violated it.

Fixed with `warning-review-example-content`, the half-replaced case: the requirement's prose
becomes the author's own and `req-example` stays. That shape is what pins the check on the
identifier, since a check reading the text would find nothing template-written in it.

Regenerating wrote one new fixture and left the other 49 byte-identical, so the frozen base held.
The gate was then proven to bite: with the fixture directory moved aside it fails naming the rule,
and with it restored it passes.

### 2. `sync` had no gate, and its report told an agent to commit the hazard

The change gated archival only. `sync` performs the same comparison and its own comment says it
makes "the same three refusals archival makes, in the same order" — there was no fourth.

Reproduced end to end in a scratch project, against the release binary:

```
$ htmlspec sync --change gate-repro
0 of 1 requirement(s) this change states have reached the accepted specifications
  absent   req-example
note: 1 requirement(s) appear in no accepted specification. Merge each into the specification
that should hold it, or confirm it was withdrawn deliberately.
exit=0
```

An agent that complies puts `req-example` into the accepted contract. The rest of the chain was
then confirmed: `sync` reports "the change's contract has been applied in full", and
`htmlspec validate` on the polluted accepted specification exits 0.

**This is worse than the archival gap, not equivalent to it.** Archival's absence would have let
example content through. This one *instructed* it, in the imperative, to an agent whose
instructions tell it to do as the tool reports.

Fixed by extracting `example_content_in` and calling it from both operations, placed before
`merge_state` so the refusal arrives *instead of* the merge instruction rather than after it. The
test asserts the absence of the note as well as the presence of the refusal, because the ordering
is the property.

After the fix:

```
$ htmlspec sync --change gate-repro
htmlspec: the comparison does not run while the specification carries a template's example
content: req-example, scenario-example, … Replace each of them with the change's own
exit=1
```

And with the identifiers replaced, `sync` proceeds and names `req-gate`.

**The requirement's wording was the deeper fault.** It said "Archival SHALL refuse", which the
code satisfied literally while the hazard sat elsewhere. It now states the property — every
operation that puts a change's contract into the accepted specifications refuses — and names the
comparison operation as the case that proves why a list of commands is the wrong shape. This is
the same defect as the count in finding 6, in a different clause.

## The five minors

### 3. The derived set was recomputed per unit — fixed, but the measurement did not reproduce

`is_example_identifier` called `example_identifiers()`, which scanned every template, sorted, and
deduped, then discarded the result. Once per unit and once per figure per document. Now a
`OnceLock`.

**The review reported about 10% on an 800-requirement document — 4.66s against 4.24s. I could not
reproduce that.** Measured warm, five to eight runs each, on a generated 800-requirement document
of 564,716 bytes:

| build | warm wall clock |
| --- | --- |
| this branch, cached | 0.71s (7 of 8 runs) |
| `7c0b4ac`, uncached | 0.72s (7 of 8 runs) |

About 1%. Both builds' first run measured ~1.2s cold, which is the likeliest source of the larger
figure. The cache is still correct — the work was genuinely per-unit and the answer cannot change
between calls — but the reason to keep it is the redundancy, not a 10% recovery.

### 4. The scan had no left boundary

`rest.find("id=\"")` matched inside any attribute name ending in `id`, so
`<a data-id="req-example">` yielded `req-example`. Today's templates carry no such attribute, so
the derived set was correct — but the set drives an archival refusal, so a template gaining a
`data-id` would have widened what the project refuses.

Now requires preceding whitespace. Confirmed the derived set is unchanged: the same seven
identifiers before and after.

What the boundary does not reach, and the doc comment says so: `id="…"` written in a template's
prose, where the preceding character is a space either way. Separating that from an attribute
needs the parser this crate deliberately does not have.

### 5. The routes test asserted order, not membership

`assert_eq!(carries, vec!["new", "ff"])` depended on `WORKFLOWS` iteration order, which that
table's own doc comment says is lifecycle order and expected to change — `sync` was inserted into
the middle of it one release ago.

Proven in both directions with the iteration reversed. The old form fails on correct bodies:

```
left: ["ff", "new"]
right: ["new", "ff"]
```

The `BTreeSet` form passes.

### 6. The accepted specification stated a count, and the count was wrong

Two lines said "seven" statements and paraphrases of one rule. There are eight.

**Fixed by removing the count rather than by correcting it to eight.** This change's own evidence
already records why: an obligation on "all seven bodies" is satisfied while an eighth carries
nothing. Correcting seven to eight reproduces that defect one workflow later. The contract now
states the property and says in prose that the omission is deliberate.

Task 4.3 carried the same count and is amended in place. The proposal and design still say seven,
left alone deliberately: they are the argument as it was written, and task 4.8's reasoning is that
re-opening a settled document re-litigates it. The normative document and the task are what bind.

### 7. The coverage-moves test drew its input from the function under test

`every_identifier_the_templates_write_is_reported` iterates `example_identifiers()` and asserts
each result is reported. Task 3.6 asks for something stronger: that a template gaining a unit is
covered with no second edit.

**The vacuity is partial, and the distinction matters.** A scan broken to return *nothing* fails
that test, because it guards on non-empty. A scan broken to drop **one** identifier and find the
rest leaves it green — measured, not assumed:

| break | validator's half | the new assets-side half |
| --- | --- | --- |
| scan returns nothing | FAILED | FAILED |
| scan drops `risk-example`, finds the rest | **ok** | **FAILED** |

A template gaining a unit is the partial case by nature, so the partial break is the one that
matters. `a_template_that_gains_an_example_unit_is_covered_without_a_second_edit` drives the
derivation from template text the test writes — the only way to state the property, since
`TEMPLATES` is `include_str!`ed and no fixture can substitute for it. Task 3.6 is amended to
describe the chain across two crates rather than one test.

My first draft of that doc comment overstated the vacuity as total. Corrected to state the
measured case.

## The two notes

**No over-report was introduced.** Errors across every conformance and dogfood fixture are
**50 before and 50 after**. Warnings go 448 to 457, and the 9 are exactly the new fixture's own
report — 7 example-content and 2 stale-reference, matching `valid-minimal`.

**The changelog note is expected.** `scripts/check-pull-request` reports `CHANGELOG.md` untouched
because writing the entry is task 6.3 and section 6 has not started. PR #20 carries
`skip-changelog` for that reason, which is the same choice PR #15 made.

## The suite after all seven

- `cargo test --workspace` — 0 failures
- `./scripts/check` — all 9 stages
- `openspec validate --all` — 11 of 11
