# Tests first, and mutation testing

**Date:** 2026-10-01
**Tasks:** 2.2, which records which tests fail against the empty rule, and 5.2

## 2.2 Against the empty rule

The 22 unit tests in `rules/change_markup.rs` ran against a `validate` that reported nothing.
**Every test asserting a report failed, and every test asserting silence passed.**

| Asserts | Test | Against the empty rule |
| --- | --- | --- |
| a report | `a_deleted_phrase_and_an_inserted_word_are_two_errors` | failed |
| a report | `markup_inside_a_unit_is_told_to_state_the_text_as_it_now_reads` | failed |
| a report | `a_deleted_paragraph_in_a_requirement_is_reported` | failed |
| a report | `an_inserted_word_alone_is_reported` | failed |
| a report | `change_markup_in_a_scenario_names_the_scenario` | failed |
| a report | `a_requirement_wrapped_in_del_is_reported_at_the_requirement` | failed |
| a report | `a_requirement_wrapped_in_ins_is_told_to_mark_it_added` | failed |
| a report | `a_long_text_is_quoted_by_its_opening_words` | failed |
| a report | `a_wrapping_element_is_reported_once_at_the_first_unit_it_encloses` | failed |
| a report | `every_unit_in_every_kind_is_reported` | failed |
| a report | `a_requirement_in_a_proposal_or_a_design_is_reported` | failed |
| a report | `no_declared_version_exempts_a_document` | failed |
| a report | `each_element_is_reported_with_its_own_text` | failed |
| a report | `a_del_nested_in_inline_markup_is_reported` | failed |
| a report | `a_del_in_a_units_heading_is_reported` | failed |
| a report | `a_unit_with_no_identifier_is_still_reported` | failed |
| a report | `markup_in_an_element_with_two_reserved_classes_is_reported` | failed |
| silence | `change_markup_outside_every_unit_reports_nothing` | passed |
| silence | `change_markup_in_the_summary_reports_nothing` | passed |
| silence | `change_markup_beside_a_unit_reports_nothing` | passed |
| silence | `a_struck_through_phrase_reports_nothing` | passed |
| silence | `markup_in_an_element_classed_requirements_reports_nothing` | passed |

Task 2.4's conformance run against the same empty rule failed two harness tests,
`every_fixture_reports_exactly_the_rules_it_declares` and
`every_fixture_is_accepted_exactly_when_it_declares_it_is`. Both failed on
`invalid-schema-change-markup` and on nothing else.

## 5.2 Ten mutations, each applied alone

Each mutation was applied by itself, then `cargo test --no-fail-fast -p htmlspec-validate -p
htmlspec-conformance` ran, and then the source was restored. After the last restore, both suites
passed. The script is in the appendix. **No mutation survived.**

| Mutation | Caught by |
| --- | --- |
| 1. Match `del` only | 5 unit tests, among them `an_inserted_word_alone_is_reported` and `a_requirement_wrapped_in_ins_is_told_to_mark_it_added` |
| 2. Match `ins` only | 15 unit tests |
| 3. Look at ancestors only | `a_requirement_wrapped_in_del_is_reported_at_the_requirement`, `a_requirement_wrapped_in_ins_is_told_to_mark_it_added`, `a_long_text_is_quoted_by_its_opening_words`, `a_wrapping_element_is_reported_once_at_the_first_unit_it_encloses` |
| 4. Look at descendants only | 13 unit tests, and both harness tests on `invalid-schema-change-markup` |
| 5. Drop the unit test, so every change element is reported | 15 unit tests, among them all four tests of silence outside a unit, and `every_valid_fixture_is_accepted_by_the_whole_rule_set` and `every_fixture_is_accepted_exactly_when_it_declares_it_is` on `valid-change-markup-outside-a-unit` |
| 6. Match a reserved class by substring | `markup_in_an_element_classed_requirements_reports_nothing` |
| 7. Look at the parent only instead of every ancestor | 12 unit tests, among them `a_del_nested_in_inline_markup_is_reported`, and both harness tests on `invalid-schema-change-markup` |
| 8. Skip documents declaring an earlier version | 17 unit tests, among them `no_declared_version_exempts_a_document`, and both harness tests, because the invalid fixture declares the generator's frozen `0.1.0` |
| 9. Report in `review/*` | 6 unit tests that assert an error or a rejected document, and four harness tests, among them `no_fixture_declares_a_rule_the_validator_does_not_have` |
| 10. Remove the rule from `ALL` | `every_namespace_has_a_live_rule`, and both harness tests on `invalid-schema-change-markup` |

Two catches are narrow, and both were expected:

- **Mutation 6 has one catcher.** Only an element classed `requirements` tells a whole-token match
  from a substring match. The template classes no element that way, so no fixture would catch it.
- **Mutation 3 has no conformance catcher.** The invalid fixture carries its markup inside a
  statement, so it reaches the unit through an ancestor. The four unit tests that wrap a unit catch
  it. The design's risk "A mutation that confines the rule to ancestors passes every inline test"
  names this case, and the wrapped-unit tests are its mitigation.

## Appendix: the mutation script

`mutate.py`, run from the repository root, outside the Nix shell:

```python
"""Applies each mutation of `schema/change-markup` alone, runs the suites, and restores the file.

Run from the repository root, outside the Nix shell: python3 mutate.py
Prints, for each mutation, every test that failed, or that the build failed.
"""

import re
import subprocess
from pathlib import Path

RULE = Path("crates/htmlspec-validate/src/rules/change_markup.rs")
REGISTRY = Path("crates/htmlspec-validate/src/rules/mod.rs")

MUTATIONS = [
    (
        "1. match del only",
        RULE,
        'if name.eq_ignore_ascii_case("ins") {',
        "if false {",
    ),
    (
        "2. match ins only",
        RULE,
        '} else if name.eq_ignore_ascii_case("del") {',
        "} else if false {",
    ),
    (
        "3. look at ancestors only",
        RULE,
        "node.descendants()\n",
        "node.descendants().take(0)\n",
    ),
    (
        "4. look at descendants only",
        RULE,
        "node.ancestors().find_map(",
        "node.ancestors().take(0).find_map(",
    ),
    (
        "5. drop the unit test, so every change element is reported",
        RULE,
        'let kind = attributes\n            .get("class")?\n            .split_whitespace()\n'
        "            .find_map(UnitKind::from_class)?;",
        "let kind = UnitKind::Requirement;",
    ),
    (
        "6. match a reserved class by substring",
        RULE,
        ".find_map(UnitKind::from_class)?;",
        ".find_map(|token| {\n                UnitKind::ALL\n                    .iter()\n"
        "                    .copied()\n"
        "                    .find(|kind| token.contains(kind.as_class()))\n            })?;",
    ),
    (
        "7. look at the parent only instead of every ancestor",
        RULE,
        "node.ancestors().find_map(|ancestor| Unit::of(&ancestor))",
        "node.parent().and_then(|ancestor| Unit::of(&ancestor))",
    ),
    (
        "8. skip documents declaring an earlier version",
        RULE,
        "    for element in document.inclusive_descendants().elements() {\n"
        "        let Some(change)",
        "    if subject.projection.model.schema_version.as_deref()\n"
        "        != Some(htmlspec_schema::CONTRACT_VERSION)\n    {\n        return;\n    }\n"
        "    for element in document.inclusive_descendants().elements() {\n"
        "        let Some(change)",
    ),
    (
        "9. report in review/*",
        RULE,
        'RuleId::new(Namespace::Schema, "change-markup")',
        'RuleId::new(Namespace::Review, "change-markup")',
    ),
    (
        "10. remove the rule from ALL",
        REGISTRY,
        "    change_markup::validate,\n",
        "",
    ),
]

FAILED = re.compile(r"^test (\S+) \.\.\. FAILED$", re.M)


def run():
    result = subprocess.run(
        [
            "nix",
            "develop",
            "-c",
            "cargo",
            "test",
            "--no-fail-fast",
            "-p",
            "htmlspec-validate",
            "-p",
            "htmlspec-conformance",
        ],
        capture_output=True,
        text=True,
    )
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
        print(f"== {title}")
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

## After the review of the pull request

The review found three defects in the rule and five wrong implementations the tests let through.
Eight tests were added. Run against the rule as first committed (`8032003`), five of them failed,
one for each defect:

| Test | Against `8032003` | The defect |
| --- | --- | --- |
| `a_change_element_that_is_itself_a_unit_is_reported` | failed | `<del class="decision">` was accepted, because the rule asked only the element's ancestors and descendants |
| `a_del_classed_as_a_requirement_is_reported_at_the_requirement` | failed | the same, reported at the scenario inside it instead |
| `markup_in_a_unit_around_another_names_the_unit_it_wraps` | failed | the fix named no change state for the unit the element wraps |
| `each_change_element_gives_its_own_reason` | failed | the `ins` reason said the replaced words were hidden, beside a `del` that shows them |
| `a_foreign_element_named_like_change_markup_reports_nothing` | failed | an SVG or MathML element named `del` or `ins` was reported |
| `a_unit_wrapped_through_an_intervening_element_is_reported` | passed | none: it pins mutation R2 |
| `a_class_token_differing_in_case_creates_no_unit` | passed | none: it pins mutation R3 |
| `the_quotation_normalizes_every_whitespace_run` | passed | none: it pins mutation R5 |

The review's five surviving mutations, and three of the code the review added, each applied alone
with `mutate_review.py` below. **All eight are caught:**

| Mutation | Caught by |
| --- | --- |
| R1. Look for an enclosed unit before an enclosing one | `markup_in_a_unit_around_another_names_the_unit_it_wraps` |
| R2. Look at direct children only for a wrapped unit | `a_unit_wrapped_through_an_intervening_element_is_reported` |
| R3. Match class tokens case-insensitively | `a_class_token_differing_in_case_creates_no_unit` |
| R4. Swap the `del` and `ins` reasons | `each_change_element_gives_its_own_reason` |
| R5. Split the quotation on spaces only | `the_quotation_normalizes_every_whitespace_run` |
| N1. Never treat the change element itself as a unit | `a_change_element_that_is_itself_a_unit_is_reported`, `a_del_classed_as_a_requirement_is_reported_at_the_requirement` |
| N2. Drop the namespace test | `a_foreign_element_named_like_change_markup_reports_nothing` |
| N3. Ignore a unit enclosed inside a unit | `markup_in_a_unit_around_another_names_the_unit_it_wraps` |

The ten mutations of section 5.2 were run again against the revised rule with the unchanged
`mutate.py`, and all ten are still caught. Mutation 6, the substring class match, still has one
catcher, because a case-insensitive match and a substring match fail on different inputs.

`mutate_review.py`, beside `mutate.py`, run the same way:

```python
"""The review's mutations, and mutations of the code the review added, each applied alone.

Run from the repository root, outside the Nix shell: python3 mutate_review.py
Each mutation is a list of (old, new) replacements in the rule's source, applied together.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from mutate import RULE, run  # noqa: E402

MUTATIONS = [
    (
        "R1. look for an enclosed unit before an enclosing one",
        [(
            "if let Some(unit) = node.ancestors().find_map(|ancestor| Unit::of(&ancestor)) {",
            "if let Some(unit) = enclosed()\n"
            "            .or_else(|| node.ancestors().find_map(|ancestor| Unit::of(&ancestor)))\n"
            "        {",
        )],
    ),
    (
        "R2. look at direct children only for a wrapped unit",
        [("node.descendants()\n", "node.children()\n")],
    ),
    (
        "R3. match class tokens case-insensitively",
        [(
            ".find_map(UnitKind::from_class)?;",
            ".find_map(|token| UnitKind::from_class(&token.to_lowercase()))?;",
        )],
    ),
    (
        "R4. swap the del and ins reasons",
        [
            ("Change::Deleted => format!(\n                    \"The projection gives change markup no meaning, so the withdrawn",
             "Change::Inserted => format!(\n                    \"The projection gives change markup no meaning, so the withdrawn"),
            ("Change::Inserted => \"The projection keeps the inserted",
             "Change::Deleted => \"The projection keeps the inserted"),
        ],
    ),
    (
        "R5. split the quotation on spaces only",
        [(
            ".text_contents()\n        .split_whitespace()",
            ".text_contents()\n        .split(' ')\n        .filter(|word| !word.is_empty())",
        )],
    ),
    (
        "N1. never treat the change element itself as a unit",
        [("if let Some(unit) = Unit::of(node) {", "if let Some(unit) = Unit::of(node).filter(|_| false) {")],
    ),
    (
        "N2. drop the namespace test",
        [("if &*element.name.ns != HTML_NAMESPACE {", "if &*element.name.ns != HTML_NAMESPACE && false {")],
    ),
    (
        "N3. ignore a unit enclosed inside a unit",
        [("enclosed: enclosed(),", "enclosed: None,")],
    ),
]


def main():
    for title, replacements in MUTATIONS:
        original = RULE.read_text()
        mutated = original
        for old, new in replacements:
            assert mutated.count(old) == 1, f"{title}: {old!r} is not unique"
            mutated = mutated.replace(old, new, 1)
        RULE.write_text(mutated)
        try:
            failed, output = run()
        finally:
            RULE.write_text(original)
        print(f"== {title}")
        if failed is None:
            print("   the build failed")
            print(output[-1500:])
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
