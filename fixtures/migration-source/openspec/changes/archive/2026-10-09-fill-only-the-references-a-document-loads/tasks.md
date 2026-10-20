## 1. Re-measure before changing anything

- [x] 1.1 In a scratch project made by `htmlspec init` and `htmlspec new change`, quote `href="…"` and `src="…"` in `<code>`, an escaped `<link>` in `<pre><code>`, and an `<a href>` to the stylesheet in a proposal. Run `format --check`, `format`, and `validate`, and record every fill. Then point the `<a>` and the `<link>` at an absent address in turn, and record what the asset rules report for each. Record the result in `evidence/the-measurement-2026-10-09.md`

## 2. Tests first

- [x] 2.1 In `crates/htmlspec-format/tests/reference_rewriting.rs`, add one test per measured case: a `<code>` quotation, an escaped `<pre><code>` example, and an `<a href>`, each in a project whose store does not hold the bytes. Each asserts that the document is byte-identical after `rewrite` and that the document's own `<link>` is still filled. Check that all three fail on HEAD
  - Three tests, `prose_quoting_an_asset_attribute_is_left_as_written`, `an_escaped_example_of_a_link_is_left_as_written`, and `a_hyperlink_to_an_asset_is_left_as_written`, share one assertion. The body survives byte for byte, the document's own `<link>` is filled, and exactly one fill is reported. A precondition asserts that the fixture body is canonical as written, so a formatting change cannot make the words differ for another reason
  - On HEAD all three fail at "the author's words were rewritten", and the existing 8 tests in the file pass
- [x] 2.2 In `crates/htmlspec-format/src/references.rs`, add unit tests for what a substring search gets wrong:
  - a reference inside a comment, and inside the contents of `<script>`, `<style>`, `<textarea>`, and `<template>`, is kept;
  - `data-src="…"` is not filled;
  - `<object data="…">`, a single-quoted value, an unquoted value, and an uppercase `SRC` are filled;
  - a repeated `href` fills the first value and leaves the second;
  - a `>` inside another attribute's quoted value does not end the tag.

  The `>` case holds on HEAD, because a substring search is right about it by accident, and is there to guard the scanner. Check that a test which held on HEAD still holds: a fragment link is untouched, and the rewrite changes one line
  - Ten tests, built on three helpers: `in_a_project` stores the shipped runtime, `assert_untouched` requires the source unchanged and nothing reported, and `assert_filled` requires one fill of the named asset, with only the value's bytes changed
  - On HEAD, 9 fail and 1 holds: `a_closing_bracket_inside_a_quoted_value_does_not_end_the_tag`, as expected. The kept cases fail at `assert_untouched`, because HEAD filled the comment, the raw text, the template, `data-src`, and the `title` quotation. The filled cases fail at `assert_filled`: HEAD missed `<object data>`, the single-quoted and unquoted values, and `SRC`, and filled both of a repeated `href`
  - The existing 11 unit tests pass on HEAD, the fragment link and one-line tests among them
  - The `title` case is an attribute value that quotes a reference, `<abbr title='written as href="…"'>`, which a substring search also fills
- [x] 2.3 In the test module of `crates/htmlspec-validate/src/rules/assets.rs`, add the cross-check from design decision 3:
  - over every `.html` file under the repository root except `target/`, the references `htmlspec_format::asset_references` finds equal `referenced_assets`, in order;
  - a property test over bodies built from the fragments the decision lists asserts the same equality, and that the parser recovered no error.

  Add `proptest` to the validator's dev-dependencies. Check that the module fails to compile on HEAD, for want of `asset_references`
  - `the_formatter_finds_the_references_these_rules_judge_in_every_repository_document` walks `README.html`, `assets/`, `fixtures/`, `openspec/`, and `runbooks/`. It names directories rather than walking the checkout, so a contributor's untracked notes do not decide it. A document the parser recovered errors in is skipped, and floors of 100 documents and 100 references stop it passing on nothing
  - `the_formatter_finds_the_references_these_rules_judge_in_generated_documents` runs 512 cases. Each body holds up to 7 of 28 fragments, with values drawn from six (padded, empty, a fragment, a remote URL) and, where a value stands unquoted, from three. A duplicated attribute and a CDATA section in HTML content are left out of the fragments, because each makes the parser recover, and both are unit-tested in group 4
  - `proptest` is a dev-dependency of the validator, and `Cargo.lock` gains that one edge. On HEAD the test module fails to compile, with E0425 for `asset_references`

## 3. One list of the attributes that load an asset

- [x] 3.1 Move the thirteen pairs to `crates/htmlspec-assets` as `ReferencingAttribute` and `REFERENCING_ATTRIBUTES`, with the doc comment that says why `<a href>` is absent. Make `referenced_assets` read it. Check with `cargo test -p htmlspec-assets -p htmlspec-validate` that every existing asset-rule test passes unchanged, `a_hyperlink_is_not_an_asset_reference` among them
  - `crates/htmlspec-assets/src/reference.rs` holds `ReferencingAttribute` (`element`, `attribute`), the thirteen pairs as `REFERENCING_ATTRIBUTES`, and `ReferencingAttribute::find`. The doc comment keeps the reason `<a href>` is absent. It adds the reason `srcset` is absent although `safety/*` reads it: its value lists several URLs with sizes. Three unit tests check that `a`/`href` is not found, that every name is lowercase, and that every pair is found by its own names
  - `referenced_assets` reads the moved list, and the validator's private copy is gone. `cargo test -p htmlspec-assets` passes, 9 of its tests matching `reference`. All 22 tests that existed in `rules/assets.rs` pass unchanged, `a_hyperlink_is_not_an_asset_reference` among them

## 4. The scanner and the rewrite

- [x] 4.1 Add the scanner module to `crates/htmlspec-format` as design decision 2 describes, total over any input: no unchecked indexing and no panic. Give it unit tests for each rule the decision lists: comments (including `<!-->` and `<!--->`), CDATA, end tags with quoted `>`, each value quoting, a repeated attribute, each raw-text element and its case-insensitive end tag, `plaintext`, nested `<template>`, and a tag cut off by the end of the source. Check with `cargo test -p htmlspec-format`
  - `crates/htmlspec-format/src/tags.rs`, private, with `start_tags` returning each `StartTag` and its `Attribute`s by name and value range. The module comment names the three places it stops following the parser: a document with recovered errors, foreign content, and the escape states of script data. Indexing goes through `get`, so the module has no unchecked index and no `unwrap`
  - 18 unit tests, one per rule, and a property test, `any_source_is_scanned_without_panic`. Over strings built from the characters the tag states turn on, mixed with arbitrary text, it checks that the scanner returns and that every value range can be sliced from the source. `cargo test -p htmlspec-format --lib tags`: 19 pass
- [x] 4.2 Add `asset_references` to `references.rs`, and make `rewrite` copy between its ranges. Remove `next_reference` and `ATTRIBUTES`. Correct the module's doc comment to say which values are references and why. Check that the tests from 2.1, 2.2, and 2.3 pass, and that `htmlspec format --check` reports `fixtures/dogfood/` and `runbooks/` canonical, unchanged
  - `asset_references` returns `SourceReference { on, value, span }`, and `span` is private to the crate. Each value is trimmed and taken only if not blank, as `referenced_assets` takes it, so the two compare without any normalizing in the test. `rewrite` copies the source between spans. `next_reference`, `Found`, and `ATTRIBUTES` are gone. The module comment now says which values are references and points at the cross-check. `lib.rs` re-exports `asset_references` and `SourceReference`
  - `cargo test -p htmlspec-format`: every test passes, the 3 from 2.1 and the 10 from 2.2 among them. `cargo test -p htmlspec-validate --lib assets`: 24 pass, both from 2.3 among them. The corpus test compared 128 of the 138 documents, holding 640 references, and skipped the 10 parse-error fixtures; the counts were printed once and the print removed
  - `htmlspec format --check fixtures/dogfood` reports all six canonical, and `runbooks` both. `git status` shows neither directory changed. `cargo clippy --workspace --all-targets -- -D warnings` is clean

## 5. Prove it

- [x] 5.1 Repeat 1.1 against the built change, and record the result beside the measurement. The expected result is no fill in the three paragraphs, and `format --check` passing without the document being changed. The `<a>` is still not judged by the asset rules, and the `<link>` still is
  - Recorded in `evidence/the-change-measured-2026-10-09.md`. Every expected result holds. In the same scratch project, at the same commit, `format --check` and `format` report the proposal canonical, and `git diff` is empty. The `<a>` pointed at an absent address gives no asset diagnostic and a canonical check. The `<link>` pointed at it gives `asset/unresolved`, and `format --check` fills it and exits 1
  - Beyond the task: in a copy of the staged migration, all 109 documents are canonical. With both of the drafter's `href<!-- -->=` workarounds restored to `href=`, `establish-a-runbook/design.html` passes the change's check and fails `htmlspec 0.18.0`'s, whose rewrite is HEAD's. Its `tasks.html` passes both, because its quotation names no file
- [x] 5.2 Mutation-test the change, and record which test catches each mutation:
  - the old substring search restored;
  - comments not skipped;
  - raw-text elements not skipped;
  - `<template>` contents not skipped;
  - a repeated attribute's last value kept rather than its first;
  - a quoted value ended at `>`;
  - attribute names compared without lowercasing;
  - `("a", "href")` added to the shared list;
  - values not trimmed;
  - a cut-off tag reported.

  A mutation that nothing catches needs a new test before this task is done
  - Recorded in `evidence/mutation-testing-2026-10-09.md`. **All ten are caught**, and so is 6b, a `>` inside a quoted value ending the tag, which was added because mutation 6 ends only the value. Every mutation compiled, the mutated files matched their copies after each run, and the `proptest-regressions/` seeds the failing runs wrote were removed
  - Two needed a new test, as the task requires. Mutation 10 survived its first run, because no cut-off case ended after a whole attribute; two such cases in `a_tag_cut_off_by_the_end_of_the_source_is_not_emitted` now catch it. Mutation 9 was caught only by the property test, at random; `a_padded_value_is_filled_inside_its_padding` now catches it directly
- [x] 5.3 Run `cargo test --workspace --all-targets`, then `scripts/check`, with every stage green
  - `cargo test --workspace --all-targets`: 1,380 passed, 0 failed. At `0.28.1`, `scripts/check` passes the nine stages that run inside `nix develop` (format, lint, test, doctest, self-conformance, self-format, self-runbooks, self-runbooks-format, self-assets), with the same 1,380 tests. The four Python stages fail there with "tool 'python3' not found", and pass outside it: `check-prompt` (4 cases), `check-installed` (45 files), `check-release --self-test` (23 shared cases), and `check-release` (declared `0.28.1`)
  - After two doc-comment corrections, `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` are clean, and the format and validate library tests pass again (85 and 305). `cargo doc` gives no warning in a file this change touches. It gives three elsewhere, which predate it: an ambiguous `lockfile` link in `htmlspec-assets/src/lib.rs`, and two links to a private `verdict` in `change_markup.rs`
- [x] 5.4 Record what the implementation revealed in `evidence/`, including any document in the corpus on which the scanner and the tree first disagreed
  - Recorded in `evidence/what-the-implementation-revealed-2026-10-09.md`. **The scanner and the tree agreed on the first run, over the corpus and the generated documents.** Three rules taken from the HTML specification were confirmed against the pinned parser: `noscript` is raw text, a `<template>`'s contents are outside the tree, and a CDATA section in `<svg>` is text. The file also records the line the moved list gained on `srcset`, the single-quoted and unquoted references the old search never filled, that only one of the two staged workarounds was needed, and two things left out of this change

## 6. Release

- [x] 6.1 Run the release runbook at `runbooks/release/tasks.html`, and report whether it worked as written
  - Recorded in `evidence/the-runbook-ran-2026-10-09.md`. It worked as written, in its order. The version and the lock moved the seven workspace members. The 45 installed files took the new stamp and nothing else, and `check-installed` passed. The changelog entry names `0.28.1`, and `check-release` passed on the declared version. The four Python stages pass at `0.28.1`
- [x] 6.2 Choose the increment, state the reasoning, and flag it for the contributor's confirmation. The expected increment is a patch, to 0.28.1: the formatter now does what "Formatting preserves meaning" already required, and nothing is added to the contract
  - **`0.28.1`, a patch increment, confirmed by the contributor on 2026-10-09 before the version moved.** The formatter now does what "Formatting preserves meaning" already required, the contract stays at `0.7.0`, and no committed document is reformatted
- [x] 6.3 Check that `scripts/check-release` passes after archival writes `VERSION` and the hand-written `runbooks.json` beside it
  - `openspec archive fill-only-the-references-a-document-loads --yes` archived the change as `2026-10-09-fill-only-the-references-a-document-loads`, on its branch before review, as the contributor asked. It modified 1 requirement in `deterministic-formatting`, and the merged "Formatting preserves meaning" equals the delta's word for word. No other accepted specification changed. `openspec validate deterministic-formatting --type spec --strict` passes. `--specs` still fails only `project-bootstrap`, which is unchanged from HEAD and keeps a placeholder Purpose. It warned of 2 incomplete tasks, this one and 7.1, and continued under `--yes`
  - `VERSION` holds `0.28.1`. `runbooks.json` is a copy of the previous archive's, because the release runbook is unchanged: the sha256 of `runbooks/release/specification.html` and `tasks.html` still match the inventory's (`3d7efaec…` and `e6a4410e…`)
  - `scripts/check-release`: `ok  35 archived change(s) record a version, and 29 version(s) are in the changelog` and `ok  declared version 0.28.1 has a changelog entry`

## 7. Review

- [x] 7.1 Resolve what a review of the pull request found, and record what was and was not taken
  - **The code review of PR #53 (medium) found three readings on which the scanner disagreed with the tree, each in a document read without error, and the contributor chose on 2026-10-09 to fix all three.** The three: an SVG `<style>` or `<title>` read as raw text; an SVG `<template>` whose contents were hidden as an HTML template's; and a script whose `<!--<script>` postpones its end, read as ending at the first `</script>`. In the first two the validator judged a reference the formatter would not fill. In the third the formatter could fill text inside a script, the defect this change exists to remove
  - **Taken.** `tags.rs` now keeps a stack of the foreign elements open, with each one's namespace and how its children are read. Raw text, `template`, and `plaintext` apply only where children are read as HTML. Integration points are SVG's `foreignObject`, `desc`, and `title`, and MathML's text elements, with `mglyph` and `malignmark` staying MathML. An element inside foreign content takes its parent's namespace, except an `svg` directly inside `annotation-xml`. A self-closing foreign tag opens nothing. `script_end` follows the tokenizer's script data, escaped, and double-escaped states. The design's decision 2 and its risks are corrected to match, and the changelog entry says the formatter follows SVG and MathML
  - **Found while fixing: the pinned parser makes no `annotation-xml` an integration point**, where the HTML specification makes one with an HTML `encoding`. kuchikiki leaves markup5ever's `is_mathml_annotation_xml_integration_point` at its default, `false`. The scanner follows the parser, because the tree is what the asset rules judge, and a fragment pins it. Recorded in `evidence/what-the-implementation-revealed-2026-10-09.md`, section 6
  - Tests: 9 scanner unit tests; 14 new fragments in the generated-document cross-check; and a new fixed test, `every_fragment_alone_is_read_as_the_tree_reads_it`, which runs each fragment alone with every value, so no fragment is left to the property test's random draw. Twelve mutations of the fixes are all caught, two after a new test each (`evidence/mutation-testing-2026-10-09.md`, "The review's fixes, mutated")
  - `scripts/check` at `0.28.1`: the nine `nix develop` stages pass, 1,388 tests among them, and the four Python stages pass outside it. `cargo fmt --check` and `cargo clippy -- -D warnings` are clean
  - **Not taken, already recorded:** the review's note that the tree decodes character references and the scanner does not. It is the design's risk "[A value with character references]", and no shipped asset name holds an `&`
