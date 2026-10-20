# The premises, checked before building on them

Tasks 1.1 to 1.3, run against `17afc05` with the debug binary built from it. **Two of the design's
premises hold and one does not.** The one that does not is the premise decisions 3 and 4 rest on.

## 1.1 A renamed requirement keeps its identifier, and `sync` calls the rename reached

A scratch project (`htmlspec init --tools none`, then `htmlspec new change rename-x`) with one
accepted specification holding `req-x` under "The old heading", and a change specification stating
`req-x` under "The new heading" with `data-change="modified"`. Both documents are the specification
template's shell with the requirement section replaced.

```
$ htmlspec validate htmlspec/specs/cap/spec.html                  # exit 0, 0 errors, 1 warning
$ htmlspec validate htmlspec/changes/rename-x/specification.html   # exit 0, 0 errors, 1 warning
$ htmlspec sync --change rename-x
1 of 1 requirement(s) this change states have reached the accepted specifications
  reached  req-x
  the change's contract has been applied in full
$ htmlspec --json sync --change rename-x
{ "absent": [], "change": "rename-x", "reached": ["req-x"], "stated": ["req-x"] }
```

The warning on each is `review/unverified-requirement`, which the scratch documents earn by carrying
no verification and which has nothing to do with the rename.

**This is the "before" state.** The accepted specification still says "The old heading", and the
operation reports the change's contract as "applied in full". No other check the project has
reports it either.

## 1.2 The projected `title` includes the `hs-id` span, so the design's premise is false

Design decisions 3 and 4 say the heading comparison "uses the projected `title`, which excludes the
`hs-id` span". **It does not.** Projection section 3.1 takes a unit's title from the text content of
its first heading, and section 8.3 says an inline `span` contributes its text. Nothing
special-cases `class="hs-id"`, because section 2.2 says that class tokens other than the reserved one are
ignored. Measured through `htmlspec_parse::project`:

| Heading markup | Projected `title` |
| --- | --- |
| `<h3>The old heading <span class="hs-id">req-x</span></h3>` (the template's form) | `The old heading req-x` |
| the same, with line breaks inside the heading | `The old heading req-x` |
| `<h3>The old heading<span class="hs-id">req-x</span></h3>` (no space) | `The old headingreq-x` |
| `<h3>The old heading</h3>` (no span) | `The old heading` |

**Whitespace normalization does make a reflowed heading compare equal.** `htmlspec format` does not
reflow heading text at all: it kept the line breaks inside the second heading. And the projection's
section 6.1 normalization collapses them, so the projected title is identical before and after
formatting. That half of the premise holds.

What fails is the other half, and it affects both decisions:

- **Decision 3, "equal to the heading".** A `renamed-from` paragraph states a previous heading as a
  reader reads it, and that is how migration writes it from an OpenSpec `FROM:` line: `The old heading`.
  Compared with the projected title `The old heading req-x`, it is never equal. So the rule would
  never fire on a document written from the template.
- **Decision 4, the untaken-rename comparison.** Comparing two projected titles works when both
  documents write the identifier the same way, because the suffix is the same identifier on both
  sides. It reports a false untaken rename when one document carries the span and the other does
  not. It also puts `req-x` into both headings it names.

The fix, and the design edit it needs, are recorded in the design under decision 3.

## 1.3 OpenSpec 1.13.1's rename rules, against their counterparts

Read from `lib/openspec/dist/core/specs-apply.js` (`buildUpdatedSpec`) and
`lib/openspec/dist/core/parsers/requirement-blocks.js` in the installed 1.13.1.

| OpenSpec rule | Counterpart here |
| --- | --- |
| A `FROM:` or `TO:` that never forms a pair is an error | Structural. The pair is the requirement's heading and its one paragraph, so it cannot be half-written. An empty paragraph is the nearest case, and the "empty" rule covers it |
| The same `FROM:` twice in one delta is an error | "more than once in one requirement". Two paragraphs on one requirement are one requirement renamed twice |
| The same `TO:` twice in one delta is an error | **None, deliberately.** See below |
| RENAMED and REMOVED naming one requirement is an error | "on a `removed` requirement" |
| A MODIFIED block naming the `FROM:` heading when a rename exists is an error | Structural. The modified content sits on the unit whose heading is the new name, so it cannot name the old one |
| A RENAMED `TO:` colliding with an ADDED requirement is an error | "on an `added` requirement", per the design's decision 1 table |
| RENAMED into a specification that does not exist is an error | The merge comparison reports the identifier as `absent`, and never as an untaken rename. This is a finding, not an error, because archival assesses the merge and never performs it |
| Apply order RENAMED → REMOVED → MODIFIED → ADDED | Not needed. Identity is the identifier, so a rename and a modification are two facts about one unit (the artifact-kinds delta) and order cannot change the result |
| Source heading gone and target heading present: a no-op, already synced | Decision 4: the accepted heading equals the change's heading, so the requirement is reached |
| Source gone, target present, and a case or spacing variant of the source still present: an error ("typo'd header") | **None, deliberately.** See below |
| Source and target both absent: an error | `absent`, as above |
| Source and target both present: an error ("target already exists") | **None, deliberately.** See below |
| A target that differs from another requirement only in case or spacing: an error | **None, deliberately.** See below |
| A case-only rename is allowed | Allowed. The "equal to the heading" rule compares after whitespace normalization and is case-sensitive, so `Foo` → `foo` is a rename |
| Header matching strips a closing `#` run and trims | Section 6.1 whitespace normalization. A closing `#` run is Markdown syntax and has no HTML counterpart |
| Chained renames inside one delta (`A → B`, then `B → C`) apply in sequence | **None, deliberately.** One `renamed-from` per requirement. The intermediate heading was never accepted, so the change states `renamed-from` `A` under heading `C`. Migration collapses such a chain to its first `FROM:` |
| Main specifications carry no RENAMED section | The artifact-kinds delta. The accepted specification carries no `renamed-from` |

**Every rule marked "none, deliberately" has one reason. OpenSpec finds the requirement a rename
applies to by its heading, and html-spec finds it by its identifier.** OpenSpec needs those rules
because a heading is its only key, so two requirements that would share a heading, or a `FROM:`
spelled slightly differently from the accepted heading, would apply a rename to the wrong
requirement or to none. Here the `renamed-from` paragraph is not a lookup key. The merge comparison
reads the accepted requirement with the same identifier, and names that requirement's actual heading
in its report. So a mistyped `renamed-from` cannot misdirect anything, and two requirements with
one heading are two identifiers. The design names these as deliberate differences.
