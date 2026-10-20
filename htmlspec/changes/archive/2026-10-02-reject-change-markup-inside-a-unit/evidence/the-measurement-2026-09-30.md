# HTML's change markup read into the model, measured against HEAD

**Date:** 2026-09-30
**Tasks:** 1.1 and 1.2, which re-measure this against HEAD before any change, and 5.1, which
repeats it against the built change
**Binary:** `htmlspec 0.18.0 (6a591bdf9d92-dirty)`. The `-dirty` mark comes from
`crates/htmlspec-cli/build.rs`, which asks `git status --porcelain` and so counts untracked files.
The untracked files are this set of planning directories. `git diff HEAD` is empty, so every
compiled source is HEAD's.
**Projection:** `~/projects/html-spec-migration/tools/project-model/target/release/project-model`,
which calls `htmlspec_parse::project` from this repository's `crates/htmlspec-parse` by path. That
crate last changed in `51c5aea` (2026-09-27), before the tool was built, so the tool prints the
model HEAD's `validate` judges.

Every scratch document was made with `htmlspec init --tools none --root <scratch>/proj`, then
`htmlspec new change <name>` and `htmlspec new document specification <path>`, and then edited. Each
one declares contract `0.2.0` and keeps the template's asset references, so the project's store
resolves them.

## 1. A deleted phrase and an inserted word in one statement

The change `measure-change-markup`. Its specification holds one requirement, one scenario, and one
verification naming the requirement. The requirement, as written:

```html
<section class="requirement" id="req-the-spellings-are-fixed" data-change="modified">
  <h3>The spellings are fixed <span class="hs-id">req-the-spellings-are-fixed</span></h3>
  <p>The spellings are fixed: <code>list</code>, <code>status</code>, <code>sync</code>, <code>instructions</code>, and <del>nothing else</del><ins><code>store</code></ins> SHALL name them.</p>
  <article class="scenario" id="scenario-the-spellings-are-fixed">…</article>
</section>
```

```
$ project-model htmlspec/changes/measure-change-markup/specification.html \
    | jq '.[…].requirements[0].statements'
[
  "The spellings are fixed: list, status, sync, instructions, and nothing elsestore SHALL name them."
]
```

**The deleted words are in the normative statement, run together with the inserted one:
`nothing elsestore`.** The projection keeps both texts. `code` and the change markup contribute
their text alike, and no space separates two adjacent elements.

## 2. `validate` on that project accepts it

Run from the project root with no path, while the project held only this change:

```
$ htmlspec validate
exit 0
```

Every diagnostic it printed. The paths are relative to the project root:

| Document | Diagnostic |
| --- | --- |
| `design.html` | warning `review/example-content` at document: the summary is the template's placeholder |
| `proposal.html` | warning `review/example-content` at document: the same |
| `specification.html` | warning `review/stale-reference` at `verify-the-spellings-are-fixed`: "this verification references req-the-spellings-are-fixed, which this change marks modified. Reconcile it when convenient; work continues either way" |
| `tasks.html` | warning `review/example-content` at `task-example` |
| `tasks.html` | warning `review/example-content` at `task-dropped-example` |
| `tasks.html` | warning `review/example-content` at document: the summary is the template's placeholder |

```
4 document(s): 0 error(s), 6 warning(s)
```

**No diagnostic names the change markup.** The one warning on the specification is about the
verification naming a requirement this change modifies, and it would be there without the markup.
The other five are the untouched template documents.

Strict mode and the structured payload say the same about the specification alone:

```
$ htmlspec validate --strict htmlspec/changes/measure-change-markup/specification.html
  warning: review/stale-reference at verify-the-spellings-are-fixed: …
  0 errors, 1 warning: the document is accepted
strict exit 0

$ htmlspec --json validate htmlspec/changes/measure-change-markup/specification.html
{"accepted":true,"documents":1,"errors":0,"warnings":1,"results":[{…,"diagnostics":[{"rule":"review/stale-reference",…}]}]}
json exit 0
```

Strict mode does not promote `review/*`, so no setting a project's checks can use turns this
document's one warning into a failure, and no setting makes any diagnostic name the markup.

## 3. A deleted paragraph becomes a second statement

The change `measure-block-deletion`. The same document, with the statement reading `… and
<code>store</code> SHALL name them.` and one more paragraph after it:

```html
<del><p>No other spelling SHALL name a command.</p></del>
```

```
$ project-model htmlspec/changes/measure-block-deletion/specification.html | jq '…requirements[0]'
{
  "id": "req-the-spellings-are-fixed",
  "change": "modified",
  "statements": [
    "The spellings are fixed: list, status, sync, instructions, and store SHALL name them.",
    "No other spelling SHALL name a command."
  ]
}

$ htmlspec validate htmlspec/changes/measure-block-deletion/specification.html
  warning: review/stale-reference at verify-the-spellings-are-fixed: …
  0 errors, 1 warning: the document is accepted
validate exit 0
```

**The withdrawn paragraph is a second obligation in the model.** The requirement's statements are
every paragraph among its own descendants, and a `del` carries no reserved class, so the walk goes
through it to the `p` inside. No parser recovery is reported, so `<del><p>` is valid markup and not
a malformed document.

`htmlspec format --check` reports this document as "not canonically formatted", and the one from
section 1 as canonical. Run on a copy, `format` changed one thing: it moved the `p` onto its own
line inside the `del`. The projected statements were the same two afterwards. That does not bear
on this change.

## 4. How many existing documents the rule would break

Counted per file and summed, for the raw strings and again for an element start tag:

```
grep -o -i '<ins' "$f" | wc -l
grep -o -i '<del' "$f" | wc -l
grep -o -i -E '<ins[[:space:]>/]' "$f" | wc -l
grep -o -i -E '<del[[:space:]>/]' "$f" | wc -l
```

over `git ls-files '*.html'` and over `find ~/projects/html-spec-migration/staged -type f -name '*.html'`.

| Set | Files | `<ins` | `<del` | start tags |
| --- | --- | --- | --- | --- |
| `git ls-files '*.html'`, without `README.html` | 121 | 0 | 0 | 0 |
| `README.html`, a 4.2 MB page with its libraries inlined | 1 | 0 | 0 | 0 |
| `~/projects/html-spec-migration/staged/`, every `.html` | 116 | 0 | 0 | 0 |

**Zero in every set.** No document the repository tracks, and none the staged migration would
promote, carries change markup, so the rule breaks none of them. `README.html` is the one tracked
page that inlines a vendored bundle. It is listed apart because the count was to exclude bundles,
and it holds none either. No tracked `.html` file lives under a vendor directory.

The counter finds markup where it exists. Over the scratch project's changes it found 5 `<ins` and
7 `<del` across the four documents that carry them.

Among tracked sources, `git grep` for `"ins"`, `"del"`, `<ins `, `<del `, `<ins>`, `<del>`, and
the backquoted names finds one file: `assets/sha256-18327bef70d96fb5/mermaid.min.js`, whose
sanitizer lists both as allowed tags. No crate, fixture, template, instruction, or accepted
specification mentions them.

## 5. Beyond the four questions: what else change markup reaches

The scope of the rule turns on where change markup reaches the model, so three more cases were
projected.

### A deleted link is still a reference

The change `measure-outside-units`, `specification.html`. The verification has no `data-refs`, and
its method paragraph names the requirement only inside deleted text:

```html
<p data-field="method">List the commands and compare each with the fixed spellings<del>, as <a href="#req-the-spellings-are-fixed">the requirement</a> states them</del>.</p>
```

```
"method": "List the commands and compare each with the fixed spellings, as the requirement states them.",
"references": [ { "origin": "link", "target": "req-the-spellings-are-fixed" } ]

validate exit 0: 0 errors, 1 warning (review/stale-reference)
```

The control is the same document with the deleted words actually removed, placed in the same
project (`measure-control`):

```
error: schema/invalid at verify-the-spellings-are-fixed (/verifications/0/references): [] has less than 1 item
warning: review/unverified-requirement at req-the-spellings-are-fixed: no verification in this specification names this requirement. …
1 error, 1 warning: validation failed
exit 1
```

**A verification whose only reference is deleted still verifies the requirement.** It satisfies
the schema's `minItems: 1` on references and clears `review/unverified-requirement`. Remove the
withdrawn words, and the document is rejected.

### A unit wrapped in `del` is still a unit

The change `measure-wrapped-unit`. A second requirement, `req-no-other-spelling`, sits whole inside
`<del>…</del>` with `data-change="unchanged"`:

```
{"id":"req-the-spellings-are-fixed","change":"modified","statements":["The spellings are fixed: list, status, sync, instructions, and store SHALL name them."]}
{"id":"req-no-other-spelling","change":"unchanged","statements":["No other spelling SHALL name a command."]}

validate exit 0: 0 errors, 1 warning (review/stale-reference)
```

**The deleted requirement projects as one the document still states.** The `del` is not inside a
unit, so a rule that looked only at a change element's ancestors would pass it.

### Outside every unit: free prose projects nowhere, and three other places do

The change `measure-outside-units`, `proposal.html`. Change markup in four places outside every
unit:

```html
<header class="hs-masthead">
  <h1>measure-outside-units</h1>
  <p>The command surface gains <del>no new spelling</del><ins>one spelling, <code>store</code></ins>.</p>
  …
</header>
…
<section id="hs-what-changes">
  <h2>What changes</h2>
  <p>The fixed spellings become <code>list</code>, <code>status</code>, <code>sync</code>, <code>instructions</code>, and <del>nothing else</del><ins><code>store</code></ins>.</p>
  <p>This replaces <del><a href="#hs-why">the argument above</a></del><ins><a href="#hs-scope">the scope below</a></ins>.</p>
  <figure id="figure-spellings">
    <figcaption>The spellings <del>before</del><ins>after</ins> this change.</figcaption>
    <table><tr><th>Spelling</th></tr><tr><td>store</td></tr></table>
  </figure>
</section>
```

```
"summary": [ "The command surface gains no new spellingone spelling, store." ],
"links": [ …, { "target": "hs-why", "text": "the argument above" }, { "target": "hs-scope", "text": "the scope below" } ],
"figures": [ { "caption": "The spellings beforeafter this change.", "id": "figure-spellings", "kind": "prose" } ]

validate exit 0: no diagnostics
```

| Where, outside every unit | Reaches the model? |
| --- | --- |
| A paragraph of free prose | **No.** The model has no field for it, and its keys are only `figures`, `kind`, `links`, `schemaVersion`, `subject`, and `summary` |
| The document's summary, from the `header` | Yes, run together: `no new spellingone spelling, store` |
| A figure's caption | Yes, run together: `beforeafter` |
| A fragment link | Yes: the deleted link is an entry in `links`, target and text |

**Free prose outside a unit reaches nothing, as expected. The summary, a caption, and a link do.**
None of them carries a contract. In this repository's code, the summary is read to compare it
with a template's placeholder (`rules/review.rs`, and `commands.rs` for archival), a caption only
for whether it exists (`rules/a11y.rs`), and a link for whether its target resolves
(`rules/refs.rs`). The design decides what the rule does about them.
