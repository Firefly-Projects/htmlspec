# The tests can fail: one mutation

**Date:** 2026-10-13
**Task:** 5.1.

The repair is one function, `project_store_root` in `crates/htmlspec-cli/src/commands.rs`. The mutation
puts back what it replaced, by returning at the top of the function:

```rust
return ProjectRoot::resolve(Path::new(".")).ok().map(|project| project.htmlspec());
```

So the working directory finds the project again, and its store is referenced wherever the document
lies.

```
$ nix develop -c cargo test -p htmlspec-cli --locked --no-fail-fast -- new_document a_document_written new_change_from_a_subdirectory a_destination_spelled
test commands::tests::a_destination_spelled_through_a_parent_is_placed_where_it_lands ... FAILED
test new_document_refuses_to_overwrite_and_exits_with_an_operational_failure ... ok
test new_document_creates_a_document_that_validates_immediately ... FAILED
test a_document_written_into_a_projects_root_outside_its_directory_gets_its_own_store ... FAILED
test a_document_written_outside_the_project_it_is_created_from_gets_its_own_store ... FAILED
test a_document_written_into_a_project_with_no_store_establishes_the_projects_store ... FAILED
test a_document_written_outside_every_project_gets_its_own_store ... ok
test a_document_written_into_a_project_from_outside_references_the_projects_store ... FAILED
test new_change_from_a_subdirectory_lands_in_the_project ... ok
test new_document_and_post_document_agree ... ok
```

Seven tests kill it: the three failing arrangements (A, B, C), the project with no store, the unit test
for a destination spelled through `..`, and the test that failed the day this repository became a
project. The three that pass are the ones this change must not move: a document written outside every
project (D), `new change` from a subdirectory, and refusing to overwrite.

**One test survived the first run, and was strengthened.** `new_document_and_post_document_agree`
passed under the mutation, because its corrected assertion, written in task 2.3, allowed a store beside
the document and did not require one. It now also requires the store:

```
== with the repair
test new_document_and_post_document_agree ... ok
== with the mutation
test new_document_and_post_document_agree ... FAILED
```

The mutation was removed after the run, and `grep -c "return ProjectRoot::resolve(Path::new"` over the
file finds 0.

## A second mutation, from the code review

The review of pull request #56 found the first version of the repair comparing the destination with the
project's `htmlspec/` by spelling, with `directory.starts_with(project.htmlspec())`. On a filesystem that
ignores case, as this machine's does, the walk finds the project through `HTMLSPEC/` and the comparison
does not, so the document is given a second store inside the project. The repair now asks the
filesystem whether the destination's first directory below the project's root and `htmlspec/` are one
directory.

The reviewed comparison, put back, against the two unit tests added for it:

```
test commands::tests::the_projects_directory_in_another_case_uses_its_store_where_case_is_ignored ... FAILED
test commands::tests::another_name_for_the_projects_directory_uses_its_store ... FAILED
```

Both fail, and both pass with the repair. The case test runs only where the filesystem ignores case, and
it ran here. The review's own scenario, end to end with the repaired binary, from outside a scratch
project into `HTMLSPEC/changes/c/proposal.html`:

```
<scratch>/p/HTMLSPEC/changes/c/proposal.html: created a proposal
exit 0
$ ls <scratch>/p/htmlspec/changes/c
proposal.html
$ htmlspec assets          # in the project
each stored asset appears once
$ htmlspec validate <scratch>/p/HTMLSPEC/changes/c/proposal.html
  0 errors, 1 warning: the document is accepted
```

The mutation was removed after the run.
