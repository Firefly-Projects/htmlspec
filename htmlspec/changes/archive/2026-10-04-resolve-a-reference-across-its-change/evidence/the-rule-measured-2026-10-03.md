# The measurement, repeated against the built change

**Date:** 2026-10-03
**Tasks:** 5.1 to 5.3
**Binary:** `htmlspec 0.21.0`, built from this change's working tree with the placement, the scope,
the reference rule, and the instructions in place.

## 5.1 Steps 1 and 2 now pass, and the plain link is unchanged

`measure.sh`, unchanged from `the-measurement-repeated-2026-10-03.md`, run from an empty directory:

| Step | Before | After |
| --- | --- | --- |
| 1. A task names its change specification's `req-example`, over the project | exit 1, `4 document(s): 1 error(s), 15 warning(s)`, `refs/unresolved` | **exit 0, `4 document(s): 0 error(s), 15 warning(s)`**, no `refs/*` diagnostic |
| 1. The tasks document named alone | exit 1, `1 error, 3 warnings` | **exit 0, `0 errors, 3 warnings`** |
| 2. A task names a requirement only `htmlspec/specs/measure/spec.html` declares | exit 1, `5 document(s): 1 error(s), 16 warning(s)` | **exit 0, `5 document(s): 0 error(s), 16 warning(s)`** |
| The plain links to `specification.html#req-example` and `#req-does-not-exist` | exit 0, `0 errors, 3 warnings`, no reference projected | the same: exit 0, no reference projected, and `links` holds only `hs-content` and `hs-tasks` |

The warnings are the template's own `review/example-content` and `review/stale-reference`, as before.
**The template's advice now validates, and a plain link is still not checked**, as design decision 2
and the proposal's "Not in this change" say.

## 5.2 The staged tree gives exactly its baseline

```
$ cd ~/projects/html-spec-migration/staged && time htmlspec --json validate
accepted: True   documents: 109   errors: 0   warnings: 781
review/stale-reference: 317   review/stale-verification: 193   review/unverified-requirement: 271
exit 0
```

Compared document by document with the baseline from task 1.3, **every document's diagnostics are
identical**: the same rules, units, and messages. No staged document carries a `data-refs` entry that
crosses a document, so this was the expected result, and design decision 12's claim holds on real
data.

**The wall time could not be measured cleanly today, and the CPU time could.** Four runs took
134, 97, 553, and 102 seconds of wall time, against 19.47 for the baseline, while their user CPU time
stayed between 19.81 and 20.03 seconds, against 19.15. A `python3` loop timed at the same moment ran
in its CPU time, but a shell's `sleep 25` in the same session took more than 300 seconds of wall time,
and a `cargo test` run paused for 16 minutes between starting and running its first binary. So the
machine paused the session's file-touching processes, and the wall times measure those pauses. **The
CPU time moved by about 0.7 seconds, 4 percent**, which is reading 13 accepted specifications and 24
change specifications once each.

## 5.3 The staged tree's 68 links, given as `data-refs`

`links-to-refs.py`, below, copies the staged tree, gives each of the 56 linking units a `data-refs`
entry naming every identifier its links name, 68 in all, and validates each document it changed:

```
units: 56 links' identifiers given: 68
   23 units linking to an accepted specification: no refs/* diagnostic
    2 units linking to another change's specification: no refs/* diagnostic
   30 units linking to its own change's specification: no refs/* diagnostic
    1 units linking to its own change's specification: refs/removed-target
units with a refs/* diagnostic:
  changes/archive/2026-08-21-distinguish-command-names-from-skill-names/tasks.html task-5-1 (its own change's specification): refs/removed-target
```

| Expected (task 5.3) | Found |
| --- | --- |
| 30 units linking to their change's specification resolve | 30 resolve |
| 23 units linking to an accepted specification resolve | 23 resolve |
| task 5.1 of `2026-08-21-distinguish-command-names-from-skill-names` reports `refs/removed-target` | it does, and nothing else |
| the 2 units linking to another change's specification report `refs/unresolved` | **they resolve** |

**The task's expectation for the last two was wrong, and the result is right.** The two units are
task 9.1 of `2026-08-17-project-bootstrap-cli`, linking to
`req-archival-requires-the-project-version-to-advance` in `2026-08-30-finish-the-archival`, and task
8.3 of `2026-09-11-migrate-an-openspec-project`, linking to
`req-the-html-to-model-projection-is-published-as-a-normative-specification` in `2026-08-10-html-spec`.
Each identifier is also declared by an accepted specification, `project-bootstrap` and
`document-schema`, because those changes' requirements were merged there. A `data-refs` entry names a
unit by identifier, never a document, so each resolves in the accepted specifications, which is the
third place document-schema's "Cross-references address units by identifier" lists. The task counted
by the file each link pointed at. The scenario "A reference to another change" still holds where only
the other change declares the identifier, which `a_task_naming_another_changes_requirement_is_unresolved`
pins.

### `links-to-refs.py`

```python
#!/usr/bin/env python3
"""In a copy of the staged tree, gives every task or decision unit that links to a unit in another
document a data-refs entry naming each linked identifier, then prints, per unit, the tier its links
point at and the refs/* diagnostics validate reports for it.

Usage: links-to-refs.py <copy-of-staged> <htmlspec-binary>
"""
import collections, glob, json, os, re, subprocess, sys
ROOT, BIN = sys.argv[1], sys.argv[2]
UNIT = re.compile(r'^(\s*)<(\w+) class="(task|decision)" id="([^"]+)"')
LINK = re.compile(r'href="([^"#]*(?:specification\.html|specs/)[^"#]*)#'
                  r'((?:req|scenario|decision|task|evidence|risk|verify)-[a-z0-9-]+)"')
os.chdir(os.path.join(ROOT, "htmlspec"))
units = []  # (doc, uid, tier, [targets])
edits = collections.defaultdict(list)
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
            found = LINK.findall("\n".join(lines[i:j + 1]))
            if found:
                tiers = set()
                for target, frag in found:
                    resolved = os.path.normpath(os.path.join(os.path.dirname(doc), target))
                    if resolved.startswith("specs/"):
                        tiers.add("an accepted specification")
                    elif os.path.dirname(resolved) == os.path.dirname(doc):
                        tiers.add("its own change's specification")
                    else:
                        tiers.add("another change's specification")
                targets = sorted({frag for _, frag in found})
                units.append((doc, uid, " + ".join(sorted(tiers)), targets))
                edits[doc].append((uid, targets))
            i = j
        i += 1
for doc, wanted in edits.items():
    text = open(doc).read()
    for uid, targets in wanted:
        opening = re.search(r'<\w+ class="(?:task|decision)" id="%s"[^>]*>' % re.escape(uid), text).group(0)
        if 'data-refs="' in opening:
            new = re.sub(r'data-refs="([^"]*)"', lambda m: 'data-refs="%s"' % " ".join(m.group(1).split() + targets), opening)
        else:
            new = opening[:-1] + ' data-refs="%s">' % " ".join(targets)
        text = text.replace(opening, new, 1)
    open(doc, "w").write(text)
results = collections.Counter()
rows = []
for doc in sorted(edits):
    out = subprocess.run([BIN, "--json", "validate", doc], capture_output=True, text=True).stdout
    report = json.loads(out)["results"][0]["report"] or {}
    by_unit = collections.defaultdict(list)
    for d in report.get("diagnostics", []):
        if d["rule"].startswith("refs/"):
            by_unit[d.get("unitId")].append(d["rule"])
    for (udoc, uid, tier, targets) in units:
        if udoc != doc:
            continue
        refs = sorted(set(by_unit.get(uid, [])))
        rows.append((tier, uid, doc, refs))
        results[(tier, tuple(refs))] += 1
print("units:", len(rows), "links' identifiers given:", sum(len(t) for *_, t in units))
for (tier, refs), n in sorted(results.items()):
    print(f"  {n:3} units linking to {tier}: {', '.join(refs) or 'no refs/* diagnostic'}")
print("units with a refs/* diagnostic:")
for tier, uid, doc, refs in rows:
    if refs:
        print(f"  {doc} {uid} ({tier}): {', '.join(refs)}")
```
