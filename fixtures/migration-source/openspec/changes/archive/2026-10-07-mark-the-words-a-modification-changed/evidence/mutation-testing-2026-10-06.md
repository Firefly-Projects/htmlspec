# Mutation testing

**Date:** 2026-10-06
**Task:** 8.3

Each mutation was applied alone to the built change, the test suites that could catch it were run
with `--no-fail-fast`, every failing test was recorded, and the file was restored from a copy taken
before the edit. The suites were `htmlspec-parse` and `htmlspec-conformance` for the projection's
mutations (1 to 7); the validator's unit tests, `htmlspec-conformance`, and the command line's
change-markup tests for the rule's (8 to 16); `presentation.rs` and `htmlspec-assets` for the
stylesheet's (17); and `htmlspec-assets` for the instruction's (18). After the run, `git diff HEAD`
over `crates/`, `assets/`, and `conformance/` was empty.

**Every mutation was caught. No test had to be added.**

| # | Mutation | Caught by |
| --- | --- | --- |
| 1 | collect_text keeps a del's text | `every_fixture_projects_to_its_expected_model`; `project::tests::a_del_in_a_nested_unit_is_that_units`; `project::tests::an_empty_del_records_nothing`; `project::tests::nested_dels_are_one_entry`; `project::tests::text_leaves_out_what_a_del_holds` |
| 2 | the walk supplies fields from inside a del | `every_fixture_projects_to_its_expected_model` |
| 3 | a deletion is recorded on the outermost unit instead of the nearest | `every_fixture_projects_to_its_expected_model`; `project::tests::a_del_in_a_nested_unit_is_that_units` |
| 4 | an anchor inside a del still projects a reference | `every_fixture_projects_to_its_expected_model` |
| 5 | an anchor inside a del is dropped from links | `every_fixture_projects_to_its_expected_model` |
| 6 | a unit inside a del stops projecting | `every_fixture_projects_to_its_expected_model` |
| 7 | deletions is serialized when empty | `every_fixture_projects_to_its_expected_model`; `project::tests::a_del_in_a_nested_unit_is_that_units`; `project::tests::an_empty_del_records_nothing` |
| 8 | the markup is permitted at every placement | `a_modified_requirement_may_carry_marks_only_in_a_change_specification`; `every_fixture_is_accepted_exactly_when_it_declares_it_is`; `every_fixture_reports_exactly_the_rules_it_declares`; `marks_beneath_a_changes_evidence_are_refused`; `marks_in_a_changes_proposal_are_refused`; and 23 more |
| 9 | it is permitted at AcceptedSpecification as well | `a_modified_requirement_may_carry_marks_only_in_a_change_specification`; `a_placement_whose_places_declare_nothing_changes_no_fixture`; `marks_in_an_accepted_specification_are_refused`; `rules::change_markup::tests::an_accepted_specification_is_refused`; `rules::change_markup::tests::each_refusal_names_its_fix`; and 4 more |
| 10 | the permission is judged by the outermost unit | `rules::change_markup::tests::change_markup_in_a_scenario_names_the_scenario`; `rules::change_markup::tests::scenario_change_markup_in_an_unchanged_scenario_of_a_modified_requirement` |
| 11 | it is permitted in a unit marked unchanged | `rules::change_markup::tests::scenario_an_inserted_word_in_an_unchanged_unit`; `rules::change_markup::tests::scenario_change_markup_in_an_unchanged_scenario_of_a_modified_requirement` |
| 12 | it is permitted in a decision | `rules::change_markup::tests::a_unit_no_accepted_specification_holds_is_refused`; `rules::change_markup::tests::scenario_any_unit_of_any_kind_at_every_placement` |
| 13 | it is permitted in a task | `rules::change_markup::tests::a_task_is_refused`; `rules::change_markup::tests::each_refusal_names_its_fix`; `rules::change_markup::tests::scenario_any_unit_of_any_kind_at_every_placement`; `rules::change_markup::tests::the_first_refusal_that_applies_is_given` |
| 14 | it is permitted in a heading | `rules::change_markup::tests::each_refusal_names_its_fix`; `rules::change_markup::tests::markup_in_a_name_is_refused`; `rules::change_markup::tests::scenario_change_markup_in_a_units_heading`; `rules::change_markup::tests::the_first_refusal_that_applies_is_given` |
| 15 | it is permitted around a unit | `rules::change_markup::tests::a_long_text_is_quoted_by_its_opening_words`; `rules::change_markup::tests::a_requirement_wrapped_in_del_is_reported_at_the_requirement`; `rules::change_markup::tests::a_requirement_wrapped_in_ins_is_told_to_mark_it_added`; `rules::change_markup::tests::a_unit_wrapped_through_an_intervening_element_is_reported`; `rules::change_markup::tests::a_wrapping_element_is_reported_once_at_the_first_unit_it_encloses`; and 4 more |
| 16 | only del is judged, and ins is always permitted | `a_modified_requirement_may_carry_marks_only_in_a_change_specification`; `an_added_requirement_may_carry_marks_nowhere`; `marks_in_an_accepted_specification_are_refused`; `rules::change_markup::tests::a_deleted_phrase_and_an_inserted_word_are_two_errors`; `rules::change_markup::tests::a_requirement_wrapped_in_ins_is_told_to_mark_it_added`; and 9 more |
| 17 | the stylesheet's double underline is removed | `an_edits_words_are_drawn_by_shape_and_announced`; `tests::every_generated_address_matches_the_derivation`; `tests::every_own_asset_matches_its_recorded_hash` |
| 18 | the sentence on removing the marks is removed from sync.md | `instructions::tests::the_merge_instructions_say_how_marks_are_written_and_removed` |

## What the table shows

- **Four projection mutations are caught by the mapping fixture alone**: 2, the walk supplying
  fields from inside a `del`; 4, a deleted anchor still projecting a reference; 5, a deleted anchor
  dropped from `links`; and 6, a scenario inside a `del` no longer found. Each is a case the
  fixture `change-markup` pins and no unit test restates. That is the fixture's job: it is the
  contract an independent implementation reproduces, and a unit test beside it would assert the
  same model twice.
- **Mutation 9, the permission widened to accepted specifications, is caught by the restated
  property** `a_placement_whose_places_declare_nothing_changes_no_fixture`, as task 2.7 required,
  and by the conformance suite, the command line's test, and the rule's own tests.
- **Mutation 17 is also caught by the store's hash checks**, because the stylesheet lives at its
  content address. Removing the double underline changes the bytes, and the presentation test is
  the one that names what went missing.
- **The `sync` and `archive --check` command-line tests catch no rule mutation**, and none was
  expected: each asserts that the marks do not refuse the change, so a mutation that permits more
  leaves them passing. The refusal at every other placement is caught by the four tests beside them.

## Found after this run, by review

The code review of PR #46 found a defect no planned mutation stood for: a `del` whose own class
names a unit, nested in a requirement, was recorded in the requirement's `deletions`. The fixture
pinned `<del class="decision">` outside every unit only. A unit test and a nested `<del
class="scenario">` in the fixture now pin it, and each fails with the fix removed. Task 10.1 records
it.

