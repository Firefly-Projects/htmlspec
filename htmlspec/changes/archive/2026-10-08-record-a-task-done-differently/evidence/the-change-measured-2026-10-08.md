# The measurement, repeated against the built change

**Date:** 2026-10-08
**Task:** 7.1, which repeats section 3 of `the-measurement-2026-09-30.md` as
`the-measurement-repeated-2026-10-08.md` recorded it on HEAD
**Binary:** `htmlspec 0.24.0 (7c4b09132aea-dirty)`, built from this change's branch after group 6,
with groups 6 and 7 uncommitted. The workspace version does not move until the release task.
**Projection:** `project-model`, rebuilt against this worktree's `crates/htmlspec-parse` at the same
point.

The scratch project was made the same way as on HEAD: `init --tools none`, `new change
measure-a-deviation`, the masthead placeholder replaced, and the template's example tasks replaced
by the same four. The template now writes three example tasks and declares `0.5.0`, and the script
replaces all three, so the document differs from HEAD's only in its declared version.

## Summary

Every expected result holds. The four-task document validates with exit 0, `list` shows
`2/4  in-progress  1 done differently`, `list --json` carries `doneDifferently: 1` beside counts
that sum to 4, `status` names `task-done-differently`, a deviation on the `todo` task is
`schema/deviation-not-done`, and a second deviation is fault 4.5.1. The note and dropped variants
print exactly what they printed on HEAD.

## The four-task document, with its deviation

`htmlspec validate htmlspec/changes/measure-a-deviation/tasks.html`, **exit 0**:

```
htmlspec/changes/measure-a-deviation/tasks.html
  no diagnostics: the document is accepted
```

`htmlspec list`, exit 0:

```
measure-a-deviation  2/4  in-progress  1 done differently
```

`htmlspec list --json`, exit 0 (the change):

```
{"doneDifferently": 1, "name": "measure-a-deviation", "state": "in-progress", "tasks": {"done": 2, "dropped": 1, "inProgress": 0, "todo": 1, "undeclared": 0}, "total": 4}
```

The per-state counts are 2 + 1 + 0 + 1 + 0 = 4, the total. `doneDifferently` is beside `total`, not
among them.

`htmlspec status --change measure-a-deviation`, exit 0:

```
  present proposal.html
  present design.html
  present tasks.html
  absent  specification.html
  3 of 4 task(s) done or dropped
  1 done differently: task-done-differently
```

With `--json`, `tasks` is
`{"doneDifferently": 1, "doneDifferentlyTasks": ["task-done-differently"], "open": 1, "openTasks": ["task-open"], "total": 4}`.

The projection holds the task as
`{"change": "added", "deviation": "There is no hand-written list. The check resolves each invocation against the command tree, so the subcommand was covered the moment it existed.", "id": "task-done-differently", "state": "done", "title": "…", "unit": "task"}`.

## Beside HEAD

| Variant | HEAD `validate` | Built `validate` | HEAD `list` | Built `list` |
| --- | --- | --- | --- | --- |
| deviation on the `done` task | exit 1, `schema/projection` 4.0 | **exit 0** | `2/4  in-progress` | `2/4  in-progress  1 done differently` |
| the same, as a note | exit 0 | exit 0 | `2/4  in-progress` | `2/4  in-progress` |
| the same, as a `dropped` task's reason | exit 0 | exit 0 | `1/4  in-progress` | `1/4  in-progress` |
| also a deviation on the `todo` task | exit 1, 4.0 twice | exit 1, `schema/deviation-not-done` at `task-open` | `2/4  in-progress` | `2/4  in-progress  1 done differently` |
| a second deviation on the `done` task | exit 1, 4.0 twice | exit 1, `schema/projection` **4.5.1** at `task-done-differently` | `2/4  in-progress` | `2/4  in-progress  1 done differently` |
| also a deviation on the `dropped` task | exit 1, 4.0 twice | exit 1, `schema/deviation-not-done` at `task-abandoned` | `2/4  in-progress` | `2/4  in-progress  1 done differently` |

`status` reports `3 of 4 task(s) done or dropped` in every row, on both binaries. On the built
binary it adds `1 done differently: task-done-differently` wherever the done task carries a
deviation, and adds nothing to the note and dropped variants, whose `--json` carries
`"doneDifferently": 0, "doneDifferentlyTasks": []`.

The three refusals, as reported:

```
  error: schema/deviation-not-done at task-open: a task with state todo states a deviation, and only a done task may. Nothing is finished yet that could differ from the task's words, so record the difference in a note and state it as the deviation once the task is done

  error: schema/projection at task-done-differently: a task contains 2 deviation paragraphs, and the projection does not choose between two accounts of what was done

  error: schema/deviation-not-done at task-abandoned: a task with state dropped states a deviation, and only a done task may. A dropped task's work did not happen, and its reason says why; a task whose work partly happened is done, with a deviation saying which part
```

With `--json` the first and third carry `"rule": "schema/deviation-not-done"` and no
`projectionRule`, and the second `"rule": "schema/projection"`, `"projectionRule": "4.5.1"`.

**An invalid document still counts as the design says.** With a deviation on the `todo` task, the
counter reads that task as `todo` only, and `doneDifferently` stays 1. With two deviations on the
done task, the first is kept, so it still counts as done differently rather than as done as written.

## A downgrade is loud

HEAD's binary, given the built change's four-task document, exits 1 with two errors: the deviation
paragraph is `schema/projection` (4.0), and `schemaVersion` `"0.5.0" is not one of "0.1.0", "0.2.0"
or 2 other candidates`. That is the rollback the Migration Plan describes.

## The staged migration is unchanged

The 116 `.html` files of a scratch copy of the staged migration were projected by `project-model`
built at HEAD and built at this change, and the two outputs are byte-identical: none carries a
`deviation` paragraph, and the new field is absent wherever it is not stated. `htmlspec validate
htmlspec` there reports `109 document(s): 0 error(s), 781 warning(s)` on both binaries.
