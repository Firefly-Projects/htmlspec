# The measurement, repeated before anything changed

**Date:** 2026-10-01
**Tasks:** 1.1 and 1.2
**Binary:** `htmlspec 0.18.0 (62403651e46d-dirty)`, built from `6240365`, the merge of #38. The
`-dirty` mark is this file, which is untracked. No compiled source differs from HEAD.
**Projection:** `~/projects/html-spec-migration/tools/project-model`, rebuilt against this tree.
Cargo found nothing to rebuild, because `crates/htmlspec-parse` has not changed since the tool was
built.

The script is `measure.sh` with `write_docs.py`, in the appendix. It makes a fresh project with
`htmlspec init --tools none`, makes each change with `htmlspec new change` and
`htmlspec new document specification`, and then writes the documents section 1, 3, and 5 of
`the-measurement-2026-09-30.md` quote. The same script, unchanged, is task 5.1's measurement.

## 1.1 Every result reproduces

| Section | 2026-09-30 | 2026-10-01 |
| --- | --- | --- |
| 1. `statements` | `… and nothing elsestore SHALL name them.` | the same |
| 2. `validate` over the project, holding only `measure-change-markup` | exit 0, 4 documents, 0 errors, 6 warnings; on the specification only `review/stale-reference` | the same, diagnostic for diagnostic |
| 2. `validate --strict` on the specification | exit 0, 1 warning | the same |
| 2. `--json validate` on the specification | `"accepted":true`, the one `review/stale-reference` | the same |
| 3. block `<del><p>`: `statements` | the statement and `No other spelling SHALL name a command.` | the same |
| 3. `validate` | exit 0, 1 warning | the same |
| 3. `format --check` | not canonical; section 1's canonical | the same. `format` moves the `p` onto its own line inside the `del` and the two statements stay |
| 5. deleted link: `method` and `references` | `… as the requirement states them.`, one `link` reference | the same |
| 5. deleted link: `validate` | exit 0, 1 warning | the same |
| 5. the control, the words removed | exit 1: `schema/invalid` `[] has less than 1 item`, and `review/unverified-requirement` | the same |
| 5. wrapped requirement | projects `req-no-other-spelling`, `unchanged`, with its statement; exit 0, 1 warning | the same |
| 5. proposal: model keys | `figures`, `kind`, `links`, `schemaVersion`, `subject`, `summary` | the same |
| 5. proposal: summary | `The command surface gains no new spellingone spelling, store.` | the same |
| 5. proposal: links | `hs-why` "the argument above" and `hs-scope` "the scope below" | the same |
| 5. proposal: caption | `The spellings beforeafter this change.` | the same |
| 5. proposal: `validate` | exit 0, no diagnostics | the same |

**One count differs, and only in when it was taken.** Over the scratch changes the counter found 5
`<ins` and 8 `<del` in five documents, where the evidence has 5 and 7 in four. The eighth is the
`del` around the wrapped requirement. The evidence counted before it made that document, and this
script counts at the end.

Three things the evidence does not quote had to be chosen to reproduce it, and each was settled by
the diagnostics the evidence records:

- The verification keeps the template's `data-outcome="not-run"`. With `passed`, the warning is
  `review/stale-verification` and not `review/stale-reference`.
- The specification's contents keep only the links to the two sections it keeps. The template's
  links to the decisions, risks, and evidence sections are otherwise `refs/broken-link`.
- In the wrapped case, the verification names both requirements. Naming one adds
  `review/unverified-requirement` on the other, and the evidence reports one warning.

## 1.2 The count is still zero

The same four `grep` counts as section 4 of the evidence, per file and summed:

| Set | Files | `<ins` | `<del` | start tags |
| --- | --- | --- | --- | --- |
| `git ls-files '*.html'`, without `README.html` | 121 | 0 | 0 | 0 |
| `README.html` | 1 | 0 | 0 | 0 |
| `~/projects/html-spec-migration/staged/`, every `.html` | 116 | 0 | 0 | 0 |

**Zero in every set**, so the contract bump in group 4 goes ahead on the premise the design states.
The staged migration has not moved since the proposal: it is still the stage-6 tree, waiting on
this change and the nine after it.

## Appendix: the script

`measure.sh`, run as `measure.sh <htmlspec binary> <output dir>`:

```bash
#!/bin/bash
# Reproduces sections 1, 2, 3, and 5 of the change's measurement in a fresh scratch project.
# Usage: measure.sh <htmlspec binary> <output dir>
set -u
H=$1
OUT=$2
HERE=$(cd "$(dirname "$0")" && pwd)
PM=~/projects/html-spec-migration/tools/project-model/target/release/project-model
rm -rf "$OUT"
mkdir -p "$OUT"
"$H" init --tools none --root "$OUT/proj" >/dev/null
cd "$OUT/proj" || exit 1
"$H" --version

make_change() { # name case
  "$H" new change "$1" >/dev/null
  "$H" new document specification "htmlspec/changes/$1/specification.html" >/dev/null
  python3 "$HERE/write_docs.py" "htmlspec/changes/$1" "$2"
}
model() { # path jq-filter
  "$PM" "$1" | jq -c --arg p "$1" ".[\$p] | $2"
}
validate() { # args...
  "$H" "$@"
  echo "exit $?"
}

echo "== 1. A deleted phrase and an inserted word in one statement"
make_change measure-change-markup measure-change-markup
S1=htmlspec/changes/measure-change-markup/specification.html
model $S1 '.requirements[0].statements'

echo "== 2. validate over the project, holding only this change"
validate validate
echo "-- strict, the specification alone"
validate validate --strict $S1
echo "-- json, the specification alone"
validate --json validate $S1

echo "== 3. A deleted paragraph"
make_change measure-block-deletion measure-block-deletion
S3=htmlspec/changes/measure-block-deletion/specification.html
model $S3 '.requirements[0] | {id, change, statements}'
validate validate $S3
echo "-- format --check, this document and section 1's"
validate format --check $S3
validate format --check $S1
cp $S3 "$OUT/formatted.html"
"$H" format "$OUT/formatted.html" >/dev/null
diff $S3 "$OUT/formatted.html"
model "$OUT/formatted.html" '.requirements[0].statements'

echo "== 5a. A deleted link, and its control"
make_change measure-outside-units measure-outside-units
S5=htmlspec/changes/measure-outside-units/specification.html
model $S5 '.verifications[0] | {method, references}'
validate validate $S5
make_change measure-control measure-control
validate validate htmlspec/changes/measure-control/specification.html

echo "== 5b. A unit wrapped in del"
make_change measure-wrapped-unit measure-wrapped-unit
SW=htmlspec/changes/measure-wrapped-unit/specification.html
model $SW '.requirements[] | {id, change, statements}'
validate validate $SW

echo "== 5c. Outside every unit, the proposal"
python3 "$HERE/write_docs.py" htmlspec/changes/measure-outside-units outside-units-proposal
P5=htmlspec/changes/measure-outside-units/proposal.html
model $P5 '{keys: keys, summary, links: [.links[] | select(.target == "hs-why" or .target == "hs-scope")], figures}'
validate validate $P5

echo "== The counter finds markup where it exists"
ins=0; del=0; files=0
for f in htmlspec/changes/*/*.html; do
  i=$(grep -o -i '<ins' "$f" | wc -l); d=$(grep -o -i '<del' "$f" | wc -l)
  if [ $((i + d)) -gt 0 ]; then files=$((files + 1)); fi
  ins=$((ins + i)); del=$((del + d))
done
echo "$files documents: $ins <ins, $del <del"
```

`write_docs.py`:

```python
"""Writes the measurement's documents into a scratch project made by `htmlspec init`.

Usage: write_docs.py <change-dir> <case>

Each case replaces the template specification's masthead summary and main, or the
template proposal's, with the document the evidence quotes. The head, the asset
references, and the declared contract stay the template's.
"""

import re
import sys
from pathlib import Path

REQUIREMENT = """<section class="requirement" id="req-the-spellings-are-fixed" data-change="modified">
            <h3>The spellings are fixed <span class="hs-id">req-the-spellings-are-fixed</span></h3>
            <p>{statement}</p>{extra}
            <article class="scenario" id="scenario-the-spellings-are-fixed">
              <h4>Every command has a fixed spelling</h4>
              <dl>
                <dt>When</dt>
                <dd>The commands are listed.</dd>
                <dt>Then</dt>
                <dd>Each spelling is one the requirement names.</dd>
              </dl>
            </article>
          </section>"""

WRAPPED = """
          <del><section class="requirement" id="req-no-other-spelling" data-change="unchanged">
            <h3>No other spelling <span class="hs-id">req-no-other-spelling</span></h3>
            <p>No other spelling SHALL name a command.</p>
            <article class="scenario" id="scenario-no-other-spelling">
              <h4>An unlisted spelling</h4>
              <dl>
                <dt>When</dt>
                <dd>A command is invoked by a spelling the list does not hold.</dd>
                <dt>Then</dt>
                <dd>Nothing runs.</dd>
              </dl>
            </article>
          </section></del>"""

VERIFICATION = """<section class="verification" id="verify-the-spellings-are-fixed" data-change="added" data-outcome="not-run"{refs}>
            <h3>The spellings were listed <span class="hs-id">verify-the-spellings-are-fixed</span></h3>
            <p data-field="method">{method}</p>
          </section>"""

SPELLINGS = (
    "The spellings are fixed: <code>list</code>, <code>status</code>, <code>sync</code>, "
    "<code>instructions</code>, and "
)
MARKED = SPELLINGS + "<del>nothing else</del><ins><code>store</code></ins> SHALL name them."
PLAIN = SPELLINGS + "<code>store</code> SHALL name them."
METHOD = "List the commands and compare each with the fixed spellings."
DELETED_LINK = (
    "List the commands and compare each with the fixed spellings<del>, as "
    '<a href="#req-the-spellings-are-fixed">the requirement</a> states them</del>.'
)
REFS = ' data-refs="req-the-spellings-are-fixed"'


def specification(statement, extra="", wrapped="", refs=REFS, method=METHOD):
    requirements = REQUIREMENT.format(statement=statement, extra=extra) + wrapped
    verification = VERIFICATION.format(refs=refs, method=method)
    return f"""<main class="hs-main" id="hs-content">
        <section id="hs-requirements">
          <h2>Requirements</h2>
          {requirements}
        </section>
        <section id="hs-verification">
          <h2>Verification</h2>
          {verification}
        </section>
      </main>"""


PROPOSAL_SUMMARY = (
    "The command surface gains <del>no new spelling</del><ins>one spelling, "
    "<code>store</code></ins>."
)

PROPOSAL_WHAT_CHANGES = """<section id="hs-what-changes">
          <h2>What changes</h2>
          <p>The fixed spellings become <code>list</code>, <code>status</code>, <code>sync</code>, <code>instructions</code>, and <del>nothing else</del><ins><code>store</code></ins>.</p>
          <p>This replaces <del><a href="#hs-why">the argument above</a></del><ins><a href="#hs-scope">the scope below</a></ins>.</p>
          <figure id="figure-spellings">
            <figcaption>The spellings <del>before</del><ins>after</ins> this change.</figcaption>
            <table><tr><th>Spelling</th></tr><tr><td>store</td></tr></table>
          </figure>
        </section>"""

CASES = {
    "measure-change-markup": ("specification", specification(MARKED)),
    "measure-block-deletion": (
        "specification",
        specification(
            PLAIN,
            extra="\n            <del><p>No other spelling SHALL name a command.</p></del>",
        ),
    ),
    "measure-outside-units": (
        "specification",
        specification(PLAIN, refs="", method=DELETED_LINK),
    ),
    "measure-control": ("specification", specification(PLAIN, refs="", method=METHOD)),
    "measure-wrapped-unit": (
        "specification",
        specification(
            PLAIN,
            wrapped=WRAPPED,
            refs=' data-refs="req-the-spellings-are-fixed req-no-other-spelling"',
        ),
    ),
}

SUMMARY = re.compile(r"(<header class=\"hs-masthead\">\s*<h1>[^<]*</h1>\s*<p>)(.*?)(</p>)", re.S)


def main():
    change_dir, case = Path(sys.argv[1]), sys.argv[2]
    if case == "outside-units-proposal":
        path = change_dir / "proposal.html"
        html = path.read_text()
        html, n = SUMMARY.subn(lambda m: m.group(1) + PROPOSAL_SUMMARY + m.group(3), html, 1)
        assert n == 1, "the proposal has a masthead summary"
        html, n = re.subn(
            r'<section id="hs-what-changes">.*?</section>', PROPOSAL_WHAT_CHANGES, html, 1, re.S
        )
        assert n == 1, "the proposal has a What changes section"
        path.write_text(html)
        return
    _, main_html = CASES[case]
    path = change_dir / "specification.html"
    html = path.read_text()
    html, n = SUMMARY.subn(
        lambda m: m.group(1) + "Measures how change markup reaches the model." + m.group(3),
        html,
        1,
    )
    assert n == 1, "the specification has a masthead summary"
    html, n = re.subn(r'<main class="hs-main" id="hs-content">.*?</main>', main_html, html, 1, re.S)
    assert n == 1, "the specification has a main"
    html, n = re.subn(
        r'\s*<li><a href="#hs-(decisions|risks|evidence)">[^<]*</a></li>', "", html
    )
    assert n == 3, "the specification's contents name the three sections removed"
    path.write_text(html)


if __name__ == "__main__":
    main()
```
