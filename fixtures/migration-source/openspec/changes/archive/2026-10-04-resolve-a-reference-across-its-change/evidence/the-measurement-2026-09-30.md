# The template's advice followed, and the links drafted instead

**Date:** 2026-09-30
**Tasks:** 1.1, 1.2, and 1.3
**Binary:** `htmlspec 0.18.0 (6a591bdf9d92-dirty)`. It is the build of HEAD `6a591bdf9d92`. The
`-dirty` suffix comes from untracked change directories under `openspec/changes/`:
`git status --porcelain` lists only `??` entries, and no tracked file differs from HEAD. Every
`htmlspec` below is this repository's `./target/debug/htmlspec`, called by its absolute path. The
`htmlspec` on `PATH` is an older build and was not used.

## The scratch project

```
htmlspec init --tools none --root <scratch>/proj
cd <scratch>/proj
htmlspec new change measure-refs
htmlspec new document specification htmlspec/changes/measure-refs/specification.html \
  --subject "measure-refs: specification"
```

Every command exited 0. The scratch project is not inside a git repository. `new change` wrote
`proposal.html`, `design.html`, and `tasks.html` and said "The specification is produced at
archival, from the completed change", so the specification was created with `new document`. The
specification template declares `req-example`, with `data-change="added"`.

The tasks template writes this note into `task-example`:

> State the work in a sentence someone else could pick up. A task may reference the requirement it
> serves with `data-refs`, once that requirement exists in this change's specification.

Before any edit, `htmlspec validate` exited 0: `4 document(s): 0 error(s), 15 warning(s)`, all of
them `review/example-content` or `review/stale-reference`.

In the outputs below, paths are shortened to be relative to the scratch project, and the
`review/example-content` warnings are left out. Every document carries them because it still holds
template content, and none of them bears on references.

## Step 1: a task references a requirement in its change's specification

The one edit, to `htmlspec/changes/measure-refs/tasks.html`:

```diff
-            <li class="task" id="task-example" data-change="added" data-state="todo">
+            <li class="task" id="task-example" data-change="added" data-state="todo" data-refs="req-example">
```

`req-example` exists in the change's specification. This follows the template's advice exactly.

```
$ htmlspec validate
htmlspec/changes/measure-refs/design.html
  0 errors, 1 warning: the document is accepted
htmlspec/changes/measure-refs/proposal.html
  0 errors, 1 warning: the document is accepted
htmlspec/changes/measure-refs/specification.html
  warning: review/stale-reference at evidence-example: this evidence references req-example, which this change marks added. Reconcile it when convenient; work continues either way
  warning: review/stale-reference at verify-example: this verification references req-example, which this change marks added. Reconcile it when convenient; work continues either way
  0 errors, 10 warnings: the document is accepted
htmlspec/changes/measure-refs/tasks.html
  error: refs/unresolved at task-example: this task references req-example, and no unit in the document declares that identifier. A reference that resolves to nothing records a relationship to a thing that does not exist
  1 error, 3 warnings: validation failed

4 document(s): 1 error(s), 15 warning(s)
exit 1
```

Naming the one document gives the same error and the same exit status:

```
$ htmlspec validate htmlspec/changes/measure-refs/tasks.html
  error: refs/unresolved at task-example: this task references req-example, and no unit in the document declares that identifier. ...
  1 error, 3 warnings: validation failed
exit 1
```

With `--json`, the diagnostic is:

```json
{
  "rule": "refs/unresolved",
  "namespace": "refs",
  "severity": "error",
  "message": "this task references req-example, and no unit in the document declares that identifier. A reference that resolves to nothing records a relationship to a thing that does not exist",
  "unitId": "task-example"
}
```

`accepted` is `false`. Exit status 1 is the document-violation code (`crates/htmlspec-cli/src/exit.rs`).

**The template's advice produces an error.**

## Step 2: a task references a requirement only an accepted specification holds

```
mkdir -p htmlspec/specs/measure
htmlspec new document specification htmlspec/specs/measure/spec.html --subject "measure: accepted specification"
```

In that file every `*-example` identifier was renamed to `*-held-by-the-accepted-contract`, and
every `data-change="added"` was removed, so the accepted specification declares
`req-held-by-the-accepted-contract` as an unchanged requirement. No document in the change
declares that identifier. `tasks.html` then carried:

```diff
-            <li class="task" id="task-example" data-change="added" data-state="todo">
+            <li class="task" id="task-example" data-change="added" data-state="todo" data-refs="req-held-by-the-accepted-contract">
```

```
$ htmlspec validate
htmlspec/changes/measure-refs/design.html
  0 errors, 1 warning: the document is accepted
htmlspec/changes/measure-refs/proposal.html
  0 errors, 1 warning: the document is accepted
htmlspec/changes/measure-refs/specification.html
  warning: review/stale-reference at evidence-example: ...
  warning: review/stale-reference at verify-example: ...
  0 errors, 10 warnings: the document is accepted
htmlspec/changes/measure-refs/tasks.html
  error: refs/unresolved at task-example: this task references req-held-by-the-accepted-contract, and no unit in the document declares that identifier. A reference that resolves to nothing records a relationship to a thing that does not exist
  1 error, 3 warnings: validation failed
htmlspec/specs/measure/spec.html
  0 errors, 1 warning: the document is accepted

5 document(s): 1 error(s), 16 warning(s)
exit 1
```

**A requirement held by an accepted specification does not resolve either.** document-schema's
"Cross-references address units by identifier" says a reference "SHALL resolve to a unit that exists
in the same document or in an accepted spec reachable from it". The second half has no
implementation.

## The workaround: a link to another document is never checked

Starting again from the template's `tasks.html`, the task's note gained two links, one to a unit
that exists and one to a unit that does not:

```html
<p data-field="note">Serves <a href="specification.html#req-example">the requirement</a> and
<a href="specification.html#req-does-not-exist">one that does not exist</a>. State the work ...</p>
```

```
$ htmlspec validate htmlspec/changes/measure-refs/tasks.html
  0 errors, 3 warnings: the document is accepted
exit 0
```

The program's projection, printed by
`~/projects/html-spec-migration/tools/project-model/target/release/project-model`, holds no
reference for `task-example`, and its `links` are only the page's two same-document fragments:

```json
"links": [{"target": "hs-content", "text": "Skip to content"}, {"target": "hs-tasks", "text": "Tasks"}],
"tasks": [{"change": "added", "id": "task-example", "notes": ["Serves the requirement and one that does not exist. ..."], "state": "todo", ...}]
```

**A link to a unit in another document enters neither the references nor the links, so validation
accepts one that points at nothing.** This matches the projection specification, section 5.1: a
reference comes from `data-refs` or from "an anchor whose `href` is a same-document fragment".

## Step 3: the workarounds in the staged migration

The staged migration is at `~/projects/html-spec-migration/staged/`. It holds 13 accepted
specifications and 24 changes. Every change is archived, under
`htmlspec/changes/archive/`, and none is active.

### What the drafters were told

The drafting brief, `~/projects/html-spec-migration/brief-draft.md`:

- lines 241-242: "Links are relative, and cross-document links are not validated, so get them right:
  the review checks every one against the staged tree."
- lines 264-265: "Never give a unit `data-refs` naming a unit in another document: references
  resolve within one document only, and it is an error."
- line 316: "A task naming a requirement may link to it in `specification.html`. Give it no
  `data-refs`."

### The count

A link is counted when all of these hold:

- it is an `<a href>` inside a `task` or `decision` unit;
- its `href` contains `specification.html` or a path under `specs/`;
- its fragment matches the schema's `unitId` grammar.

The count was made twice. The first method was a script using Python's `html.parser`, which tracks
the innermost unit around each link. The second was the script below. Both gave the same numbers.
The second is recorded because it is short enough to re-run as written. It is run from
`staged/htmlspec`, and it relies on the documents being canonically formatted: a unit's element
closes at the first line that holds its closing tag at the same indentation.

```python
import collections, glob, os, re

UNIT = re.compile(r'^(\s*)<(\w+) class="(task|decision)" id="([^"]+)"')
LINK = re.compile(r'href="([^"#]*(?:specification\.html|specs/)[^"#]*)#'
                  r'((?:req|scenario|decision|task|evidence|risk|verify)-[a-z0-9-]+)"')
REFS = re.compile(r'<\w+ class="[^"]*" id="([^"]+)"[^>]*data-refs="([^"]+)"')
IDS = re.compile(r' id="([^"]+)"')

def state_of(path, uid):
    m = re.search(r'id="%s"[^>]*' % re.escape(uid), open(path).read())
    if not m:
        return None
    s = re.search(r'data-change="([^"]+)"', m.group(0))
    return s.group(1) if s else "unchanged"

rows = []
for doc in sorted(glob.glob("changes/archive/*/*.html")):
    lines = open(doc).read().split("\n")
    i = 0
    while i < len(lines):
        m = UNIT.match(lines[i])
        if m:
            indent, tag, kind, uid = m.groups()
            j = i + 1
            while j < len(lines) and not lines[j].startswith(indent + "</" + tag + ">"):
                j += 1
            for target, frag in LINK.findall("\n".join(lines[i:j + 1])):
                resolved = os.path.normpath(os.path.join(os.path.dirname(doc), target))
                if resolved.startswith("specs/"):
                    tier = "an accepted specification"
                elif os.path.dirname(resolved) == os.path.dirname(doc):
                    tier = "its own change's specification"
                else:
                    tier = "another change's specification"
                own = os.path.join(os.path.dirname(doc), "specification.html")
                rows.append(dict(change=doc.split("/")[2], doc=doc, unit=uid, kind=kind,
                                 tier=tier, state=state_of(resolved, frag),
                                 shadowed=tier == "an accepted specification"
                                 and state_of(own, frag) is not None))
            i = j
        i += 1

units = lambda rs: {(r["doc"], r["unit"]) for r in rs}
for kind in ("task", "decision"):
    rs = [r for r in rows if r["kind"] == kind]
    print(f"{kind}: {len(units(rs))} units, {len(rs)} links, {len({r['change'] for r in rs})} changes")
for tier in ("its own change's specification", "an accepted specification",
             "another change's specification"):
    rs = [r for r in rows if r["tier"] == tier]
    print(f"  to {tier}: {len(units(rs))} units, {len(rs)} links")
print("  links whose target does not exist:", sum(r["state"] is None for r in rows))
print("  target change states:", dict(collections.Counter(r["state"] for r in rows)))
print("  links to an accepted specification whose identifier the change's own specification also declares:",
      sum(r["shadowed"] for r in rows))

crossing = 0
for doc in glob.glob("**/*.html", recursive=True):
    if "/evidence/" in doc or doc.startswith("assets/"):
        continue
    text = open(doc).read()
    declared = set(IDS.findall(text))
    crossing += sum(t not in declared for _, refs in REFS.findall(text) for t in refs.split())
print("data-refs entries naming an identifier their own document does not declare:", crossing)

held = collections.defaultdict(set)
for doc in glob.glob("specs/**/*.html", recursive=True):
    for uid in IDS.findall(open(doc).read()):
        if re.match(r"(req|scenario|decision|task|evidence|risk|verify)-", uid):
            held[uid].add(doc)
print("identifiers declared by more than one accepted specification:",
      sorted(u for u, d in held.items() if len(d) > 1))
```

Its output:

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

| What | Units | Links |
| --- | --- | --- |
| task units linking to a unit in another document | **54**, in 15 changes | 66 |
| decision units linking to a unit in another document | **2**, in 2 changes | 2 |
| of those, to their own change's specification | 31 | 36 |
| of those, to an accepted specification | 23 | 30 |
| of those, to another change's specification | 2 | 2 |
| existing `data-refs` entries that cross a document boundary | | **0** |

All 66 task links are in `tasks.html`, and both decision links are in `design.html`. No unit
links to both its own change's specification and an accepted specification. Every link's target
exists on disk. **None of the 56 units carries `data-refs`.**

The staged tree's `data-refs` entries all sit in the specifications. There are 524 of them, in 23
`specification.html` files, on 125 evidence units, 97 verification units, 3 decision units, and
3 risk units. No `tasks.html` carries one.

### Four facts in the same data that bear on the design

- **What the links point at.** Of the 68 links, 24 target a unit marked `added`, 12 a unit marked
  `modified`, 31 an unchanged unit, and 1 a removed unit.
- **The removed target.** It is task 5.1 of `2026-08-21-distinguish-command-names-from-skill-names`,
  "Withdraw the `#### Scenario: One identity per workflow` scenario from …". It links to
  `specification.html#scenario-one-identity-per-workflow`, which that specification marks
  `data-change="removed"`. The task is the work of withdrawing the scenario.
- **Two links point at an accepted requirement the change also restates.** Task 1.3 of
  `2026-08-31-sync-a-change-into-the-accepted-contract` links to the accepted
  `req-a-single-command-exposes-validate-format-and-init`. Task 1.4 of
  `2026-09-02-carry-task-progress-into-the-listing` links to the accepted
  `req-the-command-surface-reports-what-a-project-holds`. Each change's own specification declares
  the same identifier as `modified`.
- **Two identifiers are each declared by two accepted specifications.**
  `scenario-a-tasks-document-carrying-other-units` is in `artifact-kinds` and
  `document-validation`. `scenario-reference-to-a-removed-requirement` is in `document-schema` and
  `document-validation`. The 13 accepted specifications declare 724 distinct unit identifiers,
  and all 726 of their units are unchanged.

A separate count, made the same way over task, decision, evidence, risk, and verification units,
looked for links to a unit in a sibling `proposal.html`, `design.html`, or `tasks.html`. It found 2
task units, each linking to a decision in its own change's `design.html`, and nothing else.

### How often this project has withdrawn an accepted requirement

This repository's own archive has 24 changes and 46 delta specifications. One `## REMOVED
Requirements` section appears in them:

```
$ cd openspec/changes/archive
$ find . -path '*/specs/*' -name spec.md -exec grep -l '^## REMOVED Requirements' {} +
./2026-09-26-carry-a-runbook-to-another-machine/specs/runbook-authority/spec.md
$ find . -path '*/specs/*' -name spec.md -exec awk '/^## REMOVED Requirements/{p=1;next} /^## /{p=0} p&&/^### Requirement/' {} +
### Requirement: A runbook carries no version and no changelog of its own
```

So one requirement has been withdrawn, by one change in 24.

### The staged tree validates today

```
$ cd ~/projects/html-spec-migration/staged && htmlspec --json validate
accepted: True   documents: 109   errors: 0   warnings: 781
review/stale-reference: 317   review/stale-verification: 193   review/unverified-requirement: 271
exit 0
```

It took 19.2 seconds, measured with `time`. No document in it carries a `data-refs` entry that crosses a document boundary, so
this is the result the built change must reproduce exactly.
