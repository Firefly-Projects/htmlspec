# Evidence: recovered parse errors under the pinned parser (task 1.5)

**Date:** 2026-08-07
**Pins:** `html5ever =0.35.0`, `kuchikiki =0.8.9-speedreader`, `markup5ever =0.35.0`
**Spike tool:** `cargo run -p htmlspec-parse --example parse_error_spike`
**Fixture corpus:** `fixtures/parse-errors/*.html`
**Enumeration in code:** `crates/htmlspec-parse/src/diagnostics.rs`
**Tests:** `crates/htmlspec-parse/tests/parse_errors.rs`

## What the spike measured

HTML5 parsing never fails, it recovers, and `html5ever` reports each recovery through `TreeSink::parse_error` as free text. The design's rule is that any recovery is fatal. Two things had to be measured before that rule is implemented: whether a conforming document produces recoveries anyway, and whether the message text identifies the fault uniquely.

## Finding 1: a conforming document produces zero recoveries

`fixtures/parse-errors/benign-minimal.html` — a document with a short doctype, `head`, `title`, `meta charset`, and a requirement section — reports **zero** recovered errors.

This contradicts the risk register's expectation that "`html5ever` reports errors for benign-looking input", and it is the most useful result of the spike. The permitted-recovery allowlist can start empty and stay empty, because nothing in ordinary conforming markup needs an exemption. The design's fallback — a documented allowlist of recovery-free error codes — remains available and is currently unused.

## Finding 2: `exact_errors` supplies the parser state the design needed

The design records that two distinct faults share the string `Unexpected token`, so message text alone cannot distinguish a missing doctype from a stray end tag. That is true of the terse messages, and `html5ever` has a switch for it. With `exact_errors` set on both `TokenizerOpts` and `TreeBuilderOpts`, the same recovery reports the offending token and the insertion mode:

```text
terse:  Unexpected token
exact:  Unexpected token Tag(Tag { kind: StartTag, name: Atom('html' ...) ... })
        in insertion mode Initial
```

The parser therefore answers the question the design proposed to answer "by parser state", and no message needs to be rejected for ambiguity. **The parser is configured with `exact_errors` enabled**, and the mapping is keyed on a stable message *prefix* rather than on a whole formatted string, because the formatted tail embeds `Debug` output that a patch release can reword.

## Finding 3: quirks mode is a separate signal from parse errors

`fixtures/parse-errors/bad-doctype.html` carries the HTML 4.01 Strict doctype. It reports **zero** recovered errors and parses in `NoQuirks`. `fixtures/parse-errors/no-doctype.html` reports one recovery and parses in `Quirks`.

So a legacy doctype is invisible to the parse-error rule. A document that must render identically in every consumer needs a rule of our own requiring the short doctype, exactly as the offline rule needs a rule of our own for remote data URLs. Recorded here for the rule work in group 4; it is not a parse-error concern.

## The enumeration

Every message the pinned parser can emit, mapped onto a stable project-owned code. Codes are ours, not `html5ever`'s, so a parser upgrade changes the mapping and never the diagnostic surface. All entries are **fatal**. No entry is permitted, so the `permitted` column of the design's allowlist is empty for every row.

| Code | Pinned message prefix | Fault | Fixture |
| --- | --- | --- | --- |
| `parse/unexpected-token` | `Unexpected token` | A token appears in an insertion mode that does not accept it. The exact form names the token and the mode. | `no-doctype.html` |
| `parse/unexpected-open-element` | `Unexpected open element` | An element was still open when another element closed. | `code-example-escapes.html` |
| `parse/unexpected-open-tag-at-eof` | `Unexpected open tag` … `at end of body` | An element was never closed. | `unclosed-element.html` |
| `parse/no-tag-to-close` | `No <p> tag to close`, `No matching tag to close`, `No heading tag to close` | An end tag has no matching open tag. | `code-example-escapes.html` |
| `parse/formatting-element` | `Formatting element not open`, `Formatting element not in scope`, `Formatting element not current node` | Misnested inline formatting elements. | `misnested-tags.html` |
| `parse/special-tag-in-generic-close` | `Found special tag while closing generic tag` | A stray end tag forced the parser out of an open element. | `stray-end-tag.html` |
| `parse/duplicate-attribute` | `Duplicate attribute` | One attribute name appears twice on one element. The second is discarded. | `duplicate-attribute.html` |
| `parse/character-reference` | `Character reference does not end with semicolon`, `Invalid character reference`, `Numeric character reference without digits`, `EOF after '#' in character reference`, `EOF in numeric character reference` | A malformed entity. Interpretation depends on recovery. | `bare-ampersand.html` |
| `parse/table-text` | `Non-space table text` | Text appears where table structure is required, and is relocated. | `table-text.html` |
| `parse/bad-doctype` | `Bad DOCTYPE`, `DOCTYPE in` | A malformed doctype, or a doctype outside the initial insertion mode. | *(no fixture yet — see gap below)* |
| `parse/bad-character` | `Bad character`, `Saw ` | A character illegal in the current tokenizer state. | *(no fixture yet)* |
| `parse/unexpected-eof` | `Unexpected EOF`, `Saw EOF in state` | Input ended inside a construct. | *(no fixture yet)* |
| `parse/self-closing` | `Unacknowledged self-closing tag`, `Self-closing end tag` | `/>` on an element where it means nothing. | *(no fixture yet)* |
| `parse/end-tag-attributes` | `Attributes on an end tag` | Attributes on an end tag, which are discarded. | *(no fixture yet)* |
| `parse/nested-element` | `Nested <nobr>`, `nested buttons`, `nested forms`, `nested heading tags`, `Closing wrong heading tag`, `Bad open element on </form>`, `Form element not in scope on </form>`, `Null form element pointer on </form>`, `bookmark not found in active formatting elements`, `expected to close <td> or <th> with cell`, `</body> with no <body> in scope`, `</html> with no <body> in scope` | Structural faults specific to one element family. | *(no fixture yet)* |
| `parse/unmapped-recovery` | *(anything else)* | A recovery this enumeration does not name. Fatal, and it fails the enumeration-coverage test. | `crates/htmlspec-parse/tests/parse_errors.rs` |

The last row is the load-bearing one. An unmapped message is fatal rather than ignored, so a parser upgrade that introduces a new message cannot quietly widen what the validator accepts. The message inventory was taken from every `parse_error` and `emit_error` call site in the pinned crate, not only from the corpus, so the mapping covers messages the corpus does not reach.

## Gaps, stated rather than hidden

Rows marked *(no fixture yet)* are mapped from the pinned crate's source but are not yet reached by a fixture. The design requires one fixture per **permitted** recovery, and no recovery is permitted, so no row here is short of its required fixture. Group 8 authors the remaining fixtures for diagnostic-message coverage, which is a coverage obligation rather than a safety one.

Two further limits:

1. Mapping is by message prefix. A patch release that rewords a prefix moves messages into `parse/unmapped-recovery`, which is fatal and loud. It cannot move a message into the wrong specific code, because the specific codes are only reachable by prefix match.
2. `kuchikiki` receives the message as `Cow<'static, str>` with no byte offset, so a parse diagnostic names the document and the message and cannot name a line. Offsets would require a different tokenizer integration and are out of scope here.

## Version-change tripwire

`crates/htmlspec-parse/tests/parse_errors.rs` asserts the pinned `html5ever` version against a constant in the enumeration module. A version bump fails that test, which forces the enumeration to be reviewed rather than assumed. This satisfies task 4.2's requirement for a test that fails when the parser version changes without review.
