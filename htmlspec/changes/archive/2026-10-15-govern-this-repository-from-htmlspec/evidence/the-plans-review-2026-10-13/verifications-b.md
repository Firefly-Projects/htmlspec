# Verification review: reviewer `b`

Changes: `2026-08-17-project-bootstrap-cli`, `2026-08-24-report-in-the-contributors-terms`,
`2026-09-02-carry-task-progress-into-the-listing`. Steps are the brief's rule: 1 a record,
2 the requirement is this change's, 3 the same check, 4 how much of it, 5 the method.

One standard is applied throughout. A run, probe or test the record shows or names counts as the
check. A record's own conclusion that goes beyond what it shows ("Nothing was written", "Codex
correctly received skills") reaches only as far as the shown check. This is the drafter's own
standard for *Re-running on an established project*, which they left unwritten because "the
record asserts the outcome with no check shown".

After each edit, from S: `htmlspec validate` 0 errors, `htmlspec format` canonical, and
`check_draft.py` 0 findings, for all three specifications.

## 2026-08-17-project-bootstrap-cli

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-first-run-bootstrapping-an-empty-repository` | partial | kept | 4: the output reports "created the project structure" and one store in `htmlspec/assets`, and lists no directories. The record's later "specs/, changes/, and archive/ are untouched" is a conclusion with no listing shown. The unchecked stands. |
| `verify-first-run-bootstrapping-writes-the-declaration` | partial | kept | 4: the output shows `spec.nix` at the root and version `0.0.0` only. Name and program unchecked. |
| `verify-first-run-naming-the-next-command` | passed | kept | 3–4: `--tools claude` (one tool), and the output ends naming `/htmlspec:archive`, which the same record labels as Claude Code's spelling. |
| `verify-first-run-creating-a-change` | passed | kept, method amended | 4: the drafter flagged "each declaring its kind" as resting on `validate` 0 errors. I read `crates/htmlspec-validate/src/rules/schema.rs` at `d88ca93` (this change's build; also at `d88ca93~1`): `a_missing_required_property_is_rejected` shows that a document with no `htmlspec-kind` is not accepted. So the recorded 0 errors reach the declared kind. The method now says this, per step 5. |
| `verify-first-run-interactive-selection` | passed | kept | 3–4: pty, `.claude/` and `.pi/` present, all five tools listed, with those two pre-checked. The prompt appearing at all shows that no selection argument was given. |
| `verify-first-run-declining-at-the-prompt` | passed | **partial** | 4: the check shown is `ls htmlspec` → no such directory. "Nothing was written" is the record's conclusion from that probe. Unchecked: "That the program makes no further change: the record's probe looks only for the `htmlspec` directory, and does not look for `spec.nix` or any tool's instructions." |
| `verify-first-run-cancelled-operation` | partial | kept, unchecked added | 4: the scenario's WHEN is "declines **or interrupts**". The record declines with Escape and never interrupts. Added: "That the program exits with that code when a contributor interrupts an interactive operation: …". The distinct-from-violations/failure unchecked stands. |
| `verify-first-run-tools-with-different-spellings` | passed | kept | 3–4: Claude Code and Codex selected, and the shown output gives two labelled lines. |
| `verify-first-run-installing-for-a-tool-without-commands` | passed | **removed** | 1: one of the coordinator's known doubts. "Codex correctly received skills and no command file" is a sentence with no listing, probe or output behind it. The run's shown output says nothing about Codex's files. The record does not describe a check of what Codex received, and the drafter refused the same kind of sentence elsewhere. I was unsure, so I removed it. |
| `verify-first-run-no-terminal-available` | passed | kept | 3–4: I re-read `init_does_not_block_when_no_terminal_is_attached` at `d88ca93` (`tests/cli.rs:423`). With `.claude/` present, it runs `init` with no `--tools` under `Command::output` (stdin not a terminal) and asserts success in under 30 s, `detected: Claude Code`, and the skill written. That matches the method. |
| `verify-first-run-the-withdrawn-spelling` | passed | kept | 3–4: `init doc.html` exits 2. The message says the subject is the project and names `new document proposal doc.html`. That covers both *A withdrawn spelling…* and *Initializing a proposal* (WHEN: names a document path). |
| `verify-first-run-re-running-is-safe` | partial | kept | 3–4: the design ("Whether init writes a configuration file. It writes spec.nix") makes `spec.nix` the configuration, so both scenarios have the same subject. Only the edited line is read back. |
| `verify-asset-footprint-one-store-for-four-changes` | partial | kept | 3–4: four `new change` runs, `du` 6.1 MB, and no `changes/*/assets`. A `new change` creates the documents, so the trigger fits *A created document…*. References were not read, so the unchecked stands. |
| `verify-asset-footprint-checking-a-project-s-assets` | passed | **partial**, method amended | 4: the WHEN is "runs **anywhere** in a project", and the requirement is about resolving from any directory beneath the root. The record shows one run. I read the named test `the_asset_check_reports_duplication_between_two_stores` at `d88ca93`, where the testing record has every test passing; the asset record names no commit. The test passes the project's root (`run(&["assets", root])`). Neither runs the command from beneath the root. Unchecked: "That the command checks every asset the project stores when it runs anywhere in the project: …". The method now says what the test checks and the commit. |
| `verify-asset-footprint-an-orphaned-change` | passed | kept | 3–4: a copied change validated `--strict` gives 21 errors, where it gave 0 with 21 dead references before. That covers the strict scenario whole. For the plain scenario, a strict run is still a validation. Strict mode only promotes warnings (`engine.rs:207` at `d88ca93`), and the record ties the 21 errors to the 21 ascending references. |
| `verify-openspec-divergence-no-terminal-nothing-detected` | passed | kept, method typo fixed | 3–4: `init < /dev/null` with no argument and nothing detected proceeds and reports ready. Built from `prompt-selection-guards`, after the build. I added the missing space in "group 14:<code>". |
| `verify-testing-state-a-version-that-has-not-moved` | passed | **partial** | 3–4: I re-read the test at `d88ca93`. Nothing is archived, so `last_archived_version` (`commands.rs:1331`) returns `unwrap_or(initial)`. That is the fallback this specification's own class-B note says the gate always took. No test compares against a version an archived change carries. Unchecked: "That archival does not complete while the declared version equals the last archived version of a project that has archived a change: the test archives nothing first, so the last archived version it compares against is the initial one." |
| `verify-testing-state-a-project-s-first-archival` | partial | kept | 4: refused at `0.0.0` with nothing archived. No later archival is refused for comparison, so "on the same grounds as any later archival" stays unchecked. |
| `verify-testing-state-an-advanced-version` | passed | **partial** | 3–4: the same test, after `advance_version`. Exit 0 against the initial version, nothing archived. Same reasoning as the not-moved unit. Unchecked: "That the version gate permits archival to proceed when the declared version is ordered after the last archived version of a project that has archived a change: …". |
| `verify-testing-state-a-version-that-went-backwards` | passed | kept | 3–4: archived `…-0.9.0-…` against declared `0.1.0`. Exit 2, with "ordered *before*" and `0.9.0`. The two messages differ in the code (`commands.rs:1299–1316`) and in what the two tests assert. |
| `verify-testing-state-an-unorderable-version` | partial | kept | 4: asserts "cannot be ordered", not `tuesday`. |
| `verify-testing-state-an-absent-declaration` | partial | kept | 4: asserts only `htmlspec init` in the message. |
| `verify-testing-state-the-declaration-is-not-written` | passed | kept | 3–4: `spec.nix` is unchanged across both a refusal and a success. |

## 2026-08-24-report-in-the-contributors-terms

All three records are runs at `819d07a`, made after the bodies were amended. `the-report-received` predates the build and carries no unit.

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-the-new-report-against-the-register-obligation` | partial | kept, unchecked added and one reworded | 4: the report's literal first line is "All three documents validate clean.", a check result. The record measures "the first claim after the one-line preamble". Added: "that report's first claim stated in the vocabulary of what the contributor asked for: …". The check-result unchecked said "the record does not say where the report states it", but the transcript shows it in the opening line. It is reworded to say the record does not judge that line. |
| `verify-the-archive-report-against-the-register-obligation` | partial | kept, unchecked added | 4: the report opens "The archival is complete.", and the record takes the first claim to be what follows. Added the same first-claim unchecked. The other four stand. |
| `verify-the-new-report-quotes-the-proposal-s-sequence` | passed | kept | 3–4: the record quotes the proposal's sequence and shows the report reproducing it near-verbatim. |
| `verify-the-new-report-moves-artifact-facts-to-the-close` | partial | kept, unchecked reworded | 4: where the change landed is shown in the closing section. What a check reported appears in the opening line, which the record sets aside. The reason now says so instead of "does not say where". |
| `verify-the-new-report-glosses-its-task-order` | passed | kept | 3–4: an ordered list of five identifiers, each glossed in the same sentence, all checked against `tasks.html`. |
| `verify-the-apply-report-glosses-its-task-states` | passed | kept | 3–4: five task-state lines, the identifiers are the claim, and each is glossed on its line. |
| `verify-the-apply-report-against-the-gloss-obligation` | partial | kept | 4: identifiers and counts with their kinds are reached. Self-describing references are not addressed; the report in fact glosses `index.html` and others, and the record is silent on it. |
| `verify-the-new-report-leaves-a-command-unglossed` | partial | kept | 4: the record names a command and a skill name only. Flag and file path unchecked. |
| `verify-the-apply-report-against-the-settling-obligation` | partial | kept | 4: "what remains for the contributor" holds only vacuously, since no question was left. |
| `verify-the-apply-report-offers-its-added-scope-for-removal` | passed | kept | 3–4: "I added … placeholders … say the word if you want them removed". |
| `verify-the-new-report-keeps-one-open-question` | passed | kept | 3–4: "What genuinely needs you" is kept apart from "Decided, and cheap to change", and the record judges that the question changes the shape of the work. |

## 2026-09-02-carry-task-progress-into-the-listing

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-change-s-progress-is-visible-in-the-listing` | passed | kept | 3–4: `htmlspec list` over the scratch project, built at the branch through `prove-listing.py`. `beta` reads `1/3 in-progress`. |
| `verify-outstanding-work-that-was-abandoned` | passed | kept | 3–4: `gamma` (all dropped) reads `0/2 done` with `"dropped": 2`, against changes with todo tasks that read `in-progress`. The scenario asks for distinguishability, not for the same fraction. |
| `verify-the-enumeration-and-the-completion-report-agree` | passed | **partial** | 4: this is the drafter's flagged tension. The numeric comparison reaches the four changes that have a tasks document. For `epsilon-no-tasks`, `status` gives `total=0` and the listing gives no counts, and the script scores that as AGREE by construction. The record calls it a difference "in what they are able to express", which is not "the same counts". No change holds a task that declares no state, and the requirement singles out that case for agreement ("the two SHALL NOT differ"). Two unchecked paragraphs record these. |
| `verify-a-change-with-no-tasks-document` | passed | kept | 3–4: `no tasks` in the human report, and the payload is `{ "name": "epsilon-no-tasks" }`. |
| `verify-a-change-whose-contract-has-not-been-synced` | passed | kept | 3–4: the shown check-mode output says "This change has not been synced" and names `htmlspec sync --change unsynced-change`. |
| `verify-the-question-asked-about-an-unsynced-change` | passed | **removed** | 3: the named test `the_question_names_sync_for_an_unsynced_change` (re-read at `9af58cf`) calls the pure function `question_for`. That is a unit test of a function the command uses, and the brief names it as a different trigger from "archival asks". The record's quoted question is not said to come from a run. It lacks the `inquire::Confirm` decoration a prompt would show (`prompt.rs:166`). Task 6.6 lists what was confirmed against a real project, and the question is not among them. Unsure, so removed. |
| `verify-a-change-whose-contract-was-applied` | passed | kept | 3–4: the shown output after the specification was copied into `specs/` has no mention of syncing. |
| `verify-an-agent-reads-the-condition-rather-than-inferring-it` | passed | kept | 3–4: the shown payload `{ "needsSync": true, … }` names `req-example`, the real change's requirement, not the unit test's `req-gone`. So it came from the real project, not from the test. |

## Counts

| | bootstrap-cli | report-in-terms | carry-task-progress | Total |
| --- | --- | --- | --- | --- |
| Reviewed | 23 | 11 | 8 | 42 |
| Kept | 18 | 11 | 6 | 35 |
| Made partial | 4 | 0 | 1 | 5 |
| Made passed | 0 | 0 | 0 | 0 |
| Removed | 1 | 0 | 1 | 2 |

Of the 35 kept, 6 were amended with no change of outcome: 4 gained or had reworded `unchecked`
paragraphs (`verify-first-run-cancelled-operation`, the two register units, and
`verify-the-new-report-moves-artifact-facts-to-the-close`), and 2 had their method amended
(`verify-first-run-creating-a-change`, and the divergence unit's missing space). Every Verification
section still holds units, so no section, nav entry or aside changed.

Warnings after review: bootstrap-cli 68 (was 69), report-in-terms 22, carry-task-progress 16 (was
17). All are `review/*`; each drop is one `review/stale-verification` from a removed unit.
