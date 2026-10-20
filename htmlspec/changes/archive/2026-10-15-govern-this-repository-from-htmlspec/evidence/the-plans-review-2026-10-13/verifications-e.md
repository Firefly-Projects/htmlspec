# Verification review: reviewer e

Changes: `2026-10-02-see-a-modification-the-merge-did-not-take`, `2026-10-04-resolve-a-reference-across-its-change`,
`2026-10-07-mark-the-words-a-modification-changed`. Every test a unit names was read at the record's
commit: `913a87d` for the first change and `e61e3b1` for the second. The third change's units rest on
runs only.

## 2026-10-02-see-a-modification-the-merge-did-not-take

Records: `the-comparison-measured-2026-10-02.md` (the seven-step run of the first measurement's
`build.sh` and `measure.sh`, unchanged, against the built binary) and `mutation-testing-2026-10-02.md`
(the 13 mutations plus a clean run, "every test passes").

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-word-the-accepted-specifications-have-not-taken` | passed | kept | Steps 1 to 4: run step 1 (`build.sh` 1: `req-x` `modified`, "quickly" added to its one statement, accepted `req-x` without it). `sync` printed `differs req-x (statements from paragraph 1)` and `reached: []`, and all 28 runs exited 0. That is the whole of the scenario. |
| `verify-a-scenario-the-accepted-specifications-have-not-taken` | passed | kept | Steps 1 to 4: run step 2 adds `scenario-x-c` `added`. The record shows `not held: scenario-x-c`, and step 6's payload shows `scenariosNotHeld: ["scenario-x-c"]`, which is the change's identifier. Whole. |
| `verify-a-withdrawn-scenario-the-accepted-requirement-still-holds` | passed | removed | Step 3: `a_withdrawn_scenario_the_accepted_requirement_still_holds_is_not_kept` (commands.rs:7362 at `913a87d`) calls `merge_state(&project, &projection)` directly. The scenario's WHEN is "the operation is run against that change". A unit test of the function `sync` uses has a different trigger. |
| `verify-a-modification-the-accepted-specifications-have-taken` | passed | removed | Step 3: `scenario_order_and_notes_do_not_decide_reaching` (commands.rs:7524) calls `merge_state` directly, and the WHEN runs the operation. Different trigger. |
| `verify-statements-in-another-order` | passed | removed | Step 3: `statements_in_another_order_are_not_taken` (commands.rs:7395) calls `merge_state` directly, and the WHEN runs the operation. Different trigger. |
| `verify-an-added-requirement-merged-with-other-content` | passed | removed | Step 3: `an_added_requirement_is_compared_as_a_modified_one_is` (commands.rs:7418) calls `merge_state` directly, and the WHEN runs the operation. Different trigger. |
| `verify-a-requirement-carried-forward-is-compared-by-its-identifier` | partial | removed | Step 3: `a_requirement_carried_forward_is_compared_by_its_identifier` (commands.rs:7559) calls `merge_state` directly, and the WHEN runs the operation. Different trigger. Run step 4 carries `req-z` forward, but its statements equal the accepted ones, so the run does not reach this scenario either. |
| `verify-a-removal-the-accepted-specifications-have-taken` | passed | kept | Steps 1 to 4: run step 3b, with `req-y` `removed` and deleted from the accepted specification, printed `reached req-y` and `absent: []`. Whole. |
| `verify-a-removal-the-accepted-specifications-have-not-taken` | passed | kept | Steps 1 to 4: steps 3a and 6 printed `held req-y (this change marks it removed)` and `untakenRemovals: ["req-y"]`. Step 6's `reached` is `["req-z"]`, so `req-y` is not reached. Whole. |
| `verify-a-rename-and-a-modification-neither-taken` | passed | removed | Step 3: `a_rename_and_a_modification_neither_taken_are_both_reported` (commands.rs:7487) calls `merge_state` directly, and the WHEN runs the operation. Different trigger. No step of the run states a rename. |
| `verify-the-report-says-where-the-content-differs-not-what-it-says` | passed | kept | Steps 1 to 4: step 6's `sync --json` entry names the identifier, `firstDifferingStatement: 1`, and `scenario-x-c`. The record states that no statement or scenario text from either side appears in any payload. Whole. |
| `verify-a-structured-report` | passed | kept | Steps 1 to 4: step 6's payload carries `stated`, `reached`, `absent`, `untakenRenames`, `untakenContent`, and `untakenRemovals`. Whole. |
| `verify-a-change-whose-contract-has-been-applied` | passed | kept | Steps 1 to 4: in step 6 the accepted specification is back to `req-x`, `req-y`, and `req-z` (`build.sh` 6), every requirement the change states, and the payload has `"absent": []`. Whole. |
| `verify-the-operation-and-archival-agree-in-the-repeated-run` | partial | kept | Steps 1 to 4: in step 6, `archive --check`'s notes name `req-x` (content) and `req-y` (removal), and `sync` puts them in the same sets. The one `unchecked` paragraph, a rename, is right, because no step states a rename. |
| `verify-the-operation-and-archival-agree-about-what-did-not-arrive` | passed | removed | Step 3: `archival_and_sync_name_the_same_untaken_work` (commands.rs:7770) compares `merge_state` with `findings_for`, two functions. The WHEN is "archival reports …" and the THEN is about what "the operation reports". The test runs neither command. Different trigger. The run unit above carries this scenario as partial. |
| `verify-a-requirement-whose-content-did-not-arrive` | passed | kept | Steps 1 to 4: step 6's `archive --check` names `req-x (statements from paragraph 1; …)` and says archival "would report the findings above and ask". At step 3b, the real archival under a pseudo-terminal asked, and the record says this was because of `req-x`'s content. Whole. |
| `verify-a-withdrawal-the-merge-applied` | passed | kept | Steps 1 to 4: at step 5, with `req-y` deleted, archival printed "ready: archival would archive this change" with `needsSync: false`. At step 3b it still asked, "no longer because of `req-y`". Whole. |
| `verify-a-change-whose-contract-was-applied` | passed | kept | Steps 1 to 4: at step 5, `sync` printed "applied in full" and `archive --check` printed "ready: archival would archive this change", with `needsSync: false`. Whole. |
| `verify-an-agent-reads-the-condition-rather-than-inferring-it` | passed | kept | Steps 1 to 4: `archive --check --json` gave `needsSync` as `true` at steps 1 and 6 and as `false` at step 5. Whole. |
| `verify-a-change-whose-only-unapplied-work-is-a-word-in-the-repeated-run` | partial | kept | Steps 1 to 4: step 1 states only `req-x`, which is held, with one word added. Archival said "has not been synced". The record prints no step 1 notes, so the two `unchecked` paragraphs are right: naming the requirement, and naming the operation. |
| `verify-a-change-whose-only-unapplied-work-is-a-word-end-to-end` | passed | kept | Steps 1 to 4: the test `a_change_whose_only_unapplied_work_is_a_word_has_not_been_synced` (tests/archive.rs:392 at `913a87d`) runs the binary's `archive <dir> --check` through `archive_in`. It asserts "this change has not been synced", and that standard error contains `req-established` and `htmlspec sync --change a-change`. Same trigger, and the whole outcome. |
| `verify-a-change-whose-only-unapplied-work-is-a-withdrawal` | passed | kept | Steps 1 to 4: the test `a_withdrawal_the_merge_did_not_apply_has_not_been_synced` (tests/archive.rs:594) runs `archive --check`. It asserts "has not been synced", that standard error contains `req-withdrawn` and `htmlspec sync --change a-change`, and `needsSync: true`. The change states only the withdrawal, so "otherwise reached" holds because nothing else is stated. That is a condition of the scenario, not its outcome, and the outcome is fully checked. |
| `verify-every-reported-set-is-worked` | partial | kept | Steps 1 to 4: the test `every_set_the_sync_payload_carries_is_named_in_its_instructions` (tests/archive.rs:655) runs `sync --json` and checks that each array key appears in backticks in `workflow_for("sync")`. The `unchecked` paragraph, "with what the agent does about it", is right. |

## 2026-10-04-resolve-a-reference-across-its-change

Records: `the-rule-measured-2026-10-03.md` (`measure.sh` against the built binary, §5.1, and §5.3's
naming of the another-change test), `mutation-testing-2026-10-03.md` (10 mutations and a clean run,
"every test passes"), and `what-the-implementation-revealed-2026-10-03.md` (1,194 tests, none
failing, and the no-specification test's assertion).

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-task-references-a-requirement-in-its-change-s-specification` | passed | kept | Steps 1 to 4: run step 1. The task names `req-example`, which the change's specification declares. The project run exited 0 with 0 errors and no `refs/*`, and so did the tasks document validated alone. Whole. |
| `verify-a-task-references-a-requirement-only-the-accepted-contract-holds` | passed | kept | Steps 1 to 4: run step 2. Only `htmlspec/specs/measure/spec.html` declares the identifier. Exit 0, 0 errors, and no `refs/*`. Whole. |
| `verify-a-change-withdraws-a-requirement-the-accepted-contract-still-holds` | passed | kept | Steps 1 to 4: the test (cli.rs:1380 at `e61e3b1`) validates the tasks document. The accepted `req-y` is unchanged, and the change's `req-y` is `removed`. It asserts `DOCUMENT_VIOLATION`, `refs/removed-target`, and no `refs/unresolved`. Whole. |
| `verify-accepted-specifications-reference-one-another` | passed | kept | Steps 1 to 4: the test (cli.rs:1341) validates `specs/b`, whose evidence unit names `specs/a`'s `req-x`. It asserts `SUCCESS` and no `refs/`. Whole. |
| `verify-an-identifier-two-accepted-specifications-declare` | passed | kept | Steps 1 to 4: the conformance test (references.rs:184) runs `engine::run` on a task placed as a change document, with two accepted projections that both declare `req-x`, and asserts no `refs/*`. The scenario names no command, and resolution is the validator's work, so this is the same subject. Whole. |
| `verify-a-reference-to-another-change` | passed | kept | Steps 1 to 4: §5.3 names `a_task_naming_another_changes_requirement_is_unresolved` as pinning the scenario, and the clean mutation run passes every test. Read at cli.rs:1476: `b-change`'s specification declares `req-x`, and the first change has none. Validating the tasks document gives `DOCUMENT_VIOLATION` and `refs/unresolved`. Whole. |
| `verify-a-runbook-s-document-names-an-accepted-requirement` | passed | kept | Steps 1 to 4: the test (cli.rs:1544), caught under mutation 9, runs on `htmlspec/runbooks/release/tasks.html` naming an accepted `req-y`. It asserts `DOCUMENT_VIOLATION` and `refs/unresolved`. Whole. |
| `verify-a-fragment-link-to-a-unit-in-another-document` | passed | kept | Steps 1 to 4: the test (cli.rs:1495) puts a `#req-x` link in the task's note, where the change's specification declares `req-x`. It asserts `DOCUMENT_VIOLATION`, `refs/unresolved`, and `refs/broken-link`. Whole. |
| `verify-a-reference-resolved-by-the-change-s-specification` | passed | kept | Steps 1 to 4: the CLI test (cli.rs:1404) confirms the specification is invalid. It then asserts that the tasks document validates with `SUCCESS` and that its output never names `specification.html`. The conformance test (references.rs:263) has the same fixture and asserts `is_accepted()` and no `refs/*`. Could a warning still be "about the reference"? At `e61e3b1`, `review/stale-reference` reads only the document's own materially changed units (review.rs `validate`). So a reference into another document can raise nothing but `refs/*`. Drafter's doubt settled: whole. |
| `verify-a-dangling-reference-in-a-change-says-where-it-looked` | partial | kept | Steps 1 to 4: the test (references.rs:286) asserts that there is exactly one `refs/unresolved` and that its message names the three places. It does not check the outcome, the unit, or the identifier, so the three `unchecked` paragraphs are right. |
| `verify-a-reachable-document-that-cannot-be-read` | passed | partial | Step 4. The CLI test (cli.rs:1431) has the scenario's condition, a change specification with an unclosed `strong`. It asserts `DOCUMENT_VIOLATION`, and that the `refs/unresolved` line contains `specification.html`, but not that it calls the specification unread. The conformance test (references.rs:305) does assert "could not be read", but it is given a `Declared` that holds an `Unread` entry it constructs itself, with no markup. Its condition differs (step 3), so it adds nothing. Unchecked: "the report names the specification as the document that could not be read". |
| `verify-a-change-with-no-specification-yet` | passed | kept | Steps 1 to 4: `what-the-implementation-revealed` records the assertion: no `refs/*`, and output that neither names `specification.html` nor says "could not be read". It ties that to the scenario's words, and records the test stage passing. Read at cli.rs:1455, the test asserts this. Whole. |
| `verify-a-design-references-its-change-s-specification` | passed | kept | Steps 1 to 4: the test (cli.rs:1321) validates a design whose decision names the change's specification's `req-x`. It asserts `SUCCESS` and no `refs/`, and the task test at cli.rs:1296 asserts the same. Whole. |
| `verify-validating-one-document` | passed | kept | Steps 1 to 4: run §5.1. The tasks document validated alone reported 0 errors and 3 warnings. That is its own 3 from before the change, and none of the specification's 10 (`the-measurement-2026-09-30.md`, step 1). The reference resolved, so the specification was read. Whole. |

## 2026-10-07-mark-the-words-a-modification-changed

Record: `the-change-measured-2026-10-06.md`. It runs `write_spec.py ins-del` from the first
measurement's appendix A, with `project-model` rebuilt against the branch's parser, `validate`, and the
block-level copy from §3.

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-word-inserted-into-a-statement` | passed | kept | Steps 1 to 4: the projection reads "… status, sync, instructions, and store …" with `deletions: null`. This is the scenario's own statement. Whole. |
| `verify-a-deleted-word-beside-an-inserted-phrase` | passed | kept | Steps 1 to 4: the projection reads "… its active changes, its archive, and its runbooks, each in its own place." with `deletions: ["and"]`. Whole. |
| `verify-a-deleted-paragraph` | passed | kept | Steps 1 to 4: the block-level copy puts `<del><p>…</p></del>` in requirement 0, which is `modified`. Before and after `format`, it projected three statements, and the paragraph appeared in `deletions`. Whole. |
| `verify-a-document-with-no-change-markup` | passed | kept | Steps 1 to 4: 116 staged documents carry no change markup, and `diff -r` against the pre-change baseline exited 0. The check covers 116 documents, so it does not hold vacuously. Whole. |
| `verify-a-word-inserted-into-a-modified-requirement` | passed | kept | Steps 1 to 4: the change's specification, whose requirement is `modified` and states `<ins><code>sync</code>, </ins>`, validated with no `schema/change-markup` and exit 0. Whole. |
| `verify-a-word-and-a-paragraph-deleted-from-a-modified-requirement` | partial | kept | Steps 1 to 4: the `<del>and </del>` was validated with no `schema/change-markup`. The block-level copy was formatted and projected but not validated. The `unchecked` "a paragraph wrapped in `del`" is right. |
| `verify-the-same-markup-outside-a-change-s-specification` | partial | kept | Steps 1 to 4: only the accepted-specification placement was run, giving three errors and exit 1. The four `unchecked` placements are right. |

## Counts

Out of 44 units: kept 36, made partial 1, made passed 0, removed 7.

All seven removals are in `see-a-modification-the-merge-did-not-take`. The one unit made partial is
in `resolve-a-reference-across-its-change`. After editing, both specifications validate with 0
errors and are canonical, and `check_draft.py` reports 0 findings for all three changes.
`see-a-modification`'s `review/stale-verification` count fell from 21 to 14. Its Verification section
and evidence aside still hold, because three units it describes, which rest on end-to-end tests,
remain.
