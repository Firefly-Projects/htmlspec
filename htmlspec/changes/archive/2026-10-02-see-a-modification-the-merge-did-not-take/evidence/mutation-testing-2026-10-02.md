# Mutation testing

**Date:** 2026-10-02
**Task:** 5.2
**Script:** `mutate.py`, reproduced below. It applies each mutation alone, runs
`cargo test --workspace --all-targets --no-fail-fast` inside `nix develop`, records every test that
failed, and restores the file before the next mutation. A last run with nothing mutated passes.

## Every mutation is caught

| Mutation | Caught by |
| --- | --- |
| 1. Compare `modified` requirements by identifier only | 9 tests, among them `a_word_the_accepted_specifications_have_not_taken_is_not_reached`, `every_stated_requirement_is_in_exactly_one_set`, and the end-to-end `a_change_whose_only_unapplied_work_is_a_word_has_not_been_synced` |
| 2. Drop `added` from the compared states | `an_added_requirement_is_compared_as_a_modified_one_is` |
| 3. Compare `unchanged` requirements' content too | `a_requirement_carried_forward_is_compared_by_its_identifier`, `every_stated_requirement_is_in_exactly_one_set` |
| 4. Count a `removed` requirement as reached when it is held, as HEAD does | `a_removal_the_merge_did_not_apply_is_an_untaken_removal`, the partition and parity tests, and the end-to-end `a_withdrawal_the_merge_did_not_apply_has_not_been_synced` |
| 5. Compare scenarios in order | `scenario_order_and_notes_do_not_decide_reaching` |
| 6. Compare statements as a collection | `statements_in_another_order_are_not_taken` |
| 7. Include notes in the content | `scenario_order_and_notes_do_not_decide_reaching` |
| 8. Stop leaving scenarios marked `removed` out of the change's side | `a_withdrawn_scenario_the_accepted_requirement_still_holds_is_not_kept`, `scenario_order_and_notes_do_not_decide_reaching`, `the_payload_carries_where_content_differs_and_no_text` |
| 9. Leave `untaken_content` out of `reached_in_full` | `a_change_whose_only_unapplied_work_is_a_word_is_unsynced`, `a_word_the_accepted_specifications_have_not_taken_is_not_reached`, the payload test, and the end-to-end word test |
| 10. Leave `untaken_removals` out of `reached_in_full` | `a_change_whose_only_unapplied_work_is_a_withdrawal_is_unsynced`, and the end-to-end `a_withdrawal_the_merge_did_not_apply_has_not_been_synced` |
| 11. Put statement text into a payload entry | `the_payload_carries_where_content_differs_and_no_text`, and four tests whose expected entry it changes |
| 12. Report a requirement both renamed and modified in the rename set only | `a_rename_and_a_modification_neither_taken_are_both_reported`, `every_stated_requirement_is_in_exactly_one_set` |
| 13. Remove `untakenContent` from `sync.md` | `every_set_the_sync_payload_carries_is_named_in_its_instructions` |

**The guards earn their place.** Mutations 5 and 7 make the comparison stricter than the requirement,
and each is caught only by `scenario_order_and_notes_do_not_decide_reaching`, one of the three tests
task 2.2 wrote to pass on HEAD. Mutation 3 is caught by the second,
`a_requirement_carried_forward_is_compared_by_its_identifier`. A suite of tests that only look for a
finding would pass all three.

**Mutation 11 is the prohibition on merge text.** Its one dedicated catch is the payload test, which
looks for every statement and scenario text of both sides in the emitted payload. The other four fail
because the entry they expect gains a name, which a mutation putting text somewhere else would not
cause.

After the run, `every_change_state_is_judged_one_way` was added, to pin which of design decision 1's
three cases each schema state falls in. It reads the states from the schema and would also fail under
mutations 1 and 2. It was not part of this run.

## One more, for the fix the code review asked for

The code review of PR #42 found a requirement marked `unchanged` holding a scenario marked `added`,
which was judged by its own mark alone and never compared. Task 7.1 added `Declared::of_requirement`
and two tests. Undoing the fix was applied by hand and run against `cargo test -p htmlspec-cli
--no-fail-fast`:

| Mutation | Caught by |
| --- | --- |
| 14. Judge a requirement by its own change state alone, as `Declared::of(&requirement.change)` | `a_scenario_changed_inside_an_unchanged_requirement_is_compared`, and the end-to-end `a_scenario_added_inside_an_unchanged_requirement_has_not_been_synced` |

## The run

```
== 1. compare modified requirements by identifier only
   a_change_whose_only_unapplied_work_is_a_word_has_not_been_synced
   commands::tests::a_rename_and_a_modification_neither_taken_are_both_reported
   commands::tests::a_scenario_the_accepted_specifications_have_not_taken_is_named_by_its_identifier
   commands::tests::a_withdrawn_scenario_the_accepted_requirement_still_holds_is_not_kept
   commands::tests::a_word_the_accepted_specifications_have_not_taken_is_not_reached
   commands::tests::archival_and_sync_name_the_same_untaken_work
   commands::tests::every_stated_requirement_is_in_exactly_one_set
   commands::tests::statements_in_another_order_are_not_taken
   commands::tests::the_payload_carries_where_content_differs_and_no_text

== 2. drop added from the compared states
   commands::tests::an_added_requirement_is_compared_as_a_modified_one_is

== 3. compare unchanged requirements' content too
   commands::tests::a_requirement_carried_forward_is_compared_by_its_identifier
   commands::tests::every_stated_requirement_is_in_exactly_one_set

== 4. count a removed requirement as reached when it is held
   a_withdrawal_the_merge_did_not_apply_has_not_been_synced
   commands::tests::a_removal_the_merge_did_not_apply_is_an_untaken_removal
   commands::tests::archival_and_sync_name_the_same_untaken_work
   commands::tests::every_stated_requirement_is_in_exactly_one_set

== 5. compare scenarios in order
   commands::tests::scenario_order_and_notes_do_not_decide_reaching

== 6. compare statements as a collection
   commands::tests::statements_in_another_order_are_not_taken

== 7. include notes in the content
   commands::tests::scenario_order_and_notes_do_not_decide_reaching

== 8. stop leaving scenarios marked removed out of the change's side
   commands::tests::a_withdrawn_scenario_the_accepted_requirement_still_holds_is_not_kept
   commands::tests::scenario_order_and_notes_do_not_decide_reaching
   commands::tests::the_payload_carries_where_content_differs_and_no_text

== 9. leave untaken_content out of reached_in_full
   a_change_whose_only_unapplied_work_is_a_word_has_not_been_synced
   commands::tests::a_change_whose_only_unapplied_work_is_a_word_is_unsynced
   commands::tests::a_word_the_accepted_specifications_have_not_taken_is_not_reached
   commands::tests::the_payload_carries_where_content_differs_and_no_text

== 10. leave untaken_removals out of reached_in_full
   a_withdrawal_the_merge_did_not_apply_has_not_been_synced
   commands::tests::a_change_whose_only_unapplied_work_is_a_withdrawal_is_unsynced

== 11. put statement text into a payload entry
   a_change_whose_only_unapplied_work_is_a_word_has_not_been_synced
   commands::tests::a_rename_and_a_modification_neither_taken_are_both_reported
   commands::tests::a_word_the_accepted_specifications_have_not_taken_is_not_reached
   commands::tests::statements_in_another_order_are_not_taken
   commands::tests::the_payload_carries_where_content_differs_and_no_text

== 12. report a requirement both renamed and modified in the rename set only
   commands::tests::a_rename_and_a_modification_neither_taken_are_both_reported
   commands::tests::every_stated_requirement_is_in_exactly_one_set

== 13. remove untakenContent from sync.md
   every_set_the_sync_payload_carries_is_named_in_its_instructions

== restored: every test passes
```

## Appendix: `mutate.py`

```python
"""Applies each mutation of see-a-modification-the-merge-did-not-take alone, runs the workspace's
tests, and restores the file.

Run from the repository root, outside the Nix shell: python3 mutate.py
Prints, for each mutation, every test that failed, or that the build failed.
"""

import re
import subprocess
from pathlib import Path

COMMANDS = Path("crates/htmlspec-cli/src/commands.rs")
SYNC = Path("crates/htmlspec-assets/instructions/sync.md")

MUTATIONS = [
    (
        "1. compare modified requirements by identifier only",
        COMMANDS,
        '.filter(|state| state.contains("add") || state.contains("modif"))',
        '.filter(|state| state.contains("add"))',
    ),
    (
        "2. drop added from the compared states",
        COMMANDS,
        '.filter(|state| state.contains("add") || state.contains("modif"))',
        '.filter(|state| state.contains("modif"))',
    ),
    (
        "3. compare unchanged requirements' content too",
        COMMANDS,
        "state.place(id, untaken_rename(requirement, id, &headings), None);",
        "state.place(id, untaken_rename(requirement, id, &headings), "
        "untaken_content(requirement, id, accepted));",
    ),
    (
        "4. count a removed requirement as reached when it is held",
        COMMANDS,
        "Declared::Withdraws => state.untaken_removals.push(id.clone()),",
        "Declared::Withdraws => state.reached.push(id.clone()),",
    ),
    (
        "5. compare scenarios in order",
        COMMANDS,
        "        match unmatched\n"
        "            .iter()\n"
        "            .position(|(_, _, accepted)| *accepted == text)\n"
        "        {",
        "        match unmatched.first().filter(|(_, _, accepted)| *accepted == text).map(|_| 0) {",
    ),
    (
        "6. compare statements as a collection",
        COMMANDS,
        "    let first_differing = first_differing_statement(\n"
        "        &normalized_all(&requirement.statements),\n"
        "        &normalized_all(&held.statements),\n"
        "    );",
        "    let mut stated_sorted = normalized_all(&requirement.statements);\n"
        "    stated_sorted.sort();\n"
        "    let mut held_sorted = normalized_all(&held.statements);\n"
        "    held_sorted.sort();\n"
        "    let first_differing = first_differing_statement(&stated_sorted, &held_sorted);",
    ),
    (
        "7. include notes in the content",
        COMMANDS,
        "        &normalized_all(&held.statements),\n    );\n",
        "        &normalized_all(&held.statements),\n    )\n"
        "    .or_else(|| (normalized_all(&requirement.notes) != normalized_all(&held.notes))\n"
        "        .then_some(std::num::NonZeroUsize::MIN));\n",
    ),
    (
        "8. stop leaving scenarios marked removed out of the change's side",
        COMMANDS,
        "        if Declared::of(&scenario.change) == Declared::Withdraws {\n"
        "            continue;\n"
        "        }\n",
        "",
    ),
    (
        "9. leave untaken_content out of reached_in_full",
        COMMANDS,
        "            && self.untaken_content.is_empty()\n",
        "",
    ),
    (
        "10. leave untaken_removals out of reached_in_full",
        COMMANDS,
        "            && self.untaken_removals.is_empty()\n",
        "",
    ),
    (
        "11. put statement text into a payload entry",
        COMMANDS,
        "    UntakenContent::new(id, first_differing, not_held, not_kept)\n",
        "    not_held.extend(first_differing.and_then(|position| "
        "requirement.statements.get(position.get() - 1).cloned()));\n"
        "    UntakenContent::new(id, first_differing, not_held, not_kept)\n",
    ),
    (
        "12. report a requirement both renamed and modified in the rename set only",
        COMMANDS,
        "        self.untaken_renames.extend(rename);\n"
        "        self.untaken_content.extend(content);\n",
        "        let renamed = rename.is_some();\n"
        "        self.untaken_renames.extend(rename);\n"
        "        if !renamed {\n"
        "            self.untaken_content.extend(content);\n"
        "        }\n",
    ),
    (
        "13. remove untakenContent from sync.md",
        SYNC,
        "`untakenContent`",
        "the content list",
    ),
]

FAILED = re.compile(r"^test (\S+) \.\.\. FAILED$", re.M)


def run():
    result = subprocess.run(
        ["nix", "develop", "-c", "cargo", "test", "--workspace", "--all-targets", "--no-fail-fast"],
        capture_output=True,
        text=True,
    )
    output = result.stdout + result.stderr
    if "error[E" in output or "could not compile" in output:
        return None, output
    return sorted(set(FAILED.findall(output))), output


def apply(path, old, new):
    original = path.read_text()
    if path == SYNC:
        assert old in original, old
        return original, original.replace(old, new)
    assert original.count(old) == 1, f"the pattern is not unique in {path}: {old[:60]!r}"
    return original, original.replace(old, new, 1)


def main():
    for title, path, old, new in MUTATIONS:
        original, mutated = apply(path, old, new)
        path.write_text(mutated)
        try:
            failed, output = run()
        finally:
            path.write_text(original)
        print(f"== {title}", flush=True)
        if failed is None:
            print("   the build failed")
            print(output[-2000:])
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
