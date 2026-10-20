# What the implementation revealed

**Date:** 2026-10-08
**Task:** 6.8.

This change was implemented on a branch from `main` at `b768860`, as the top of a stack of four:
`record-a-task-done-differently`, `record-a-partial-verification`, and `record-how-a-risk-was-settled`
were implemented beside it, in their own branches, to be placed below it. So the vocabulary this
change maps the migration into was not in this branch's schema, and two pieces of work wait for the
stack to be assembled. Both are recorded here and in `tasks.md`.

## 6.7: the checks

`nix develop -c cargo test --workspace --all-targets`: 41 test binaries, **1,300 passed, 0 failed,
1 ignored**. The ignored one is `the_destination_expresses_each_distinction_migrate_md_maps`, as
below. This change added 14 tests, of which 13 run: eight in `migrate::tests`, one in
`commands::tests`, three in `tests/migrate.rs`, and two in `tests/promote.rs`.

`nix develop -c scripts/check`: the nine stages that do not need Python pass (`format`, `lint`,
`test`, `doctest`, `self-conformance`, `self-format`, `self-runbooks`, `self-runbooks-format`,
`self-assets`). The four Python stages fail there with `error: tool 'python3' not found`, as they
have for the last several changes, and pass outside the shell, with the toolchain's `bin` on `PATH`:
`scripts/check-prompt` ("all 4 prompt cases passed"), `scripts/check-installed` ("45 installed
instruction file(s) match what `init` writes"), `scripts/check-release --self-test` ("23 shared
fixture case(s) read the same way both readers read them"), and `scripts/check-release` ("30
archived change(s) record a version, and 24 version(s) are in the changelog", "declared version
0.24.0 has a changelog entry"). **All 13 stages are green.**

## Corrections to the design

- **Decision 6 counted 53 of the 56 linking units as resolving. 55 do.** It quoted
  `resolve-a-reference-across-its-change`'s task 5.3 as written, and that change's own measurement
  of the task found the two units linking to another change's specification resolve in the accepted
  specifications, which declare the same identifiers. Only task 5.1 of
  `distinguish-command-names-from-skill-names` stays a link, because its target is withdrawn. The
  decision now says so, and the delta's list of units a document cannot reach now reads "a unit that
  only another change's documents declare", which agrees with its own scenario. Details are in
  `the-measurement-repeated-2026-10-08.md`, section 1.0.
- **Decision 3 marked the scenario-identifier sentence for review.** `state-a-modified-requirement-whole`
  kept it at archival, so the paragraph now says so, and its sentence about dropping the paragraph is
  gone.
- **Decision 12's test is two tests.** The rows half is
  `each_distinction_the_destination_expresses_has_a_row`, and it runs. The schema half,
  `the_destination_expresses_each_distinction_migrate_md_maps`, reads fields this branch's schema does
  not hold, so it is marked `#[ignore]`, with a reason naming the three changes. **The attribute must
  be removed when the stack is assembled.** Run with `--ignored` here, it fails at its first
  assertion, as it should. **Removed on 2026-10-08, when the stack was assembled: the test passes
  against the three changes' schema.**

## What the work found

- **Promotion's check over the staged tree takes twice as long as it did.** 39.65 s of user CPU,
  against 21.3 s on 2026-09-30, and the main checkout's `0.24.0 (facdaca)` binary takes the same. It
  arrived with one of the changes since `resolve-a-reference-across-its-change`, which measured 20 s.
  It does not gate anything, and is recorded for whoever runs promotion.
- **`check_draft.py` reported two RENAMED-pair lines as missing source sentences** on
  `finish-the-archival`, in the backed-up checker too, so the pilot's acceptance as "clean" in
  `review-log.md` carried them as expected findings. With task 6.3's criterion of 0 findings, they
  would have blocked the proof, so they are now structural.
- **Decision 2's amendment needed a record in the plan, not only in the packets.** The plan records
  each qualified scenario identifier as a `scenario:` note. A scenario carried forward with its
  qualified accepted counterpart's identifier would otherwise have had an identifier no plan line
  derives, so `make_decisions.py` writes one note for it, which makes it read `replay.json` and run
  after `replay.py`.
- **The survey's human report was read by no test.** `the_survey_report_names_each_change_that_states_a_predecessor`
  is the first.
- **`follows:` with no value is null in YAML.** The survey reports such a file's key and no
  predecessor, and `a_predecessor_key_with_no_value_states_none` pins it.
- **A configuration file that cannot be opened reads as no file**, as `Change::version` reads its
  file. The file is still enumerated as an entity, so the plan still accounts for it. It is read with
  lossy UTF-8 decoding, so a file that opens is always read.
- **Every test in `tests/migrate.rs` now depends on the payload carrying `configuration`.** The
  harness refuses a payload that omits it rather than reading the omission as `null`. That is why the
  nine tests that read the survey failed on HEAD, and why mutation 2 is caught by ten tests.

## The re-stage's handoff

The self-migration's next step re-stages the whole project. It inherits the following.

**The worklist from 6.5.** A preliminary run, recorded in
`the-working-folder-revised-2026-10-08.md`, found 116 findings of the new kinds over the 31 drafts
the proof does not re-draft: 45 links with no `data-refs` in 9 packets, 39 modified units with no
mark in 16, 30 modified units whose previous text the unmarked draft no longer gives in 7, and the
newly qualified scenario identifier in `record-a-renamed-requirement`. Task 6.5, run on the
assembled stack, gives the definitive one.

**Ten new archives the reviewed plan predates.** Six are archived at `b768860`:
`reject-change-markup-inside-a-unit`, `state-a-modified-requirement-whole`,
`see-a-modification-the-merge-did-not-take`, `resolve-a-reference-across-its-change`,
`mark-the-words-a-modification-changed`, and `show-what-a-modification-changed`. Four will be:
`record-a-task-done-differently`, `record-a-partial-verification`, `record-how-a-risk-was-settled`,
and this change. Each holds an `.openspec.yaml`, and the six archived state 3 predecessors, so the
re-survey reports 30 or more archives, 21 or more configuration files, and at least 6 `FOLLOWS`.
`name-each-command-by-its-request-verb`, outside the ten, adds one more when it archives.

**Amendments awaiting the plan's next review.** Each keeps its reviewed text and gains an amendment
dated 2026-10-08:

- decision 1: the date crosses only inside the carried file because the project records none, and
  `FOLLOWS` comes from the survey;
- decision 2: a scenario a change carries forward takes its accepted counterpart's identifier;
- convention 2: a struck-through task is dropped, and a task done differently states a deviation;
- convention 4: the archival commit decides where the replay and the history differ, and the
  drafter marks the changed words;
- convention 7: a scenario a block left out is completed from the merge, and is no longer open.

**What the re-stage must run, in this order**, against a survey of HEAD made with this change's
survey: `replay.py`, `make_decisions.py`, `build_plan.py`, `stage_carry.py`, `build_packets.py`, then
the drafting under `brief-draft.md`, then `check_draft.py`. `derive.py` must also cover the new
archives first, which this change did not touch.
