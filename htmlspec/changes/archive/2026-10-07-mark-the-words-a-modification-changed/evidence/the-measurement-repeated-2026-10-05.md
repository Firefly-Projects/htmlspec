# The measurement repeated against HEAD, before any change

**Date:** 2026-10-05
**Tasks:** 1.2 to 1.4
**Binary:** `htmlspec 0.22.0 (e112067f6a90-dirty)`, built from `main` at `e112067` with only this change's
documents edited. Every `htmlspec` below is `./target/debug/htmlspec`.
**Projection:** `~/projects/html-spec-migration/tools/project-model/target/release/project-model`,
rebuilt against this tree's `crates/htmlspec-parse`. It is written `project-model` below.

The scratch project was built in the session scratchpad with `the-measurement-2026-09-30.md`'s
appendix A script, run with the `ins-del` variant, so the specification holds both requirements. The
same script and a shell wrapper repeat it for task 8.1.

## Task 1.2: sections 2, 3, and 5

### Section 2: the projection, unchanged

```
$ project-model htmlspec/changes/measure-marked-words/specification.html | jq -c '.[] | .requirements[] | {id, change, statements, deletions}'
{"id":"req-a-single-command-exposes-validate-format-and-init","change":"modified","statements":["Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec's name for it. This holds for the subcommand and for the options that select what it acts on.","The spellings are therefore fixed rather than left to an implementer: validate, archive, init, new, list, status, sync, instructions, and store name the operations OpenSpec also provides. format and assets name operations it does not."],"deletions":null}
{"id":"req-a-project-has-a-declared-root-holding-one-asset-store","change":"modified","statements":["An HTMLSpec project SHALL be identified by a directory containing an htmlspec/ directory. That directory SHALL hold the project's accepted specifications, its active changes, and its archive, and its runbooks, each in its own place."],"deletions":null}
```

The inserted word is kept, and the deleted word is still in the statement: "its active changes, and
its archive, and its runbooks". **As measured on 2026-09-30.** No requirement has `deletions`.

### Section 2: validation, different, as expected

```
$ htmlspec validate htmlspec/changes/measure-marked-words/specification.html
  error: schema/change-markup at req-a-single-command-exposes-validate-format-and-init: <ins>sync,</ins> sits in the requirement req-a-single-command-exposes-validate-format-and-init. The projection keeps the inserted words as ordinary text and gives the markup no meaning, so the edit the page shows is in no field the model holds. Write the text as it now reads, without the ins element; a requirement whose words this change edits is marked data-change="modified"
  error: schema/change-markup at req-a-project-has-a-declared-root-holding-one-asset-store: <del>and</del> sits in the requirement … The projection gives change markup no meaning, so the withdrawn words are still part of what the requirement states. …
  error: schema/change-markup at req-a-project-has-a-declared-root-holding-one-asset-store: <ins>and its runbooks,</ins> sits in the requirement … 
  warning: review/stale-reference at verify-the-spellings: …
  warning: review/stale-reference at verify-the-places: …
  3 errors, 2 warnings: validation failed
validate exit 1
$ htmlspec --json validate … | jq -c '{accepted, errors, warnings, rules: [.results[].report.diagnostics[].rule]}'
{"accepted":false,"errors":3,"warnings":2,"rules":["schema/change-markup","schema/change-markup","schema/change-markup","review/stale-reference","review/stale-reference"]}
```

**On 2026-09-30 this exited 0.** `reject-change-markup-inside-a-unit` (0.19.0) landed since and
rejects all three elements in a change's specification as anywhere else. That is the starting point
this change relaxes: the same file is to validate with no `schema/change-markup` in task 8.1. Each
message tells the author to write the text without the element, and that a unit whose words a
change edits is marked `modified`, which these units already are.

The same bytes copied to `htmlspec/specs/cap/spec.html`, an accepted specification, report three
`schema/change-markup` lines and exit 1. Task 8.1 expects the same three there after the change.

### Section 3: the formatter, unchanged

- `format --check` on the specification: `is canonical`, exit 0. `format` on a copy, then `cmp`: exit 0.
- The block-level `<del><p>` and `<ins><p>` put after the list paragraph, one line each:
  `format --check` exits 1, and `format` writes each element with its paragraph on its own line,
  the same six lines as the 2026-09-30 diff. A second `format` changes nothing (`cmp` exit 0).
- `project-model` gives the same four statements before and after formatting, the deleted paragraph
  among them, and no `deletions`.

**As measured on 2026-09-30.**

### Section 5: the stylesheets, unchanged

```
assets/sha256-0350261bb5666167/htmlspec-print.css: 0
assets/sha256-04ac9a24fc436a87/htmlspec-print.css: 0
assets/sha256-aac8967609f80d9f/htmlspec.css: 0
assets/sha256-e4f5de7cecc4f691/htmlspec.css: 0
```

No revision names `ins` or `del`. **As measured on 2026-09-30.**

## Task 1.3: the counts in sections 4 and 7

### Section 4: modified units in the staged migration, unchanged

Appendix B's script, run from `~/projects/html-spec-migration/staged`:

```
[(('evidence', 'added'), 122), (('requirement', 'added'), 137), (('requirement', 'modified'), 34), (('requirement', 'removed'), 1), (('scenario', 'added'), 594), (('scenario', 'modified'), 13), (('scenario', 'removed'), 4), (('scenario', 'unchanged'), 176), (('verification', 'added'), 94)]
[(('change-spec', 'requirement'), 34), (('change-spec', 'scenario'), 13)]
34 modified requirements in 19 changes
```

**As measured on 2026-09-30.** The migration has not been re-staged.

### Section 7: start tags of `ins` and `del`

Counted with `<(ins|del)(?=[\s>/])`, case-insensitive, over `git ls-files '*.html'` and over every
`.html` under the staged `htmlspec/`:

```
tracked start tags: 4 in 125 files
  fixtures/conformance/invalid-schema-change-markup/input.html: 2
  fixtures/conformance/valid-change-markup-outside-a-unit/input.html: 2
staged start tags: 0 in 116 files
```

**The tracked count is no longer zero.** Both files are conformance fixtures that
`reject-change-markup-inside-a-unit` generated to test its rule, and each carries
`<del>every obligation</del><ins>one obligation</ins>`:

| Fixture | Where the markup sits | Its model today | After this change | Its verdict |
| --- | --- | --- | --- | --- |
| `invalid-schema-change-markup` | the statement of a requirement marked `added` | "The system SHALL state every obligationone obligation per requirement, …" | "The system SHALL state one obligation per requirement, …" | invalid, `schema/change-markup`, at every placement: the unit is not `modified` |
| `valid-change-markup-outside-a-unit` | a paragraph of the requirements section, outside every unit | the paragraph projects into no field | the same | valid at every placement |

So one fixture's model changes and neither verdict does. The invalid fixture's note gives as its
reason that "the projection gives change markup no meaning, so the withdrawn words would be part of
the obligation", which this change makes false. Task 1.3 stops the work on a nonzero count until the
contributor decides what these documents should say.

**Settled with the contributor on 2026-10-05:** keep both fixtures and their markup, and reword the
invalid fixture's note in `conformance/src/bin/generate_fixtures.rs`, so its reason is that the
requirement is marked `added` and a fixture sits in no change's specification. Tasks 2.5 and 5.3 then
expect that fixture's `expected.json` to change beside the new fixture's directory.

## Task 1.4: the baseline for task 8.2

`project-model` over every `.html` under the staged `htmlspec/`, one output kept for each document in
the session scratchpad (`baseline-before/models/`): **116 documents projected.**

`htmlspec --json validate` from the staged project's root:

```
{"documents":109,"accepted":true,"errors":0,"warnings":781}
317 warning review/stale-reference
193 warning review/stale-verification
271 warning review/unverified-requirement
```

109 documents rather than 116, because a change's evidence is not discovered. These are the counts
the migration's last `migrate promote --check` recorded. It took 20.45 seconds of CPU time.

`cargo run -p htmlspec-parse --example mapping_models -- --write` printed "every fixture model is
current", and `git status --porcelain fixtures/projection/` printed nothing.
