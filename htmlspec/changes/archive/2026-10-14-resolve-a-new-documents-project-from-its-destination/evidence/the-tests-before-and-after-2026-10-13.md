# The tests, before the repair and after it

**Date:** 2026-10-13
**Tasks:** 1.1 to 1.3, and 2.2.

## Before the repair

The six tests tasks 1.1 to 1.3 add, and the two existing `new document` tests, run against the
unrepaired `new_document`, which resolves its project from the working directory:

```
$ nix develop -c cargo test -p htmlspec-cli --test cli --locked -- new_document a_document_written new_change_from_a_subdirectory
test new_document_refuses_to_overwrite_and_exits_with_an_operational_failure ... ok
test new_document_creates_a_document_that_validates_immediately ... FAILED
test a_document_written_outside_the_project_it_is_created_from_gets_its_own_store ... FAILED
test a_document_written_into_a_projects_root_outside_its_directory_gets_its_own_store ... FAILED
test a_document_written_into_a_project_with_no_store_establishes_the_projects_store ... FAILED
test a_document_written_into_a_project_from_outside_references_the_projects_store ... FAILED
test a_document_written_outside_every_project_gets_its_own_store ... ok
test new_change_from_a_subdirectory_lands_in_the_project ... ok
test result: FAILED. 3 passed; 5 failed; 0 ignored; 0 measured; 63 filtered out; finished in 1.34s
```

The four arrangements fail as `evidence/the-four-cases-measured-2026-10-13.md` predicts: written outside
the project it is created from (A), into a project's root outside its directory (B), and into a
project from outside (C) fail, and written outside every project (D) passes. The project with no store
fails too: run from outside it, the document's store is put beside the document, inside the project's
`htmlspec/`, and the project's own store is never established. `new change` from a subdirectory
passes, as it must, because this change does not touch it.

The first version of the no-store test passed here, because it put the directory it ran from inside the
project's root, so the working directory found the project. The test was corrected to make the two
directories siblings before this run.

## After the repair

`new_document` resolves the project from the destination's directory, normalized lexically, and
references the project's store only for a destination beneath its `htmlspec/`. The same tests, the two
unit tests the repair adds, and `new_document_and_post_document_agree` in `tests/verbs.rs`, run from
inside this repository with the promoted project in place:

```
$ nix develop -c cargo test -p htmlspec-cli --locked -- new_document a_document_written new_change_from_a_subdirectory a_destination_spelled normalize_resolves
test commands::tests::normalize_resolves_dot_components_lexically ... ok
test commands::tests::a_destination_spelled_through_a_parent_is_placed_where_it_lands ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 462 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 74 filtered out; finished in 0.00s
test new_document_refuses_to_overwrite_and_exits_with_an_operational_failure ... ok
test a_document_written_into_a_project_with_no_store_establishes_the_projects_store ... ok
test new_document_creates_a_document_that_validates_immediately ... ok
test a_document_written_outside_every_project_gets_its_own_store ... ok
test new_change_from_a_subdirectory_lands_in_the_project ... ok
test a_document_written_into_a_project_from_outside_references_the_projects_store ... ok
test a_document_written_outside_the_project_it_is_created_from_gets_its_own_store ... ok
test a_document_written_into_a_projects_root_outside_its_directory_gets_its_own_store ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 63 filtered out; finished in 1.51s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 35 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s
test new_document_and_post_document_agree ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 2.45s
```

Every one passes, `new_document_creates_a_document_that_validates_immediately` among them.

## Found while running them: a document in a project's root

`new_document_and_post_document_agree` failed after the repair. It writes `notes.html` into a project's
root and asserted that the document was the only file written, so that it referenced
`htmlspec/assets/`. Its references descend into `htmlspec/`, and validation checks no reference when it
finds no store above the document. Measured in a scratch project, with a document copied to the root
and its references rewritten to `htmlspec/assets/…`:

```
$ htmlspec validate notes.html
  0 errors, 1 warning: the document is accepted
$ htmlspec validate bogus.html      # the stylesheet's hash replaced with sha256-0000000000000000
  0 errors, 1 warning: the document is accepted
```

A reference naming bytes no store holds is accepted as readily as a correct one. After the repair the
document gets a store beside it, which validation checks, and the test asserts that instead. Design
decision 2 and the risk "A document in a project's root gets a second copy of the runtime" record it.
