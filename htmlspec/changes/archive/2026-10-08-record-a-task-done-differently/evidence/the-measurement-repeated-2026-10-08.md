# The measurement, repeated against HEAD before any change

**Date:** 2026-10-08
**Tasks:** 1.1, 1.2, and 1.3, which repeat sections 1, 2, and 3 of `the-measurement-2026-09-30.md`
**Binary:** `htmlspec 0.24.0 (b768860d5b82)`, built from `main` at `b768860` in this change's
worktree, with nothing changed. A copy was kept outside the tree before any edit, so every "on HEAD"
check in this change runs the same bytes. Every `htmlspec` below is that binary, called by its path.
**OpenSpec:** `openspec` 1.13.2, from `PATH`. The original measurement used 1.13.1.
**Projection:** `project-model`, its source copied out of `~/projects/html-spec-migration/tools/` and
rebuilt against this worktree's `crates/htmlspec-parse` at `b768860`. The copy under the migration
folder was built on 2026-10-07 against an earlier `main`, and nothing under that folder was written.

Scratch files are in the session scratchpad. Nothing in the repository or in the staged migration
was written: the staged project was copied before anything ran over it.

## Summary

Every result of sections 1 to 3 holds. OpenSpec reports 27/28 and 31/31; the staged migration
still holds 76 task units with a migration note, and the 33 class-A tasks are all `done` with their
notes; and HEAD refuses `data-field="deviation"` with `schema/projection` (section 4.0) at
`task-done-differently` while the listing reads `2/4  in-progress`. The only difference is
OpenSpec's patch version, which changes no output measured here.

## Section 1: OpenSpec's own counts (task 1.1)

The scratch project was built as the original recipe states: the two archived changes copied into
`openspec/changes/` under their names without the date, with `openspec/config.yaml`, in a fresh
`git init`.

| Change | Task lines | Markers |
| --- | --- | --- |
| `accept-a-described-change` | 28 | 27 `[x]`, 1 `[~]` |
| `sync-a-change-into-the-accepted-contract` | 31 | 31 `[x]` |

`openspec list`, exit 0:

```
Changes:
  sync-a-change-into-the-accepted-contract     ✓ Complete    just now
  accept-a-described-change                    27/28 tasks   just now
```

`openspec list --json`, exit 0 (excerpt):

```
{ "name": "sync-a-change-into-the-accepted-contract", "completedTasks": 31, "totalTasks": 31, "status": "complete" },
{ "name": "accept-a-described-change", "completedTasks": 27, "totalTasks": 28, "status": "in-progress" }
```

`openspec instructions apply --change <name> --json`, exit 0 each:

```
accept-a-described-change {"total": 28, "complete": 27, "remaining": 1} ready
   {"id": "20", "done": false, "description": "5.2 **Dropped.** Confirm a change is created on the first ex"}
sync-a-change-into-the-accepted-contract {"total": 31, "complete": 31, "remaining": 0} all_done
   {"id": "22", "done": true, "description": "4.5 ~~Add the subcommand to `invocations.rs`, whose hand-wri"}
```

**Both expectations reproduce: 27/28 and 31/31.** Task 5.2, marked `[~]`, is counted not done, and
task 4.5, ticked and struck through, is counted done.

## Section 2: the staged migration (task 1.2)

**Input:** `~/projects/html-spec-migration/staged/`, copied whole into the scratchpad. Its archive
directory was last modified on 2026-09-29, and it holds the same 24 `tasks.html` files under
`changes/archive/*/` (25 in the whole tree, the 25th preserved as evidence and not counted) and 116
`.html` files in all.

**Method:** the evidence's script, unchanged, over the rebuilt `project-model`:

```
{"documents": 24, "tasks": 970, "states": {"done": 965, "dropped": 4, "todo": 1}}
task units with a migration note: 76
migration notes: 76
by state: Counter({'done': 74, 'todo': 1, 'dropped': 1})
```

Cross-checked by `grep -o '<p data-field="note">Migration note'` over the 24 files: 76.

The 76 rows of the evidence's table were read back from the evidence itself and compared with the
76 rows the script finds, by archived change and task identifier:

| Check | Result |
| --- | --- |
| rows in the evidence's table | 76: A 33, B 20, C 21, D 2 |
| class-A rows | 33: another way 19, only part 11, more 3 |
| class-A rows no longer carrying a migration note | none |
| class-A rows whose state is not `done` | none |
| every row present today in the state the table records | yes |
| rows today that the table does not list | none |

**No count changed.** The 33 class-A tasks are still `done`, each with its migration note, so every
count still reads them as done as written.

## Section 3: what html-spec can express at HEAD (task 1.3)

`htmlspec init --tools none --root …/proj` and `htmlspec new change measure-a-deviation`, both exit 0.
The template's two example tasks were replaced by the evidence's four, with the paragraph marked
`data-field="deviation"`, the name decided on 2026-10-01. The masthead's placeholder paragraph was
replaced too: the template's own placeholder draws `review/example-content` today, and the original
measurement reported no warning, so it had replaced it.

The four tasks, as written by the scratch script:

```html
<li class="task" id="task-as-written" data-change="added" data-state="done">…</li>
<li class="task" id="task-done-differently" data-change="added" data-state="done">
  <h3>Add the subcommand to the hand-written list <span class="hs-id">task-done-differently</span></h3>
  <p data-field="deviation">There is no hand-written list. The check resolves each invocation against the command tree, so the subcommand was covered the moment it existed.</p>
</li>
<li class="task" id="task-abandoned" data-change="added" data-state="dropped">… <p data-field="reason">…</p></li>
<li class="task" id="task-open" data-change="added" data-state="todo">…</li>
```

`htmlspec validate htmlspec/changes/measure-a-deviation/tasks.html`, **exit 1**:

```
  error: schema/projection at task-done-differently: data-field="deviation" names no field of this unit; it accepts "reason", "note"

  1 error, 0 warnings: validation failed
```

With `--json`: `"rule": "schema/projection"`, `"projectionRule": "4.0"`,
`"unitId": "task-done-differently"`, exit 1.

`htmlspec list`, exit 0:

```
measure-a-deviation  2/4  in-progress
```

`htmlspec list --json`, exit 0:

```
{"name": "measure-a-deviation", "state": "in-progress", "tasks": {"done": 2, "dropped": 1, "inProgress": 0, "todo": 1, "undeclared": 0}, "total": 4}
```

`htmlspec status --change measure-a-deviation`, exit 0, ends `3 of 4 task(s) done or dropped`. Its
JSON carries `"tasks": {"open": 1, "openTasks": ["task-open"], "total": 4}`.

The projection holds `task-done-differently` as
`{"change": "added", "id": "task-done-differently", "state": "done", "title": "…", "unit": "task"}`.
**The deviation reaches no field, and the counts cannot see it.**

### The two ways an author can write it today

The same document, with only the second task's paragraph changed:

| Written as | `validate` | `list` | `list --json` tasks | `status` |
| --- | --- | --- | --- | --- |
| `data-field="note"`, state `done` | exit 0, no diagnostics | `2/4  in-progress` | done 2, dropped 1, todo 1 | 3 of 4 done or dropped |
| `data-field="reason"`, state `dropped` | exit 0, no diagnostics | `1/4  in-progress` | done 1, dropped 2, todo 1 | 3 of 4 done or dropped |

Both reproduce. As a note the task is indistinguishable from one done as written, and as dropped
the numerator falls by one.

### Three more variants, recorded as the baseline for task 7.1

| Variant | `validate` |
| --- | --- |
| a second `deviation` paragraph in `task-done-differently` | exit 1, two `schema/projection` (4.0) at `task-done-differently` |
| also a `deviation` on the `todo` task `task-open` | exit 1, `schema/projection` (4.0) at `task-done-differently` and at `task-open` |
| also a `deviation` on the `dropped` task `task-abandoned` | exit 1, `schema/projection` (4.0) at `task-done-differently` and at `task-abandoned` |

In each, `list` still reads `2/4  in-progress` and `status` `3 of 4`.

### A repeated reason is still discarded silently

In the note variant, `task-abandoned` given a second `<p data-field="reason">`: `validate` exits 0
with `no diagnostics: the document is accepted`, and the projection holds the first reason only.
