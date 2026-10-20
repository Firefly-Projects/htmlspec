# Mutation testing: nineteen mutations, and a stale literal they found

Task 9.1. Each new rule, each projection behavior the rules depend on, and each part of the
untaken-rename set was neutered on its own, the affected packages' tests were run, and the file was
restored. A script made each edit as a unique string replacement, ran
`cargo test -q --no-fail-fast` over the packages named in the table, and wrote the original file
back in a `finally` block. After the run, the five edited files were compared with checksums taken
before it, and all five matched byte for byte.

**Every mutation was caught.** The first pass ran without `--no-fail-fast`, so it recorded only the
first test binary to fail, which was the conformance suite for every validator mutation. The
second pass, below, ran every binary.

## The table

| # | Mutation | Packages | Caught by |
| --- | --- | --- | --- |
| 1 | `rename-outside-requirement` reports nothing | validate, conformance | `a_rename_outside_a_requirement_is_reported_once_per_unit`, and the fixture harness's two assertions |
| 2 | the `added` state never matches | validate, conformance | `a_rename_on_an_added_requirement_is_reported`, `independent_faults_on_one_requirement_are_each_reported`, fixtures |
| 3 | the `removed` state never matches | validate, conformance | `a_rename_on_a_removed_requirement_is_reported`, `a_diagnostic_names_the_requirement`, fixtures |
| 4 | "repeated" needs three paragraphs rather than two | validate, conformance | `independent_faults_on_one_requirement_are_each_reported`, fixtures |
| 5 | `rename-empty` reports nothing | validate, conformance | `an_empty_rename_is_reported`, `independent_faults_…`, fixtures |
| 6 | the same-name check compares the title, not the heading name | validate, conformance | `a_rename_to_the_requirements_own_heading_name_is_reported`, `independent_faults_…`, fixtures |
| 7 | the projection does not record a misplaced rename | parse, validate, conformance | `section_4_1_1_a_rename_outside_a_requirement_is_recorded_rather_than_faulted`, `a_rename_outside_…`, fixtures |
| 8 | the projection discards an empty `renamed-from` | parse, validate, conformance | `section_4_1_1_an_empty_rename_is_kept_and_projects_nothing`, `section_4_1_1_the_field_holds_the_first_paragraph_with_text`, `an_empty_rename_is_reported`, `independent_faults_…`, fixtures |
| 9 | a `renamed-from` paragraph becomes a statement | parse, validate, conformance | `every_fixture_projects_to_its_expected_model` (the `renamed-from` mapping fixture), `section_4_1_1_a_rename_outside_…` |
| 10 | a heading name keeps the identifier | parse, validate, conformance, cli | two `model::tests`, `section_4_1_2_…`, two `renames` tests, two `commands` tests, fixtures |
| 11 | a rename is never untaken | cli | `a_rename_the_accepted_specifications_have_not_taken_is_not_reached`, `archival_and_sync_name_the_same_untaken_renames`, `every_stated_requirement_is_in_exactly_one_set`, the CLI test `a_change_whose_only_unapplied_work_is_a_rename_has_not_been_synced` |
| 12 | every differing heading is compared, stated rename or not | cli | `a_heading_that_differs_with_no_rename_stated_is_reached` |
| 13 | accepted titles compared rather than heading names | cli | `a_rename_the_accepted_specifications_have_not_taken_…`, `…_have_taken_is_reached_and_nothing_else`, `every_stated_requirement_is_in_exactly_one_set` |
| 14 | an untaken rename is also reached | cli | `a_rename_the_accepted_specifications_have_not_taken_…`, `every_stated_requirement_…`, the CLI test |
| 15 | an identifier nobody holds is reported as an untaken rename | cli | `a_renamed_requirement_whose_identifier_is_not_held_is_absent_only` |
| 16 | `needsSync` ignores untaken renames | cli | `a_change_whose_only_unapplied_work_is_a_rename_is_unsynced`, `the_payload_carries_each_untaken_rename`, the CLI test |
| 17 | an untaken rename is not a finding | cli | `a_change_whose_only_unapplied_work_is_a_rename_is_unsynced`, the CLI test |
| 18 | `sync` reports "applied in full" while a rename is untaken | cli | the CLI test alone |
| 19 | the plan's collision key drops the destination | cli (`plan::`) | `one_obligation_renamed_across_changes_is_not_a_collision`, `one_heading_stated_by_several_changes_is_not_a_collision` |

## What the table says beyond "caught"

- **Mutation 13 is not caught by the test named for it.** `the_identifier_shown_beside_a_heading_is_not_a_rename`
  writes the accepted specification *without* the identifier span, so comparing accepted titles
  instead of heading names changes nothing on that side. Three other tests catch the mutation,
  because their accepted specifications carry the span. The named test pins the other half, the
  change's side, and mutation 10 is what shows it does.
- **Mutations 12, 15, and 18 are each caught by exactly one test.** Each is a single guard, and it
  is deliberate: one condition, tested where it is decided. It is recorded here so that the next
  person to delete one of those tests knows it is the only one.
- **Mutation 19 confirms the claim task 7.2 wrote into `plan.rs`.** Now that a rename keeps one
  identifier, the rename test catches a collision key that drops the destination. Before this
  change, the comment on the test after it recorded that the rename test could not.

## The finding: a hard-coded address, stale since task 8.1

`a_stale_asset_reference_is_not_reported_as_a_formatting_fault`, in
`crates/htmlspec-cli/tests/promote.rs`, failed under every CLI mutation, including ones that cannot
reach it. It was failing on its own. The test simulates a stale reference by replacing the
stylesheet's address in a staged document, and it named that address as the literal
`sha256-e4f5de7cecc4f691`. Task 8.1 gave the stylesheet a new address, so the replacement found
nothing, and the test's own guard stopped it (`the stylesheet address was not found`).

The full CLI suite had passed before task 8.1 and was not rerun after it, so the mutation run is
where this surfaced. **The fix reads the address from the embedded asset table**, as the
presentation checks already do. The test now follows the stylesheet rather than pinning one
revision of it. No other test names an asset address as a literal.
