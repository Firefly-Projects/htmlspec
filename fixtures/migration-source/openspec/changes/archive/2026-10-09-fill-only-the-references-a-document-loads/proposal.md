## Why

**`htmlspec format` rewrites an author's prose.** When the store does not hold the bytes an asset
reference names, the formatter fills that reference. It finds references by searching the raw
source for `href="` and `src="`, so it also finds them in text. In a project with a store, prose
that quotes `href="assets/htmlspec.css"` inside a `<code>` element has its words changed to a hashed
path. The same happens to an escaped example in a `<pre>`. Then `format --check` fails until the
author's words are wrong.

`evidence/the-measurement-2026-10-09.md` measured this against HEAD. One `format` run on a scratch
change made four fills:

- three changed the author's words: two `<code>` quotations and one escaped `<pre><code>` example;
- one changed an `<a href>`, a hyperlink the document cites and does not load.

`validate` accepted the document before and after, because the words were valid either way.

deterministic-formatting's "Formatting preserves meaning" forbids this: formatting "SHALL NOT
change a document's semantic content". The fill itself was decided in
`content-address-runtime-assets` (design, "Rewrite references in the formatter, not by hand"), but
no requirement states it. Nothing says which values are references and which are words.

**The formatter and the asset rules disagree on what an asset reference is.**

- The asset rules (`asset/unresolved`, `asset/mismatch`) judge thirteen element and attribute
  pairs, the attributes through which an element loads a file. `<a href>` is deliberately not
  among them: "a hyperlink is a citation, not a dependency".
- The formatter judges every substring `href="` or `src="`. So it fills text and hyperlinks, and it
  misses `<object data>`, which the asset rules judge.

**Why now.** The self-migration's drafters have met the fault twice:

- in drafting, on 2026-09-29;
- in the proof run of `migrate-into-the-extended-vocabulary`, on 2026-10-08.

Each time, they changed the author's markup to get past it. One split a `<code>`, and another put an
HTML comment between `href` and `="`. The comment workaround is still in two staged documents. The
re-stage that comes next re-drafts every document in the new vocabulary, and each document that
quotes an attribute meets the fault again.

## What Changes

- **The formatter fills only an attribute through which an element loads an asset.** It reads the
  same pairs the asset rules judge, from one list that both read. Text is never a reference: a
  quotation in a `<code>` element, an escaped example, a comment, and the contents of a `<script>`
  or `<style>` are all left as written. An `<a href>` is left as written.
- **References are found in the HTML tokenizer's terms.** A tag's attributes are read as the parser
  reads them: by name rather than substring, in single, double or no quotes, and in any letter case.
  So `<object data>`, `SRC='…'` and an unquoted value are filled, and `data-src="…"` is not.
- **The rewrite stays textual.** Only the bytes of a filled value change, so formatting still does
  not widen a diff.
- **A test holds the formatter to the parsed tree.** For every HTML document in the repository, and
  for generated documents, the references the formatter finds are exactly the ones the asset rules
  find in the tree, in the same order.
- **"Formatting preserves meaning" states the fill and its bounds.** A reference naming absent bytes
  is filled. A reference naming stored bytes is kept. Text that quotes a reference is content.
- **Unchanged:**
  - what a fill writes, and when (only when the bytes are absent);
  - the refusal to re-point a reference that names stored bytes;
  - the asset rules' verdicts;
  - the contract version;
  - every committed document. None of this repository's tracked HTML quotes an asset attribute or
    links to an asset from an `<a>`, so none is reformatted.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `deterministic-formatting`: "Formatting preserves meaning" gains the fill as its one bounded
  exception, and gains three scenarios:
  - prose quoting an asset reference;
  - a hyperlink to an asset;
  - a reference that names stored bytes.

  The requirement is modified rather than added to, because the exception qualifies its "every
  attribute value". Stated alone, the two would contradict each other.

## Impact

- `crates/htmlspec-assets`:
  - The list of attributes that load an asset moves here from the validator's `rules/assets.rs`.
  - It becomes a named type, so that the asset rules and the formatter read one list.
- `crates/htmlspec-format/src/references.rs`: `next_reference` and `ATTRIBUTES` are replaced. A new
  `asset_references` returns each reference with its element, attribute, and value.
- `crates/htmlspec-format`: a new private module that reads tags in the tokenizer's terms. It
  covers comments, CDATA sections, end tags, attribute quoting, duplicate attributes, the raw-text
  elements, and `<template>` contents.
- `crates/htmlspec-validate/src/rules/assets.rs`: the asset rules read the moved list. A
  cross-check test compares the formatter's references with `referenced_assets`, over the
  repository's HTML and over generated documents. `proptest` becomes a dev-dependency.
- `crates/htmlspec-format`: regression tests for each case in the measurement, and for the cases
  the scanner must get right.
- `crates/htmlspec-cli`: no code change. `format` and the migration's staged-document check both
  call `rewrite`, and both gain the fix.
- **Not in this change**, and recorded so it is not assumed to follow:
  - Removing the workarounds from the staged migration. That is the re-stage's work, and the staged
    tree is outside this repository.
  - The `safety/*` rules' own list of loading attributes. It adds `srcset`, which names several
    URLs in one value and is no asset reference this program fills, so the safety rules keep their
    own list.
  - The wording of `format --check`'s progress lines, which say a reference "now names" bytes when
    nothing was written.
