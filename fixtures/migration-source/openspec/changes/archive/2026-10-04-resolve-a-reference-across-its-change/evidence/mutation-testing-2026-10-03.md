# Mutation testing

**Date:** 2026-10-03
**Task:** 5.4
**Script:** `mutate.py`, reproduced below. It applies each mutation alone, runs
`cargo test --workspace --all-targets --no-fail-fast` inside `nix develop`, records every test that
failed, and restores the file before the next mutation. A last run with nothing mutated passes.

## Every mutation is caught

| Mutation | Caught by |
| --- | --- |
| 1. Skip the change's specification | 15 tests, in the CLI, the conformance suite, `references.rs`, and `refs.rs` |
| 2. Skip the accepted specifications | 10 tests, among them `a_task_naming_an_accepted_requirement_resolves` and `an_accepted_specification_resolves_in_another` |
| 3. Try the accepted specifications before the change's specification | `a_task_naming_a_requirement_its_change_withdraws_is_a_removed_target`, `the_change_specifications_withdrawal_comes_before_the_accepted_unit`, `a_change_specifications_unit_comes_before_the_accepted_one`, and three more |
| 4. Resolve a link-origin reference through the scope | `a_fragment_link_resolves_only_in_its_own_document`, `a_fragment_link_to_the_change_specification_is_unresolved_and_broken`, `a_link_reference_is_not_resolved_through_the_scope` |
| 5. Treat an unread document as an empty set | `a_specification_that_needs_recovery_is_named_as_unread` |
| 6. Apply `REMOVED_TARGET` only in the document itself | the CLI, conformance, and `refs.rs` tests of a withdrawal in the change's specification |
| 7. Place an archived change's documents as loose | `an_archived_changes_task_naming_its_own_specification_resolves`, `every_document_is_placed_by_where_it_sits` |
| 8. Place a change's evidence as a change document | `every_document_is_placed_by_where_it_sits` |
| 9. Let a runbook's document reach the accepted specifications | `a_runbook_document_reaches_only_itself`, `a_runbooks_task_naming_an_accepted_requirement_is_unresolved`, `each_placement_reaches_the_places_of_the_design_in_order` |
| 10. Let an accepted specification reach only itself | `an_accepted_specification_naming_anothers_requirement_resolves`, `an_accepted_specification_resolves_in_another`, `each_placement_reaches_the_places_of_the_design_in_order` |

**Two mutations have one catch each, and both are deliberate.**

- **Mutation 5 is caught only by the end-to-end test.** Reading a document, and calling it unread, is
  the command line's work, so the validator's own tests build an unread place directly and cannot see
  the reading go wrong.
- **Mutation 8 is caught only by the placement test.** Under it, a file in a change's evidence is
  placed as a change document. Its change's specification is then looked for beside the file, in the
  evidence directory, where there is none, so the end-to-end evidence case still reports
  `refs/unresolved`. The placement test is what pins the row.

## One more, for the fix the code review asked for

The code review of PR #43 found a document named by its bare file name placed loose. Task 7.1 added
`project::directory_of` and `a_document_named_by_its_bare_file_name_is_placed_where_it_sits`. Undoing the
fix in `ProjectRoot::placement`, so that it splits the path with `Path::parent` again, fails that test on
`tasks.html` and passes `./tasks.html`, which is the defect as found.

## The run

```
== 1. skip the change's specification
   a_dangling_reference_names_the_places_searched
   a_decision_in_the_design_naming_its_change_specification_resolves
   a_design_reaches_what_a_tasks_document_does
   a_reached_specification_is_read_and_not_reported
   a_specification_that_needs_recovery_is_named_as_unread
   a_task_naming_a_requirement_its_change_withdraws_is_a_removed_target
   a_task_naming_its_change_specifications_requirement_resolves
   a_task_resolves_in_its_change_specification
   an_archived_changes_task_naming_its_own_specification_resolves
   an_unread_document_is_named_and_not_treated_as_empty
   references::tests::each_placement_reaches_the_places_of_the_design_in_order
   rules::refs::tests::a_change_specifications_unit_comes_before_the_accepted_one
   rules::refs::tests::an_unread_document_is_named_in_the_report
   the_change_specifications_withdrawal_comes_before_the_accepted_unit
   validating_one_document_reports_nothing_about_the_documents_it_reaches

== 2. skip the accepted specifications
   a_change_with_no_specification_reaches_the_accepted_contract
   a_dangling_reference_names_the_places_searched
   a_task_naming_an_accepted_requirement_resolves
   a_task_resolves_in_the_accepted_specifications
   an_accepted_specification_naming_anothers_requirement_resolves
   an_accepted_specification_resolves_in_another
   an_identifier_two_accepted_specifications_declare_resolves
   references::tests::each_placement_reaches_the_places_of_the_design_in_order
   rules::refs::tests::an_identifier_two_accepted_documents_declare_resolves
   rules::refs::tests::an_unread_document_is_named_in_the_report

== 3. try the accepted specifications before the change's specification
   a_dangling_reference_names_the_places_searched
   a_task_naming_a_requirement_its_change_withdraws_is_a_removed_target
   references::tests::each_placement_reaches_the_places_of_the_design_in_order
   rules::refs::tests::a_change_specifications_unit_comes_before_the_accepted_one
   rules::refs::tests::an_unread_document_is_named_in_the_report
   the_change_specifications_withdrawal_comes_before_the_accepted_unit

== 4. resolve a link-origin reference through the scope
   a_fragment_link_resolves_only_in_its_own_document
   a_fragment_link_to_the_change_specification_is_unresolved_and_broken
   rules::refs::tests::a_link_reference_is_not_resolved_through_the_scope

== 5. treat an unread document as an empty set
   a_specification_that_needs_recovery_is_named_as_unread

== 6. apply REMOVED_TARGET only in the document itself
   a_task_naming_a_requirement_its_change_withdraws_is_a_removed_target
   rules::refs::tests::a_change_specifications_unit_comes_before_the_accepted_one
   the_change_specifications_withdrawal_comes_before_the_accepted_unit

== 7. place an archived change's documents as loose
   an_archived_changes_task_naming_its_own_specification_resolves
   project::tests::every_document_is_placed_by_where_it_sits

== 8. place a change's evidence as a change document
   project::tests::every_document_is_placed_by_where_it_sits

== 9. let a runbook's document reach the accepted specifications
   a_runbook_document_reaches_only_itself
   a_runbooks_task_naming_an_accepted_requirement_is_unresolved
   references::tests::each_placement_reaches_the_places_of_the_design_in_order

== 10. let an accepted specification reach only itself
   an_accepted_specification_naming_anothers_requirement_resolves
   an_accepted_specification_resolves_in_another
   references::tests::each_placement_reaches_the_places_of_the_design_in_order

== restored: every test passes
```

## Appendix: `mutate.py`

```python
"""Applies each mutation of resolve-a-reference-across-its-change alone, runs the workspace's tests,
and restores the file.

Run from the repository root, outside the Nix shell: python3 mutate.py
Prints, for each mutation, every test that failed, or that the build failed.
"""

import re
import subprocess
from pathlib import Path

REFERENCES = Path("crates/htmlspec-validate/src/references.rs")
REFS = Path("crates/htmlspec-validate/src/rules/refs.rs")
COMMANDS = Path("crates/htmlspec-cli/src/commands.rs")
PROJECT = Path("crates/htmlspec-cli/src/project.rs")

CHANGE_DOCUMENT = "            Self::ChangeDocument => &[Place::ChangeSpecification, Place::AcceptedSpecifications],\n"
MUTATIONS = [
    ("1. skip the change's specification", REFERENCES, CHANGE_DOCUMENT,
     "            Self::ChangeDocument => &[Place::AcceptedSpecifications],\n"),
    ("2. skip the accepted specifications", REFERENCES,
     CHANGE_DOCUMENT
     + "            Self::ChangeSpecification | Self::AcceptedSpecification => {\n"
       "                &[Place::AcceptedSpecifications]\n"
       "            }\n",
     "            Self::ChangeDocument => &[Place::ChangeSpecification],\n"
     "            Self::ChangeSpecification | Self::AcceptedSpecification => &[],\n"),
    ("3. try the accepted specifications before the change's specification", REFERENCES,
     CHANGE_DOCUMENT,
     "            Self::ChangeDocument => &[Place::AcceptedSpecifications, Place::ChangeSpecification],\n"),
    ("4. resolve a link-origin reference through the scope", REFS,
     "            ReferenceOrigin::Link => Found::Nowhere {\n"
     "                searched: Vec::new(),\n"
     "                unread: Vec::new(),\n"
     "            },\n",
     "            ReferenceOrigin::Link => scope.find(target),\n"),
    ("5. treat an unread document as an empty set", COMMANDS,
     "            Reading::Unread(document) => unread.push(document.clone()),\n",
     "            Reading::Unread(_) => {}\n"),
    ("6. apply REMOVED_TARGET only in the document itself", REFERENCES,
     "                    withdrawn: declared.withdrawn.contains(target),\n",
     "                    withdrawn: false,\n"),
    ("7. place an archived change's documents as loose", PROJECT,
     "            && (holder == Some(canonical(&self.changes()).as_path())\n"
     "                || holder == Some(archive.as_path()));\n",
     "            && holder == Some(canonical(&self.changes()).as_path());\n"),
    ("8. place a change's evidence as a change document", PROJECT,
     "        if self.is_within_change_evidence(path) {\n            return Placement::Loose;\n        }\n",
     "        if self.is_within_change_evidence(path) {\n            return Placement::ChangeDocument;\n        }\n"),
    ("9. let a runbook's document reach the accepted specifications", REFERENCES,
     "            Self::Loose | Self::RunbookDocument => &[],\n",
     "            Self::Loose => &[],\n            Self::RunbookDocument => &[Place::AcceptedSpecifications],\n"),
    ("10. let an accepted specification reach only itself", REFERENCES,
     "            Self::ChangeSpecification | Self::AcceptedSpecification => {\n"
     "                &[Place::AcceptedSpecifications]\n"
     "            }\n",
     "            Self::ChangeSpecification => &[Place::AcceptedSpecifications],\n"
     "            Self::AcceptedSpecification => &[],\n"),
]

FAILED = re.compile(r"^test (\S+) \.\.\. FAILED$", re.M)


def run():
    result = subprocess.run(
        ["nix", "develop", "-c", "cargo", "test", "--workspace", "--all-targets", "--no-fail-fast"],
        capture_output=True, text=True)
    output = result.stdout + result.stderr
    if "error[E" in output or "could not compile" in output:
        return None, output
    return sorted(set(FAILED.findall(output))), output


def main():
    for title, path, old, new in MUTATIONS:
        original = path.read_text()
        assert original.count(old) == 1, f"{title}: the pattern is not unique in {path}"
        path.write_text(original.replace(old, new, 1))
        try:
            failed, output = run()
        finally:
            path.write_text(original)
        print(f"== {title}", flush=True)
        if failed is None:
            print("   the build failed")
            print(output[-1500:])
        elif not failed:
            print("   SURVIVED: no test failed")
        else:
            for name in failed:
                print(f"   {name}")
        print(flush=True)
    clean, _ = run()
    print(f"== restored: {'every test passes' if clean == [] else clean}", flush=True)


if __name__ == "__main__":
    main()
```
