# The tests, before and after the fix

**Date:** 2026-10-14
**Tree:** this change's branch, after tasks 1.1 to 1.4 and before any of 2.1 to 2.3.

## Before the fix

The new types existed so the tests compiled, with the unrepaired behaviour behind them:
`htmlspec_format::relocate` returned its source unchanged, which is what 0.29.1's archival does to a
document, and `Store::seen_through` returned the store as it was, which resolves a reference by its
address alone, as 0.29.1 does. Archival and recovery were 0.29.1's.

### Task 1.2, the rewrite (`cargo test -p htmlspec-format --lib relocate`)

```
test relocate::tests::a_place_spelled_with_parent_steps_is_resolved ... ok
test relocate::tests::a_relative_place_is_refused ... ok
test relocate::tests::text_and_unread_attributes_are_unchanged ... ok
test relocate::tests::a_move_to_the_same_place_changes_nothing ... ok
test relocate::tests::a_reference_within_the_directory_is_unchanged ... ok
test relocate::tests::a_reference_naming_no_relative_path_is_unchanged ... ok
test relocate::tests::a_reference_out_of_the_directory_reaches_the_same_file_from_the_new_one ... FAILED
test relocate::tests::a_query_and_a_fragment_are_kept ... FAILED
test relocate::tests::a_reference_leaving_and_reentering_is_expressed_by_where_it_lands ... FAILED
test relocate::tests::a_directory_reference_keeps_its_trailing_slash ... FAILED
test relocate::tests::every_loading_attribute_and_both_hyperlinks_are_relocated ... FAILED
test relocate::tests::every_byte_outside_a_rewritten_value_is_copied ... FAILED
test result: FAILED. 6 passed; 6 failed
```

The six that pass are the ones that assert a value is left alone, which a function that changes
nothing satisfies. They pin what the fix must not touch. The six that fail assert a rewrite.

### Task 1.3, the path (`htmlspec-assets` and `htmlspec-validate`, and `tests/cli.rs`)

```
test store::tests::seen_from_a_document_a_path_that_misses_the_store_leads_elsewhere ... FAILED
    left: Match
   right: Elsewhere { leads_to: "../../assets/", store: "../../../assets/" }
test store::tests::seen_from_a_document_an_absolute_path_leads_elsewhere ... FAILED
    /assets/sha256-277089d91c0bdf4f/htmlspec.css resolved through a path that does not reach the store
test rules::assets::tests::a_reference_from_beside_the_store_reaches_it ... FAILED
test rules::assets::tests::a_reference_whose_path_reaches_another_directory_is_unresolved ... FAILED
test rules::assets::tests::a_reference_whose_path_reaches_the_store_is_accepted ... ok
test a_reference_whose_path_misses_the_store_is_unresolved ... FAILED
    a change whose references reach no store was accepted (validate exited 0)
```

Each failure is the defect: a reference whose path misses the store resolves by its address and is
accepted. The one that passes asserts acceptance, which the address-only rule also gives.

### Tasks 1.1 and 1.4, archival and recovery (`tests/archive.rs`)

```
test archival_keeps_every_reference_reaching_what_it_reached ... FAILED
    the archived proposal's references do not reach what they reached
    left:  <link href="../../assets/sha256-03627137bcbb81c9/htmlspec.css" ...
    right: <link href="../../../assets/sha256-03627137bcbb81c9/htmlspec.css" ...
test an_interruption_before_the_move_removes_the_staged_documents ... FAILED
test an_interruption_after_the_move_puts_the_staged_documents_in_place ... FAILED
test an_edited_staged_document_is_refused ... FAILED
    each: the journal "declares the format "htmlspec-archive-journal/v2" rather than
    "htmlspec-archive-journal/v1"", so recovery refused it as unreadable
test a_leftover_staged_document_is_refused ... FAILED
    archived to .../2026-10-14-leftover-staged at version 0.1.0
```

A unit test in `crates/htmlspec-cli/src/commands.rs`, `a_journal_replaces_only_documents_in_the_change`,
states the v2 journal's shape: a replacement names a document in the change by its name, and a name
reaching anywhere else is refused. Before the fix it does not compile, because the journal has no
replacements to read.

Task 1.4's "A v1 journal is still settled" adds no CLI test. The five interrupted-publication tests
already in `tests/archive.rs` that leave a journal to settle write a v1 one, and they pass before and
after.

## After the fix

After tasks 2.1 to 2.3, every test above passes, and so does the rest of the workspace:
`nix develop -c cargo test --locked --workspace` ran 1456 tests, with 0 failures.

```
test relocate::tests::… (all 12) ... ok
test store::tests::seen_from_a_document_a_path_that_misses_the_store_leads_elsewhere ... ok
test store::tests::seen_from_a_document_an_absolute_path_leads_elsewhere ... ok
test rules::assets::tests::a_reference_from_beside_the_store_reaches_it ... ok
test rules::assets::tests::a_reference_whose_path_reaches_another_directory_is_unresolved ... ok
test rules::assets::tests::a_reference_whose_path_reaches_the_store_is_accepted ... ok
test commands::tests::a_journal_replaces_only_documents_in_the_change ... ok
test a_reference_whose_path_misses_the_store_is_unresolved ... ok
test archival_keeps_every_reference_reaching_what_it_reached ... ok
test an_interruption_before_the_move_removes_the_staged_documents ... ok
test an_interruption_after_the_move_puts_the_staged_documents_in_place ... ok
test an_edited_staged_document_is_refused ... ok
test a_leftover_staged_document_is_refused ... ok
```

### Two fixtures that carried the defect

The first full run after task 2.3 failed two existing tests in `tests/cli.rs`,
`an_archived_changes_task_naming_its_own_specification_resolves` and
`marks_in_an_archived_changes_specification_are_accepted`. Each built an archived change by renaming
an active one into `htmlspec/changes/archive/`, which is what 0.29.1's archival did, so each archived
document referenced `../../assets/`, and validation now rejects that:

```
error: asset/unresolved at /link[href]: <link href="../../assets/sha256-03627137bcbb81c9/htmlspec.css">
leads to ../../assets/ from this document, and the project's store is at ../../../assets/. …
```

Neither test is about the store. One asserts that an archived change's task resolves a requirement in
its own specification, and the other that an archived specification keeps its change marks. So the
fixtures were corrected rather than the rule: a helper, `archive_by_hand`, renames the change and
re-expresses its references out of it as archival now does. Both tests keep every assertion they had,
and pass.

One unit test, `a_journal_holds_only_absolute_paths`, wrote a journal in the current format with no
`replacements` list, which the v2 format requires, so the parser would refuse it for that before
reaching the relative path the test is about. The list was added, empty, and the test asserts what it did.
