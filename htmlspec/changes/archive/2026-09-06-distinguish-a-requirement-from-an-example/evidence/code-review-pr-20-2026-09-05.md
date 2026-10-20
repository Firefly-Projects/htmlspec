# Code review — PR #20 (`distinguish-a-requirement-from-an-example`)

**Date:** 2026-09-05
**Scope:** `git diff main...HEAD`, 19 files
**Method:** built both `main` and the branch, ran the full workspace suite (green), clippy and fmt
(clean), and reproduced the runtime behavior end-to-end in a scratch `htmlspec` project. Each
finding below was verified behaviorally where a behavior could be observed.

## Majors

### 1. The new rule is absent from `known_rules()`, so the coverage gate passes vacuously

`conformance/tests/conformance.rs:350`

`known_rules()` is a hand-maintained list, and the file's own comment says "a rule absent from this
list is a rule with no coverage requirement." `review::EXAMPLE_CONTENT` was added to the validator
but not to this list. The list ends at `STALE_REFERENCE, STALE_VERIFICATION,
UNVERIFIED_REQUIREMENT`, and `grep -rn EXAMPLE_CONTENT conformance/` returns nothing.

Consequences:

- `every_rule_has_both_a_valid_and_an_invalid_fixture` never asks for a fixture for the rule.
- `no_fixture_declares_a_rule_the_validator_does_not_have` cannot catch the omission.
- No conformance fixture pins the rule. It ships with unit tests only.

Confirmed by adding `rules::review::EXAMPLE_CONTENT,` to the list, which immediately fails with
`these rules have no fixture that pins them: review/example-content`.

This is the exact rot the suite's own comment says it exists to prevent. It also means task 3.7's
claim — checked `[x]` — that the rule was proven not to over-report across fixtures rests on a suite
that never asserts on it.

**Fix:** add the rule to `known_rules()` and add the valid/invalid fixture pair the gate then
demands.

### 2. `htmlspec sync` has no example-content gate, and its report instructs the agent to commit the hazard

`crates/htmlspec-cli/src/commands.rs:1477`

The change gates only `archive`. Every example-content check sits inside `archive`
(`commands.rs:954-982`); `sync` (from `commands.rs:1403`) has none. But PR #16/#19 made `sync` a
first-class operation that performs the same merge into `htmlspec/specs/`.

Reproduced end-to-end:

1. Create a change; write a specification from the template.
2. `htmlspec sync --change X` reports `absent req-example`, plus
   `note: 1 requirement(s) appear in no accepted specification. Merge each into the specification
   that should hold it`.
3. The agent complies. `req-example` now sits in the accepted contract.
4. `htmlspec validate specs` exits 0. `sync` then reports "the change's contract has been applied
   in full."
5. The archive refusal is trivially cleared by renaming the ids in the *change's* specification.
   The polluted accepted specification is never re-examined.

The `contract-authority` requirement added by this change says "Archival SHALL refuse while a
change's specification carries one," which the code satisfies literally. The defect is that the
requirement's wording under-covers its own hazard: the live path is now `sync`, and `sync` does not
merely skip the check — it prints instructions to commit the very thing the gate was added to stop.

**Fix:** gate `sync` on the same condition, and widen the requirement's wording from "archival" to
any operation that merges a change's contract into the accepted specifications.

## Minors

### 3. `is_example_identifier` re-scans and re-sorts every template on every call

`crates/htmlspec-assets/src/lib.rs:165`

`example_identifiers()` allocates a `Vec`, scans ~12.7 KB of template text, sorts, and dedups;
`is_example_identifier` then linear-searches it and discards the whole thing. It is called once per
unit and once per figure per document.

Measured: 3.8 µs/call release, 30.6 µs/call debug. On an 800-requirement document, branch `validate`
runs 4.66 s vs 4.24 s on `main` — about 10% slower, purely from this.

**Fix:** a `OnceLock<Vec<&'static str>>`, or have callers hoist the set out of the loop.

### 4. `identifiers_in` matches the bare substring `id="`, with no left boundary

`crates/htmlspec-assets/src/lib.rs:174`

`rest.find("id=\"")` has no check for what precedes the match, so the scan also harvests values from
any attribute whose name ends in `id`, and from `id="…"` appearing in text or comments. Confirmed
standalone:

- `<a data-id="req-example">` yields `["req-example"]`
- `<p>id="not-a-tag-example"</p>` yields `["not-a-tag-example"]`

Today's four templates contain no such attribute, so the derived set is currently correct. But the
doc comment justifies the scan by saying "the templates are files this project writes, so their
attributes are double-quoted" — which addresses quoting, not attribute-name boundaries. A template
gaining a `data-id`, an `…id="…"`-suffixed attribute, or a comment mentioning an example id would
silently widen `example_identifiers()`; and because the archive gate is derived from it, that widens
a refusal.

**Fix:** require a preceding space or `<`.

### 5. The routes test asserts `WORKFLOWS` order rather than set membership

`crates/htmlspec-assets/src/instructions.rs:389`

`the_routes_out_of_silence_reach_the_bodies_that_need_them` collects `carries` in `WORKFLOWS`
iteration order and compares with `assert_eq!(carries, vec!["new", "ff"])`. `WORKFLOWS`' own doc
comment says the order "carries weight" and is lifecycle order — i.e. it is expected to change when
a stage is inserted, as `sync` was. If a future workflow lands between `new` and `ff`, or the two
are reordered, the test fails with "the three routes are stated in the wrong set of bodies" while
the bodies are in fact correct.

**Fix:** sort both sides, or compare `BTreeSet`s — which asserts the property the doc comment
actually describes.

### 6. The accepted specification says "seven" workflows twice; there are eight

`openspec/specs/contract-authority/spec.md:189` and `:201`

The two lines read "seven independently maintained statements of one rule" and "seven paraphrases of
one rule." `WORKFLOWS` has eight entries: `explore`, `new`, `continue`, `ff`, `apply`, `verify`,
`sync`, `archive` (`sync` shipped in 0.7.0). `instructions.rs:215` already says "eight," so the code
and the contract now disagree on the count.

The change's own evidence file `the-guidance-is-silent-not-wrong-2026-09-03.md:14` records exactly
this: "The proposal, design, and tasks all say seven… The count is now eight," and gives the reason
it matters — "a task that says 'all seven bodies' is satisfiable while leaving one body without the
wording." That reasoning applies with more force to the accepted specification, which is the
normative document and the one thing this PR makes binding. Task 4.3 is also checked `[x]` while
still reading "all seven bodies."

### 7. `every_identifier_the_templates_write_is_reported` does not test what task 3.6 says

`crates/htmlspec-validate/src/rules/review.rs:533`

Task 3.6 reads "add an example unit to a template in a test fixture and confirm the check reports it
with no other edit." The test instead iterates the identifiers the templates *currently* write and
asserts each is reported — a restatement of `is_example_identifier`'s own derivation, not a
demonstration that coverage moves. If `identifiers_in` broke such that a newly added template unit
were missed, the test would still pass, because the loop's input comes from the same function under
test. The task is marked `[x]`.

The property is genuinely hard to test without a template fixture seam. Worth either adding one, or
amending the task the way task 3.2 was honestly amended.

## Notes, not findings

- **The identical-wording claim holds.** The extracted block was diffed across all eight bodies:
  byte-identical in six, and `new`/`ff` identical to each other with the routes section appended.
- **`scripts/check-pull-request` reports one fault on this branch:** `CHANGELOG.md` is untouched.
  Tasks 5.x and 6.x are still open, so this is presumably expected mid-change, but CI's `pr-guard`
  job will fail until the changelog lands or `skip-changelog` is applied.
- **The severity split works as designed.** The same document validates at exit 0 (warning, not
  promoted under `--strict`) and refuses archival at exit 1, naming all seven identifiers.
