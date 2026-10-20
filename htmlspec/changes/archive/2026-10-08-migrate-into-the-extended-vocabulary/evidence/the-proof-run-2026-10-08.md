# The proof, run on the assembled stack

**Date:** 2026-10-08
**Tasks:** 6.3, 6.4, and 6.5, deferred from this branch until `record-a-task-done-differently`,
`record-a-partial-verification`, and `record-how-a-risk-was-settled` were below it, and the rest of 6.8
**Program:** `htmlspec 0.27.0 (375a9e286c90)`, built from this branch with the three changes below it.
**Projection:** `project-model`, copied from the working folder into the scratch directory and built
against the stack's `htmlspec-parse`. The working folder's own tool was not touched.
**Inputs:** `scratchpad/migrate-into-the-extended-vocabulary/staged-proof/`, the staged tree's copy over
which the revised `stage_carry.py` ran, and `source/`, the `6a591bd` extraction, both prepared at
task 6.2.

## Summary

| Check | Result |
| --- | --- |
| The six packets re-drafted under the revised brief, one drafter each | all six written, every document valid (0 errors) and canonical |
| `check_draft.py` on the six | **0 findings on each**, after two faults the proof found in the checker were fixed |
| `check_draft_selftest.py` | 17 of 17 cases pass, two of them added for those faults |
| `migrate promote --check` over the copy | passes: 109 documents and 9 stored assets checked, 301 surveyed entities and 301 plan items |
| `show --diff` on `finish-the-archival`'s rebuilt modifications | each of the three reads as `author`, matching the draft's marks |
| `check_draft.py` on the 31 other drafts (6.5) | 12 draw no finding, and 19 draw the worklist below |

## Tasks 6.3 and 6.4: the six re-drafts

Six drafters ran in parallel, one per packet, each told to use the scratch copy as S, the stack's
program as H, the `6a591bd` extraction as the source, and a report directory in the scratch directory.
Each removed its packet's drafted documents from the copy first and touched no carried file;
`check_draft.py` reports the carried files intact.

| Packet | Documents | New vocabulary used |
| --- | --- | --- |
| `spec-runbook-portability` | the accepted specification | none applies: every unit is `unchanged`, and an accepted specification holds no task, verification, or risk |
| `2026-09-26-carry-a-runbook-to-another-machine` | proposal, design, tasks, specification | two deviations, on tasks 5.2 and 5.3, each naming where the build is recorded |
| `2026-08-30-finish-the-archival` | the four | `ins` and `del` in three modified requirements, one withdrawn sentence kept as a recorded deletion; seven deviations; `data-refs` on five tasks |
| `2026-08-13-content-address-runtime-assets` | the four | three `partial` verifications with `unchecked`; whole added paragraphs in `ins`; one deviation; five `data-refs` |
| `2026-08-14-separate-artifact-kinds` | the four | `del`/`ins` inside a sentence and whole added paragraphs; the two scenarios the merge kept as `unchanged` units; two `partial` verifications; three deviations |
| `2026-09-06-distinguish-a-requirement-from-an-example` | the four | two risks `data-response="accepted"`, one keeping its own measure and one with none; five deviations; two `partial` verifications; one `data-refs` |

6.4's criteria hold: the two partial checks design decision 14 names in `content-address` are
`partial` with `unchecked`; the two kept scenarios are `unchanged` units; and the risk units declare
`accepted`. **One more than decision 14 names:** the `content-address` drafter also made
`verify-the-check-suite-at-86c33f4` `partial`, because its record shows the check suite passing and not
that the reference rule is evaluated by it. That is for review.

**The second check of the marks (6.3).** A scratch project made with `init` held, as its accepted
specification, `finish-the-archival`'s three modified requirements as they read with each `del`
unwrapped and each `ins` dropped, and as an active change, the re-drafted specification. The change's
specification validates (0 errors). `show finish-the-archival --diff --json` shows all three as
`differs` with `marks` `author`, and the paragraphs it pairs and the paragraphs it shows whole are the
ones the draft marks: a pair for the paragraph that keeps the withdrawn sentence in a `del`, and a
paragraph stated only by the change for each whole `ins`. `check_draft.py`'s marks check had already
compared the same text with the packet's previous text in both directions.

## Two faults in the revised checker, found by the proof

`separate-artifact-kinds` drew three findings from `check_draft.py` that its drafter traced to the
checker, not the draft. Both causes are fixed in the working folder, and each has a self-test case
that fails without the fix:

- **A deletion in the middle of a sentence hid the sentence.** The source-sentence check read every
  word of the document, a `del`'s included, so `declare <del>its lifecycle status</del><ins>which
  artifact kind it is</ins>` never held the delta's sentence. The visible text now leaves out what a
  `del` holds, as the projection does (section 8.6.1). Case: "a deletion in the middle of a source
  sentence does not hide the sentence, and keeping the deleted words as current text does".
- **A scenario the merge kept was compared with nothing.** For a scenario inside a modified
  requirement, the expected entries came from the delta's block, which left these scenarios out, and
  the packet's `text` for them, which `build_packets.py` carries for exactly this case, was not read.
  It now is. Case: "a scenario the source's merge kept is compared with the packet's text".

**Two self-test cases assumed the old drafts' faults.** "A task whose source says it was amended and
states no deviation is reported" and "a link to a reachable unit with no data-refs is reported" each
expected the draft to lack the deviation or the reference, and the re-drafts carry both. They now
plant the fault themselves, by removing the deviation or the reference first. A third case added a
second `data-refs` attribute beside an existing one and passed only because the parser keeps the last;
it now replaces the attribute. The checker and the self-test as they stood before these edits are kept
in the scratch directory, beside the working folder's backup.

## Task 6.5: the re-stage's worklist

The revised `check_draft.py` over the 31 other drafts, which predate the revised brief: 12 draw no
finding (the eleven accepted specifications other than `runbook-portability`, and
`2026-08-24-report-in-the-contributors-terms`), and 19 draw these:

| Kind | Findings |
| --- | --- |
| a link to a reachable unit with no `data-refs` | 45 |
| a modified unit whose text changed, with no mark | 39 |
| previous text not given with `del` unwrapped | 30 |
| source sentences not found, or scenario entries differing, each reported before this change | 7 |
| `record-a-renamed-requirement`'s "An invalid specification", whose identifier the packet now qualifies | 2 |

The three new kinds are what the revised brief exists to change, and their counts equal the
preliminary run's, so the checker's two fixes moved none of them. By packet, the most are in
`2026-08-17-project-bootstrap-cli`, `2026-09-11-migrate-an-openspec-project`, and
`2026-09-26-record-a-renamed-requirement` (19 each), then `2026-09-18-establish-a-runbook` (11). No
draft draws a `FOLLOWS`, task-state, or deviation finding.

## For the re-stage, found by the proof

- **The stylesheet revision.** `htmlspec new document` from this program writes the newest stylesheet,
  `sha256-03627137bcbb81c9`, into the staged store, where the 109 staged documents name
  `sha256-aac8967609f80d9f`. The store now holds 9 assets, and `htmlspec assets` still passes. The
  re-stage decides whether re-drafted documents take the new revision or link the existing one.
- **The formatter rewrites a quoted attribute in prose.** `htmlspec format` rewrites any `href="` or
  `src="` in a document's text, an author's `<code>href="assets/htmlspec.css"</code>` included, so
  `format --check` fails until the author's words are changed. The `content-address` drafter kept the
  words with an HTML comment between `href` and `="` inside the `<code>`. The fault is in
  `crates/htmlspec-format/src/references.rs`, predates this stack, and any migrated document that
  quotes such an attribute meets it.
- **The brief's "Read first" paths name the main checkout**, whose files hold none of the new
  vocabulary until the stack lands. Three drafters read the worktree's copies instead. Once the stack
  is on `main`, the paths are right again.
- **What the drafters put to review:** a SHALL the `carry-a-runbook` specification now states from its
  design (archival skips a version it cannot parse); `finish-the-archival`'s two build-differs notes
  (`--store` reaches `list` and `status` only, and `--check` passes a change whose target exists, both
  still so at `b768860`) and the tension between "Archival produces…" and "Archival assesses the merge
  and never performs it"; `distinguish-a-requirement-from-an-example`'s task 3.7 deviation, which
  records two measurements it does not reconcile; and the third `partial` check above. Each is in that
  packet's report in the scratch directory.
