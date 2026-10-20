# Verification review: reviewer `c`

Four changes, 43 verification units. Each verdict names the step of the rule that decided it. Tests a
record gives by name were read at the commit shown: `06b3508` (#20), `4a89958` (#25), `0ce5ad5` (#26).

## 2026-09-02-state-a-changes-place-in-a-sequence

Every unit rests on `the-sequence-against-real-projects-2026-09-02.md` and the `prove-sequence.py`
assertions it reports, run against the built binary (step 1).

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-predecessor-that-has-been-archived` | passed | kept | Steps 3–4. Scene 5.2 archives `the-device-work` after its predecessor sits in `2026-09-03-the-graded-work`. The script asserts exit 0 and no `this change follows` in stderr. Both other resolutions print that phrase: `active` as "which is not archived" (5.1), `unmatched` as "no active or archived change carries that name" (5.3). So silence at archival is how the archived resolution shows, and the record states it resolved as archived. |
| `verify-a-predecessor-still-being-worked-on` | passed | kept | Step 4. 5.1 asserts `unresolvedPredecessor.resolution == "active"`. That is the whole outcome. |
| `verify-a-predecessor-that-no-longer-exists-under-that-name` | passed | kept | Step 4. 5.3 asserts `unmatched` in archival and in the listing, and asserts the report does not say "which is not archived". That covers "reported distinctly". |
| `verify-two-changes-that-each-follow-the-other` | passed | kept | Step 4. 5.4 asserts `followsCycles` as one finding, the word contradiction in both listing and archival, and name order kept. That covers "no ordering is chosen". |
| `verify-a-change-archived-after-its-predecessor` | passed | kept | Step 4. 5.2 asserts exit 0 with no question asked, and neither ordering phrase in stderr, for each archival. |
| `verify-a-change-archived-before-its-predecessor` | partial | kept | Step 4. The partial is right: no question was ever put to anyone, and the archiving run used `--yes`. The one `unchecked` paragraph named two parts, so it is now two paragraphs, "asks" and "archives on an affirmative answer" (step 4's one-per-part form). |
| `verify-an-agent-archiving-out-of-order` | passed | kept | Step 4. 5.1 and 5.5 assert `outOfOrder: true` and `unresolvedPredecessor.change` in the `--check --json` payload. |
| `verify-a-change-whose-predecessor-is-unfinished` | passed | kept | Step 4. 5.5 asserts `follows.resolved` false for exactly the three successors. The record's transcript of the human listing shows each predecessor named beside its successor and marked `(active)`. |
| `verify-order-does-not-follow-the-stated-sequence` | passed | kept | Step 4. The 5.4 assertion is about a cycle, which has no sequence to follow, so on its own it only shows name order. The 5.5 transcript the record shows separates the two: `compare-…` is listed before `contract-…`, the change it follows. Note: the script's 5.5 never runs the human `list`, so that transcript is recorded output, not an assertion. |

## 2026-09-06-distinguish-a-requirement-from-an-example

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-every-destination-names-a-requirement-or-the-silence` | passed | partial | Step 4. `the-guidance-changed-the-transcript` checked that every destination is cited or recorded as silence. It also checked that the 20 cited titles exist (`grep`) and that one quotation is verbatim. It never checked that each cited requirement fixes the point it was cited for. Unchecked: "it names the requirement that fixes it". |
| `verify-the-rule-did-not-over-fire` | passed | kept | Steps 3–4. Both records (the after-run and the `apply` run) state that claims about runs were uncited, and that the agent's reasoning about its ordering was uncited. Same trigger as both scenarios. |
| `verify-silence-where-a-shape-exists-preserves-it` | passed | kept | Step 4. The record states four points were recorded as silence, and the preserving route was taken where a shape existed. |
| `verify-silence-where-nothing-exists` | partial | kept | Step 4. The record checks only that the agent attributed the `.html` choice to itself. Its unchecked "cheapest-to-change" stands. |
| `verify-a-newly-created-specification-validates` | passed | kept | Step 4. `the-namespace-decides-the-severity`: every kind `new document` writes was accepted, the specification with 9 warnings. The code-review note says the same document validates at exit 0, with the example content as a warning that `--strict` does not promote. |
| `verify-archival-refuses-example-content` | passed | kept | Step 3. The run was `archive keeps-example --check`. I read `commands.rs` at `06b3508`: the example-content refusal comes before the check-mode branch. So `--check` runs archival up to its own refusal, the same trigger as the scenario. Exit 1, all seven identifiers named. |
| `verify-the-comparison-refuses-before-reporting-a-merge` | passed | kept | Step 4. Finding 2 shows `sync` before the fix (merge note, exit 0) and after it (refusal, exit 1, no merge note). |
| `verify-a-half-replaced-example-is-reported` | passed | kept | Step 4. The fixture keeps `req-example` under the author's own prose. Its report holds 7 example-content warnings, one per template identifier, `req-example` among them. Moving the fixture aside fails the gate. |
| `verify-a-template-that-gains-a-unit` | partial | removed | Step 3. Both tests read at `06b3508`. The first feeds `identifiers_in` strings the test writes itself, a unit test of a helper. The second runs the rule over today's derived identifiers only. Neither changes a template, and neither runs the check on a new identifier. The unit's own `unchecked` paragraph names the scenario's whole WHEN and THEN. The record says the templates are `include_str!`ed, so no test can make this trigger happen. That makes it a check of a different thing, not a short one. |
| `verify-one-wording-across-the-bodies` | passed | kept | Step 4. The code-review note diffed the extracted block across all eight bodies: byte-identical in six. `new` and `ff` are identical to each other, with the routes section appended beside the statement. |

## 2026-09-11-migrate-an-openspec-project

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-surveying-this-repository` | passed | kept | Step 4. Read at `4a89958`, `the_survey_enumerates_every_file_the_project_holds` runs the compiled program's `--json` against this repository. It compares the result, file for file, with an independent walk. Each kind matched `find`. |
| `verify-a-file-loose-at-any-level` | passed | kept | Step 4. The record shows 4 of 7 files missed before the fix, and all 7 enumerated after it, the 4 as `unrecognized`. Read at `4a89958`, the test asserts each loose path is enumerated and that 4 are `unrecognized`. |
| `verify-nested-evidence` | passed | kept | Steps 3–4. Read at `4a89958`: three files at three depths under `evidence/`, and the evidence count is 3. A depth-1 mutation failed it. |
| `verify-the-survey-writes-nothing` | partial | kept | Step 4. Read at `4a89958`: both tests compare lists of paths (`walk` / `every_file`), never contents, and neither involves a destination. Both `unchecked` paragraphs stand. |
| `verify-the-survey-is-repeatable` | passed | kept | Step 4. Read at `4a89958`: two surveys of one unchanged source are compared, entities and payloads both. |
| `verify-the-payload-the-gate-read` | passed | kept | Step 4. `migrating-this-repository`: `migrate promote --survey survey.json` checked the plan against the kept survey payload, 201/201. |
| `verify-every-entity-has-a-disposition` | partial | kept | Step 4. Coverage at archival never read `destination` (five-more-defects, finding 5), so the unchecked "with a destination" stands. |
| `verify-the-plan-is-a-tasks-document` | passed | kept | Step 4. The record describes `tests/plan.rs`, run through the compiled program. One plan made of task units goes through `validate` (exit 0), `format`, `format --check`, and `migrate promote`. |
| `verify-an-entity-missing-from-the-plan` | passed | kept | Step 3. Read at `4a89958`: `migrate_promote` runs in `Perform` mode against a plan missing three entities. It asserts `DocumentViolation`, each entity named, and "promotion does not proceed". |
| `verify-the-program-does-not-judge` | passed | kept | Step 3. Read at `4a89958`: `Coverage::of` on a plan that sends a capability specification into `evidence/` is complete. The record quotes the passing report, which says the check makes no judgment. |
| `verify-checks-run-in-the-staging-location` | passed | kept | Step 4. In the hand run, `--check` checked the complete staged project. Read at `4a89958`, `a_complete_migration_promotes_and_the_result_is_a_project` promotes the project and then passes `validate`, `assets`, and `format --check` at the destination. |
| `verify-promotion-is-a-move` | passed | kept | Step 4. Four documents at four depths: each reference was resolved in staging and again after the move, and the reference text was identical. |
| `verify-an-abandoned-migration` | passed | partial | Step 4. Read at `4a89958`, the test labels its boundaries Stage 4, 5, 6, and 7 (declined). They are coverage, structure, and conformance failures, plus a clean `--check`. No check stops migration after the survey, read, or gather-evidence stage and then compares the destination. The survey tests involve no destination at all. Unchecked: "migration stops at any stage before promotion". |
| `verify-checks-gate-promotion` | passed | kept | Step 3. Read at `4a89958`: boundary 3 runs promotion in perform mode, not `--check`. Its identifier `REQ-Example` gets `DOCUMENT_VIOLATION` and leaves the destination's bytes unchanged. The systematic-fault test also refuses and leaves the destination empty. |
| `verify-a-systematic-fault-reported-as-a-set` | passed | kept | Step 4. Read at `4a89958`: the test asserts all 12 documents are named, the count stated, the word "systematic", and an empty destination. A return-at-first mutation failed it. |
| `verify-a-clean-staged-project` | passed | kept | Step 4. In the hand run, the run without `--check` promoted `htmlspec`, `spec.nix`, and `CHANGELOG.md`, with exit 0. |
| `verify-the-gate-reads-the-carried-version` | partial | kept | Step 4. Only one carried version (0.1.0) exists, so "the greatest" holds only vacuously. The `unchecked` paragraph stands. |
| `verify-one-obligation-renamed` | partial | removed | Step 3. Read at `4a89958`: the test builds a plan by hand with three distinct identifiers and calls `Coverage::of`. It migrates no change, so nothing derives the identifiers from the headings each change stated. The record says the test passes whatever the table is keyed on, including the keying `project-migration` forbids. When unsure, remove. |

## 2026-09-15-catch-a-templates-placeholder-prose

Every unit rests on `mutation-testing-2026-09-13.md`: each mutation, the tests it failed, and a suite
that passes with zero failures (step 1). All tests were read at `0ce5ad5`.

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-newly-created-document` | partial | kept | Step 4. The test renders each kind through `render_kind` and validates it. It counts only diagnostics at the document level (the summary's) and asserts the document is accepted. The template's example units are never counted, so the unchecked stands. |
| `verify-a-document-whose-units-were-replaced-and-whose-summary-was-not` | passed | kept | Step 4. In the unit test, a document whose units are all the author's own and whose masthead is the placeholder gets one diagnostic at no unit and is accepted. Fixture 6.1 fails under mutation 4. The 49-document run fired on one real instance. |
| `verify-a-summary-that-quotes-the-placeholder-within-an-author-s-own-statement` | passed | kept | Step 4. Three quoting shapes report nothing, while the placeholder alone does. The conformance test asserts the quoting fixture's report is empty. Mutation 1 fails both. |
| `verify-a-placeholder-summary-reaching-an-operation-that-applies-a-contract` | passed | kept | Step 4. `tests/archive.rs` runs archival through the binary and asserts nonzero exit, "the masthead summary", the summary's reason, and not the example-requirement reason. |
| `verify-every-artifact-kind` | passed | kept | Step 4. The test loops over every template in `TEMPLATES` (asserted non-empty) and asserts one diagnostic at the document level for each. Mutation 3 fails it. |
| `verify-an-absent-summary` | passed | kept | Step 4. Both shapes (a header with no paragraph, and no header at all) draw no `review/example-content`. The suite passes. |

## Counts

- Kept: 39
- Made partial: 2 (`verify-every-destination-names-a-requirement-or-the-silence`, `verify-an-abandoned-migration`)
- Made passed: 0
- Removed: 2 (`verify-a-template-that-gains-a-unit`, `verify-one-obligation-renamed`)

After the edits, each specification validates with 0 errors and is canonical. `check_draft.py` differs from its baseline only in
the verification list. Warnings went from 42 to 41 and from 75 to 74: each removal took away the
unit's `review/stale-reference`.
