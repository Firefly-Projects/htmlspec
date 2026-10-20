# The measurement, repeated against the built change

**Date:** 2026-10-02
**Task:** 4.1
**Binary:** `htmlspec 0.19.0 (6fd228eb392d-dirty)`, built from this change's working tree after the
instructions were edited and the installed copies regenerated. The `-dirty` mark is this change.

## The served instructions

`./target/debug/htmlspec instructions sync` and `instructions archive` each exit 0, and each prints
its source file byte for byte (`diff` against `crates/htmlspec-assets/instructions/<name>.md` is
empty). Read with every whitespace run collapsed, so that a phrase wrapped across lines still
counts:

| Body | "partial update" | "A change's specification states a modified requirement whole" | Lines containing "partial" |
| --- | --- | --- | --- |
| `sync` | absent | 2 times: "Leave every requirement the change does not state untouched" and "Merging a modified requirement" | none |
| `archive` | absent | 2 times: step 2's "A requirement the change modified" and step 5's opening paragraph | none |

Before the change, the same reading found "partial update" at `sync.md` lines 105 and 135 and at
`archive.md` lines 260 and 264, and the requirement named in neither.

## 1.4, repeated

The same `grep -n -i "partial\|restate\|wholesale\|modified"` over the instructions and
`assets/template-specification.html` now finds 41 lines, against 19 before. Every added line is in
`sync.md` or `archive.md`, in the text this change wrote. **No line matches "partial".** The hits in
`continue.md`, `ff.md`, and `migrate.md` are the same lines as before, and the template still has
none.

`grep -rn -i "partial update"` over `crates`, `conformance`, `scripts`, and the product's installed
copies (`.claude/commands/htmlspec`, `.claude/skills/htmlspec-*`, `.codex`, `.pi`) finds no product
file. The only hits outside the test that names the phrase are OpenSpec's own generated files. Three
carry `generatedBy: "1.7.0"`. The Claude Code command and the Pi prompt carry no stamp, and all five
arrived in one commit, `2989654`, and are unchanged since:

- `.claude/commands/opsx/sync.md`
- `.claude/skills/openspec-sync-specs/SKILL.md`
- `.codex/skills/openspec-sync-specs/SKILL.md`
- `.pi/prompts/opsx-sync.md`
- `.pi/skills/openspec-sync-specs/SKILL.md`

The proposal's "Not in this change" names the first two. **The other three are the same files for
two more tools**, and the proposal's reason covers them: they are OpenSpec's generated files and
govern this repository's OpenSpec-form work, not the product.

## 1.3, repeated: the scratch project's results are unchanged

Appendix C of `the-measurement-2026-09-30.md`, run into a fresh `after/` directory. Its output is
byte-identical to the run before the change (`diff rerun/html-spec-today.txt after/html-spec-today.txt`
is empty):

| Document | `validate` | Scenarios projected for `req-x` |
| --- | --- | --- |
| `htmlspec/specs/cap/spec.html` | exit 0, 0 errors, 1 warning | A, B, C, all `unchanged` |
| `changes/partial/specification.html` | exit 0, 0 errors, 1 warning | D `added` |
| `changes/whole/specification.html` | exit 0, 0 errors, 1 warning | A `unchanged`, B `unchanged`, C `removed`, D `added` |

`htmlspec --json sync` still reports `req-x` as `reached` for both changes. **That is the expected
result.** This change adds no check, so the partial form still validates and still reads as reached.
The rule is now stated in the contract and in the instructions that perform the merge, and the
comparison that would report the partial form is `see-a-modification-the-merge-did-not-take`'s.

## The fixture from 2.2 validates

`cargo test -p htmlspec-conformance --test conformance`: 16 passed, 0 failed. For this fixture the
harness asserts three things: it is accepted (`every_fixture_is_accepted_exactly_when_it_declares_it_is`
and `every_valid_fixture_is_accepted_by_the_whole_rule_set`), it formats to a fixed point, and
formatting it does not change what the validator reports. **It does not assert that the fixture
reports nothing.** `every_fixture_reports_exactly_the_rules_it_declares` skips a fixture whose rule
list is empty, as the harness's own comment on `a_summary_quoting_the_placeholder_reports_nothing`
records. So the fixture pins that the shape is accepted, which is what the design asks of it, and
not that it is silent. Its projection:

```
req-example modified [('scenario-example', 'unchanged'), ('scenario-altered', 'modified'),
                      ('scenario-introduced', 'added'), ('scenario-withdrawn', 'removed')]
```

`htmlspec --json validate` on the file reports `"accepted": true` and 0 errors. Its 10 warnings are
the same rules at the same units as `valid-minimal`'s: 8 `review/example-content` on the template's
example identifiers, and 2 `review/stale-reference` on the evidence and the verification that
reference `req-example`. None names any of the three scenarios the fixture adds.

## `archive.md`'s example validates where an author would paste it

The `html` block under "A requirement the change modified" was extracted from `archive.md` and
pasted into the requirements section of a document from
`htmlspec new document specification spec.html --subject "The pasted example"`, beside the
template's own example requirement:

```
  0 errors, 11 warnings: the document is accepted
validate exit 0
```

The 11 warnings are the template's own 10, the same as an unedited specification, and one
`review/unverified-requirement` on `req-the-accepted-identifier`, because no verification names the
pasted requirement. After `htmlspec format spec.html`, `validate` reports the same and
`format --check` exits 0. The projection holds the requirement as `modified`, with its one statement
and its four scenarios in the states the example declares.
