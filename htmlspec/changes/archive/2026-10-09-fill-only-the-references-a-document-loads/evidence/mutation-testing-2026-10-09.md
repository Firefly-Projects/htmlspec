# Mutation testing

**Date:** 2026-10-09
**Task:** 5.2

Each mutation was applied to the working tree by a script in the session scratchpad. It ran one
`cargo test -q -p htmlspec-assets -p htmlspec-format -p htmlspec-validate --no-fail-fast`, and
then restored the file from a copy taken before the run. After each run, the restored files were
compared byte for byte with their copies, and they matched. Every mutation compiled.

Failing property tests write their seeds to a `proptest-regressions/` directory beside the crate.
The repository tracks no such directory, and these seeds came from mutated code, so each one was
removed after its run.

**All ten mutations the task names are caught, and so is a sharper version of one of them (6b).**
Two needed a new test first, as the task requires: mutation 10 survived its first run, and
mutation 9 was caught only by a property test.

| # | Mutation | Caught by |
|---|---|---|
| 1 | The old substring search restored, inside `asset_references` | 14 tests: all three from 2.1, nine of the ten from 2.2, and both cross-checks |
| 2 | Comments not skipped: scanning resumes just after `<!--` | `a_reference_inside_a_comment_is_not_a_reference`, `a_comment_holds_no_tags_however_it_ends`, the generated-documents cross-check |
| 3 | Raw-text elements not skipped | `a_reference_inside_a_raw_text_element_is_not_a_reference`, `a_raw_text_element_holds_no_tags_until_its_own_end_tag`, `a_raw_text_element_left_open_holds_the_rest`, the generated-documents cross-check |
| 4 | `<template>` contents not skipped | `a_reference_inside_a_template_is_not_a_reference`, `template_contents_are_not_in_the_tree`, the generated-documents cross-check |
| 5 | A repeated attribute's last value kept | `a_repeated_attribute_fills_only_its_first_value`, `the_first_of_a_repeated_attribute_is_kept` |
| 6 | A quoted value ended at its first `>` | `values_are_read_in_every_quoting` |
| 6b | A `>` inside a quoted value ends the tag | `a_closing_bracket_inside_a_quoted_value_does_not_end_the_tag`, `a_quoted_bracket_does_not_end_an_end_tag`, `values_are_read_in_every_quoting`, the generated-documents cross-check |
| 7 | Names not lowercased | `an_uppercase_attribute_is_filled`, `names_are_lowercased`, `the_first_of_a_repeated_attribute_is_kept`, the generated-documents cross-check |
| 8 | `("a", "href")` added to the shared list | `a_hyperlink_to_an_asset_is_left_as_written`, `a_hyperlink_is_not_a_referencing_attribute`, and the validator's existing `a_hyperlink_is_not_an_asset_reference` |
| 9 | Values not trimmed | first run: the generated-documents cross-check only; second run: also `a_padded_value_is_filled_inside_its_padding` |
| 10 | End of source before an attribute's name emits the tag | first run: **nothing**; second run: `a_tag_cut_off_by_the_end_of_the_source_is_not_emitted` |

## Notes

**Mutation 1** reproduces HEAD's search as closely as the new API allows. It finds `href="` and
`src="` anywhere, and reports each as a `link` or a `script` reference. The one 2.2 test it passes is
`a_closing_bracket_inside_a_quoted_value_does_not_end_the_tag`, which HEAD also passes.

**Mutation 6 is weaker than its name.** Ending a value at a `>` does not end the tag. The scanner
reads what follows as further attributes, so the `href` after `title="a > b"` is still found, and the
`references.rs` test that guards this case passes. So **6b** was added: a `>` inside a quoted value
ends the tag, which is the mistake that test describes. Four tests catch 6b.

**Mutation 9** was first caught only by the generated-documents property test. Its cases are drawn
at random, and a padded value is one of six values, so with 512 cases a run is almost certain to
draw one. "Almost certain" depends on the draw, and a direct test does not. So
`a_padded_value_is_filled_inside_its_padding` was added in `references.rs`. It fills
`src="  assets/htmlspec.js "` and keeps the spaces.

**Mutation 10** treats the end of the source as a `>` when it comes between attributes. It survived
the first run: the cut-off cases in the tests ended inside a name, an unquoted value, or a quoted
value, and none ended after a whole attribute. Two such cases were added to
`a_tag_cut_off_by_the_end_of_the_source_is_not_emitted`: `<p><img src="a" ` and `<p><img src='a'/`.
The second run caught it.

The script's first anchor for mutation 10 matched two places in `tags.rs`. The script's check that an
anchor matches exactly once stopped it before applying anything, and it restored every file. The
re-run used an anchor unique to the place before an attribute's name.

## The review's fixes, mutated (task 7.1)

The code review of the pull request found three readings where the scanner disagreed with the tree,
and the fixes added foreign content and a script's escape states to `tags.rs`. They were mutated the
same way, twelve mutations to `tags.rs`, each run against the format and validate crates' tests
(`cargo test -q -p htmlspec-format -p htmlspec-validate --no-fail-fast`), and the file was restored
byte for byte after each.

**All twelve are caught.** Two needed a new test first.

| # | Mutation | Caught by |
|---|---|---|
| F1 | Foreign content not tracked | 7 tests: the five foreign-content unit tests and both generated-document cross-checks |
| F2 | Integration points ignored | `an_integration_point_holds_html`, `a_foreign_element_takes_its_parent_s_namespace`, both generated-document cross-checks |
| F3 | Namespace switched by an element's name inside foreign content | `a_foreign_element_takes_its_parent_s_namespace`, both generated-document cross-checks |
| F4 | `annotation-xml` made an integration point, as the HTML specification has it | `an_integration_point_holds_html`, both generated-document cross-checks |
| F5 | `template` counted inside foreign content | `a_foreign_template_is_an_ordinary_element`, both generated-document cross-checks |
| F6 | `/>` ignored | `html_resumes_after_foreign_content` |
| F7 | A script's escape states ignored | `a_script_s_escaped_inner_script_postpones_its_end`, both generated-document cross-checks |
| F8 | The double escape never entered | the same three |
| F9 | `-->` never returns to script data | `a_script_s_escape_ends_where_the_tokenizer_ends_it` |
| F10 | The opener's own dashes not counted toward `-->` | first run: **nothing**; second run: `a_script_s_escape_ends_where_the_tokenizer_ends_it`, both generated-document cross-checks |
| F11 | The inner script's end does not return to the escape | `a_script_s_escaped_inner_script_postpones_its_end`, the random generated-document cross-check |
| F12 | `mglyph` read as HTML inside a MathML text integration point | first run: **nothing**; second run: `an_integration_point_holds_html`, both generated-document cross-checks |

"Both generated-document cross-checks" means the 512-case property test and
`every_fragment_alone_is_read_as_the_tree_reads_it`. That second test was added with the fixes. It
runs every fragment alone, with every value, so that no fragment is left to the property test's
random draw.

**F10** survived its first run, because no test opened a script's escape with `<!-->`. The case
`<script><!--><script></script><img src=a>` was added to the unit test, and the same case, with a
value, was added as a fragment. **F12** survived for the same reason, with `mglyph`.
`<math><mi><mglyph><style>…` was added to both.

**F12's first run never compiled.** Its patch added a second `if` guard to a match arm, which is a
syntax error. The script judged compilation by searching for `error[E`, which a syntax error
lacks, so it reported "compiled" and no failing test. Re-run alone, it showed the error. The patch
was corrected and the check changed to "could not compile". The first round's ten mutations each
listed failing tests, so each of them compiled.
