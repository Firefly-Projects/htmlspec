# A one-word modification, and change markup inside a modified unit, measured against HEAD

**Date:** 2026-09-30
**Tasks:** 1.1 to 1.4, which re-measure this against HEAD before any change, and 5.1, which
repeats it against the built change
**Binary:** `htmlspec 0.18.0 (6a591bdf9d92)`. It prints `6a591bdf9d92-dirty`, because the working
tree holds this set's untracked change directories. `git status --porcelain` lists only `??`
entries, so every compiled source is HEAD's. Every `htmlspec` below is this repository's
`./target/debug/htmlspec`, called by its absolute path.
**Projection:** `~/projects/html-spec-migration/tools/project-model/target/release/project-model`,
which calls `htmlspec_parse::project` from this repository's `crates/htmlspec-parse`. It is written
`project-model` below.

Scratch files are under `scratchpad/mark-the-words-a-modification-changed/`. The script that writes
the scratch specification is reproduced in appendix A, so the measurement does not depend on that
directory surviving.

## Summary

| Question | Answer |
| --- | --- |
| What did the one-word change change? | One line of a 10-paragraph, 18-scenario requirement: `` `sync`, `` inserted into the fixed list of spellings. The archived delta equals the accepted text after the merge. |
| Where is that word recorded today? | In git, and in a sentence of the staged specification's summary. The staged requirement marks the whole unit `modified`, and its paragraph with `sync` in it looks like the other nine. |
| What does `project-model` make of `<ins><code>sync</code>, </ins>`? | The inserted word is kept, and the statement is correct. `validate` exits 0. |
| And of a real deletion beside an insertion? | The deleted word stays in the statement: "its active changes, and its archive, and its runbooks". `validate` exits 0. |
| Does the formatter handle both forms? | Inline markup is canonical as written. A block-level `<del><p>` is laid out with the `p` on its own line, a second run changes nothing, and the statements are the same before and after. |
| How do authors say what moved today? | In summary prose. 34 modified requirements in 19 staged changes, and 5 of those changes' summaries narrate what moved inside a statement. One author bolded the added words, and the bold reached the accepted contract, where it still is. The word the same edit deleted was never marked. |
| What does the stylesheet do with `ins` and `del`? | Nothing. No revision of either stylesheet names them, so the browser's defaults apply: an underline and a line-through. The stylesheet also underlines every link. |
| Is `withdrawn` free as a model name? | No. It names a unit marked `removed` across the accepted specifications, the instructions, the code, and changes 2 and 4. `deletions` names nothing in the contract or the code. |

## 1. The one-word example, from git

`2026-08-31-sync-a-change-into-the-accepted-contract` was archived and merged in one commit:

```
$ git log --format='%h %ad %s' --date=short -- openspec/changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract | tail -1
ee4301c 2026-08-31 Add the sync operation archival tells you to run (#16)

$ git log --format='%h %ad %s' --date=short -- openspec/specs/toolchain-packaging/spec.md
6a591bd 2026-09-29 Keep a change's evidence out of document discovery (#37)
51c5aea 2026-09-27 Record the heading a renamed requirement replaced (#35)
17afc05 2026-09-26 Sync the run-record and portability contracts, and archive both changes (#34)
80558b5 2026-09-02 Let a change state the change it comes after (#19)
595e38c 2026-09-02 Apply the listing change's contract and archive it (#18)
ee4301c 2026-08-31 Add the sync operation archival tells you to run (#16)
cf11132 2026-08-30 Finish the archival, and fix the gate that never fired (#13)
…
```

So "before" is `ee4301c^` and "after" is `ee4301c`. The requirement "The command surface names its
operations as OpenSpec names them" was extracted from each, and from the archived delta, from its
`### Requirement:` line to the next one:

```
$ wc -l before.md after.md delta.md
      96 before.md
      96 after.md
      95 delta.md
$ grep -c '^#### Scenario' before.md after.md delta.md
before.md:18
after.md:18
delta.md:18
$ wc -w before.md
    1191 before.md
```

The requirement holds 10 statement paragraphs and 18 scenarios, 1,191 words in all.

```
$ diff before.md after.md
7c7
< The spellings are therefore fixed rather than left to an implementer: `validate`, `archive`, `init`, `new`, `list`, `status`, `instructions`, and `store` name the operations OpenSpec also provides. `format` and `assets` name operations it does not. The selectors `--json`, `--specs`, `--change`, and `--store` carry OpenSpec's spellings for the same reason.
---
> The spellings are therefore fixed rather than left to an implementer: `validate`, `archive`, `init`, `new`, `list`, `status`, `sync`, `instructions`, and `store` name the operations OpenSpec also provides. `format` and `assets` name operations it does not. The selectors `--json`, `--specs`, `--change`, and `--store` carry OpenSpec's spellings for the same reason.
diff exit 1

$ git diff --no-index --word-diff=plain before.md after.md | grep -E '\[-|\{\+'
The spellings are therefore fixed rather than left to an implementer: `validate`, `archive`, `init`, `new`, `list`, `status`, {+`sync`,+} `instructions`, and `store` name the operations OpenSpec also provides. …

$ diff delta.md after.md
95a96
>
diff exit 1
```

**The whole edit is `` `sync`, ``, in the third of ten paragraphs.** The archived delta is the
accepted text after the merge, apart from a trailing blank line. It restates the requirement whole,
and nothing in it says which word moved.

The staged migration states the same requirement in
`~/projects/html-spec-migration/staged/htmlspec/changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract/specification.html`,
line 144, as `<section class="requirement" id="req-a-single-command-exposes-validate-format-and-init" data-change="modified">`,
with 10 statement paragraphs and 18 scenarios. Its paragraph at line 148 carries `<code>sync</code>`
like every other spelling, and the file holds no `ins` or `del`. The one place the word is named is
the document's summary:

> Its only change to The command surface names its operations as OpenSpec names them, which is
> restated here in full, is `sync` added to that requirement's fixed list of spellings.

## 2. The same paragraph written with change markup

### The scratch project

```
$ htmlspec init --tools none --root <scratch>/proj                                   # exit 0
$ cd <scratch>/proj
$ htmlspec new change measure-marked-words                                           # exit 0
$ htmlspec new document specification htmlspec/changes/measure-marked-words/specification.html \
    --subject "measure-marked-words: specification"                                  # exit 0
$ python3 write_spec.py htmlspec/changes/measure-marked-words/specification.html ins  # exit 0
```

The scratch project is not inside a git repository. The script (appendix A) replaces the template's
units with one requirement marked `modified`, carrying the staged identifier, one scenario marked
`unchanged`, and one verification naming the requirement. The requirement holds two statements: the
paragraph before the list, which carries the requirement's SHALL, and the list paragraph with the
inserted word marked:

```html
<section class="requirement" id="req-a-single-command-exposes-validate-format-and-init" data-change="modified">
  <h3>The command surface names its operations as OpenSpec names them <span class="hs-id">req-a-single-command-exposes-validate-format-and-init</span></h3>
  <p><strong>Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec's name for it.</strong> This holds for the subcommand and for the options that select what it acts on.</p>
  <p>The spellings are therefore fixed rather than left to an implementer: <code>validate</code>, <code>archive</code>, <code>init</code>, <code>new</code>, <code>list</code>, <code>status</code>, <ins><code>sync</code>, </ins><code>instructions</code>, and <code>store</code> name the operations OpenSpec also provides. <code>format</code> and <code>assets</code> name operations it does not.</p>
  <article class="scenario" id="scenario-the-spellings-are-found" data-change="unchanged">…</article>
</section>
```

A first version held the list paragraph alone, and `validate` rejected it with `schema/weak-modal`,
because that paragraph states no SHALL. The accepted requirement's SHALL is in the paragraph before
it, so that paragraph was added.

### An inserted word is kept, and the statement is correct

```
$ project-model htmlspec/changes/measure-marked-words/specification.html | jq '.[] | .requirements[] | {id, change, statements}'
{
  "id": "req-a-single-command-exposes-validate-format-and-init",
  "change": "modified",
  "statements": [
    "Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec's name for it. This holds for the subcommand and for the options that select what it acts on.",
    "The spellings are therefore fixed rather than left to an implementer: validate, archive, init, new, list, status, sync, instructions, and store name the operations OpenSpec also provides. format and assets name operations it does not."
  ]
}

$ htmlspec validate htmlspec/changes/measure-marked-words/specification.html
  warning: review/stale-reference at verify-the-spellings: this verification references req-a-single-command-exposes-validate-format-and-init, which this change marks modified. Reconcile it when convenient; work continues either way
  0 errors, 1 warning: the document is accepted
validate exit 0
strict exit 0
$ htmlspec --json validate … | jq -c '{accepted, errors, warnings}'
{"accepted":true,"errors":0,"warnings":1}
```

**Today an `ins` is read like `code` or `strong`, and here that gives the right statement.** The one
warning is about the verification naming a requirement this change modifies, and it would be there
without the markup.

### A deleted word beside an inserted one is kept too

For the deletion, a real edit from this repository's history was used rather than an invented one.
`2026-09-18-establish-a-runbook` changed project-bootstrap's "A project has a declared root holding
one asset store". The merge commit shows it word by word:

```
$ git log --format='%h %ad %s' --date=short -S'**its runbooks**' -- openspec/specs/project-bootstrap/spec.md
48ffe3b 2026-09-18 Sync the runbook capability into the contract, and archive the change (#30)
$ git diff --word-diff=plain 48ffe3b^ 48ffe3b -- openspec/specs/project-bootstrap/spec.md | grep 'runbooks\*\*'
An HTMLSpec project SHALL be identified by a directory containing an `htmlspec/` directory. That directory SHALL hold the project's accepted specifications, its active changes,[-and-] its archive, {+and **its runbooks**,+} each in its own place.
The bootstrap operation SHALL be safe to run on a project it has already established. On such a run it SHALL refresh the instructions for the selected tools, and SHALL leave the project's accepted specifications, active changes, [-and archive-]{+archive, **and runbooks**+} unchanged.
```

The first sentence, marked with both elements, went into the same specification as a second
requirement marked `modified` (`write_spec.py … ins-del`):

```html
<p>An HTMLSpec project SHALL be identified by a directory containing an <code>htmlspec/</code> directory. That directory SHALL hold the project's accepted specifications, its active changes, <del>and </del>its archive, <ins>and its runbooks, </ins>each in its own place.</p>
```

```
$ project-model … | jq '.[] | .requirements[1] | {id, change, statements}'
{
  "id": "req-a-project-has-a-declared-root-holding-one-asset-store",
  "change": "modified",
  "statements": [
    "An HTMLSpec project SHALL be identified by a directory containing an htmlspec/ directory. That directory SHALL hold the project's accepted specifications, its active changes, and its archive, and its runbooks, each in its own place."
  ]
}

$ htmlspec validate htmlspec/changes/measure-marked-words/specification.html
  warning: review/stale-reference at verify-the-spellings: …
  warning: review/stale-reference at verify-the-places: …
  0 errors, 2 warnings: the document is accepted
validate exit 0
strict exit 0
{"accepted":true,"errors":0,"warnings":2}
```

**The statement reads "its active changes, and its archive, and its runbooks", which is neither the
text before the edit nor the text after it.** `validate` accepts it. This is the defect
`reject-change-markup-inside-a-unit` measured with an invented phrase ("nothing elsestore"), and
its evidence also shows a deleted paragraph becoming a second statement, a deleted link still
counting as a reference, and a requirement wrapped in `del` still projecting
(`openspec/changes/reject-change-markup-inside-a-unit/evidence/the-measurement-2026-09-30.md`,
sections 1, 3, and 5). Those are not repeated here.

## 3. The formatter

The specification from section 2, with both requirements:

```
$ htmlspec format --check htmlspec/changes/measure-marked-words/specification.html
htmlspec/changes/measure-marked-words/specification.html is canonical
format --check exit 0
$ cp … fmt-inline.html && htmlspec format fmt-inline.html && diff … fmt-inline.html
diff exit 0
```

Inline `ins` and `del` are canonical as written. The formatter's element table
(`crates/htmlspec-format/src/element.rs`) names neither, so each is an unknown element, and an
unknown element is inline.

Then two block-level forms were put after the list paragraph in a copy, `block.html`, one line each:

```html
<del><p>No other spelling SHALL name a command.</p></del>
<ins><p>A spelling OpenSpec does not provide SHALL be one this project chooses.</p></ins>
```

```
$ htmlspec format --check block.html
block.html: not canonically formatted
format --check exit 1
$ cp block.html block-fmt.html && htmlspec format block-fmt.html && diff block.html block-fmt.html
41,42c41,46
<             <del><p>No other spelling SHALL name a command.</p></del>
<             <ins><p>A spelling OpenSpec does not provide SHALL be one this project chooses.</p></ins>
---
>             <del>
>               <p>No other spelling SHALL name a command.</p>
>             </del>
>             <ins>
>               <p>A spelling OpenSpec does not provide SHALL be one this project chooses.</p>
>             </ins>
$ cp block-fmt.html block-fmt2.html && htmlspec format block-fmt2.html && cmp block-fmt.html block-fmt2.html
second format cmp exit 0
```

```
$ for f in block.html block-fmt.html; do project-model $f | jq -c '.[] | .requirements[0].statements'; done
["Where an operation … acts on.","The spellings are therefore fixed … operations it does not.","No other spelling SHALL name a command.","A spelling OpenSpec does not provide SHALL be one this project chooses."]
["Where an operation … acts on.","The spellings are therefore fixed … operations it does not.","No other spelling SHALL name a command.","A spelling OpenSpec does not provide SHALL be one this project chooses."]
```

**The formatter lays a block-level `del` or `ins` out with its paragraph on its own line, a second
run changes nothing, and the projected statements are the same before and after.** Both still
include the deleted paragraph, as today's projection does. Placed inside the scratch project,
`block.html` validates with 0 errors and the two `review/stale-reference` warnings. Outside it, the
same file reports 7 `asset/unresolved` errors, because no store sits above it.

## 4. Where authors say what a modification changed today

### How many modified requirements there are

Counted over every `specification.html` in the staged archive, and then over every `.html` under
`~/projects/html-spec-migration/staged/htmlspec`, placed by where each file sits (script in appendix
B):

| Unit, in a change's specification | `added` | `modified` | `removed` | `unchanged` |
| --- | --- | --- | --- | --- |
| requirement | 137 | **34** | 1 | 0 |
| scenario | 594 | **13** | 4 | 176 |
| evidence | 122 | 0 | 0 | 0 |
| verification | 94 | 0 | 0 | 0 |

The 34 modified requirements sit in 19 changes. **Every unit marked `modified` in the staged tree is
a requirement or a scenario in a change's specification.** No proposal, design, tasks document,
accepted specification, or runbook document marks a unit `modified`. The staged accepted
specifications hold 138 requirement units and 590 scenario units, and no unit of any other kind.

### Summary prose that narrates the edit

The summaries of the 19 changes were read in full. Five of them say what moved inside a modified
requirement's statements, which no unit can say today:

| Change | The sentence |
| --- | --- |
| `2026-08-31-sync-a-change-into-the-accepted-contract` | "Its only change to The command surface names its operations as OpenSpec names them, which is restated here in full, is `sync` added to that requirement's fixed list of spellings." |
| `2026-08-18-conform-generated-frontmatter-names` | "This change restated Bootstrapping installs agent instructions for the tools a project uses in full and added its last two paragraphs and its last four scenarios, …" and "The requirement's first eight paragraphs and first eight scenarios are as the change that installed the full workflow set stated them." |
| `2026-08-24-pass-the-description-through` | "This change added four paragraphs and five scenarios to the one requirement it restates, …" and "The requirement's other paragraphs and scenarios are carried as `distinguish-command-names-from-skill-names` last stated them." |
| `2026-08-21-distinguish-command-names-from-skill-names` | "It added to the requirement's prose that no two files written for one tool declare the same name, … and it removed the clause that the same identity is used for every kind of file a workflow produces." |
| `2026-09-15-catch-a-templates-placeholder-prose` | "The requirement's rule that a unit is checked on its identifier rather than on its text is kept, and bounded to units." |

The other 14 summaries say what the change made the contract require, and not where in a restated
requirement the new words are.

### Added words marked with bold, and the deleted word not marked

`2026-09-18-establish-a-runbook` marked its added words with Markdown bold in its delta,
`openspec/changes/archive/2026-09-18-establish-a-runbook/specs/project-bootstrap/spec.md`:

- line 5: "…its active changes, its archive, and **its runbooks**, each in its own place."
- line 35: "…active changes, archive, **and runbooks** unchanged."

The staged migration carries both as `<strong>` in
`…/2026-09-18-establish-a-runbook/specification.html`, lines 289 and 337:
`its archive, and <strong>its runbooks</strong>, each in its own place` and
`archive, <strong>and runbooks</strong> unchanged`.

**The bold reached the accepted contract and is still there.** `openspec/specs/project-bootstrap/spec.md`
line 10 reads "…its archive, and **its runbooks**, each in its own place." and line 313 reads
"…archive, **and runbooks** unchanged." The staged accepted
`~/projects/html-spec-migration/staged/htmlspec/specs/project-bootstrap/spec.html` carries them as
`<strong>` at lines 43 and 566. The word diff in section 2 shows how they arrived at `48ffe3b`, and
that the same edit **deleted** "and" before "its archive" without marking it. In the accepted
contract the bold now reads as emphasis, and nothing says it once meant "added here".

## 5. What the stylesheet and the browser do with the two elements

Each stylesheet revision in the store, searched for either element name as a whole word:

```
$ for f in assets/sha256-*/*.css; do printf '%s: ' $f; grep -c -E '(^|[^a-zA-Z0-9_-])(ins|del)([^a-zA-Z0-9_-]|$)' $f; done
assets/sha256-0350261bb5666167/htmlspec-print.css: 0
assets/sha256-04ac9a24fc436a87/htmlspec-print.css: 0
assets/sha256-aac8967609f80d9f/htmlspec.css: 0
assets/sha256-e4f5de7cecc4f691/htmlspec.css: 0
```

**No revision styles `ins` or `del`.** `assets/vendor.lock.json` records `aac8967609f80d9f` and
`04ac9a24fc436a87` as the current revisions under `own`, and the other two under `retained`.

The browser's own stylesheet therefore decides. The HTML Standard's rendering section, 15.3.4
"Phrasing content" (html.spec.whatwg.org/multipage/rendering.html, read 2026-09-30), states:

```
ins, u { text-decoration: underline; }
del, s, strike { text-decoration: line-through; }
```

The project's screen stylesheet underlines every link, `assets/sha256-aac8967609f80d9f/htmlspec.css`
lines 414-421:

```css
a {
  color: var(--hs-accent);
  /* Underlined rather than colored alone, because color is never the only
   * signal that something is a link. */
  text-decoration: underline;
  text-decoration-thickness: 1px;
  text-underline-offset: 0.13em;
}
```

So on screen, an inserted word and a link both carry an underline, and the stylesheet's own
distinction between them is the link's accent color. The template links `htmlspec.css` with no
`media` attribute, so it applies to print as well. The print stylesheet, loaded with
`media="print"`, sets `a { text-decoration: none; }` (`assets/sha256-04ac9a24fc436a87/htmlspec-print.css`,
lines 138-143), so on paper only an inserted word is underlined.

The screen stylesheet states its rule for change state at line 562: "three redundant channels,
never hue alone", of which the first is "A text label, rendered from the attribute value by
`content`, so the state is a word a reader reads and a screen reader announces." MDN's reference for
`del` (developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/del, read 2026-09-30) says:

> The presence of the `del` element is not announced by most screen reading technology in its
> default configuration. It can be made to be announced by using the CSS `content` property, along
> with the `::before` and `::after` pseudo-elements.

It adds that the technique should be kept to "situations where not knowing content has been deleted
would adversely affect understanding". This was read, not measured with a screen reader.

## 6. Whether the working name is free

`withdrawn`, as a whole word, case-insensitive, in tracked files:

| Where | Lines |
| --- | --- |
| `crates/` | 63 |
| of which `crates/htmlspec-assets/instructions/` | 6 |
| `conformance/` | 2 |
| `openspec/specs/` | 11 |

In every place read, it means a unit marked `removed` or a spelling taken out of use. document-schema's
"Semantic units declare a change state" has the scenario "Requirement being withdrawn", for a
requirement carrying `data-change="removed"`. `sync.md` line 153 says an identifier may be absent
"because the requirement was withdrawn". Among this set's changes,
`resolve-a-reference-across-its-change` names a field `withdrawn` in its `Declared` type, holding the
identifiers of units marked `removed`, and `state-a-modified-requirement-whole` says "A scenario the
change withdraws SHALL be stated as well, as a unit marked `removed`."

`deletions`, as a whole word: one line in `crates/` (a doc comment in
`crates/htmlspec-assets/src/lib.rs` about assembling templates), none in `conformance/` or
`openspec/specs/`. In the staged tree it appears only in prose quoting `git diff --stat` output.

## 7. No existing document carries the markup

Recounted over start tags, as `reject-change-markup-inside-a-unit` counted them:

```
tracked start tags: 0 in 122 files
staged start tags: 0 in 116 files
```

## Appendix A: `write_spec.py`

```python
#!/usr/bin/env python3
"""Replaces the <main> of a specification made by `htmlspec new document specification`
with the units this measurement needs, and its summary with one sentence.

Usage: write_spec.py <specification.html> <variant>
  variant "ins"     : the sync paragraph with <ins><code>sync</code>, </ins>
  variant "ins-del" : the same, plus the runbook paragraph with <del>and </del> and an <ins>
"""
import re, sys

path, variant = sys.argv[1], sys.argv[2]

NAMES = ('<strong>Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec\'s '
         'name for it.</strong> This holds for the subcommand and for the options that select what it acts on.')
SYNC = ('The spellings are therefore fixed rather than left to an implementer: '
        '<code>validate</code>, <code>archive</code>, <code>init</code>, <code>new</code>, '
        '<code>list</code>, <code>status</code>, <ins><code>sync</code>, </ins><code>instructions</code>, '
        'and <code>store</code> name the operations OpenSpec also provides. <code>format</code> and '
        '<code>assets</code> name operations it does not.')
RUNBOOK = ('An HTMLSpec project SHALL be identified by a directory containing an <code>htmlspec/</code> '
           'directory. That directory SHALL hold the project\'s accepted specifications, its active changes, '
           '<del>and </del>its archive, <ins>and its runbooks, </ins>each in its own place.')

def requirement(rid, heading, paragraphs, sid, stitle):
    body = ''.join(f'''
            <p>{p}</p>''' for p in paragraphs)
    return f'''
          <section class="requirement" id="{rid}" data-change="modified">
            <h3>{heading} <span class="hs-id">{rid}</span></h3>{body}
            <article class="scenario" id="{sid}" data-change="unchanged">
              <h4>{stitle}</h4>
              <dl>
                <dt>When</dt>
                <dd>A contributor runs the operation.</dd>
                <dt>Then</dt>
                <dd>It is found under the spelling the requirement fixes.</dd>
              </dl>
            </article>
          </section>'''

def verification(vid, rid):
    return f'''
          <section class="verification" id="{vid}" data-change="added" data-outcome="not-run" data-refs="{rid}">
            <h3>Checked by reading the command list <span class="hs-id">{vid}</span></h3>
            <p data-field="method">List the commands and compare each with the fixed spellings.</p>
          </section>'''

reqs = [requirement('req-a-single-command-exposes-validate-format-and-init',
                    'The command surface names its operations as OpenSpec names them', [NAMES, SYNC],
                    'scenario-the-spellings-are-found', 'The spellings are found')]
vers = [verification('verify-the-spellings', 'req-a-single-command-exposes-validate-format-and-init')]
if variant == 'ins-del':
    reqs.append(requirement('req-a-project-has-a-declared-root-holding-one-asset-store',
                            'A project has a declared root holding one asset store', [RUNBOOK],
                            'scenario-the-places-are-found', 'The places are found'))
    vers.append(verification('verify-the-places', 'req-a-project-has-a-declared-root-holding-one-asset-store'))

main = ('\n        <section id="hs-requirements">\n          <h2>Requirements</h2>' + ''.join(reqs) +
        '\n        </section>\n        <section id="hs-verification">\n          <h2>Verification</h2>' +
        ''.join(vers) + '\n        </section>\n      ')
t = open(path).read()
t = re.sub(r'(<main class="hs-main" id="hs-content">).*?(</main>)', lambda m: m.group(1) + main + m.group(2), t, flags=re.S)
t = re.sub(r'<p>Replace this paragraph with what this document is for\..*?</p>',
           '<p>A scratch specification for measuring how change markup projects today.</p>', t, flags=re.S)
t = re.sub(r'\s*<li><a href="#hs-(decisions|risks|evidence)">[^<]*</a></li>', '', t)
open(path, 'w').write(t)
```

## Appendix B: the counts in section 4

Run from `~/projects/html-spec-migration/staged`:

```python
import re, glob, collections, os
by_state = collections.Counter()      # (kind, state) in change specifications
modified = collections.Counter()      # (place, kind) for units marked modified, whole tree
changes = collections.Counter()       # modified requirements per change
UNIT = r'<(\w+)\s+([^>]*class="[^"]*\b(requirement|scenario|decision|risk|evidence|verification|task)\b[^"]*"[^>]*)>'
for f in sorted(glob.glob('htmlspec/**/*.html', recursive=True)):
    if '/evidence/' in f: place = 'evidence'
    elif f.startswith('htmlspec/specs/'): place = 'accepted'
    elif f.startswith('htmlspec/runbooks/'): place = 'runbook'
    elif os.path.basename(f) == 'specification.html' and '/changes/' in f: place = 'change-spec'
    elif '/changes/' in f: place = 'change-doc'
    else: place = 'other'
    t = open(f).read()
    for m in re.finditer(UNIT, t):
        s = re.search(r'data-change="(\w+)"', m.group(2))
        s = s.group(1) if s else '(none)'
        kind = m.group(3)
        if place == 'change-spec': by_state[(kind, s)] += 1
        if s == 'modified':
            modified[(place, kind)] += 1
            if kind == 'requirement': changes[f.split('/')[3]] += 1
print(sorted(by_state.items())); print(sorted(modified.items()))
print(sum(changes.values()), 'modified requirements in', len(changes), 'changes')
```

Output:

```
[(('evidence', 'added'), 122), (('requirement', 'added'), 137), (('requirement', 'modified'), 34), (('requirement', 'removed'), 1), (('scenario', 'added'), 594), (('scenario', 'modified'), 13), (('scenario', 'removed'), 4), (('scenario', 'unchanged'), 176), (('verification', 'added'), 94)]
[(('change-spec', 'requirement'), 34), (('change-spec', 'scenario'), 13)]
34 modified requirements in 19 changes
```

The summaries in section 4 were printed with this, from `htmlspec/changes/archive`, and read by
hand:

```python
import re, glob
for f in sorted(glob.glob('*/specification.html')):
    t = open(f).read()
    if not re.findall(r'class="requirement" id="[^"]+" data-change="modified"', t): continue
    header = re.search(r'<header[^>]*>(.*?)</header>', t, re.S).group(1)
    ps = [re.sub(r'<[^>]+>', '', re.sub(r'\s+', ' ', p)).strip() for p in re.findall(r'<p>(.*?)</p>', header, re.S)]
    print('==', f.split('/')[0]); print('  ', ' '.join(ps))
```
