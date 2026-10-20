# What the implementation revealed

**Date:** 2026-10-08
**Task:** 7.4

## The design was written against a contract two versions old

The design was written on 2026-09-30 and 2026-10-01, and six changes landed before it was
implemented. Reading the code first, at task 1.4, found what had moved, and `design.md` was
corrected before any code (`what-it-builds-on-2026-10-08.md`):

- **The contract was `0.4.0`, not `0.2.0`,** so this change moves it to `0.5.0`.
  `reject-change-markup-inside-a-unit` and `mark-the-words-a-modification-changed` moved it.
- **The stem `"done"` was the literal the same decision forbids.** `schema_rust_split.rs` matches each
  owned literal as a whole quoted string, so `status_named("done", …)` in rule code would have failed
  the test the decision adds. The rule reads `done` through `don`, a fragment, as `dropp` and
  `supersed` already read theirs. Had the design been followed literally, task 4.2 would have stopped
  on its own check.
- **The fixture generator no longer deletes the four hand-authored fixtures.** Task 2.4 asked for
  them to be restored, and `HAND_AUTHORED`, added by `reject-change-markup-inside-a-unit`, already
  keeps them. Nothing was restored, and `git status` after each generator run showed only this
  change's fixtures.
- **`status` could not be reached from a unit test.** It resolves its project from the working
  directory. It is now `status`, which resolves, and `status_of`, which reports, as `list` is already
  `list` and `list_in`. Nothing it prints changed: every `status` test in `tests/archive.rs` and
  `tests/cli.rs` passes unchanged.

## A blank column still has a separator

Decision 5 says the count's column is widened to its widest member, blank at zero, and trimmed at
the end of a line, and also that a listing of changes done as written is byte-identical to today's.
The two meet on a listing where some change follows another and no change has a task done
differently. A column of width zero with the two spaces that separate every column still puts two
more spaces before `follows …`, and the end-of-line trim does not reach them, because they are not
at the end of the line. The column and its separator are therefore written only where some row has
a count. `a_listing_with_no_task_done_differently_prints_what_it_printed_before` pins today's bytes
for exactly that listing, a predecessor included, and passes on HEAD and on the change. Task 7.2's
mutation 9 is the other side of it.

## A task that declares no state

Neither the design nor the delta specification says what the rule does with a deviation on a task
that declares no state, or a state outside the four. The specification makes a deviation invalid on
`todo`, `in-progress`, and `dropped`, and the schema's description, written at task 3.1, says a
deviation is permitted only when the state is `done`. The rule follows the second reading: it keys on
the one state that permits a deviation and reports every other, an absent state included, so a state
the schema gains later is refused until someone decides it may carry one. Such a task then draws two
errors, the schema's missing state and this rule. The counter agrees: it reads the task as
undeclared, not as done differently. Task 9.1 puts the double report to the contributor.

## The template's third example counts

A change just created by `htmlspec new change` lists as `1/3  in-progress  1 done differently`, where
it listed `0/2  in-progress`, and `status` names `task-deviation-example`. The example task is `done`,
as decision 8 has it, and it counts until an author replaces it, as the dropped example already
made a fresh change read `in-progress` rather than `todo`. `review/example-content` reports all three
example identifiers, so the example does not pass unnoticed. Task 9.1 asks whether a fresh change
reading as partly done is acceptable.

## Smaller findings

- **Section 6.1 of the projection specification enumerates every normalized-text field**, and now
  names deviations. Task 3.2 listed four edits and this is a fifth.
- **`README.md` states the contract version**, as `mark-the-words-a-modification-changed` kept it,
  and now says `0.5.0`.
- **The new `apply` section renumbered the steps after it.** The contradiction step is 5 and "Check
  as you go" 6, and step 2's cross-reference moved with them. The design's decision 8 and its second
  risk now name the step by its title.
- **The task fault for an unknown field now lists `deviation`.** A paragraph a task does not accept
  is reported as `… it accepts "reason", "deviation", "note"`. No test or fixture held the old
  wording.
- **The formatter needs nothing for the paragraph.** It canonicalizes the tag and leaves the text,
  line breaks included, as for every paragraph, and a second run changes nothing.
- **An empty deviation paragraph is discarded**, as decision 3 says, so the task reads as done as
  written. `section_4_5_a_task_stating_no_deviation_projects_none` and the `task-deviation` mapping
  fixture both pin it.
- **A second deviation was two 4.0 faults on HEAD and is one 4.5.1 fault now**, at the task, reported
  after the task's title faults. The first paragraph is kept, so the counter still reads the task as
  done differently.
- **One of this change's own test literals was wrong.** `the_row_shows_the_tasks_done_differently_after_the_state`
  expected 33 spaces before the predecessor where decision 5's widths give 29. The listing was right
  and the test was corrected.
- **The staged migration projects byte-identically** under the built change and under HEAD, all 116
  documents, because none carries a deviation and the field is absent where it is not stated.
