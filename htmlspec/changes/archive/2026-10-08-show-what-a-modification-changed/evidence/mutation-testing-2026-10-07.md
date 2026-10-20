# Mutation testing

**Date:** 2026-10-07
**Task:** 7.3

Each mutation was applied alone by a script that edits the source, runs `nix develop -c cargo test -p
htmlspec-cli -p htmlspec-assets --no-fail-fast`, collects every test that reports `FAILED`, and
restores the files from copies taken before the first mutation. The SHA-256 of the four files the
mutations touch (`show.rs`, `commands.rs`, `main.rs`, and `archive.md`) matched the copies after the
run. No mutation failed to compile.

## Results

| # | Mutation | Caught by |
| --- | --- | --- |
| 1 | Pair the change's requirement with an accepted one by heading rather than by the comparison's index | `a_renamed_requirement_shows_both_headings` |
| 2 | Decide "differs" by the view's own comparison, with scenarios compared in order | **survived the first run**; now `a_modification_merged_with_its_scenarios_reordered_is_held_as_stated` |
| 3 | Drop the common-prefix step from `align` | `a_shorter_side_that_agrees_to_its_end_leaves_the_rest_unpaired` |
| 4 | Pair leftover paragraphs by position instead of by likeness | `a_paragraph_added_beside_an_edited_one_is_shown_apart_from_the_pair` |
| 5 | Drop the half-words threshold, so every leftover pair is marked | `a_paragraph_added_beside_an_edited_one_is_shown_apart_from_the_pair`, `a_paragraph_rewritten_throughout_is_shown_whole_on_each_side` |
| 6 | Show the author's marks without checking them against the accepted text | `the_authors_marks_are_set_aside_where_they_leave_part_unmarked` |
| 7 | Ignore the author's marks | `the_authors_marks_are_set_aside_where_they_leave_part_unmarked`, `the_authors_marks_are_shown_where_they_account_for_the_difference` |
| 8 | Pair scenarios by order instead of by identifier | `scenarios_added_withdrawn_and_left_out_are_each_shown_as_such` |
| 9 | Show a scenario marked `removed` as held only by the accepted specification | `scenarios_added_withdrawn_and_left_out_are_each_shown_as_such` |
| 10 | Decide whether to compare by the declared state alone, so an `added` requirement and an `unchanged` one holding a changed scenario are listed | `the_view_and_sync_agree_about_what_differs`, `a_scenario_changed_inside_an_unchanged_requirement_is_compared`, `an_added_requirement_held_otherwise_is_compared`, `the_one_word_case_as_a_payload`, `the_one_word_case_shows_one_pair_with_the_word_marked` |
| 11 | Accept an archived change | `an_archived_change_is_refused` |
| 12 | Treat `show <change>` without `--diff` as `--diff` | `showing_a_change_without_a_view_names_diff`, `the_view_writes_nothing` |
| 13 | Print a `@@` hunk header in the human form | `nothing_reads_as_merge_text` |
| 14 | Write a progress line to standard output under `--json` | `nothing_reads_as_merge_text`, `the_one_word_case_as_a_payload`, `the_payload_is_alone_on_standard_output`, `the_view_through_a_registered_store_is_the_same` |
| 15 | Drop the refusal of a specification that does not validate | `an_invalid_specification_is_refused`; after the run, also `sync_refuses_a_specification_that_does_not_validate` |
| 16 | Remove the sentence naming `htmlspec show` from `archive.md` | `the_review_instructions_send_a_reader_to_the_view` |
| 17 | Compare a requirement marked `unchanged` whose every scenario is marked `unchanged` | `requirements_the_comparison_does_not_compare_are_listed` |

**17 of 17 are caught**, two of them by tests added because of this run.

## How some mutations were realized

- **2.** In `view`, a requirement was held where its normalized statements and its scenarios, as
  heading, given, when, and then in order and less those withdrawn, equalled the first holder's, and
  differed otherwise, with an empty difference where the comparison found none.
- **4.** The run between two paragraphs held by both was paired by position, keeping the threshold,
  as the evidence's prototype pairs: a leftover pair alike is paired, and one not alike is shown
  whole on each side.
- **5.** `alike` returns true.
- **8.** The accepted scenario taken for a stated one is the first of those not kept, whatever its
  identifier.
- **14.** `streams.payload("comparing the change")` before the payload, which is a line on standard
  output that is not the payload.
- **16.** The paragraph from "**Read what you restated beside the accepted text, before step 5.**" to
  the blank line after it was removed. The instruction bodies are compiled in, so the test sees the
  edit.

## What the run found

**Mutation 2 survived.** Every fixture held as stated kept its scenarios in the accepted order, so a
view that compared scenarios in order agreed with the comparison on all of them. The comparison reads
scenarios as a collection, as toolchain-packaging's reporting requirement says ("A scenario is a unit a
reader checks on its own, so its place among the others obliges nothing"), so a merge that put them in
another order took them. `a_modification_merged_with_its_scenarios_reordered_is_held_as_stated` now
states that case, and the mutation fails it.

**`sync`'s refusal of an invalid specification had no test of its own.** Mutation 15 removes the
refusal from `load_specification`, which `sync` and the view share since task 3.3, and only the view's
`an_invalid_specification_is_refused` failed. `tests/cli.rs` held `sync` only to not refusing a marked
specification. `sync_refuses_a_specification_that_does_not_validate` now holds the requirement's
"SHALL refuse … on a specification that does not validate" for `sync`, and a rerun of the mutation
fails it too.

## Found after this run, by review

The code review of PR #47 found a case no planned mutation reached: a withdrawn scenario with no
identifier. The view found a withdrawn scenario's accepted counterpart by identifier only, so such a
scenario was always shown as no longer held, and its counterpart, left unpaired, was listed again as
held only by the accepted specification. The fixtures gave every scenario an identifier.
`a_withdrawn_scenario_with_no_identifier_is_found_by_its_text` reproduced it and failed before the
fix, which finds the counterpart by the comparison's `ScenarioText` where no identifier does.
