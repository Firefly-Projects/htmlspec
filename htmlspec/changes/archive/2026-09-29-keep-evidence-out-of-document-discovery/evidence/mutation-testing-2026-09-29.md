# Mutation testing the exclusion

**Date:** 2026-09-29
**Task:** 5.2
**Method:** a script applies each mutation with a Perl substitution. It refuses to run the suite
when the substitution did not change the file, which would be a mutation that silently did not
apply. It then runs `cargo test -p htmlspec-cli --no-fail-fast` and records every failing test.
After each run it restores the file, and the MD5 sums of both sources are compared with sums taken
before the first mutation.

```
sources restored
```

## The six mutations and what caught each

| Mutation | Caught by |
| --- | --- |
| 1. **Name only**: any directory called `evidence` is a change's evidence | `a_capability_named_evidence_is_still_validated`, `an_evidence_directory_directly_in_the_archive_is_walked`, `outside_a_project_an_evidence_directory_is_walked`, `discover::tests::a_projects_walk_passes_over_a_changes_evidence`, `project::tests::a_directory_is_a_changes_evidence_exactly_when_a_change_holds_it` |
| 2. **The root is excluded too**: naming a change's evidence yields nothing | `naming_the_evidence_directory_checks_what_it_holds`, `discover::tests::a_root_inside_a_changes_evidence_is_walked_in_full` |
| 3. **Nothing is skipped**: the walk as it was on HEAD | `validating_a_project_passes_over_a_changes_evidence`, `formatting_a_project_leaves_a_changes_evidence_byte_identical`, `evidence_nested_below_the_evidence_directory_is_evidence`, `an_archived_changes_evidence_is_evidence`, `carried_evidence_in_this_format_does_not_gate_promotion`, `discover::tests::a_projects_walk_passes_over_a_changes_evidence` |
| 4. **Resolved from the working directory** rather than from the walk's root | `carried_evidence_in_this_format_does_not_gate_promotion`, `discover::tests::a_projects_walk_passes_over_a_changes_evidence` |
| 5. **Compared lexically**: the parent is not canonicalized | `project::tests::a_changes_evidence_is_recognized_however_its_path_is_spelled`, `project::tests::a_changes_evidence_is_recognized_through_a_symbolic_link`, `discover::tests::a_projects_walk_passes_over_a_changes_evidence` |
| 6. **The archive counts as a change**: `changes/archive/evidence` is excluded | `an_evidence_directory_directly_in_the_archive_is_walked`, `project::tests::a_directory_is_a_changes_evidence_exactly_when_a_change_holds_it` |

**Six of six caught, each by at least two tests.** Every mutation is caught
by a unit test in `project.rs` or `discover.rs` and by a test that drives the compiled program,
except mutation 5, which only unit tests catch. No program-level test is sensitive to path spelling,
and the reason is the operating system rather than the tests: a command with no path composes its
walk from the working directory, which the process is given with links already resolved, and
promotion's scratch directory is canonicalized when it is created. So in every program-level test
the walk's paths are already canonical, and skipping canonicalization changes nothing.

## Two results worth stating

**Mutation 4 is caught by promotion and not by `validate` or `format`.** Every CLI test for those
two commands that expects an exclusion runs the binary from inside the project it walks, so the working directory and the
walk's root resolve to the same project, and a mutation confusing the two cannot show. Promotion is
different: it is run from the directory above the staged tree, and the staged tree is a project the
working directory is not in. So promotion is the one command where "resolve from the root" is
observable, and it is also the command this change exists for. The discovery unit test catches the
mutation too, because it runs with the crate directory as its working directory.

**Mutation 5 is caught in `discover.rs` by accident as well as on purpose.** On macOS the temporary
directory is reached through a link: `/var/folders/…` or, inside `nix develop`, `/tmp/nix-shell.…`,
whose real paths begin `/private`. `discover.rs`'s scratch directory is not canonicalized, so the walk
composes paths through the link. The mutated code compares the uncanonicalized parent with the
canonicalized `changes()`, and they differ. The two `project.rs` tests are the deliberate coverage.
The `discover.rs` test catches the mutation only on a platform whose temporary directory is a link.

## After the review: two more

The review's finding added a writer's question, `ProjectRoot::is_within_change_evidence`, and a
refusal in `format`'s writing mode. Both were mutated the same way, and the sources were restored
and fingerprinted afterwards.

| Mutation | Caught by |
| --- | --- |
| 7. **`format` writes a named record**: the refusal never fires | `formatting_a_named_record_is_refused_and_writes_nothing` |
| 8. **A record is judged lexically**: the path is not canonicalized first | `project::tests::a_path_lies_within_a_changes_evidence_exactly_when_an_evidence_directory_holds_it` |

Eight of eight. Mutation 8 is caught by the `evidence/../design.html` case: a path that names
`evidence` on the way to a file lying beside it.
