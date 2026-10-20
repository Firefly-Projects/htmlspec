# Mutation testing the partition

Task 5.1. Each mutation edits `partition` in `crates/htmlspec-cli/src/topology.rs` alone, runs
`cargo test -q --no-fail-fast -p htmlspec-cli`, and restores the file. It was checked restored
after each run.

| Mutation | Caught by |
| --- | --- |
| **Every store is the project's**, which is the behavior before this change | `a_project_beside_a_fixture_store_is_not_a_duplicate_of_it`, `a_project_beside_a_toolchain_source_store_is_not_a_duplicate_of_it`, `the_asset_check_and_promotion_give_one_verdict`, `a_store_is_the_projects_exactly_when_it_lies_within_htmlspec`, `a_project_reached_through_a_symbolic_link_keeps_its_stores` |
| **No store is the project's**, so nothing is compared across stores | `two_stores_inside_the_project_are_still_compared`, the older `the_asset_check_reports_duplication_between_two_stores`, `a_project_beside_a_toolchain_source_store_…` (its payload names one project store), `the_asset_check_and_promotion_give_one_verdict`, and three `topology::tests` |
| **Paths compared without resolving symbolic links** | `a_project_reached_through_a_symbolic_link_keeps_its_stores` alone |

**Both directions the task names are caught, and by the CLI tests as well as the unit tests.** The
first mutation is the defect this change removes, restated, so it failing the two scenarios that
measured the defect is the point. The second is the failure the design's risk section names: a
partition that excuses everything would hide a project's real second copy. The tests written
against HEAD, which passed there, are what catch it.

The third mutation is caught by exactly one test. That test exists for this purpose: on macOS the
temporary directory itself is reached through a link (`/var` → `/private/var`), so a comparison
of raw paths would be wrong on the machine this project is developed on, and wrong without
anything else failing.
