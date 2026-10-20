## Context

See proposal.md for why. The current mechanism is `rewrite` in
`crates/htmlspec-format/src/references.rs`:

1. Two callers run it: `format` in `crates/htmlspec-cli/src/commands.rs`, before canonicalizing,
   and the migration's staged-document check, before checking the canonical form.
2. It returns the source unchanged when no store is found above the document.
3. Otherwise `next_reference` finds the earliest `href="` or `src="` in the remaining source and
   takes the value up to the next `"`. `decide` fills that value when the bytes it names are absent
   and its file name is one the toolchain ships. Everything between values is copied byte for byte.

Step 3 is where text gets in. `find` does not know whether a match is inside a tag. The
module's doc comment explains why the rewrite is textual: "a tree-level rewrite would reserialize
the document and risk reflowing markup the edit did not touch". That reason still holds. What the
textual walk lacks is the tokenizer's one distinction, between a tag and the text around it.

The asset rules find references in the parsed tree, in `referenced_assets` in
`crates/htmlspec-validate/src/rules/assets.rs`. They walk every element and read the attributes
listed in `REFERENCING_ATTRIBUTES`, a private list of thirteen element and attribute pairs. The
tree is what the parser reads, so it is the authority on what is an element and what is text.

## Goals / Non-Goals

**Goals:**

- The formatter fills exactly the references the asset rules judge, and nothing else.
- The rewrite stays textual: a fill changes only the bytes of one attribute value.
- A test that compares the formatter with the tree. If the two drift, the test fails, so nobody
  has to remember to keep them in step.

**Non-Goals:**

- **Changing what a fill writes, or when.** The absent-only trigger and the refusal to re-point
  stored bytes are `content-address-runtime-assets`' and stay as they are.
- **A general HTML tokenizer.** The scanner reads only enough to place a tag's attributes. It does
  not decode character references or build a tree.
- **The safety rules' list of loading attributes.** See the proposal's Impact.

## Decisions

### 1. One list of the attributes that load an asset, in `htmlspec-assets`

The thirteen pairs move from the validator to `htmlspec-assets`, as a `ReferencingAttribute`
type with an `element` and an `attribute`, and a public `REFERENCING_ATTRIBUTES` list. The asset
rules and the formatter both read it. `htmlspec-assets` is the lowest crate both depend on, and it
already owns `Store` and `Resolution`, which answer the next question about a reference.

**Alternatives considered.**

- **Copy the list into the formatter.** Rejected: there are already two lists, and the measurement
  shows they drifted (`<object data>`, `<a href>`). A third copy is the same fault again.
- **Make `htmlspec-format` depend on `htmlspec-validate`.** Rejected: the validator already
  dev-depends on the formatter, and a formatter that pulls in the rule engine to read one constant
  inverts the layering.

### 2. A scanner that reads tags as the tokenizer does

A private module in `htmlspec-format` walks the source and reports each start tag that the tree
would hold. It gives the tag's lowercased name and its attributes, and for each attribute its
lowercased name and the byte range of its value. It follows the HTML tokenizer where that decides
whether a `<` opens a tag:

- **Comments**, including the abrupt `<!-->` and `<!--->`, end at `-->` or `--!>`.
- **CDATA sections** end at `]]>`. Other markup declarations and processing instructions end at the
  next `>`.
- **End tags** are read the way start tags are, so a `>` inside a quoted value does not end them.
  Their attributes are discarded.
- **A start tag** is `<` followed by an ASCII letter. Its attribute values may be double-quoted,
  single-quoted, unquoted, or absent. A `>` inside a quoted value does not end the tag. When an
  attribute name repeats, the first occurrence wins, as the tokenizer keeps it.
- **Raw-text elements** have contents that are text up to the matching end tag: `script`, `style`,
  `xmp`, `iframe`, `noembed`, `noframes`, `noscript`, `title`, and `textarea`. `noscript` is one of
  them because the pinned parser runs with scripting enabled. `plaintext` makes the rest of the
  source text. A script's text follows the tokenizer's escape states: inside `<!--`, a `<script`
  postpones the end tag until its own `</script` has passed, and `-->` returns to plain script
  text. *(Added at review.)*
- **`<template>` contents** are skipped. The parser puts them in a separate fragment, so the tree
  the asset rules walk does not hold them.
- **Foreign content** is followed as the pinned parser builds it. *(Added at review.)* Inside
  `<svg>` and `<math>`, every element is ordinary: `style`, `title`, and `script` are not raw text,
  and `template` is no template. The scanner keeps a stack of the foreign elements open, so that
  each end tag closes its own. A self-closing foreign tag opens nothing. At an integration point,
  children are read as HTML again: SVG's `foreignObject`, `desc`, and `title`, and MathML's `mi`,
  `mo`, `mn`, `ms`, and `mtext`, where `mglyph` and `malignmark` stay MathML. An element inside
  foreign content takes its parent's namespace whatever its name, except an `svg` directly inside
  `annotation-xml`, which is SVG.

  **The pinned parser, not the HTML specification, decides one case.** The specification also makes
  an `annotation-xml` with an HTML `encoding` an integration point. html5ever asks its tree sink,
  and kuchikiki keeps markup5ever's default answer, which is no. The tree is what the asset rules
  judge, so the scanner follows the parser. A fragment in the cross-check pins it, so a parser
  upgrade that changed the answer would fail that test.
- **A tag cut off by the end of the source** is not reported, because the tokenizer emits nothing
  for it.

`asset_references` keeps each attribute of a start tag that is in the shared list and has a value
that is not blank. It returns the value trimmed, with its range. `rewrite` copies the source
between those ranges and decides each value as before.

**Alternatives considered.**

- **Rewrite in the parsed tree and reserialize.** Rejected for the reason the module already
  gives: it would reflow markup the edit did not touch. Also, `rewrite` runs before the formatter
  decides whether a document can be formatted at all.
- **Ask the parser for positions.** `html5ever` reports line numbers in its errors and no byte
  offsets for tokens, so there is nothing to map a tree attribute back to its bytes.
- **Exclude `<code>` and `<pre>` from the substring search.** Rejected: it fixes the reported case
  and leaves escaped examples elsewhere, comments, `<script>` contents and hyperlinks wrong. The
  distinction the bug violates is tag against text, not one element against another.

### 3. The tree is the oracle, in a test

A hand-written scanner can disagree with the parser in ways no one thought to test. So the scanner
is held to the tree, not to its own author's expectations:

- **The corpus.** For every `.html` file under the repository root (excluding `target/`), the
  references `asset_references` finds equal the ones `referenced_assets` finds, as element,
  attribute and value, in document order.
- **Generated documents.** A property test builds bodies from fragments that each place a
  reference somewhere a substring search would wrongly or rightly find it: in a code element, an
  escaped example, a comment, a raw-text element, a template, a hyperlink, an unquoted or
  single-quoted value, a repeated attribute, an uppercase name, and a `data-src`. It asserts the
  same equality, and that the parser recovered no error, so the vocabulary stays inside the
  documents the formatter will format.

The tests live in the validator's `rules/assets.rs` test module, where `referenced_assets` is in
scope. The oracle is then the asset rules' own function, not a copy of it.

### 4. The rewrite's report keeps its shape

`RewriteOutcome`, `ReferenceChange`, and `KeptReason` do not change. A value outside the shared list
was `NotAnAsset` before, unless it contained `assets/`. Now it is never considered, so it appears in
no report. That is what happened before to a fragment link, and the callers already treat the
absence of a report as "nothing to say".

## Risks / Trade-offs

- **[The scanner and the parser disagree on a document with recovered errors]** → Not held to
  agreement, and not harmful. An `<img>` inside `<select>` is an example: the tree drops it and the
  scanner reports it. `format` refuses any document whose parse recovered an error, and writes
  nothing for it. The migration's check reports that refusal beside any fill. The property test
  holds the scanner to the tree only for error-free documents, and says so.
- **[A value with character references]** → The tree decodes `&amp;` and the scanner does not. An
  asset path holds no `&`, and the corpus test would fail on a document where the two disagreed.
  Decoding would make the scanner a second tokenizer, which decision 2 declines.
- **[The scanner follows the parser more closely than first planned]** → This design first
  recorded two divergences as risks: foreign content, and a script's escape states. The code
  review of the pull request showed each disagreeing with the tree on a document read without
  error. It also showed that a `template` inside `<svg>` hid its contents. The validator then
  judged a reference that the formatter would not fill, or the formatter filled text inside a
  script. The excuse given for the script case, that an inline script is invalid, did not hold,
  because the formatter runs whether or not a document validates. All three are now followed (see
  decision 2), and each has a fragment in the cross-check and a unit test.
- **[A link to an asset that a contributor did want filled]** → After this change, an `<a href>`
  naming absent bytes stays as written, and nothing reports it, because the asset rules never did.
  A hyperlink is a citation by the asset rules' own definition, and they have never reported one.
  The formatter now holds the same view rather than a second one.

## Migration Plan

No document changes. None of the repository's tracked HTML quotes an asset attribute or links to
an asset from an `<a>`, so `format --check` gives the same result before and after. The contract
version does not move, because the document format is unchanged. The release is a patch increment,
flagged for confirmation at its task. Rollback is a revert.

Once this is released, the self-migration's re-stage can remove the drafters' workarounds. That is
its own work.
