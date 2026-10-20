# Mutation testing

**Date:** 2026-10-08
**Task:** 7.2

A script applied each mutation alone, ran `nix develop -c cargo test -p htmlspec-parse -p
htmlspec-validate -p htmlspec-conformance -p htmlspec-cli -p htmlspec-assets --no-fail-fast`,
collected every test that reported `FAILED`, and restored the files from copies it took before the
first mutation. The three files the mutations touch (`project.rs`, `relations.rs`, and
`commands.rs`) hashed the same after the run as before it: `1a39150a…`, `62d82bb5…`, and
`1fad29e9…`. Every mutation compiled.

## Results

| # | Mutation | Failed | Caught by |
| --- | --- | --- | --- |
| 1 | The rule never fires | 6 | `a_deviation_on_a_todo_task_is_reported`, `a_deviation_on_an_in_progress_task_is_reported`, `a_deviation_on_a_dropped_task_is_reported`, `a_deviation_on_a_task_declaring_no_state_is_reported`, and the conformance suite's `every_fixture_reports_exactly_the_rules_it_declares` and `every_fixture_is_accepted_exactly_when_it_declares_it_is` |
| 2 | The rule also fires on `done` | 162 | `a_deviation_on_a_done_task_is_accepted`, `every_valid_fixture_is_accepted_by_the_whole_rule_set`, `every_kind_init_can_produce_validates_immediately`, and 159 more, below |
| 3 | The rule ignores `dropped` | 1 | `a_deviation_on_a_dropped_task_is_reported` |
| 4 | The projection drops `deviation` | 14 | `section_4_5_a_deviation_projects_into_its_field_and_not_into_the_notes`, `section_4_5_1_a_second_deviation_is_invalid_and_the_first_is_kept`, `every_fixture_projects_to_its_expected_model`, the four rule tests of mutation 1, the two conformance tests of mutation 1, and five counter and report tests in `commands.rs` |
| 5 | No fault 4.5.1 is reported | 2 | `section_4_5_1_a_second_deviation_is_invalid_and_the_first_is_kept`, `every_fixture_projects_to_its_expected_model` |
| 6 | The counter counts a deviation on any state | 1 | `a_deviation_on_a_task_that_is_not_done_counts_under_its_own_state_only` |
| 7 | The listing subtracts tasks done differently from the numerator | 1 | `the_row_shows_the_tasks_done_differently_after_the_state` |
| 8 | `doneDifferently` moves inside `tasks` | 2 | `the_payload_carries_the_tasks_done_differently_beside_the_per_state_counts`, `the_enumeration_and_the_completion_report_agree` |
| 9 | The listing prints `0 done differently` | 2 | `a_listing_with_no_task_done_differently_prints_what_it_printed_before`, `the_row_shows_the_tasks_done_differently_after_the_state` |

**9 of 9 are caught**, none by a test added because of this run.

## How the mutations were realized

- **1.** The `diagnostics.violation(DEVIATION_NOT_DONE, …)` call was replaced by discarding its
  arguments.
- **2.** The guard became `if task.deviation.is_none()`, dropping the `done` exemption.
- **3.** The guard also skipped a task whose state matched `dropped`.
- **4.** `project_task` wrote `deviation: None`, still reporting 4.5.1.
- **5.** The 4.5.1 branch was made unreachable, keeping the first deviation.
- **6.** The push onto `done_differently` moved out of the `done` arm, after the match, so every
  state's deviation was counted.
- **7.** The fraction's numerator became `done - done_differently.len()`.
- **8.** `doneDifferently` was written into the `tasks` object and removed from beside `total`.
- **9.** The column's text was written for every change with a tasks document, zero included.

## What the run shows

**Mutation 2 failed 162 tests, because the template now writes a `done` task with a deviation.**
Every document `new change` or `init` creates from the tasks template carries
`task-deviation-example`, so a rule that refused a deviation on `done` made every such document
invalid, and every test building a change from the template failed: archival, `sync`, the view,
runs, preflight, the plan, and the README's declared examples among them. The unit test
`a_deviation_on_a_done_task_is_accepted` and the conformance fixture
`valid-task-done-with-a-deviation` catch it without the template.

**Three mutations are caught by one test each**: 3 by the dropped-task unit test, 6 by the negative
counter test, and 7 by the exact listing. Each is the test written for that case in task 2.3 or
2.5, and each is the only place the case is stated. Mutation 6 is the one that shows
`a_deviation_on_a_task_that_is_not_done_counts_under_its_own_state_only` bites, which it could not
show against the stub, where it passed as a negative case must.

**Mutation 9 is caught on both sides of the column.** The exact listing with a deviation fails
because its predecessor row moves, and the listing with none fails because every row gains `0 done
differently`. Without the second test, a column reserved at width zero, which the implementation
found at task 5.2, would have been caught by nothing that compares bytes.
