# Verification review: reviewer f

Eight changes, 54 verification units. Each was judged in the brief's order: record (1), requirement
of this change (2), same check (3), how much (4), method (5). Where a record gives a test only by
name, the test was read at the commit named. Every edited document validates with 0 errors and is
canonical, and `check_draft.py` reports 0 findings on all eight changes.

## 2026-09-26-log-a-runbook-run

Every test was read at `16df91e`, the squashed merge commit. It is the only commit that holds the
record, because the branch's own commits were not kept. `the-review-fixes-2026-09-20.md` lists the
tests the review changed, and none of these five is among them.

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-run-s-record-created-by-the-operation-that-starts-a-run` | passed | kept | 1–4: the mutation record itself states the three assertions: no `added` survives, every `data-change` reads `unchanged`, and no `review/stale-verification`. The test reaches them through `run --as`, the operation that starts a run. The whole Then. |
| `verify-a-record-offered-where-a-change-is-expected` | partial | kept | 3–4: archival pointed at `runs/` by three spellings refuses, which is "the place holding them". No route points at a record file, so "a run's record" stays unchecked. |
| `verify-a-run-started-over-an-existing-record` | passed | kept | 3–4: read at `16df91e`, the test runs `run --as first` twice and asserts a nonzero exit, "already exists", and the bytes unchanged. Mutation 6 fails it. The whole scenario. |
| `verify-a-run-under-way` | passed | kept | 3–4: `run --check` over a just-started record exits 0 and names both requirements and the record. At `16df91e`, `run` takes no strictness option, so "the strictest setting available" is the only setting it has. |
| `verify-a-run-that-recorded-a-failure` | passed | kept | 3–4: every outcome is rewritten to `failed`, then `run --check` exits 0 with no `review/unrecorded-requirement`. Mutation 4 fails it. The whole Then. |

## 2026-10-08-migrate-into-the-extended-vocabulary

Each unit was judged against the project-migration requirement it references. These requirements
bind the migration, so a record that checks a migration's own output (a scratch re-draft, the first
attempt's draft, or a staging tool run) can be the same check. A record that checks only the checker
cannot. What `check_draft.py` checks is read from the records: `the-measurement-2026-09-30.md` §3
("validation, the masthead, unit identifiers and states against the packet, source sentences, links,
and carried digests"), `the-working-folder-revised-2026-10-08.md` §5.6, and the proof run.

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-task-ticked-and-struck-through` | partial | kept | 3–4: the subject is exactly the scenario's Given, source task 4.5 of `sync`, and the records show its migrated task is `dropped` (measurement §4, self-test). The reason, the words, and the absence of `del` are unchecked. Close call: the draft is the first attempt's, made before the revised brief. The check still reads a migration's output for the scenario's own task, so kept. |
| `verify-the-author-s-own-account-of-the-difference` | partial | kept | 3–4: in the re-draft, every task carrying "Amended during implementation." has a deviation (checker, 0 findings). That the deviation *is* the account, in the author's words, is unchecked. |
| `verify-a-task-that-names-its-change-s-requirement` | passed | **partial** | 4: the checker matches a link to a `data-refs`, and does not match the words that give a heading. The record says "data-refs on five tasks", without naming the tasks or the headings their source words give, so the scenario's outcome for such a task would have to be inferred. The record does reach "its document is valid". Unchecked: "the migrated task names that requirement's identifier in its `data-refs`". A sentence was added to the method saying so. |
| `verify-a-reference-checked-before-promotion` | passed | kept | 3–4: both product tests were read at `08f12a9`. They stage a change-only reference and a dangling reference and run `migrate promote`, then assert exit 1, `refs/unresolved` once naming the dangling one, and nothing moved. The companion asserts no `refs/` for a change-only reference. Both are named as catchers of mutation 8, and §6.7 records that both ran. The whole scenario. |
| `verify-a-check-that-reached-part-of-its-scenario` | partial | kept | 3–4: the re-draft records the checks decision 14 names as `partial` with `unchecked`. Whether the unchecked words are the scenario's is unchecked. |
| `verify-an-accepted-risk-with-no-measure` | passed | kept | 3–4: the source's headed risk "the assertion rule cannot be checked mechanically" is marked Accepted and states no measure of its own. The re-draft gives it `data-response="accepted"` and "none" for its measure, which is the whole Then. |
| `verify-an-accepted-risk-that-names-a-partial-measure` | partial | kept | 4: the record says the risk "keeps its own measure", and not in which field. "That paragraph is its mitigation" stays unchecked. |
| `verify-a-predecessor-the-source-states` | passed | kept | 3–4: the built survey of `6a591bd` carries `log-a-runbook-run` for `carry-a-runbook`. Staging over a copy with no `FOLLOWS` wrote `FOLLOWS: log-a-runbook-run`. Both halves are reached. |
| `verify-a-configuration-that-states-no-predecessor` | passed | kept | 3–4: staging wrote `FOLLOWS` in exactly the three changes whose files state `follows:`. A planted `FOLLOWS` was refused ("the survey reports no predecessor"). |
| `verify-a-change-with-no-configuration-file` | passed | kept | 3–4: the built survey over the repository: every change carries `configuration`, and its presence agrees with the files (9 hold none). Mutation 2 is caught. |
| `verify-a-key-the-survey-has-no-name-for` | partial | kept | 3–4: `Survey::of` is the survey itself, not a helper of the command, and the test (read at `08f12a9`) asserts that `skip_specs` is reported. "The plan accounts for it" is unchecked. |
| `verify-a-creation-date-the-source-states` | partial | kept | 4: `created` is among the keys and no value appears (§6.1), and the file was carried to `evidence/`. "Only inside the file" is unchecked. |
| `verify-a-modified-requirement-stated-whole` | passed | kept | 3–4: the checker's identifiers, states, and source sentences match the packets on all six re-drafts. The packets are checked against the accepted specifications at each archival commit (0 disagreements, 0 identifier problems). |
| `verify-a-scenario-the-block-left-out-and-the-merge-kept` | passed | kept | 3–4: the re-drafted `separate-artifact-kinds` carries both kept scenarios as `unchanged`, with entries checked against the packet's text, which is the accepted text. 6.4's criterion is recorded as holding. |
| `verify-the-words-a-modification-changed` | partial | kept | 3–4: kept partial, with `deletions` unchecked. The method cited only `finish-the-archival`, whose marks are whole added paragraphs and a deleted sentence, which is a nearby case. It now also cites `separate-artifact-kinds`' `del`/`ins` inside one sentence, from the same proof record, which is the scenario's condition (5). |

## 2026-10-08-record-a-partial-verification

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-partial-verification-that-does-not-say-what-it-left` | passed | kept | 3–4: variant b gives exit 1 and `schema/missing-unchecked` at the verification. The whole Then. |
| `verify-an-unmarked-paragraph-does-not-state-it` | passed | kept | 3–4: the unit test, read at `c3a9716`, validates a `partial` verification with an unmarked paragraph, then with a note, and asserts `schema/missing-unchecked` once for each. Mutation 3 is caught. Variant b shows the rule is an error, so the document is invalid. |
| `verify-two-parts-left-unchecked` | passed | kept | 3–4: the record says `named-fields` "pins both paragraphs in order", and mutation 4 is caught. The fixture was read at `c3a9716`. |
| `verify-any-other-outcome-without-it` | passed | kept | 3–4: 92 `passed` verifications with no `unchecked`: 0 errors under both binaries, the same diagnostics, and a byte-identical projection. |
| `verify-a-requirement-with-no-verification` | passed | kept | 2–4: the scenario is stated here (`unchanged`). Variant f exits 0 with `review/unverified-requirement`. |
| `verify-verification-is-not-a-gate` | passed | kept | 2–4: the same variant f, exit 0. |

## 2026-10-08-record-a-task-done-differently

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-task-done-another-way-than-its-words-say` | passed | kept | 3–4: `validate` exits 0, and the projection gives `state: done` and the `deviation` text. |
| `verify-a-task-done-as-its-words-say` | passed | kept | 3–4: the projection of 116 files is byte-identical, with the field absent where it is not stated. The note variant lists no done-differently count and gives `doneDifferently: 0`. |
| `verify-a-deviation-on-a-task-not-yet-done` | partial | kept | 4: only the `todo` case was run. `in-progress` is unchecked. |
| `verify-a-deviation-on-a-dropped-task` | passed | kept | 3–4: exit 1, `schema/deviation-not-done` at `task-abandoned`. |
| `verify-two-deviations-in-one-task` | passed | kept | 3–4: exit 1, `schema/projection` 4.5.1. |
| `verify-a-change-with-a-task-done-differently-in-the-listing` | passed | kept | 3–4: `2/4 in-progress 1 done differently`, and in JSON `done: 2` with `doneDifferently: 1` outside `tasks`. |
| `verify-a-deviation-leaves-the-per-state-counts-whole` | passed | kept | 3–4: 2+1+0+1+0 = 4, with `doneDifferently` beside `total`. |
| `verify-a-change-whose-every-task-was-done-as-written` | partial | kept | 4: the human rows show no count, but the zero is shown only in `status --json`, not in the enumeration's payload. |
| `verify-a-deviation-on-a-task-that-is-not-done` | passed | kept | 3–4: the record says the counter reads the `todo` task as `todo` only, and `doneDifferently` stays 1, the done task's. |
| `verify-a-change-with-a-task-done-differently-in-the-completion-report` | passed | kept | 3–4: `3 of 4 task(s) done or dropped`, then `1 done differently: task-done-differently`. |

## 2026-10-08-record-how-a-risk-was-settled

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-risk-written-before-the-response-attribute-existed` | passed | kept | 3–4: variant D exits 0 and projects `mitigated`. |
| `verify-a-mitigated-risk-with-no-mitigation` | partial | kept | 4: variant A covers only the default case. No record validates a risk that declares `mitigated` and has no mitigation, so the declared case stays unchecked. |
| `verify-an-accepted-risk-with-no-mitigation` | passed | kept | 3–4: variant B exits 0, projects `accepted`, and has no `mitigation`. |
| `verify-an-accepted-risk-that-names-a-partial-measure` | passed | kept | 3–4: variant C exits 0 and projects `accepted` with its mitigation. Its model keeps every key HEAD gave, and HEAD's C equals D. |

## 2026-10-08-show-what-a-modification-changed

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-a-one-word-modification` | passed | kept | 3–4: 9 of 10 paragraphs and 18 of 18 scenarios are held by both, paragraph 3 is paired with `{+sync,+}`, and the exit is 0. |
| `verify-the-view-writes-nothing` | partial | kept | 4: the hashes cover one change's two shown views. "For any change" and "any outcome" are unchecked. |
| `verify-the-author-s-marks-account-for-the-difference` | partial | kept | 3–4: kept as part of the condition, not a nearby case. The marks were an `ins` only, read with the insertion left out, so `del` is unchecked. Close call. |
| `verify-the-author-s-marks-leave-part-of-the-difference-unmarked` | passed | kept | 3–4: `sync,` is marked with `ins` and "therefore" is removed without a mark. The result is `unreconciled`, both words are marked, and the line says so. |
| `verify-the-view-and-the-reporting-operation-agree` | passed | **partial** | 3–4: the parity test, read at `facdaca`, asserts both halves of the Then. But it is a unit test that calls `merge_state` and `show::view`, while the scenario's When runs the view and the reporting operation, so its trigger differs and it does not count. The CLI harness over 34 modifications found no disagreement with `sync`, which the record gives as differing-set agreement. Unchecked: the first-differing-paragraph half. The method was rewritten to say so. |
| `verify-showing-a-change-without-a-view` | passed | kept | 3–4: no view, names `--diff`, exit 2, and empty stdout under `--json`. |
| `verify-an-archived-change` | passed | kept | 3–4: refused with exit 2, saying the change is archived and why. |
| `verify-a-structured-view` | partial | kept | 4: the payload's states, pairing, and `onlyThisSide` words are shown. The four unchecked parts are right. |

## 2026-10-09-fill-only-the-references-a-document-loads

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-the-scratch-proposal-s-quoted-markup` | passed | kept | 3–4: quotations that HEAD filled are left unchanged, with `git diff` empty. With the document's own `<link>` pointed at an absent address, only `htmlspec.css` is filled. Both halves are reached. |
| `verify-the-scratch-proposal-s-hyperlink` | passed | kept | 3–4: an `<a>` pointing at an absent address is reported canonical, exit 0. |
| `verify-the-formatter-s-references-against-the-tree` | partial | kept | 1–4: a requirement-level unit. The cross-checks compare the formatter's references with the asset rules', which is the "SHALL NOT disagree" sentence itself, over 128 documents and generated fragments. The four unchecked parts are right. Weak point: the recorded pass (§1) came before the review's fix. §6 says the cross-checks had passed while the claim did not hold, and that the three readings "are now followed", each with a fragment that mutation round 7.1 shows the cross-checks catching. Kept, as part of the claim reached by a check of the right thing. |

## 2026-10-12-name-each-command-by-its-request-verb

| Unit | Before | After | Why |
| --- | --- | --- | --- |
| `verify-get-validity-against-validate` | partial | kept | 4: the exit status and the tree are equal for bare `get validity`. What is reported is unchecked, and is left to the parity tests. |
| `verify-every-get-invocation-of-task-1-2` | partial | kept | 4: all 25 `get` invocations of the 10 operations changed nothing. "With any of the options it accepts" is unchecked. |
| `verify-put-format-check-against-format-check` | partial | kept | 3–4: `put format --check` over a project holding a non-canonical document gives exit 1 and an unchanged tree, as `format --check` does. The report is unchecked. |

## Counts

- Kept: 52. One of them, `verify-the-words-a-modification-changed`, had a sentence added to its
  method.
- Made partial: 2 (`verify-a-task-that-names-its-change-s-requirement`,
  `verify-the-view-and-the-reporting-operation-agree`).
- Made passed: 0.
- Removed: 0.
