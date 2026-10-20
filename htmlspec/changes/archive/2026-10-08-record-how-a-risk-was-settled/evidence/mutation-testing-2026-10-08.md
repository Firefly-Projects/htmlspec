# Mutation testing

**Date:** 2026-10-08
**Task:** 7.3

Each mutation was applied alone by a script that edits the source, runs `nix develop -c cargo test -p
htmlspec-schema -p htmlspec-parse -p htmlspec-validate -p htmlspec-format -p htmlspec-conformance -p
htmlspec-assets --no-fail-fast`, collects every test that reports `FAILED`, and restores the files from
copies taken before the first mutation. The SHA-256 of the four files the mutations touch
(`model.rs` and `project.rs` in `htmlspec-parse`, `rules/relations.rs` in `htmlspec-validate`, and
`attributes.rs` in `htmlspec-format`) matched the copies after the run, and `git status` showed no
source change. Every mutation compiled.

## Results

| # | Mutation | Caught by |
| --- | --- | --- |
| 1 | A default of `accepted` | `the_default_response_matches_the_schema`, `section_4_7_1_a_risk_that_declares_no_response_projects_mitigated`, `a_risk_that_declares_no_response_and_states_no_mitigation_is_reported`, `the_report_names_both_remedies_and_the_default`, `every_fixture_projects_to_its_expected_model`, `every_fixture_reports_exactly_the_rules_it_declares`, `every_fixture_is_accepted_exactly_when_it_declares_it_is` |
| 2 | A rule that skips a risk with no attribute | `a_risk_that_declares_no_response_and_states_no_mitigation_is_reported`, `the_report_names_both_remedies_and_the_default`, `every_fixture_reports_exactly_the_rules_it_declares`, `every_fixture_is_accepted_exactly_when_it_declares_it_is` |
| 3 | A rule that also fires for `accepted` | `an_accepted_risk_needs_no_mitigation`, `every_valid_fixture_is_accepted_by_the_whole_rule_set`, `every_fixture_is_accepted_exactly_when_it_declares_it_is` |
| 4 | A projection that ignores `data-response` | `section_4_7_1_an_accepted_risk_with_no_mitigation_projects_none`, `section_4_7_1_an_accepted_risk_keeps_a_mitigation_it_states`, `section_4_7_1_a_response_is_trimmed_and_an_unrecognized_one_survives`, `an_accepted_risk_needs_no_mitigation`, `a_response_outside_the_enumeration_is_left_to_the_schema`, `every_fixture_projects_to_its_expected_model`, `every_fixture_reports_exactly_the_rules_it_declares`, `every_fixture_is_accepted_exactly_when_it_declares_it_is`, `every_valid_fixture_is_accepted_by_the_whole_rule_set` |
| 5 | A projection that drops an accepted risk's mitigation | `section_4_7_1_an_accepted_risk_keeps_a_mitigation_it_states`, `every_fixture_projects_to_its_expected_model` |
| 6 | A `status_named` stem that matches nothing | `a_risk_declared_mitigated_with_no_mitigation_is_reported`, `a_risk_that_declares_no_response_and_states_no_mitigation_is_reported`, `the_report_names_both_remedies_and_the_default`, `every_fixture_reports_exactly_the_rules_it_declares`, `every_fixture_is_accepted_exactly_when_it_declares_it_is` |
| 7 | `data-response` left out of `LEADING` | `a_risks_response_leads_with_the_other_state_attributes` |

**7 of 7 are caught**, each by at least one test this change added. None needed a new test.

## How some mutations were realized

- **2.** The model cannot tell a declared `mitigated` from the default, because the projection always
  fills `response`, which is design decision 4. So the rule was made to look up the risk's element in
  `projection.document` by its identifier and skip it when the element carries no `data-response`.
  That is the defect a rule keyed on an optional field would have.
- **3.** The rule's first test became `risk.response != mitigated && risk.response != accepted`.
- **4.** `response` became `DEFAULT_RISK_RESPONSE` whatever the markup says.
- **5.** `mitigation` became `None` when the element's `data-response` is `accepted`.
- **6.** The rule's stem became `mitigatx`, so `status_named` finds nothing and the rule does not run.
- **7.** The `data-response` entry was removed from `LEADING`.

## What the run found

**`every_conditional_rule_finds_its_state` does not catch mutation 6.** The test checks the stems it
names itself, not the stems the rule passes, so a typo in the rule leaves it passing. The rule's
behaviour tests catch the mutation, because the rule then reports nothing, and so does the
conformance fixture `invalid-schema-missing-mitigation`. The two older conditional rules are built
the same way, with the stem written once in the rule and once in the test, and have the same gap.
Sharing each stem as a constant between the rule and the test would close it for all three. This
change keeps the existing pattern, and the gap is recorded for review rather than closed here.

**Mutation 7 has one test, and only one can catch it.** Outside `LEADING`, `data-response` sorts
after every leading attribute and alphabetically among the ignored ones. Inside it, it sorts after
`data-outcome`. The two orders differ only on an element that also carries a leading attribute
listed after `data-outcome`, such as `data-refs`, or an ignored attribute that sorts before it, such
as `aria-label`. A risk carrying only `class`, `id`, `data-change`, and `data-response` is written
the same either way, so the template's own risk, and `the_rendered_template_is_already_canonical`,
cannot tell. The unit test carries `data-refs`, `aria-label`, and `lang` for that reason.

**Mutation 5 is caught only by the projection.** A risk whose mitigation is dropped while it is
accepted still validates, because an accepted risk needs none. So no conformance fixture can catch
it, and the projection test and the `risk-response` mapping fixture are what pin it.
