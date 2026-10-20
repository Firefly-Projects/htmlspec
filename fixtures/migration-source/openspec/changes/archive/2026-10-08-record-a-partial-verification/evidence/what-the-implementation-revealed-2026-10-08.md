# What the implementation revealed

**Date:** 2026-10-08
**Task:** 7.4
**Revision:** this change's branch, from `main` at `b768860`.

## Summary

The change was built as designed, and no decision changed. Three facts the design rested on had
moved under it before any code, and each was corrected in `design.md` at task 1.0. Beside the
design, building and measuring found one guard the tasks did not ask for, one prediction that held
for the reason the design gave, one fault in the mutation procedure itself, and four things for the
contributor to see at review.

## 1. The design's facts, corrected before any code

`evidence/what-it-builds-on-2026-10-08.md` records them. The stylesheet revision to retire is
`ebf35a36fe4dc1a7`, which `mark-the-words-a-modification-changed` added, and not
`aac8967609f80d9f`, which was already retained. The contract moves from `0.5.0` to `0.6.0`,
because `record-a-task-done-differently`, below this change in the stack, takes `0.5.0`. And the
outcome enumeration and the record composition had moved by a few lines. None of the three changed
a decision.

## 2. Every reader of the outcome took `partial` with no code change

The design said that `review.rs` `has_run`, `requirements_are_verified`, and `runs.rs` `has_run`
need no edit, because each takes its values from the schema. That held. Appending `partial` to the
enumeration made `a_verification_of_any_outcome_clears_the_unverified_warning`,
`the_not_run_outcome_is_found_in_the_schema`, `only_an_outcome_the_schema_enumerates_counts_as_recorded`,
and `a_run_that_recorded_a_partial_check_reports_nothing` cover it, the last two having failed on
HEAD. The only code this change adds outside the projection is the rule.

**One of those passed on HEAD for the wrong reason, and still does for one input.**
`a_partial_verification_of_a_moved_requirement_is_a_stale_verification` passed before `partial`
existed, because `review.rs`'s `has_run` counts any value but the not-run one as having run. It now
passes because `partial` is a value that ran. The typo `notrun` still draws
`review/stale-verification` beside its `schema/invalid`, as task 7.1 measured. The proposal leaves
that out of this change, and nothing here moved it.

## 3. A guard the tasks did not ask for, and where coverage rests on one test

`the_archive_instructions_say_when_a_check_is_partial` holds `archive.md` to naming the outcome, the
field, the requirement, and the boundary "is not a partial check of it". No task asked for it.
Without it, removing the instruction's paragraph would fail nothing but `check-installed`'s
comparison, which a regenerated copy satisfies. Mutation 10 shows it as that mutation's only
catcher.

**Two planned mutations have one catcher each, and both are tests this change added.**
`requirements_are_verified` ignoring `partial` (mutation 7) is caught only by
`a_verification_of_any_outcome_clears_the_unverified_warning`, and the rule satisfied by a note
(mutation 3) only by `an_unmarked_paragraph_does_not_state_what_a_partial_check_missed`. Design
decision 6 found the first gap before any code: no test held the unverified warning to a
verification's outcome, because the review module's tests declare a `tasks` document and the
conformance harness checks only that a fixture's declared rules fire.

## 4. The generator's deletions are no longer a hazard

Task 2.4 asked whether the four hand-authored conformance fixtures survive regeneration. They do:
the generator now keeps them through its `HAND_AUTHORED` table and refuses to run if one is missing,
so the restore step earlier changes needed is gone. Each regeneration rewrote the twelve `rename`
fixtures' version lines, which `declaring_the_current_version` builds, as every contract bump does.

## 5. Restoring a mutated file must not restore its modification time

The first mutation run restored each file with `shutil.copy2`, which copies the modification time
with the bytes. Cargo decides what to rebuild by modification time, so a restored file looked older
than the build made from its mutated version, and the mutated build stayed. Mutation 5's run also
failed to compile, on a transient `failed to create dependency graph` in `target/debug/incremental`,
and runs 6 to 10 then tested a parser that still carried mutation 5: each listed mutation 5's
failures beside its own. The hashes after the run were all equal, which is why nothing looked wrong.

The runner now writes the saved bytes back and gives the file a new modification time, and it
retries a run whose build fails, because a build failure is no test result. All ten mutations were
run again from a clean baseline, and `evidence/mutation-testing-2026-10-08.md` records that run
alone. **A hash check proves the bytes were restored, not that the next build reads them.**

## 6. For the contributor at review (task 9.1)

- **`passed` with an `unchecked` paragraph is now valid** (task 7.1, variant d). Design decision 4
  chose not to reject it: the overstatements measured carried no such paragraph, so a rule would
  not have caught the case that exists. It is a contradiction the contract now accepts.
- **On the page, an `unchecked` paragraph looks exactly like the method**
  (`evidence/the-presentation-2026-10-08.md`). Design decision 7 gave it no generated label, as
  `method` and `reason` have none. Only the author's wording separates "what was done" from "what
  was not". A label like `renamed-from`'s would be a new stylesheet revision.
- **The prose is written for the assembled stack.** The schema changelog's entry, the projection
  specification's header, and the README say `0.1.0` to `0.5.0` stay accepted. On this branch alone
  the enumeration skips `0.5.0`, which arrives with the change below it.
- **The retired stylesheet revision is kept with a reason the others do not give.** No document
  this repository tracks, and none the staged migration holds, names `ebf35a36fe4dc1a7`. It is kept
  because the procedure keeps every superseded revision and `init` in releases 0.23.0 to 0.25.0
  stored it in every project it set up.
