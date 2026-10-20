# Verification review: reviewer d

Changes: establish-a-runbook, record-a-renamed-requirement, confine-the-asset-check-to-the-project,
keep-evidence-out-of-document-discovery, reject-change-markup-inside-a-unit. 41 units reviewed.
Every test named below was read with `git -C . show <commit>:<path>`.
After the edits, each edited specification has 0 errors, is canonical, and has 0 checker findings.
Carried files are intact.

## 2026-09-18-establish-a-runbook

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-bootstrapped-project-has-the-place` | passed | removed | Step 3. The scenario's WHEN is "a contributor bootstraps a project", which runs the bootstrap operation. `establishing_a_project_creates_the_directories_it_is_made_of` (read at `224acfa`) is a unit test of `ProjectRoot::establish`, a function bootstrap calls, so its trigger is different. No carried record runs `htmlspec init` on the built change and looks at `runbooks/`. `the-location-is-blocked` measurement 2 runs `init` only after `mkdir -p htmlspec/runbooks`, and the cookie recipe's `init` predates the build. |
| `verify-bootstrapping-an-empty-repository` | partial | removed | Step 3, for the same reason. It is the same `ProjectRoot::establish` unit test, and the scenario's WHEN is "a contributor bootstraps a project". |
| `verify-a-procedure-in-the-canonical-document` | passed | kept | Steps 1 to 4. The mutation record names the test and the clean 245-test suite. I read `the_two_document_split_is_forced_by_validation` at `224acfa`: it puts a task unit into a runbook's specification, runs `validate_source` (the validation the `validate` command runs), and asserts the document is not accepted and that the report names `kind/task-in-specification`. That is the whole of the WHEN and the THEN. |
| `verify-a-runbook-validated-by-the-ordinary-path` | partial | kept | Step 4. I read `a_runbooks_documents_validate_as_documents_of_their_kinds` at `224acfa`. It runs `validate_source` over both of a runbook's documents and asserts each is accepted, default and strict. Nothing in it examines which rules read a document. The unchecked "with no rule reading the fact that they belong to a runbook" stands. |
| `verify-a-verification-in-a-runbook-s-specification` | passed | kept | Steps 1 to 4. `the-canonical-documents-change-state` records it: the release runbook's specification, which states hazards measured before the runbook existed, drew three `review/stale-verification` warnings with its units marked `added`. With them marked `unchanged` it drew none, and ended at 0 errors and 0 warnings. That is the whole THEN. Step 5: the method's "through the real binary" is not the record's wording, so it now says "validated, as the record states". |

## 2026-09-26-record-a-renamed-requirement

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-rename-on-a-new-requirement` | passed | kept | Steps 1 to 4. `the-implementation` 4.2 records that each invalid fixture, run through the built binary, reports exactly its one error. I read `invalid-schema-rename-on-added` at `51c5aea`: an `added` requirement carrying `renamed-from`. |
| `verify-a-rename-on-a-withdrawn-requirement` | passed | kept | Steps 1 to 4. The same run. The fixture, read at `51c5aea`, is a `removed` requirement carrying `renamed-from`, and it reported its one error. |
| `verify-a-rename-outside-a-requirement` | passed | kept | Steps 1 to 4. 4.1 states the assertion of `a_rename_outside_a_requirement_is_reported_once_per_unit` and that all twelve tests pass. I read it at `51c5aea`: one `schema/rename-outside-requirement` for each of the six units the scenario lists, and that rule is an error. The decision fixture's run through the binary reports its one error. |
| `verify-two-previous-headings` | passed | kept | Steps 1 to 4. The 4.2 run. The fixture, read at `51c5aea`, holds two `renamed-from` paragraphs in one requirement. |
| `verify-a-rename-to-the-same-name` | passed | kept | Steps 1 to 4. The 4.2 run of `invalid-schema-rename-to-same-name`. |
| `verify-a-heading-that-shows-its-identifier` | passed | kept | Steps 1 to 4. The same run. I read the fixture at `51c5aea`: its heading is `The example requirement <span class="hs-id">req-example</span>` and `renamed-from` states `The example requirement`. That is the scenario's condition exactly. |
| `verify-a-rename-the-accepted-specifications-have-not-taken` | passed | kept | Steps 1 to 4. The mutation record names the CLI test, and `the-runbook-ran` step 2 records `scripts/check` green. I read `a_change_whose_only_unapplied_work_is_a_rename_has_not_been_synced` at `51c5aea`. It runs `sync` on the real binary and asserts exit 0, "0 of 1", no "applied in full", and the `renamed` line with both headings; `--json` gives `reached` empty and the one untaken rename. |
| `verify-a-rename-the-accepted-specifications-have-taken` | passed | kept | Step 4. The test rewrites the accepted heading and asserts `sync` exits 0 and prints "applied in full". At `51c5aea`, `commands.rs` prints that line only when both `absent` and `untaken_renames` are empty, so for a change stating one requirement it is the report that `req-x` reached and is in no other set. This was the closest call in the change. The test does not assert the "reached" line itself. |
| `verify-a-structured-report-carries-the-untaken-renames` | passed | kept | Steps 1 to 4. The `--json` assertion is exactly one entry with `id`, `heading`, and `acceptedHeading`. |
| `verify-a-structured-report` | partial | kept | Step 4. The `--json` run asserts `reached`, `absent`, and `untakenRenames`, and not `stated`. The unchecked "the stated identifier set" stands. The scenario is one this specification states (marked `modified`), so step 2 holds. |
| `verify-a-change-whose-only-unapplied-work-is-a-rename` | passed | kept | Steps 1 to 4. The mutation record names it as the only catcher of mutation 18. Archival asserts "this change has not been synced", names `req-established`, both headings, and `htmlspec sync --change a-change`; `--check --json` gives `needsSync` true. |

## 2026-09-28-confine-the-asset-check-to-the-project

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-the-toolchain-copy-in-this-repository` | passed | kept | Steps 1 to 4. `the-measurement` 4.2: the built binary, run on the same clone after the same `init`, reported "each stored asset appears once" and exited 0. Section 1.1 of the same record shows that clone's root `./assets/` holding the eight runtime files that `init` put in `./htmlspec/assets/`, so the GIVEN is met. |
| `verify-the-behavior-before-this-change-restored-by-mutation` | passed | kept | Steps 1 to 4. The mutation record names both tests as catchers of the old behavior, and `the-runbook-ran` records `scripts/check` passing all 13 stages. I read both at `dfb2b04`. The fixture test copies the project's store into `fixtures/conformance/a-fixture/assets`, runs `assets`, and asserts SUCCESS and "checked on its own" naming that store. The toolchain test asserts SUCCESS, `accepted: true`, and one project store. Each covers its scenario whole. |
| `verify-a-partition-that-excuses-every-store-restored-by-mutation` | passed | kept | Steps 1 to 4. I read both tests at `dfb2b04`. `two_stores_inside_the_project_are_still_compared` asserts DOCUMENT_VIOLATION and that both store paths are named. `the_asset_check_reports_duplication_between_two_stores` asserts DOCUMENT_VIOLATION and that both stores are named. Step 2 holds for the unchanged "Duplicate copies of one library", because this specification states it. On step 3, its "the project is validated" is the `assets` check: at `dfb2b04` the topology check runs only from `assets` and promotion. |
| `verify-a-second-store-planted-inside-the-project` | partial | kept | Step 4. 4.2 gives `8 duplicated asset(s)` and exit 1, and not the locations. The unchecked paragraph now holds only the scenario's words, "names both locations". Its explanation moved into the method, where steps 4 and 5 put it. |

## 2026-09-29-keep-evidence-out-of-document-discovery

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-record-in-this-format-at-a-superseded-version` | passed | kept | Steps 1 to 4. `evidence-read-as-documents` "After: task 5.1": `validate` exited 0 with `3 document(s): 0 error(s), 5 warning(s)`, the three being the change's own documents. The 5 warnings are theirs (1+1+3 in the "before" table), so no diagnostic names a record. |
| `verify-evidence-nested-below-the-evidence-directory` | passed | kept | Steps 1 to 4. The same run, with four records in `evidence/converted/`. None of them was among the 3 documents. |
| `verify-a-capability-that-happens-to-be-named-evidence` | passed | kept | Steps 1 to 4. The mutation record names the test as a mutation-1 catcher. I read it at `6a591bd`: a non-conforming specification at `htmlspec/specs/evidence/spec.html`, `validate` with no path, and assertions of DOCUMENT_VIOLATION and that the report names the file. The review revised `format`'s writing, not the walk `validate` uses. |
| `verify-an-archived-change-keeps-its-evidence` | passed | kept | Steps 3 and 4. Promotion's conformance check is an operation that collects a project's documents by walking a directory. On 0.17.0 it refused on exactly these records, and against the built change every check passed. So the file was not among them. The staged tree held no other document, but this scenario states no "every", so the outcome holds without vacuity. |
| `verify-validating-a-project-that-holds-html-evidence` | passed | kept | Steps 1 to 4. `validate` with no path, exit 0, the three conforming documents, and no diagnostic for a record. The GIVEN's "documents all conform" has three real documents behind it. |
| `verify-formatting-leaves-evidence-untouched` | passed | kept | Steps 1 to 4. The run shows `format` leaving all seven byte-identical and `format --check` exiting 0. The test the method cites, read at `6a591bd` and named as a mutation-3 catcher, runs `format` and then `format --check` in the scenario's order, and asserts both. |
| `verify-naming-an-evidence-file-checks-it` | passed | kept | Steps 1 to 4. `validate <one record>` exited 1, and the record states that a contributor asking about a record "still gets the ordinary answer, including its `asset/unresolved` errors". |
| `verify-naming-the-evidence-directory-checks-what-it-holds` | passed | kept | Steps 1 to 4. `validate <the change>/evidence` exited 1, and the record states "the records are checked when named". |
| `verify-formatting-a-named-record-is-refused` | passed | kept | Steps 1 to 4. The mutation record names the test as the catcher of post-review mutation 7. I read it at `6a591bd`. For the record, the evidence directory, and `evidence/transcripts` it asserts OPERATIONAL_FAILURE, "refusing to write" with the record's name, "nothing was written", and unchanged bytes. Then `format --check` gives DOCUMENT_VIOLATION. Each named path holds only that record, so the violation is that file being reported. |
| `verify-outside-a-project-every-file-is-a-document` | passed | kept | Steps 1 to 4. I read the test at `6a591bd`: `evidence/a-record.html` in a scratch directory that is not a project, `validate` naming the directory, and DOCUMENT_VIOLATION asserted. The record's dangling reference fails wherever the file sits (what-revealed section 3). |
| `verify-carried-evidence-in-the-destination-format-does-not-gate-promotion` | passed | partial | Step 4. The run reached "report nothing about that file, promotion proceeds, and the file arrives byte-identical". The staged project held no document besides the three records, so "a staged project whose documents all conform" holds only vacuously. That is the brief's own example. Unchecked: "a staged project whose documents all conform". The method now says the run had no conforming document beside the records. |

## 2026-10-02-reject-change-markup-inside-a-unit

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-deleted-phrase-and-an-inserted-word-in-one-statement` | passed | kept | Steps 1 to 4. `the-rule-measured` gives the second run, after the review's fixes. The statement `write_docs.py` writes is the scenario's. It exited 1 with 2 errors, and `--json` gave two `schema/change-markup` at `req-the-spellings-are-fixed`. |
| `verify-a-deleted-paragraph-in-a-requirement` | passed | kept | Steps 1 to 4. The block-level `<del><p>`: 1 error at the requirement. |
| `verify-a-unit-wrapped-in-change-markup` | passed | kept | Steps 1 to 4. The requirement wrapped whole in `del`: 1 error saying the `del` encloses `req-no-other-spelling`. |
| `verify-any-unit-of-any-kind-in-the-measurement` | partial | kept | Step 4. Only a verification in a specification was measured. Both unchecked paragraphs stand. |
| `verify-change-markup-outside-every-unit` | passed | kept | Steps 1 to 4. The proposal with markup in four places outside every unit: exit 0, no diagnostics. |
| `verify-an-inserted-word-with-nothing-marked-as-removed` | passed | removed | Step 1. The record's only statement that this test passed is 5.2's "After the last restore, both suites passed". That run was against the rule as first committed (`8032003`, whose `change_markup.rs` equals `ab54590`'s). The review then rewrote that same rule, 237 lines of `change_markup.rs` in `6fd228e`, including the `ins` reason. The post-review section says only that the ten mutations are "still caught". Unlike 5.2, it does not state the clean run that `mutate.py` prints, and it names no catchers for them. So no record states a pass against the build this specification describes, and the pass would be inferred. The drafter applied the same reasoning to "A change element that is itself a unit" and wrote no unit for it. I confirmed the test body is identical at `8032003` and `6fd228e`. I was unsure, so I removed it, as the brief directs. |
| `verify-change-markup-in-a-nested-unit` | partial | removed | Step 1, for the same reason. Had it stayed, it would have been partial: the test asserts only the location, not an error or a rejection. |
| `verify-any-unit-of-any-kind-in-the-tests` | passed | removed | Step 1, for the same reason. On steps 3 and 4 the test does reach every unit in every kind, so it would have been passed if a post-review pass were recorded. |
| `verify-a-document-declaring-an-earlier-contract-version` | passed | removed | Step 1, for the same reason. Mutation 8's catchers, this test and the 0.1.0 fixture's harness tests, are named only in the pre-review 5.2 run. The measured documents declare the template's version, 0.3.0 at 0.19.0, so no surviving unit covers this scenario. |
| `verify-a-struck-through-phrase-is-not-change-markup` | passed | removed | Step 1, for the same reason. It is a silence test, so passing against the empty rule (2.2) shows nothing. Its only meaningful pass is 5.2's, which was pre-review. |

## Counts

- Kept: 33
- Made partial: 1
- Made passed: 0
- Removed: 7

Method and unchecked wording was edited, with the outcome unchanged, in
`verify-a-verification-in-a-runbook-s-specification` and `verify-a-second-store-planted-inside-the-project`.

## For the coordinator

Check the five removals in reject-change-markup first. If a pass recorded against the rule as first
committed, with the test unchanged since, is judged enough for the archived build, restore four of
them as written and `verify-change-markup-in-a-nested-unit` as partial. Copies of each specification
as it was before this review are in
the review session's scratch directory
(`<change>.before.html`, and `runbook-spec.before.html` for establish-a-runbook). A `diff` against
them shows the exact blocks.
