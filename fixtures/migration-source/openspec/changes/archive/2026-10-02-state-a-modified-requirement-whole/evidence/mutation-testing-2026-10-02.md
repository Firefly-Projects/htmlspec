# Mutation testing

**Date:** 2026-10-02
**Task:** 4.2
**Script:** `mutate.py`, reproduced below. It applies each mutation alone, runs
`cargo test --workspace --all-targets --no-fail-fast` inside `nix develop`, records every test that
failed, and restores the file before the next mutation. A last run with nothing mutated passes.

## Every mutation is caught

| Mutation | Caught by |
| --- | --- |
| 1. Restore `sync.md`'s "**Apply partial updates.**" paragraph, as HEAD states it | `no_workflow_tells_an_agent_to_restate_part_of_a_requirement` |
| 2. Restore `archive.md`'s step 5 paragraph and its quotation of OpenSpec's sync guidance | `no_workflow_tells_an_agent_to_restate_part_of_a_requirement` |
| 3. Remove the requirement's name from `sync.md`, both times it appears | `no_workflow_tells_an_agent_to_restate_part_of_a_requirement` |
| 4. A one-line rule in `crates/htmlspec-validate/src/rules/kinds.rs` reporting a `modified` requirement holding a `removed` scenario as an error | `every_fixture_is_accepted_exactly_when_it_declares_it_is`, `every_valid_fixture_is_accepted_by_the_whole_rule_set`, and `rule_code_does_not_restate_schema_expressible_values` |

```
== 1. restore sync.md's "Apply partial updates" paragraph
   instructions::tests::no_workflow_tells_an_agent_to_restate_part_of_a_requirement
== 2. restore archive.md's paragraph and its OpenSpec quotation
   instructions::tests::no_workflow_tells_an_agent_to_restate_part_of_a_requirement
== 3. remove the requirement's name from sync.md
   instructions::tests::no_workflow_tells_an_agent_to_restate_part_of_a_requirement
== 4. report a removed scenario in a modified requirement as an error
   every_fixture_is_accepted_exactly_when_it_declares_it_is
   every_valid_fixture_is_accepted_by_the_whole_rule_set
   rule_code_does_not_restate_schema_expressible_values
== restored: every test passes
```

**Mutations 1 to 3 are caught by the new test alone.** No other test reads the wording of a merge
instruction. `scripts/check-installed` would also refuse each of them, because the installed copies
would no longer match what `init` writes, but that is a script stage and not a test, and it compares
copies rather than content.

**Mutation 4 is caught by the new fixture alone, among the conformance fixtures.** Run again against
`cargo test -p htmlspec-conformance --test conformance`, both failing tests name only
`valid-modified-requirement-stated-whole`. No other fixture holds a `modified` requirement with a
`removed` scenario, so without this fixture the rule would pass every conformance test. The third
catch, `rule_code_does_not_restate_schema_expressible_values`, is incidental: the mutation spells
the change states as the string literals `"modified"` and `"removed"`, which that test refuses in rule
code. A rule written through the schema's own facts would not trip it.

## Two more, for the fix the review took

Task 6.1 took the fold-in step out of `sync.md` and added
`no_workflow_tells_an_agent_to_fold_an_added_requirement_into_another`. Each mutation was applied by
hand and run against `cargo test -p htmlspec-assets --lib`:

| Mutation | Caught by |
| --- | --- |
| 5. Restore `sync.md` as committed in `81b0a80`, with step 3's "an addition to one already accepted" | `no_workflow_tells_an_agent_to_fold_an_added_requirement_into_another` |
| 6. Replace step 3's `document-schema's "Semantic units declare a change state" gives` with `the contract gives` | `no_workflow_tells_an_agent_to_fold_an_added_requirement_into_another` |

Both tests are wording tests, and they share one limit: a body that reworded the fold-in instruction
would pass. Mutation 6 is why the test also requires `sync` to name the requirement its report rests on.

## Three more, for the stop the code review asked for

The code review of PR #41 found that `archive.md` went from its merge step to archiving with nothing
between them, so a held requirement was archived unmerged. Task 6.1 added the stop to both workflows,
and `no_workflow_archives_a_change_while_a_modified_requirement_is_held` to pin it. Each mutation was
applied by hand and run against `cargo test -p htmlspec-assets --lib`:

| Mutation | Caught by |
| --- | --- |
| 7. Restore `archive.md` and `sync.md` as committed in `4327a49`, without either stop | `no_workflow_archives_a_change_while_a_modified_requirement_is_held` |
| 8. Move `archive.md`'s stop from the end of step 5 to the start of step 6, after the heading | `no_workflow_archives_a_change_while_a_modified_requirement_is_held` |
| 9. Remove the stop from `sync.md`'s closing paragraph | `no_workflow_archives_a_change_while_a_modified_requirement_is_held` |

## What no mutation here can reach

The fixture pins that the shape is accepted, not that it is silent. A rule reporting the same shape
as a `review/*` warning would be accepted, and `every_fixture_reports_exactly_the_rules_it_declares`
skips a fixture whose rule list is empty. That is the harness's standing limit for every valid
fixture, as its comment on `a_summary_quoting_the_placeholder_reports_nothing` records, and not this
change's to lift.

`archive.md`'s example under "A requirement the change modified" is not read by any test. Task 3.4
checked it by hand, by pasting it into a new specification (`the-rule-measured-2026-10-02.md`). No
earlier example in `archive.md` is tested either, the renamed-requirement one included.

## Appendix: `mutate.py`

```python
"""Applies each mutation of state-a-modified-requirement-whole alone, runs the workspace's tests,
and restores the file.

Run from the repository root, outside the Nix shell: python3 mutate.py
Prints, for each mutation, every test that failed, or that the build failed.
"""

import re
import subprocess
from pathlib import Path

SYNC = Path("crates/htmlspec-assets/instructions/sync.md")
ARCHIVE = Path("crates/htmlspec-assets/instructions/archive.md")
KINDS = Path("crates/htmlspec-validate/src/rules/kinds.rs")

NAME = "A change's specification states a modified requirement whole"
WRAPPED_NAME = r"\s+".join(re.escape(word) for word in NAME.split())

# The paragraphs as HEAD (6fd228e) states them, before this change.
PARTIAL_SYNC = """**Apply partial updates.** Adding one scenario to an accepted requirement means adding that
scenario. It does not mean restating the requirement, and it does not mean replacing the
accepted text with the change's. Content you are not merging must survive untouched — the
accepted specifications hold requirements from every change before this one, and they are not
yours to rewrite while you are here.
"""
PARTIAL_ARCHIVE = """**Apply partial updates. Preserve what you do not mention.** OpenSpec's sync guidance
states the governing principle, and it holds here unchanged:

> This is an **agent-driven** operation… Unlike programmatic merging, you can apply
> partial updates. The delta represents *intent*, not a wholesale replacement.
"""


def between(text, start, end):
    """The text from `start` up to, not including, `end`."""
    i = text.index(start)
    return text[i:text.index(end, i)]


def m1(text):
    old = between(text, "**Leave every requirement the change does not state untouched.**",
                  "**Never copy a change's specification")
    return text.replace(old, PARTIAL_SYNC + "\n", 1)


def m2(text):
    old = between(text, "**Preserve what the change does not state, and apply what it modifies as",
                  "Concretely:")
    return text.replace(old, PARTIAL_ARCHIVE + "\n", 1)


def m3(text):
    mutated, count = re.subn(WRAPPED_NAME, "the contract's rule", text)
    assert count == 2, count
    return mutated


def m4(text):
    old = "    a_tasks_document_enumerates_work(kind, projection, diagnostics);\n"
    new = old + (
        "    for r in projection.model.requirements.iter().filter(|r| r.change == \"modified\" "
        "&& r.scenarios.iter().any(|s| s.change == \"removed\")) { diagnostics.violation("
        "TASK_IN_SPECIFICATION, Location::maybe_unit(r.id.as_deref()), \"mutation\"); }\n"
    )
    assert text.count(old) == 1
    return text.replace(old, new, 1)


MUTATIONS = [
    ("1. restore sync.md's \"Apply partial updates\" paragraph", SYNC, m1),
    ("2. restore archive.md's paragraph and its OpenSpec quotation", ARCHIVE, m2),
    ("3. remove the requirement's name from sync.md", SYNC, m3),
    ("4. report a removed scenario in a modified requirement as an error", KINDS, m4),
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


def main():
    for title, path, mutate in MUTATIONS:
        original = path.read_text()
        mutated = mutate(original)
        assert mutated != original, title
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
    clean, _ = run()
    print(f"== restored: {'every test passes' if clean == [] else clean}")


if __name__ == "__main__":
    main()
```
