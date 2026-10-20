# Mutation testing

**Date:** 2026-10-08
**Task:** 7.2
**Revision:** this change's branch at `c3a9716`, the work-in-progress commit of groups 1 to 6 and
task 7.1, with nothing else in the working tree.

Each mutation was applied alone, and the suites of five crates were run with `--no-fail-fast`:
`cargo test -p htmlspec-validate -p htmlspec-conformance -p htmlspec-parse -p htmlspec-cli -p
htmlspec-assets`, every unit, integration, and conformance test the change's code reaches, 1,182 in
all. Every test that reported `FAILED` was recorded. Each touched file was saved and hashed before
the run, written back from its copy after its mutation, given a new modification time, and hashed
again. After the run all six files hashed as before and `git status --porcelain` was empty.

The run began with an unmutated baseline: 1,182 passed, 0 failed, and no build failure. No
mutation's build failed.

**Every mutation was caught.** Nine are the task's. The tenth is the instruction paragraph, which
task 6.1's added guard covers.

| # | Mutation | Failed | Caught by |
| --- | --- | --- | --- |
| 1 | the rule never reports | 4 | `rules::relations::tests::a_partial_verification_stating_nothing_it_did_not_check_is_reported`; `rules::relations::tests::an_unmarked_paragraph_does_not_state_what_a_partial_check_missed`; `every_fixture_reports_exactly_the_rules_it_declares`; `every_fixture_is_accepted_exactly_when_it_declares_it_is` |
| 2 | the rule keys on the wrong stem, `pass` for `parti` | 48 | `rules::relations::tests::a_partial_verification_stating_nothing_it_did_not_check_is_reported`; `rules::relations::tests::a_verification_that_is_not_partial_needs_no_unchecked_paragraph`; `rules::relations::tests::an_unmarked_paragraph_does_not_state_what_a_partial_check_missed`; `rules::relations::tests::every_dogfood_document_satisfies_the_relational_rules`; `every_valid_fixture_is_accepted_by_the_whole_rule_set`; and 43 more, every command-line test whose project holds a `passed` verification |
| 3 | the rule accepts an unmarked paragraph as `unchecked` (a note satisfies it) | 1 | `rules::relations::tests::an_unmarked_paragraph_does_not_state_what_a_partial_check_missed` |
| 4 | `unchecked` is single-valued | 1 | `every_fixture_projects_to_its_expected_model` (`named-fields`) |
| 5 | `project_verification` does not accept `unchecked` | 5 | `every_fixture_projects_to_its_expected_model`; `rules::relations::tests::a_partial_verification_stating_what_it_did_not_check_is_accepted`; `every_fixture_is_accepted_exactly_when_it_declares_it_is`; `every_valid_fixture_is_accepted_by_the_whole_rule_set`; `a_run_that_recorded_a_partial_check_reports_nothing` |
| 6 | `review.rs` `has_run` treats `partial` as not run | 2 | `rules::review::tests::a_partial_verification_of_a_moved_requirement_is_a_stale_verification`; `rules::review::tests::the_not_run_outcome_is_found_in_the_schema` |
| 7 | `requirements_are_verified` ignores a `partial` verification | 1 | `rules::review::tests::a_verification_of_any_outcome_clears_the_unverified_warning` |
| 8 | `runs.rs` `has_run` treats `partial` as not recorded | 2 | `runs::tests::only_an_outcome_the_schema_enumerates_counts_as_recorded`; `a_run_that_recorded_a_partial_check_reports_nothing` |
| 9 | the stylesheet rule for `partial` is removed | 176 | `every_enumerated_state_value_has_a_label` ("`[data-outcome="partial"]` has no rule"); `tests::every_own_asset_matches_its_recorded_hash`; `tests::every_generated_address_matches_the_derivation`; and 173 more, every test whose project `init` sets up |
| 10 | the `partial` paragraph is removed from `archive.md` | 1 | `instructions::tests::the_archive_instructions_say_when_a_check_is_partial` |

## What the table shows

- **Mutation 7 has one catcher, and it is the test task 2.2 added.** No other test reads
  `review/unverified-requirement` on a `specification` document with a `partial` verification: the
  review module's other tests declare a `tasks` document, and the conformance harness checks that
  a fixture's declared rules fire, never that others stay silent. Before this change nothing would
  have caught it, which is what design decision 6 found.
- **Mutation 3 has one catcher**, the unit test of document-schema's "An unmarked paragraph does not
  state it". No fixture holds a `partial` verification with a note and no `unchecked` paragraph.
- **Mutation 4 is caught by the mapping fixture alone**, which is its job: `named-fields` is the
  contract an independent implementation reproduces, and it pins both paragraphs in order.
- **Mutation 6 is caught twice.** `the_not_run_outcome_is_found_in_the_schema` needed no edit, as
  design decision 6 said: it iterates the schema, so it reaches `partial` once the schema lists it.
- **Mutations 2 and 9 fail widely, and each is still named by the test meant for it.** A rule keyed
  on `passed` rejects every document whose verification passed, which most command-line projects
  hold. A stylesheet edited in place no longer matches its address, so every project `init` sets up
  reports `asset/mismatch`. `every_enumerated_state_value_has_a_label` is the test that says what
  went missing.

## A first run that was discarded

A first run of the same ten mutations restored each file with `shutil.copy2`, which copies the
file's modification time along with its bytes. Cargo rebuilds by modification time, so a restored
file looked older than the build made from its mutated version. Mutation 5's build then failed on a
transient `failed to create dependency graph … No such file or directory` under
`target/debug/incremental`, and mutations 6 to 10 ran against a parser that still carried mutation
5: each listed mutation 5's failures beside its own. The hash check after that run passed, because
the bytes were restored; what was stale was the build. That run is discarded. The runner now gives
each restored file a new modification time and retries a run whose build fails, and the table above
is the second run alone, from a clean baseline.
