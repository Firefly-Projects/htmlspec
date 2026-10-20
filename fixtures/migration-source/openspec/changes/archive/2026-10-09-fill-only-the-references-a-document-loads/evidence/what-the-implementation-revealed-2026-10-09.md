# What the implementation revealed

**Date:** 2026-10-09
**Task:** 5.4

## 1. The scanner and the tree agreed from the first run

Neither cross-check found a disagreement on its first run. That was not assumed in advance. Three
of the scanner's rules were taken from the HTML specification, and each could have been wrong for
the pinned parser:

- `noscript` is raw text because scripting is enabled. `kuchikiki::ParseOpts::default()` leaves
  html5ever's `scripting_enabled` on, and `parse_document` does not change it. The property test's
  `<noscript><img src=…></noscript>` fragment shows the tree holds no `img` there.
- A `<template>`'s contents are not in the tree. kuchikiki keeps them in the element's
  `template_contents` fragment, and `inclusive_descendants` does not enter it. The `<template>`
  fragment shows this.
- A CDATA section inside `<svg>` is text up to `]]>`. The fragment that puts an `<image href>` in
  one shows the tree holds no `image` there.

**The corpus.** The test compared 128 of the repository's 138 HTML documents, which hold 640
references. It skipped the 10 under `fixtures/parse-errors/`, whose parse recovers errors on
purpose. The counts were printed once and the print removed. The test keeps floors of 100 of each,
so it fails if it finds nothing.

## 2. The validator's list moved with one addition to its comment

`REFERENCING_ATTRIBUTES` moved unchanged. Its doc comment gained the reason `srcset` is absent
although `safety/*` reads it: a `srcset` value lists several URLs with their sizes, and the store
cannot resolve any of them as written. The safety rules keep their own list, as the proposal's
Impact says.

## 3. Two forms of value the fill now reaches

The old search found only double-quoted values, so a single-quoted or unquoted reference was never
filled. The consequence depended on what it named:

- an absent address was reported by the asset rules as `asset/unresolved`, and `format` could not
  repair it;
- no address at all was reported by nothing and filled by nothing. `references_match_stored_bytes`
  passes over `Resolution::NotAddressed`.

This is read from the code, and not run against HEAD. No committed document has either form. A
migrated or hand-written one could, and the 2.2 tests for both forms now pin the fill.

## 4. Of the drafters' two workarounds, one was needed

See `the-change-measured-2026-10-09.md` section 2. The quotation in `establish-a-runbook`'s
`tasks.html` names no file (`sha256-…`). The rewrite keeps a reference whose file the runtime does
not ship, so that quotation was safe under the old search too. Only `design.html` needed the comment
split.

## 5. Recorded, not changed

- **`format --check`'s progress lines** say a reference "now names the bytes stored at …" when the
  check wrote nothing. The proposal's Impact lists this as out of scope.
- **The installed binary.** This machine's Nix profile holds `htmlspec 0.18.0`, pinned to a branch
  that predates all ten changes from PR #38. The measurement used `./target/debug/htmlspec`.
  Updating the profile is the contributor's call, outside this change.

## 6. At review: three readings the cross-checks did not cover, and the parser's own answer

The code review found three documents, each read without error, on which the scanner and the tree
disagreed:

- an SVG `<style>` or `<title>` holding a reference;
- an SVG `<template>` holding one;
- a script whose `<!--<script>` postpones its end.

The first two were risks this design had recorded, and the third was excused because inline scripts
are invalid. Neither the corpus nor the generated fragments held any of them, so the cross-checks
passed while the claim "SHALL NOT disagree" did not hold. All three are now followed, and each has
a fragment.

**Writing those fragments found one more disagreement, between the HTML specification and the
pinned parser.**

- The specification makes `<math><annotation-xml encoding="text/html">` an integration point, whose
  children are read as HTML. The fix followed it at first.
- The fragment's document then made the parser recover errors. Its `<style>` was not raw text, and
  the `<img>` written inside it became a misplaced tag.
- html5ever decides this case by asking its tree sink
  (`is_mathml_annotation_xml_integration_point`). kuchikiki does not answer, so markup5ever's
  default answer, `false`, applies.

**So the scanner follows the parser, not the specification.** The asset rules judge the tree, and
the tree is what this parser builds. A fragment pins the parser's answer, so an upgrade that changes
it fails the cross-check rather than passing silently.

The same reading of html5ever's dispatch also corrected a namespace rule. Inside foreign content an
element takes its parent's namespace whatever its name, so an `<svg>` inside MathML is a MathML
element and its `title` is no integration point. The exception is an `<svg>` directly inside
`annotation-xml`, which is SVG.
