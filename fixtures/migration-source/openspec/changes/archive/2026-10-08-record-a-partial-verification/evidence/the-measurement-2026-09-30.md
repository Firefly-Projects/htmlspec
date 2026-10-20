# A check that covered part of its scenario, measured against HEAD

**Date:** 2026-09-30
**Tasks:** 1.1, 1.2, and 1.3
**Binary:** `htmlspec 0.18.0 (6a591bdf9d92)`. The binary reports itself as `6a591bdf9d92-dirty`
because the working tree holds untracked change directories. No tracked file was modified
(`git status --porcelain | grep -vc '^??'` printed `0`).

## 1. Two verification units removed on review for covering part of their scenario

The source is this repository's own migration, staged and waiting in
`~/projects/html-spec-migration/`. Its review log records each draft's verdict. Lines 124 and 126
of `review-log.md`, under "Convention-6 recheck results (units added)":

```
  - confine 5 → 4 (removed a-second-store-planted: the record lacks "names both locations");
  - record-a-renamed 5 → 4 (removed mutation-19: it checks only the collision check, not what migration produces);
```

Each draft states the removal in full.

**`verify-a-second-store-planted-inside-the-project`**, in `drafts/2026-09-28-confine-the-asset-check-to-the-project.md`, line 117:

> I wrote this unit and flagged it, and review removed it. The Given matches, down to "one beside a
> change's documents". But the Then is "the duplication is reported and names both locations", and
> the record gives only "reports `8 duplicated asset(s)` and exits 1", with no diagnostics. A
> `passed` unit would infer the second half.

The scenario is document-validation's "Two stores inside the project":

```
- **THEN** the duplication is reported and names both locations
```

**`verify-mutation-19`**, in `drafts/2026-09-26-record-a-renamed-requirement.md`, line 125:

> I first wrote `verify-mutation-19` for "One obligation renamed across changes". The review ruled
> it out. The scenario's Then states what migration produces: both changes carry the one
> identifier, and each states the heading it used. Mutation 19's test takes those as input and
> checks only that the plan's collision check does not report them, so a `passed` unit would claim
> the rest of the Then.

The scenario is project-migration's "One obligation renamed across changes":

```
- **THEN** both changes' specifications carry the identifier derived from the heading the earlier change established, each states the heading that change used, and this is not reported as a collision
```

**In both, the check ran, reached one clause of the Then, and found it to hold.** The reviewer's
only choices were `passed`, which claims the clauses the check did not reach, or no unit, which
loses the record of the clause it did. The review chose the second both times.

## 2. Two verification units kept as `passed` although they covered part of their scenario

`drafts/2026-08-13-content-address-runtime-assets.md`, line 138, under "Also for a reviewer":

> **V2 and V4 partly cover their THEN.** The record gives the rule id and the exit code. It does not
> show "reports the document, the reference" or "names both locations". Drop them if that is too thin.

The draft does not number its units. The two quoted clauses identify them, because each is the
Then of exactly one scenario a unit names. Both are staged as `passed`, in
`staged/htmlspec/changes/archive/2026-08-13-content-address-runtime-assets/specification.html`:

| Unit | `data-outcome` | Its scenario's Then | What the method paragraph admits |
| --- | --- | --- | --- |
| `verify-reference-names-a-fabricated-address` (line 365) | `passed` | "validation fails, reports the document, the reference, and that the bytes are absent" (line 219) | "The record gives the rule identifier and the exit code." |
| `verify-a-second-copy-of-one-library` (line 373) | `passed` | "validation fails and names both locations" (line 267) | "The record gives the rule identifier and the exit code." |

**The gap is recorded, and only as prose.** Each method paragraph says what the record lacks. The
projected model says `passed`, so a consumer reading the model, and the review warning that reads
it, sees a whole check.

## 3. What the staged tree holds, by outcome

Every staged document outside a change's `evidence/`, projected with the program's own projection:

```
$ cd ~/projects/html-spec-migration/staged
$ find htmlspec -name '*.html' -not -path '*/evidence/*' -print0 | sort -z \
    | xargs -0 ~/projects/html-spec-migration/tools/project-model/target/release/project-model > pm-staged.json
exit 0
```

Counted with a short Python reading of `verifications[].outcome` in that JSON:

```
not-run htmlspec/changes/archive/2026-08-10-html-spec/specification.html verify-print
blocked htmlspec/changes/archive/2026-08-22-accept-a-described-change/specification.html verify-described-change-run-2026-08-22-a-description-with-no-name
109 documents projected; 21 hold verifications; 94 units; {'passed': 92, 'not-run': 1, 'blocked': 1}
```

| Outcome | Units |
| --- | --- |
| `passed` | 92 |
| `not-run` | 1 |
| `blocked` | 1 |
| `failed` | 0 |
| **Total** | **94**, in 21 specifications |

This reproduces the expected counts. A plain `grep` over the whole staged tree finds 97 units and
95 `passed`. The three extra are all `passed`, and all sit in
`2026-08-10-html-spec/evidence/documents/deterministic-formatting.html`, a record preserved as it
was written in August rather than a migrated document.

**The two units in section 2 are among the 92 `passed`.** So at least 2 of 92 `passed` units
claim more than their record shows. The drafters flagged those two. This measurement does not
establish how many others do.

The drafting brief gave every drafter the same four values. `brief-draft.md`, lines 377-386:

```
  unit needs `data-outcome` set to what the record says happened (`passed`, `failed`, `blocked`, or
  `not-run`), a `data-refs`, and a `<p data-field="method">` stating how it was checked, as the
  record states it. When unsure whether a record supports one, write none, and say so in your
  report. A test in the code is not a record in the change and supports no verification unit.
  **Clarified after review:** a record need not use the requirement's heading. Write a
  verification unit where a carried record describes a check made against the built program (a
  run, a probe, a mutation run, a suite the record says was run and what it asserts) with its
  outcome, and that check exercises the behaviour a requirement or scenario states. Write none
  where the check only resembles the scenario, where the outcome would be inferred, or where the
  record predates the build.
```

## 4. Notes declining a verification because the check "only resembles the scenario"

```
$ grep -n -i "resembl" ~/projects/html-spec-migration/drafts/*.md
```

13 lines match, in 9 drafts. Two of them flag a unit that *was* written and leave it to review:
`accept-a-described-change` line 171 and `migrate-an-openspec-project` line 191. The other 11
decline a unit. `migrate-an-openspec-project` line 159 is a heading over three items. So the
drafts hold **13 declines, in 9 drafts**. `record-a-renamed-requirement` line 122 covers three
scenarios in one note.

The last column is my reading of each note against its scenario's Then, and it is a reading, not a
count the program made. "Part" means the note states both a clause of the Then the check reached
and one it did not. "Other" means the check reached no clause of the Then, because its condition,
subject, or trigger differed from the scenario's.

| Draft, line | Scenario declined | The note's own words for what was missing | Reading |
| --- | --- | --- | --- |
| conform-generated-frontmatter-names, 109 | A malformed name is caught before release | "Its trigger is the writer's invocation, not a workflow added with a malformed identity" | Other |
| conform-generated-frontmatter-names, 112 | Every selected tool receives something | "the run shows three tools receiving files. The scenario's substance is that receiving nothing is not representable, which a run cannot show" | Part |
| distinguish-command-names-from-skill-names, 121 | Generated instructions name their version | "the frontmatter checked is a command file's" | Other |
| accept-a-described-change, 136 | A description already carrying its scope | "the run's request states no exclusion" | Other |
| catch-a-templates-placeholder-prose, 117 | A template that rewords its placeholder | "no rewording was tested" | Other |
| migrate-an-openspec-project, 159 | Every entity has a disposition | "the gate did not check destinations at archival" | Part |
| migrate-an-openspec-project, 159 | A source change with no version | "the survey reported none, but no migrated archive or plan was checked" | Other |
| migrate-an-openspec-project, 159 | A unit kind the source format has none of | "0 verification units were produced, but no record says the plan recorded why" | Part |
| establish-a-runbook, 157 | A runbook validated by the ordinary path | "Zero diagnostics does not show which rules ran" | Part |
| record-a-renamed-requirement, 122 | three valid-case scenarios | "assert only that the rules do not fire, which resembles those scenarios without matching them (they also state what a consumer reads)" | Part |
| log-a-runbook-run, 138 | a run leaving a record | "checks an accessor's path relationship, not a run leaving a record" | Other |
| log-a-runbook-run, 147 | A runbook that states nothing about its runs | "a runbook with no specification could not be started, not a conformance report" | Other |
| confine-the-asset-check-to-the-project, 118 | A project beside a fixture that carries its own store | "Its Given needs a fixture store holding bytes identical to the project's" | Other |

**Five read as a check that reached part of its scenario, and eight as a check of something
else.** The brief's rule sent both kinds to the same place, no unit. A value for "reached part of
it" would give the five a truthful record. It would not fit the eight, and they stay correctly
unrecorded.

## 5. What the contract accepts today: a scratch project

```
$ H=target/debug/htmlspec
$ S=…/scratchpad/record-a-partial-verification
$ $H init --tools none --root $S/proj                     # exit 0
$ cd $S/proj && $H new change measure-partial             # exit 0
$ $H new document specification htmlspec/changes/measure-partial/specification.html \
    --subject "A partial verification, measured"          # exit 0
```

Each variant below is that specification with its seven template identifiers renamed and its
masthead placeholder replaced, so that `review/example-content` does not mix into the results.
The renames are `req-example` → `req-a-check-covers-its-scenario`, `scenario-example` →
`scenario-the-check-runs`, `decision-example` → `decision-measure-on-head`, `risk-example` →
`risk-the-measure-misleads`, `evidence-example` → `evidence-the-head-binary`, `figure-example` →
`figure-the-pipeline`, and `verify-example` → `verify-the-check-reached-part`. Each variant is
written as `htmlspec/changes/v-<name>/specification.html`, at the depth its relative asset paths
were written for, by a short script (`variants.py`, appendix A)
that makes only the edits the table names.
The requirement is `req-a-check-covers-its-scenario`, marked `added`. The verification is
`verify-the-check-reached-part`, which names that requirement. The template's evidence unit also
names the requirement, which is why every row carries one `review/stale-reference` on
`evidence-the-head-binary`. That warning is omitted from the Diagnostics column.

The `unchecked` paragraph in c and d is
`<p data-field="unchecked">The Then names both locations, and the check recorded only the rule and the exit status.</p>`.

| Variant | `data-outcome` | Other edit | Exit | Diagnostics on the verification or requirement |
| --- | --- | --- | --- | --- |
| a | `not-run` | none | 0 | `review/stale-reference` |
| **b** | **`partial`** | none | **1** | **`schema/invalid`**, `review/stale-verification` |
| c | `partial` | an `unchecked` paragraph | 1 | `schema/projection`, `schema/invalid`, `review/stale-verification` |
| d | `passed` | an `unchecked` paragraph | 1 | `schema/projection`, `review/stale-verification` |
| **e** | **`failed`** | none | **0** | `review/stale-verification`, and **no `review/unverified-requirement`** |
| **f** | none | the verification removed | **0** | **`review/unverified-requirement`** |
| g | `failed` | requirement marked `modified` | 0 | `review/stale-verification` |
| h | `not-run` | requirement marked `modified` | 0 | `review/stale-reference` |
| i | `notrun` (a typo) | none | 1 | `schema/invalid`, `review/stale-verification` |
| j | `not-run` | a second `method` paragraph | 0 | `review/stale-reference` |

**Variant b: `partial` is rejected as a value outside the permitted set.** The rule is
`schema/invalid`, at the outcome's schema location. From `--json validate`:

```json
{
  "rule": "schema/invalid",
  "namespace": "schema",
  "severity": "error",
  "message": "\"partial\" is not one of \"passed\", \"failed\" or 2 other candidates",
  "unitId": "verify-the-check-reached-part",
  "schemaPath": "/verifications/0/outcome"
}
```

**Variants c and d: no paragraph can state what a check missed.** `data-field="unchecked"` is a
projection fault on any outcome:

```
  error: schema/projection at verify-the-check-reached-part: data-field="unchecked" names no field of this unit; it accepts "method", "note"
```

The projected model of variant c keeps the outcome as written and drops the paragraph:

```json
[{"change": "added", "id": "verify-the-check-reached-part",
  "method": "State how the requirement was checked, precisely enough that someone else could repeat it.",
  "outcome": "partial",
  "references": [{"origin": "attribute", "target": "req-a-check-covers-its-scenario"}],
  "title": "The example verification verify-the-check-reached-part", "unit": "verification"}]
```

**Variants e and f: `failed` clears the unverified warning today.** The same specification with
no verification names the requirement and draws `review/unverified-requirement`. With a `failed`
verification naming it, the warning is gone. Variant a shows the same for `not-run`. The rule
reads whether a verification names the requirement and never reads its outcome
(`crates/htmlspec-validate/src/rules/review.rs`, `requirements_are_verified`, lines 132-165, whose
check is at line 148).

**Variants g and h: which staleness warning a verification draws depends on whether it ran.** A
`failed` verification against a `modified` requirement draws `review/stale-verification`. A
`not-run` one draws `review/stale-reference`. `has_run` (`review.rs`, line 315) counts every value
except the schema's "not run" value as having run.

## 6. Two results this change did not set out to find

**`partial` already draws `review/stale-verification` on HEAD** (variant b), although it is not a
permitted value. So does the typo `notrun` (variant i). `review.rs`'s `has_run` asks only whether
the value is the not-run one, so a value outside the enumeration counts as having run. The CLI's
own `has_run`, which decides whether a run's record recorded an outcome
(`crates/htmlspec-cli/src/runs.rs`, line 1184), was fixed for exactly this. Its comment records
that "`data-outcome="notrun"`, a typo, reported the run as conforming", and it now returns `false`
for any value the schema does not enumerate. The two functions answer the same question two ways.
For `partial` the difference ends when the schema enumerates it. For a typo it remains, in a
document that already fails `schema/invalid`.

**A second single-valued paragraph is dropped without a diagnostic** (variant j). The document is
accepted, and the projected `method` is the first paragraph alone. `Paragraphs::single`
(`crates/htmlspec-parse/src/project.rs`, line 408) documents that "the caller reports the
repetition", and no caller does: `project_task`, `project_risk`, and `project_verification` each
call it and report nothing. For a field that states what a check did not cover, a dropped paragraph
would understate the gap.

## 7. Where else the outcome set is read

Read from the files, not run:

| Reader | What it does with the outcome | With `partial` enumerated |
| --- | --- | --- |
| `review.rs` `requirements_are_verified` (132-165) | nothing: any verification naming the requirement clears the warning | clears it, as `failed` does |
| `review.rs` `has_run` (315-322) | stale-verification for every value but the not-run one | `review/stale-verification`, as now |
| `runs.rs` `has_run` (1184-1208) | a run recorded an outcome if the value is enumerated and is not the not-run one | a recorded outcome, so `review/unrecorded-requirement` clears |
| `runs.rs` record composition (1596-1614) | writes `passed` for a machine check preflight passed, and `not-run` for every other requirement | never writes `partial` |
| `assets/sha256-aac8967609f80d9f/htmlspec.css` (698-724) | one `content` rule per outcome value, and no `attr()`, because "it would render an invalid value as though the vocabulary contained it" | no rule, so no label |
| `conformance/tests/presentation.rs` `every_enumerated_state_value_has_a_label` (239) | fails for any enumerated value the stylesheet does not label | fails until the stylesheet labels `partial` |

No accepted requirement lists the outcome values. They appear only in
`crates/htmlspec-schema/schema/document.schema.json`, lines 595-603, and in the stylesheet.

The script that section 5 names is reproduced below, so the measurement does not depend on the scratch directory surviving.

## Appendix A: `variants.py`

```python
"""Writes one specification per variant into the scratch change, from the template `new document` wrote.

Each variant differs from the base only in the verification unit (and, for the staleness pair,
the requirement's change state). The base replaces every template identifier and the masthead
placeholder, so no `review/example-content` warning mixes into the measurement.
"""
import pathlib, sys

change = pathlib.Path(sys.argv[1])
template = (change / "specification.html").read_text()

renames = {
    "req-example": "req-a-check-covers-its-scenario",
    "scenario-example": "scenario-the-check-runs",
    "decision-example": "decision-measure-on-head",
    "risk-example": "risk-the-measure-misleads",
    "evidence-example": "evidence-the-head-binary",
    "figure-example": "figure-the-pipeline",
    "verify-example": "verify-the-check-reached-part",
}
base = template
for old, new in renames.items():
    base = base.replace(old, new)
base = base.replace(
    "Replace this paragraph with what this document is for. It projects into the document's summary, so the argument the change rests on lives in the model rather than in markup the model cannot address.",
    "Measures what the contract at HEAD accepts as a verification's outcome.",
)

OPEN = '<section class="verification" id="verify-the-check-reached-part" data-change="added" data-outcome="not-run" data-refs="req-a-check-covers-its-scenario">'
METHOD = '<p data-field="method">State how the requirement was checked, precisely enough that someone else could repeat it.</p>'
UNCHECKED = '<p data-field="unchecked">The Then names both locations, and the check recorded only the rule and the exit status.</p>'
assert OPEN in base and METHOD in base

def outcome(value, extra=""):
    doc = base.replace(OPEN, OPEN.replace('data-outcome="not-run"', f'data-outcome="{value}"'))
    return doc.replace(METHOD, METHOD + ("\n            " + extra if extra else ""))

def without_verification(doc):
    start = doc.index('          <section class="verification"')
    end = doc.index("</section>", start) + len("</section>\n")
    return doc[:start] + doc[end:]

variants = {
    "a-not-run": outcome("not-run"),
    "b-partial": outcome("partial"),
    "c-partial-with-unchecked": outcome("partial", UNCHECKED),
    "d-passed-with-unchecked": outcome("passed", UNCHECKED),
    "e-failed": outcome("failed"),
    "f-no-verification": without_verification(outcome("not-run")),
    "g-modified-failed": outcome("failed").replace(
        'id="req-a-check-covers-its-scenario" data-change="added"',
        'id="req-a-check-covers-its-scenario" data-change="modified"'),
    "h-modified-not-run": outcome("not-run").replace(
        'id="req-a-check-covers-its-scenario" data-change="added"',
        'id="req-a-check-covers-its-scenario" data-change="modified"'),
}
# Each variant is the specification of its own change directory, at the depth the template's
# relative asset paths were written for.
for name, doc in variants.items():
    target = change.parent / f"v-{name}" / "specification.html"
    target.parent.mkdir(exist_ok=True)
    target.write_text(doc)
    print(target)
```
