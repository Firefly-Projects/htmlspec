# The measurement, repeated before anything changed

**Date:** 2026-10-03
**Tasks:** 1.1 to 1.3
**Binary:** `htmlspec 0.21.0 (913a87dbaa51-dirty)`, built from `913a87d`, the merge of #42. The `-dirty`
mark is this change's own design and tasks, corrected before the measurement ran (see "Before the
measurement"), and this file. No compiled source differs from HEAD.

## Before the measurement

The design's Migration Plan and task 6.2 said the contract version stays `0.2.0`. They were written
before `reject-change-markup-inside-a-unit` moved it to `0.3.0`. Both now say `0.3.0`.

Each of this change's MODIFIED blocks was diffed against the accepted specifications at HEAD. Three
changes have archived since they were written: `reject-change-markup-inside-a-unit` (document-validation,
an added requirement only), `state-a-modified-requirement-whole` (artifact-kinds), and
`see-a-modification-the-merge-did-not-take` (four other toolchain-packaging requirements). None of them
touched a requirement this change modifies, and every block differs from the accepted text only by
this change's own edits. `openspec validate resolve-a-reference-across-its-change --strict` reports the
change valid.

## 1.1 Steps 1 and 2, and the plain link, reproduce

`measure.sh`, in the appendix, rebuilds the scratch project from an empty directory with
`htmlspec init --tools none`, `new change`, and `new document specification`.

| Step | 2026-09-30 | 2026-10-03 |
| --- | --- | --- |
| 1. A task names its change specification's `req-example`. `validate` over the project | exit 1, `4 document(s): 1 error(s), 15 warning(s)`, `refs/unresolved at task-example: this task references req-example, and no unit in the document declares that identifier` | the same, word for word |
| 1. The tasks document named alone | exit 1, `1 error, 3 warnings` | the same |
| 2. A task names `req-held-by-the-accepted-contract`, which only `htmlspec/specs/measure/spec.html` declares | exit 1, `5 document(s): 1 error(s), 16 warning(s)`, the same `refs/unresolved` | the same |
| The workaround: plain links to `specification.html#req-example` and `#req-does-not-exist` | exit 0, `0 errors, 3 warnings`. No reference projected, and `links` holds only `hs-content` and `hs-tasks` | the same |

**The template's advice still produces an error, and a link to a unit that does not exist is still
accepted.**

## 1.2 The count is unchanged

The count script from the evidence, run from `~/projects/html-spec-migration/staged/htmlspec`:

```
task: 54 units, 66 links, 15 changes
decision: 2 units, 2 links, 2 changes
  to its own change's specification: 31 units, 36 links
  to an accepted specification: 23 units, 30 links
  to another change's specification: 2 units, 2 links
  links whose target does not exist: 0
  target change states: {'added': 24, 'unchanged': 31, 'modified': 12, 'removed': 1}
  links to an accepted specification whose identifier the change's own specification also declares: 2
data-refs entries naming an identifier their own document does not declare: 0
identifiers declared by more than one accepted specification: ['scenario-a-tasks-document-carrying-other-units', 'scenario-reference-to-a-removed-requirement']
```

Every line equals the first run's. **No design decision rests on a number that moved.**

## 1.3 The baseline for task 5.2

```
$ cd ~/projects/html-spec-migration/staged && time htmlspec --json validate
accepted: True   documents: 109   errors: 0   warnings: 781
review/stale-reference: 317   review/stale-verification: 193   review/unverified-requirement: 271
exit 0
real 19.47   user 19.15   sys 0.23
```

The counts equal the evidence's exactly, and the time is within a few percent of its 19.2 seconds.

## Appendix: `measure.sh`

```bash
#!/usr/bin/env bash
# Rebuilds steps 1 and 2 of the-measurement-2026-09-30.md and its plain-link workaround, from an
# empty directory, and prints validate's exit status and refs/* diagnostics for each.
set -u
BIN=target/debug/htmlspec
S=$(cd "$(dirname "$0")" && pwd)
P=$S/proj
rm -rf "$P"; mkdir -p "$P"; cd "$P"
"$BIN" --version
"$BIN" init --tools none --root "$P" >/dev/null 2>&1; echo "init exit $?"
"$BIN" new change measure-refs >/dev/null 2>&1; echo "new change exit $?"
"$BIN" new document specification htmlspec/changes/measure-refs/specification.html --subject "measure-refs: specification" >/dev/null 2>&1; echo "new specification exit $?"
T=htmlspec/changes/measure-refs/tasks.html
cp $T $S/tasks.orig.html
refs() {  # the refs/* diagnostics and the exit status of validating $1, or the project
  "$BIN" validate "$@" > out.txt 2>&1; code=$?
  grep "refs/" out.txt | sed 's/^ *//'; tail -1 out.txt; echo "exit $code"
}
echo "=== step 1: a task references its change specification's req-example"
sed 's/<li class="task" id="task-example" data-change="added" data-state="todo">/<li class="task" id="task-example" data-change="added" data-state="todo" data-refs="req-example">/' $S/tasks.orig.html > $T
grep -c 'data-refs="req-example"' $T
refs
echo "--- naming the one document"
refs $T
echo "=== step 2: a task references a requirement only an accepted specification holds"
mkdir -p htmlspec/specs/measure
"$BIN" new document specification htmlspec/specs/measure/spec.html --subject "measure: accepted specification" >/dev/null 2>&1
sed -i '' -e 's/-example/-held-by-the-accepted-contract/g' -e 's/ data-change="added"//g' htmlspec/specs/measure/spec.html
sed 's/<li class="task" id="task-example" data-change="added" data-state="todo">/<li class="task" id="task-example" data-change="added" data-state="todo" data-refs="req-held-by-the-accepted-contract">/' $S/tasks.orig.html > $T
refs
echo "=== the workaround: plain links to another document"
python3 - "$S/tasks.orig.html" "$T" <<'PY'
import re, sys
t = open(sys.argv[1]).read()
t2 = re.sub(r'(<li class="task" id="task-example"[^>]*>.*?<p data-field="note">)',
            lambda m: m.group(1) + 'Serves <a href="specification.html#req-example">the requirement</a> and '
            '<a href="specification.html#req-does-not-exist">one that does not exist</a>. ', t, count=1, flags=re.S)
assert t2 != t
open(sys.argv[2], "w").write(t2)
PY
refs $T
~/projects/html-spec-migration/tools/project-model/target/release/project-model $T | python3 -c '
import json,sys
m=list(json.load(sys.stdin).values())[0]
t=[x for x in m["tasks"] if x.get("id")=="task-example"][0]
print("task-example refs:", t.get("refs", []), "| links:", [l["target"] for l in m.get("links", [])])'
```
