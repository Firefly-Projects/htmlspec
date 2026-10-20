# The measurement, repeated before anything changed

**Date:** 2026-10-02
**Tasks:** 1.1 to 1.4
**Binary:** `htmlspec 0.19.0 (6fd228eb392d-dirty)`, built from `6fd228e`, the merge of #40. The
`-dirty` mark is this change's own proposal and design, corrected before the measurement ran (see
"Before the measurement"), and this file. No compiled source differs from HEAD. Also
`openspec 1.13.1`, from `/nix/store/9q2ajjlnbnglbbdbxxm4dp1ypk89j7i3-openspec-1.13.1`, the same
build as on 2026-09-30.
**Projection:** `~/projects/html-spec-migration/tools/project-model`, rebuilt against this tree.

The scripts are the appendices of `the-measurement-2026-09-30.md`, run unchanged except that each
writes into a fresh `rerun/` directory beside the first run's. The one script the first run describes
without reproducing, the control in section 4, is reproduced in the appendix here.

## Before the measurement

The proposal and the design each said the contract version stays `0.2.0`. They were written before
`reject-change-markup-inside-a-unit` archived as `0.19.0` and moved the contract to `0.3.0`. Both now
say it stays `0.3.0`. No other planning text changed.

The change's one MODIFIED block, "Every synthesis and every contract update is agent-driven", was
diffed against `openspec/specs/artifact-kinds/spec.md` at HEAD. It differs only in the partial-updates
paragraph and in the outcome of "Merging a scenario into an existing requirement", which are the
intended edits. `openspec validate state-a-modified-requirement-whole --strict` reports the change
valid.

## 1.1 The archive count is unchanged

`count.py` writes one JSON row per block and exits 0. Summarized from `rerun/count.json`, in the first
run's terms:

```
34 blocks, 19 changes
whole: 31    omits accepted content: 3    no accepted requirement at the time: 0
merged text equals the delta: 33 of 34
blocks where >= 1 accepted paragraph is not verbatim in the delta: 13, and in all 13 the merged text equals the delta
accepted paragraphs the delta left out and the merge kept: 0
scenario omissions: 4 (kept by the merge: 2, withdrawn by the merge: 2)
```

`rerun/count.json` equals the first run's `count.json` row for row. The three blocks that omit a
scenario are the same three, with the same merge commits:

| Change | Capability | Omitted | Merge |
| --- | --- | --- | --- |
| `2026-08-14-separate-artifact-kinds` | toolchain-packaging | "Unknown subcommand", "Discoverable help", both kept | `9c3a3c7` |
| `2026-08-14-separate-artifact-kinds` | document-template | "Status visible to a reviewer", withdrawn | `9c3a3c7` |
| `2026-08-21-distinguish-command-names-from-skill-names` | project-bootstrap | "One identity per workflow", withdrawn | `c864727` |

**No row is added.** One change has archived since 2026-09-30:
`2026-10-02-reject-change-markup-inside-a-unit`. Its one delta, for document-validation, holds only
`## ADDED Requirements`, so it modifies no requirement. The archive now holds 25 changes.

## 1.2 OpenSpec 1.13.1 still refuses all three

`openspec --version` prints `1.13.1`. Appendix B of the first measurement, rerun:

```
=== separate-artifact-kinds / toolchain-packaging (accepted at 9c3a3c7^, delta at 9c3a3c7)
✗ [ERROR] toolchain-packaging/spec.md: MODIFIED "A single command exposes validate, format, and init" omits scenario(s) the current spec still has: "Unknown subcommand", "Discoverable help". Copy them into the MODIFIED block (a MODIFIED requirement replaces the whole block, so archive refuses to drop them).
validate exit 1
toolchain-packaging MODIFIED failed for header "### Requirement: A single command exposes validate, format, and init" - current spec contains scenario(s) not present in the modified block: "Unknown subcommand", "Discoverable help". Refresh the change spec before archiving to avoid dropping scenarios.
Aborted. No files were changed.
archive exit 1
=== separate-artifact-kinds-dt / document-template (accepted at 9c3a3c7^, delta at 9c3a3c7)
✗ [ERROR] document-template/spec.md: MODIFIED "A document declares its identity and provenance" omits scenario(s) the current spec still has: "Status visible to a reviewer". …
validate exit 1
document-template MODIFIED failed for header "### Requirement: A document declares its identity and provenance" - current spec contains scenario(s) not present in the modified block: "Status visible to a reviewer". …
Aborted. No files were changed.
archive exit 1
=== distinguish-command-names-from-skill-names / project-bootstrap (accepted at c864727^, delta at 3b1ca92)
✗ [ERROR] project-bootstrap/spec.md: MODIFIED "Bootstrapping installs agent instructions for the tools a project uses" omits scenario(s) the current spec still has: "One identity per workflow". …
validate exit 1
project-bootstrap MODIFIED failed for header "### Requirement: Bootstrapping installs agent instructions for the tools a project uses" - current spec contains scenario(s) not present in the modified block: "One identity per workflow". …
Aborted. No files were changed.
archive exit 1
```

The control, which restates the two kept scenarios in the first delta (appendix):

```
Change 'separate-artifact-kinds' is valid
validate exit 0
Specs updated successfully.
Change 'separate-artifact-kinds' archived as '2026-10-02-separate-artifact-kinds'.
archive exit 0
diff against 9c3a3c7's accepted block: exit 0
```

**Stated whole, the delta still produces the author's merge byte for byte.** The block extracted at
`9c3a3c7` is also byte-identical to the first run's `after.md`.

## 1.3 html-spec still cannot tell the two forms apart

Appendix C, rerun with the rebuilt binary:

| Document | `validate` | Scenarios projected for `req-x` |
| --- | --- | --- |
| `htmlspec/specs/cap/spec.html` | exit 0, 0 errors, 1 warning | `unchanged` requirement: A, B, C, all `unchanged` |
| `changes/partial/specification.html` | exit 0, 0 errors, 1 warning | `modified` requirement: D `added` |
| `changes/whole/specification.html` | exit 0, 0 errors, 1 warning | `modified` requirement: A `unchanged`, B `unchanged`, C `removed`, D `added` |

The warning on each is `review/unverified-requirement`, read from `--json validate`.

```
$ htmlspec --json sync --change partial
{ "absent": [], "change": "partial", "reached": ["req-x"], "stated": ["req-x"], "untakenRenames": [] }
exit 0
$ htmlspec --json sync --change whole
{ "absent": [], "change": "whole", "reached": ["req-x"], "stated": ["req-x"], "untakenRenames": [] }
exit 0
```

Every result matches section 5 of the first measurement.

## 1.4 The list of instructions is complete, with one sentence added

`grep -n -i "partial\|restate\|wholesale\|modified"` over `crates/htmlspec-assets/instructions/*.md`
and `assets/template-specification.html` finds 19 lines. Each was read in context:

| Hit | What it says | In section 6? |
| --- | --- | --- |
| `sync.md` 105, 111 | "Apply partial updates" and "Never copy … wholesale" | yes, lines 105-114 |
| `sync.md` 134-135 | the rename step's "as a partial update like any other" | yes |
| `archive.md` 260, 264, 269, 272 | step 5's partial paragraph, its quotation, and two bullets | yes, lines 260-273 |
| `archive.md` 167, 175 | step 2's renamed-requirement example and its first bullet | yes, lines 162-185 |
| `archive.md` 276 | step 5's rename bullet: "Where the change also modified it, apply the modification to the same requirement" | **no** |
| `archive.md` 125 | "a restatement of intent", about the specification as a whole | not about a modified requirement |
| `archive.md` 372 | the report names "what you added, modified, or removed" | not about how to state or merge |
| `continue.md` 69, 85, `ff.md` 108 | "restate" in the sense of repeating a proposal or a workflow | not about a requirement |
| `migrate.md` 85 | the `## RENAMED Requirements` row takes the state `modified` where the delta also restates the requirement | about a source delta's state, not its completeness |
| `migrate.md` 212, 238 | identifier derivation for "a later change that restates `C`" | about identifiers |

**The one sentence section 6 left out** is `archive.md` line 276, in step 5's bullet about a renamed
requirement. It tells an agent to apply a modification "to the same requirement" and points to the
`sync` instructions for the procedure. It needs no edit. Under the new rule, applying the modification
means applying it as the change states it, and the `sync` section this change adds says how. Task 3.5
keeps the bullet, because it replaces only the paragraph, the quotation, and the two bullets named.

**Two passages the `grep` does not reach** were read because they sit in the procedure a modified
requirement passes through:

- `sync.md` "Merging one requirement", step 3: "Decide whether it is a new requirement or an addition
  to one already accepted. A change often sharpens a requirement that exists rather than adding a fresh
  one." It works on identifiers in `absent`, which a modified requirement is never in. It is not edited
  here. The question it raises is recorded in `what-the-implementation-revealed`.
- `sync.md` "What to report", first bullet: "whether you added a requirement or folded a scenario into
  one already there". Task 3.2 adds the modified-requirement reports beside it.

`assets/template-specification.html` has no hit. Its one example requirement is `data-change="added"`.

## Appendix: `control.sh`

```bash
#!/usr/bin/env bash
# The control for appendix B: restate the two kept scenarios in the separate-artifact-kinds delta,
# then validate and archive it, and diff the merged requirement with the block at 9c3a3c7.
set -u
REPO=.
R=$(dirname "$0")
P=$R/os/separate-artifact-kinds
HEADING='### Requirement: A single command exposes validate, format, and init'
block() {  # print one requirement block, up to the next requirement or section heading
  awk -v h="$HEADING" '$0==h{on=1; print; next} on && (/^### Requirement:/ || /^## /){exit} on{print}'
}
git -C $REPO show 9c3a3c7^:openspec/specs/toolchain-packaging/spec.md | block > "$R/before.md"
git -C $REPO show 9c3a3c7:openspec/specs/toolchain-packaging/spec.md | block > "$R/after.md"
DELTA=$P/openspec/changes/separate-artifact-kinds/specs/toolchain-packaging/spec.md
python3 - "$R/before.md" "$DELTA" <<'PY'
import sys
before, delta = sys.argv[1], sys.argv[2]
kept = open(before).read()
kept = kept[kept.index("#### Scenario: Unknown subcommand"):]
t = open(delta).read()
i = t.index("## ADDED Requirements")
open(delta, "w").write(t[:i] + kept.rstrip("\n") + "\n\n" + t[i:])
PY
(cd "$P" && openspec validate separate-artifact-kinds --strict 2>&1; echo "validate exit $?")
(cd "$P" && openspec archive separate-artifact-kinds --yes 2>&1 | tail -2; echo "archive exit ${PIPESTATUS[0]}")
block < "$P/openspec/specs/toolchain-packaging/spec.md" > "$R/openspec-after.md"
diff "$R/openspec-after.md" "$R/after.md"; echo "diff against 9c3a3c7's accepted block: exit $?"
```

It runs after appendix B, against the scratch project appendix B leaves for the first case.
