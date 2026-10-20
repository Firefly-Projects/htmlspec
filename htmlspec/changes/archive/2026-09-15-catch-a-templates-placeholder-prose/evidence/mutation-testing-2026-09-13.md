# Mutation testing: every fix reverted, and what actually failed

Task 6.3, plus the verification 6.4 asks for and the reading 6.5 asks for. Seven mutations, each
applied alone against the finished implementation, with `cargo test --no-fail-fast` — the flag
matters, because `cargo test` stops at the first failing target and the first run of this exercise
reported one failure where there were three.

**Two mutations found gaps in the tests rather than confirming them.** Both are recorded below with
what was changed in response. The exercise's value was in those two, not in the five that passed.

## The mutations

### 1. Whole-summary equality reverted to a substring search

`is_example_placeholder` changed from `example_placeholders().contains(&normalized)` to
`.any(|p| normalized.contains(p.as_str()))`.

**First run — 3 tests failed, and fixture 6.2 was not among them:**

- `htmlspec-assets`: `only_a_summary_that_is_the_placeholder_is_reported` FAILED
- `htmlspec-validate`: `rules::review::tests::a_summary_quoting_the_placeholder_is_not_reported` FAILED
- `htmlspec-cli`: `a_specification_quoting_the_placeholder_is_not_refused` FAILED

`valid-summary-quoting-the-placeholder` — the fixture task 6.2 exists to add, whose whole purpose
is that a substring search would report it — **did not fail the suite.** Confirmed directly: with
the mutation in place, `htmlspec validate` on that fixture reports
`warning: review/example-content at document`, and `cargo test -p htmlspec-conformance` passed.

The cause is the harness, and it is the finding the proposal named as its second one:
`every_fixture_reports_exactly_the_rules_it_declares` does `if expected.rules.is_empty() { continue; }`
at line 213. A fixture declaring no rules is never validated at all. So a fixture whose subject is
that *nothing* fires cannot state that subject through its declaration — the file would have sat
there looking like a guard and guarding nothing.

**Fixed** by adding `a_summary_quoting_the_placeholder_reports_nothing` to
`conformance/tests/conformance.rs`, which names that fixture and asserts its report is empty.
Named rather than derived, for the reason stated in its doc comment: the general property (no
accepted fixture reports an undeclared rule) is a different and larger change, since `valid-minimal`
reports `review/example-content` seven times with an empty rule list today.

**Re-run after the fix — 4 tests failed, fixture 6.2 among them:**

- `only_a_summary_that_is_the_placeholder_is_reported` FAILED
- `rules::review::tests::a_summary_quoting_the_placeholder_is_not_reported` FAILED
- `a_specification_quoting_the_placeholder_is_not_refused` FAILED
- **`a_summary_quoting_the_placeholder_reports_nothing` FAILED**

### 2. Normalization broken, first attempt: `split_whitespace().join(" ")` → `trim()`

- `inline_markup_inside_a_paragraph_contributes_its_text` FAILED
- `only_a_summary_that_is_the_placeholder_is_reported` FAILED

**Task 3.4's test did not fail.** Not a gap in the test — a mistake in the mutation. The templates
write each masthead paragraph on one line, so `trim()` and `split_whitespace().join(" ")` produce
the same string for every real template. The scanned and projected sides still agreed, which is
exactly what 3.4 asserts, so it passed correctly.

### 2b. Normalization broken so the two sides genuinely disagree: `join(" ")` → `join("  ")`

The drift 3.4 exists to catch: a normalization the projection never produces.

- `inline_markup_inside_a_paragraph_contributes_its_text` FAILED
- `the_scan_reads_the_masthead_alone` FAILED
- **`the_scanned_placeholder_is_the_projected_summary_of_a_new_document` FAILED** — task 3.4

The failure message is the one the test was written to give: the scanned placeholder differs from
the projected summary, so the rule would silently match nothing.

### 3. The rule restricted to the `specification` kind

An early return added to `summary_is_the_authors_own` for any kind but `specification`.

- **`every_artifact_kind_carrying_its_placeholder_summary_is_reported` FAILED** — task 4.5
- `a_document_whose_units_were_replaced_and_whose_summary_was_not_is_reported` FAILED
- `a_placeholder_summary_is_never_promoted_to_an_error` FAILED
- `a_summary_quoting_the_placeholder_is_not_reported` FAILED

Three of those four use a `tasks` fixture, which is why they fell too — and `tasks` is the kind the
original report was made against, so their falling is the point rather than noise.

### 4. The rule never runs: the call to `summary_is_the_authors_own` deleted

- `rules::review::tests::a_document_whose_units_were_replaced_and_whose_summary_was_not_is_reported` FAILED
- `rules::review::tests::a_placeholder_summary_is_never_promoted_to_an_error` FAILED
- `rules::review::tests::a_summary_quoting_the_placeholder_is_not_reported` FAILED
- `rules::review::tests::every_artifact_kind_carrying_its_placeholder_summary_is_reported` FAILED
- `a_placeholder_summary_warns_at_validation_and_refuses_at_archival` FAILED
- **`every_fixture_reports_exactly_the_rules_it_declares` FAILED** — fixture 6.1

This is the run that shows fixture 6.1 is load-bearing. It declares `review/example-content` and
that rule is present, so the harness does validate it and does fail.

The `a_summary_quoting_...` tests fail here because each asserts the whole-summary case is still
reported, so it cannot pass by the rule having gone silent. That assertion earning its place is
visible in this run.

### 5. The archival refusal never sees a placeholder summary

`placeholder_summary` in `example_content_in` forced to `false`.

- **`a_placeholder_summary_warns_at_validation_and_refuses_at_archival` FAILED** — task 5.3
- **`the_comparison_refuses_a_placeholder_summary_instead_of_reporting_a_merge` FAILED** — task 5.4

Both, and only these two. The validation half of the first test still passed, which is the severity
split behaving: the mutation touched the refusal and not the warning.

### 6. The absent-summary early return deleted

`if projection.model.summary.is_empty() { return; }` removed from `summary_is_the_authors_own`.

**Nothing failed.**

Investigated rather than patched. The return is genuinely redundant:
`projection.model.summary.join(" ")` on an empty vector is `""`, and
`is_example_placeholder("")` is false, because no template writes an empty masthead paragraph. The
guard cannot change any outcome.

**Not fixed by adding a test, because there is nothing to test at this layer.** The property task
4.4 asks for — an absent summary is not reported — is real and is asserted twice
(`a_document_with_no_summary_is_not_reported`, both shapes projection 9.1.1 names), and it holds
through `is_example_placeholder`, where `only_a_summary_that_is_the_placeholder_is_reported`
asserts the empty string is not a placeholder. Break *that* and tests fail.

What was done instead: the doc comment now says so. It previously implied the return carried the
property. A comment claiming a line is load-bearing when it is not is the same defect this change
closes one scale down — a check whose subject is real, whose reach is not what it says.

### 7. The comparison made per paragraph rather than over the joined summary

`is_example_placeholder(&whole)` → `.summary.iter().any(|p| is_example_placeholder(p))`.

- **`a_summary_quoting_the_placeholder_is_not_reported` FAILED**

The second of its three shapes is the one that catches this: the placeholder as one paragraph with
the author's own paragraph beside it. Under this mutation that reports, which would make the
boundary depend on whether an author edited the masthead in place or added below it.

## 6.4 No existing fixture breaks — verified against `main`, not reasoned

The proposal argues the 50 fixtures are safe because
`every_fixture_reports_exactly_the_rules_it_declares` checks only that declared rules are present.
That reasoning is correct, and it is also the shape that is right until it is not, so this was
measured by running **both binaries** — `main` at `4a89958` in a git worktree, and this branch —
over all 50 pre-existing fixtures and comparing.

| Measurement | Result |
| --- | --- |
| Pre-existing fixtures compared | 50 |
| Acceptance (`valid`) changed on | **0** |
| Reported ≥1 undeclared rule **before** this change | 49 of 50 |
| Report ≥1 undeclared rule **after** this change | 49 of 50 |
| All 52 fixtures carry the placeholder prose somewhere in `input.html` | 52 |
| The new summary diagnostic fires on | 50 |
| Of those, do not declare `review/example-content` | 48 |

That last row looks alarming and is not a change in kind. **49 of 50 fixtures already reported
undeclared rules before this change**, so the count of fixtures in that state moved by zero. The new
diagnostic joins an existing set rather than creating one. Not fixed here, for the reason the
proposal gives: tightening the harness means deciding what all 50 fixtures should declare, which is
a change of its own.

`cargo test` passes with zero failures, all 16 conformance tests included.

## 6.5 This repository's own documents, read rather than counted

The rule was run over every `.html` in the repository except the templates (which define the
placeholder) and the conformance fixtures (covered above). 49 documents.

**It fires on exactly one:**

```
openspec/changes/archive/2026-09-11-migrate-an-openspec-project/evidence/
  the-plan-for-this-repository-2026-09-09.html
```

Which is the same document the independent `git log` scan found, arrived at by a different route.

**It is a true positive, and reading it is what says so.** The document's `title` is *"Migrating
html-spec's own OpenSpec project"*. Its masthead — the whole of its projected `summary`, the field
`document-schema` describes as "what the document is for, in its own prose" — reads *"Replace this
paragraph with what this work delivers."* A consumer reading that document's summary gets
instructions addressed to its author.

It fires on nothing else. All 6 dogfood documents, all 39 projection and parse-error fixtures, the
README, and the two hand-written explainers are clean — including the explainers, which is the
whole-summary decision doing its job: a document discussing this rule is not reported for
discussing it.

**Not fixed in this change.** The document is inside `openspec/changes/archive/`, and an archived
change is a permanent record. Editing one to satisfy a rule introduced afterwards would rewrite
history to make a check look clean, which is the opposite of what the check is for. It is reported,
it stays reported, and the report is the evidence that the condition was real before the rule
existed.
