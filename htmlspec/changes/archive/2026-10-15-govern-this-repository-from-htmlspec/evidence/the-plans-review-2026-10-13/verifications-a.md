# Verification review: reviewer `a`

Eight changes, 38 verification units. Every edit is to a verification unit in the change's
`specification.html`: a `data-outcome`, a `method` or `unchecked` paragraph, or the whole unit.
After each edit, `htmlspec validate` reports 0 errors, `htmlspec format` reports the file canonical,
and `check_draft.py` reports 0 findings. The checker's output differs from the run before the review
only in the verification list and the warning count.

When a unit's record gives a test only by its name, I read the test at the commit named in the row.
"Unchecked trimmed" means the drafter's `unchecked` paragraph ended in a colon and an explanation.
I cut it back to the scenario's or the requirement's own words (rule step 4).

## 2026-08-10-html-spec

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-idempotence-and-canonicality` | passed | partial | Steps 3–4. The record names `properties.rs`, which I read at `778e451`. `formatting_is_a_fixed_point` reaches all of *Second pass changes nothing*. *Two agents converge* is not reached. `any_two_authoring_styles_converge` renders one set of units twice, so every unit keeps its own attribute order and indentation in both documents. The two styles differ only in CRLF line endings and blank lines. That is a nearby case, not the scenario's condition. `data-refs` cannot be edited, so the unit cannot drop that scenario. It becomes partial instead. Unchecked: "two agents that produce the same semantic document with different attribute order and indentation". The method now names the commit and states what the test compares. |
| `verify-absence-of-formatter-churn` | passed | kept | Steps 1–4. Read at `778e451`: `editing_one_unit_leaves_every_other_line_untouched` and `inserting_a_unit_between_two_others_rewrites_neither` check both scenarios' conditions and outcomes. Step 5: the method now names the commit. |
| `verify-check-mode` | passed | kept | Steps 1–4. Read at `778e451`: `check_mode_exits_nonzero_and_writes_nothing` runs `format --check` on an unformatted document through the built binary. It asserts exit 1, "not canonically formatted", the file named, and the file unchanged. That is all of *Check fails* and *Check mode on unformatted files*. Step 5: the method now names the test and the commit. |
| `verify-diff-locality` | passed | kept | Steps 1–4. A recorded harness run against the built formatter. Rows 1 and 3: a sentence edit in a one-line paragraph changes that one line and nothing else. Row 2: with the paragraph one sentence per line, the same edit changes one line and rewraps nothing. |
| `verify-invalid-chart-rejected` | partial | kept | Steps 1–4. The record states that a chart with an unknown mark is rejected. It shows no location and no underlying error, so the unchecked part stands. |
| `verify-no-request-leaves-the-machine` | passed | removed | Step 3: the subject differs (the known doubt, confirmed). The only observation is the access log of a local `python3 -m http.server`. It records requests made to that server. A request to any other host never reaches it, so the log cannot show that no request left the machine. The `file://` run shows the page rendering, not an absence of requests, and networking was never disabled. Nothing of "no request leaves the machine" was reached. |
| `verify-approved-libraries` | partial | kept | Steps 1–4. The log shows the four vendored libraries fetched, and the DOM probe shows one SVG per figure, drawn into a sibling. MathML was not checked. Method reworded: it said "the only scripts the browser requested", which the local log cannot show (see the row above). It now says the log shows the browser fetching the vendored libraries and `htmlspec.js` from the project. |
| `verify-narrow-width` | passed | removed | Step 3: the condition differs. The probe measured overflow at 1100px. `htmlspec.css` at `778e451` caps the page at 62rem and switches to its two-column layout at `min-width: 60rem` (960px). Its narrow-screen rules apply at `max-width: 34rem`. At 1100px the converted document is in its widest layout, which is not "a narrow window". The record's "which is the narrow-width requirement" is the author's reading, not an observation. |
| `verify-diagnostics-explain-themselves` | passed | kept | Steps 1–4. The record reports reading the diagnostics of the deliberately broken fixtures, which are failing documents. It found that each names the rule, the unit, and what to do. One caveat: at `778e451` a parse-recovery diagnostic is document-wide, as the requirement's own migration note says. So "the unit" holds for every fixture except `invalid-parse-recovered-error`. I kept the outcome because the record states it and the document is still a location. |

## 2026-08-13-content-address-runtime-assets

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-scripts-check-passes` | partial | kept | Steps 1–4. §8.1 records the suite passing with `self-assets`, and `self-assets` failing while the duplication stood. It shows neither the reference rule in the suite nor the documents validating individually. Unchecked trimmed. |
| `verify-each-library-appears-exactly-once` | passed | partial | Step 4: vacuous. §8.2 ran `htmlspec assets .` at `86c33f4` and found each of 8 stored assets once. But at `86c33f4` only one document in the project references the store: `deterministic-formatting.html`. `README.html` inlines its assets, `assets/template.html` is a template, and task 7.1's deviation says the same. So "once in the project rather than once per document" is vacuous for documents. Unchecked: "a project containing many documents". |
| `verify-a-fabricated-address` | partial | kept | Steps 1–4. §8.4 shows `asset/unresolved` and exit 1. It does not show the document or the reference. Unchecked trimmed to "reports the document, the reference". |
| `verify-stored-bytes-replaced-in-place` | passed | kept | Steps 1–4. §8.4 shows `asset/mismatch`, exit 1, and `asset/unresolved` fired zero times: validation fails, and reports a mismatch rather than an absent asset. |
| `verify-a-second-copy-of-one-library` | partial | kept | Steps 1–4. §8.4 shows `asset/duplicate` and exit 1 from the project-level pass. It does not show the locations being named. Unchecked trimmed to "names both locations". |

## 2026-08-14-separate-artifact-kinds

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-task-in-a-specification-is-reported-and-named` | passed | kept | Steps 1–4. Read at `9c3a3c7`: the test asserts one `kind/task-in-specification` error, located at the task, with a message about the obligation. An error fails validation whatever other rules run. §2 and §6 record the same rule firing through the built binary. |
| `verify-a-tasks-document-with-no-task-is-reported` | passed | kept | Steps 1–4. Read at `9c3a3c7`: the test asserts one document-wide `kind/no-tasks` error. §2 records the rule firing through the binary. "Because the document enumerates no work" gives the scenario's reason; it is not an output the scenario requires. |
| `verify-a-tasks-document-carrying-other-units-is-reported-for-none-of-them` | passed | partial | Step 4. Read at `9c3a3c7`: the test runs the kind rules alone (`&[validate]` from `kinds.rs`) and asserts no diagnostic. That shows the completeness rule says nothing about the decision. It does not show that validation as a whole says nothing about it. No record runs full validation on such a document, and the dogfood tasks document carries no decision. Unchecked: "validation reports nothing about the decision". |
| `verify-a-proposal-carrying-every-unit-draws-no-diagnostic` | passed | partial | Step 4. §4's run of the built binary validates a proposal only. *Guidance departed from* names "a proposal or a design", and no record validates a design. The kinds test that does is not in any record, and §1 only counts the fixtures. Unchecked: "a design carries a unit outside its intended content". The proposal reaches the other three scenarios. |
| `verify-an-unverified-requirement-in-a-proposal-reports-nothing` | passed | kept | Steps 1–4. §2 records two documents through the binary: the same requirement fires in a specification and draws nothing in a proposal. |
| `verify-strict-mode-does-not-gate-an-unverified-specification` | passed | kept | Steps 1–4. §5's `--strict` run shows the warnings and "0 errors, 3 warnings: the document is accepted", which is all of both scenarios. |
| `verify-an-invalid-specification-stops-archival` | passed | kept | Steps 1–4. Read at `9c3a3c7`: the test drives `htmlspec archive` through the binary and asserts a nonzero exit, `refs/unresolved` reported, and "does not proceed". |
| `verify-an-invalid-specification-is-never-archivals-output` | partial | kept | Steps 1–4. The same test shows the program refusing. The agent reconciling the error is not something it reaches. |
| `verify-the-archive-operation-ships-no-requirement-extractor` | passed | kept | Steps 1–4. §6 states what the tests assert: a written specification is validated, and SHALL/MUST prose in a proposal and design produces no specification. I also read the tests at `9c3a3c7`, and they match the record. |
| `verify-init-change-creates-a-valid-change-directory` | passed | kept | Steps 3–4. Read at `9c3a3c7`: the test calls `init_change`, the command function that `init --change` dispatches to. It asserts that the three documents are written, each declares and shows its kind, each validates, and no specification is written. I judged that function to be the command, not a helper the command uses: only argument parsing is skipped. §6 also records `init --change` run through the binary to create each archive test's change. |
| `verify-a-document-declaring-no-kind-is-invalid` | passed | kept | Steps 1–4. Read at `9c3a3c7`: the test validates a document with no `htmlspec-kind`. It asserts that the document is not accepted and that the report names `kind`. |

## 2026-08-18-conform-generated-frontmatter-names

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-name-that-is-not-an-invocation` | passed | kept | Steps 1–4. The regeneration after the fix ran the built binary. Claude Code's and Pi's command files, whose invocations carry `/` and `:`, changed `name:` from the invocation to the workflow's directory name. The scenario names no tool, so Cursor not being run leaves no part of it unreached. |

## 2026-08-18-install-the-full-workflow-set

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-what-each-tool-received` | passed | partial | Step 4. The record counts each tool's files: 7 skills and 7 commands each, and 7 skills with no commands for Codex. That reaches *Installing for a tool without commands* and "one of each per workflow". It shows neither where the files went nor that the tool reads them there. Unchecked: "written where that tool reads them". |
| `verify-section-order` | passed | kept | Steps 1–4. The region's seven headings, with their line numbers, run from explore to archive in lifecycle order, and so name each stage. |
| `verify-version-stamping` | partial | kept | Steps 1–4. The grep covered the 49 skill and command files. The `AGENTS.md` region was not checked. |
| `verify-the-closing-message` | passed | kept | Steps 1–4. The captured message states each spelling with its tools and names `new`. At `5f7e5cf`, `new` is the workflow set's flagged `entry_point`, so it is "the one that begins the lifecycle" in the requirement's sense. It is not archival. |

## 2026-08-21-check-release-hygiene

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-changelog-with-no-entry-for-the-declared-version` | passed | kept | Steps 1–4. The 6.1 run, with no changelog heading for the shipped `0.1.0`, fails and names the version. |
| `verify-two-archives-claiming-one-version` | passed | kept | Steps 1–4. The 6.2 run, with two archives at `0.2.0` above the baseline, fails with the distinctness fault. |
| `verify-an-archive-with-no-recorded-version` | passed | kept | Steps 1–4. The 6.3 run fails and names the one archive. |
| `verify-seven-archives-recording-no-version` | passed | kept | Steps 1–4. The 1.5 run of the built check names each of the seven archives and exits 1. It comes before the data was fixed, not before the check was built, so it does not predate the build. |
| `verify-seven-archives-at-the-baseline` | partial | kept | Steps 1–4 (the known doubt). The 6.1 output itself shows "1 fault(s) across 7 archived change(s)", and that fault is the changelog's. So there is no distinctness fault while seven archives record `0.1.0`. The explanatory sentence adds only that the seven still recorded `0.1.0`, and the 6.2 section ("a duplicate at 0.1.0 is correct") agrees. That reaches "the suite treats them as one release". Unchecked trimmed to "one changelog entry names them all". |

## 2026-08-21-distinguish-command-names-from-skill-names

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-repeated-name-is-caught-before-release` | passed | kept | Steps 1–4. The check ran on a tree that wrote each name twice, failed, and named each repeat for each tool. The defect being present is the scenario's condition, so the run before the fix does not predate what it checks. |
| `verify-a-workflow-is-listed-once-per-tool` | partial | passed | Step 4. The regeneration shows Claude Code's command files without `name:` while its skills keep theirs, so the name is declared once across the two. The 2026-08-21 session load shows each workflow listed once. That is the whole scenario for a tool that receives both. The drafter's unchecked part was the same scenario for Cursor, which is not a part of the scenario: it says "for one tool" and names none, and the gap could not be written in its words. Task 4.5's "must not be reported as verified" concerns whether Cursor's loader accepts a command file with no name, which is a different claim. |

## 2026-08-24-pass-the-description-through

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-an-invocation-carrying-a-description` | passed | kept | Steps 1–4. In the live Pi run, the change was created on the first exchange, with no exchange spent asking. Its content came from the description, not from the example that ships with the workflow. |

## Counts

- Kept: 30. Seven of these had a `method` or `unchecked` paragraph edited: three methods, four trims.
- Made partial: 5.
- Made passed: 1.
- Removed: 2.

The Verification section in 2026-08-10 still holds units, so its migration-note aside is unchanged.

`2026-08-21-distinguish-command-names-from-skill-names/tasks.html` also changed during this review,
at task 5.3's migration note, around 14:21. I did not make that edit.
