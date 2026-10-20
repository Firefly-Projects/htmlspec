# A task done another way than its words say, measured before the change

**Date:** 2026-09-30
**Tasks:** 1.1, 1.2, and 1.3
**Binary:** `htmlspec 0.18.0 (6a591bdf9d92)`, built at HEAD. It prints
`6a591bdf9d92-dirty`, because the working tree holds untracked files. `openspec` is 1.13.1.

**Note, 2026-10-01.** The field this change proposes was renamed `deviation` after this
measurement, so that a task's field does not share a word with a verification's `data-outcome`.
The commands and outputs below are kept as run, with `data-field="outcome"` and the scratch change
`measure-an-outcome`. The behaviour they measure is the same for either name: a task accepts only
`reason` and `note`, so any other field is projection fault 4.0. With `data-field="deviation"`, the
same binary reports the same `schema/projection` error at `task-done-differently`.

## 1. OpenSpec's own counts, for comparison (task 1.1)

A scratch OpenSpec project holding copies of two archived changes, unarchived under their names
without the date:

```
mkdir -p ospec/openspec/changes ospec/openspec/specs
cp -R openspec/changes/archive/2026-08-22-accept-a-described-change \
      ospec/openspec/changes/accept-a-described-change
cp -R openspec/changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract \
      ospec/openspec/changes/sync-a-change-into-the-accepted-contract
cp openspec/config.yaml ospec/openspec/
cd ospec && git init -q .
```

What the two `tasks.md` files hold, by `grep -oE "^\s*- \[.\]"`:

| Change | Task lines | Markers | The task in question |
| --- | --- | --- | --- |
| `accept-a-described-change` | 28 | 27 `[x]`, 1 `[~]` | 5.2, marked `[~]` and headed "**Dropped.**" |
| `sync-a-change-into-the-accepted-contract` | 31 | 31 `[x]` | 4.5, ticked, its words struck through with `~~`, then "**No action: the premise was wrong.**" |

`openspec list`, exit 0:

```
Changes:
  sync-a-change-into-the-accepted-contract     ✓ Complete    10m ago
  accept-a-described-change                    27/28 tasks   10m ago
```

`openspec list --json`, exit 0 (excerpt):

```
{ "name": "sync-a-change-into-the-accepted-contract", "completedTasks": 31, "totalTasks": 31, "status": "complete" },
{ "name": "accept-a-described-change", "completedTasks": 27, "totalTasks": 28, "status": "in-progress" }
```

`openspec instructions apply --change <name> --json`, exit 0, which is the progress the apply phase
works from:

```
accept-a-described-change {"total": 28, "complete": 27, "remaining": 1} ready
   {"id": "20", "done": false, "description": "5.2 **Dropped.** Confirm a change is created on the first ex…"}
sync-a-change-into-the-accepted-contract {"total": 31, "complete": 31, "remaining": 0} all_done
   {"id": "22", "done": true, "description": "4.5 ~~Add the subcommand to `invocations.rs`, whose hand-wri…"}
```

`openspec view` reports `Task Progress: 27/28 (96% complete)` and lists `sync-a-change-into-the-accepted-contract`
under "Completed Changes". `openspec status --change <name>` reports artifacts only
(`Progress: 4/4 artifacts complete`), not tasks. `openspec show <name> --json` carries deltas and no
task progress. For `sync-a-change-into-the-accepted-contract` it fails with exit 1 and
`"message": "Change must have a What Changes section"`, because that proposal has none.

OpenSpec states the rule itself. `openspec instructions tasks --change accept-a-described-change`:

> A box holding only `x` counts as done, upper or lower case and with any spacing, so `- [ x]` is
> done too. Every other marker, including `- [~]`, `- [-]` and an empty `- []`, reads as
> unfinished. A line with no checkbox is not tracked at all.

**Both expectations reproduce.** Task 5.2 is counted not done, so the change reads 27/28 with one
task left that nobody will ever finish. Task 4.5, ticked and struck through, is counted done, so
the change reads complete.

Neither task was done another way. Task 5.2's own text says the confirmation could not be made, and
task 4.5's says "No action". Both are abandoned work. For comparison, the same two changes as the
staged migration carries them (section 2), with each `tasks.html` copied into an active change of a
scratch html-spec project:

```
$ htmlspec list
accept-a-described-change                 27/28  done
sync-a-change-into-the-accepted-contract  30/31  done
```

`list --json` gives `{"done": 27, "dropped": 1, …}` and `{"done": 30, "dropped": 1, …}`, and
`status --change` gives `28 of 28 task(s) done or dropped` and `31 of 31`. html-spec already tells
these two cases apart with `dropped`. The case neither format can state is the one section 2
counts: done, but not as the words say.

## 2. How often the gap shows up in the migration (task 1.2)

**Input:** the 24 tasks documents at `~/projects/html-spec-migration/staged/htmlspec/changes/archive/*/tasks.html`.
A 25th file with that name, `2026-08-14-separate-artifact-kinds/evidence/documents/tasks.html`, is a
document preserved as evidence and is not counted.

**Method:** the program's own projection, through
`~/projects/html-spec-migration/tools/project-model/target/release/project-model`, and every note
of every task unit that begins "Migration note":

```python
docs = sorted((root / "staged/htmlspec/changes/archive").glob("*/tasks.html"))
models = json.loads(subprocess.run([tool, *map(str, docs)], capture_output=True,
                                   text=True, check=True).stdout)
for path, model in sorted(models.items()):
    for task in model.get("tasks", []):
        notes = [n for n in task.get("notes", []) if n.startswith("Migration note")]
        if notes:
            rows.append((pathlib.Path(path).parent.name, task["id"], task["state"], notes))
```

```
{"documents": 24, "tasks": 970, "states": {"done": 965, "dropped": 4, "todo": 1}}
task units with a migration note: 76
migration notes: 76
by state: Counter({'done': 74, 'todo': 1, 'dropped': 1})
```

Cross-checked by `grep`: 76 occurrences of `<p data-field="note">Migration note` across the 24
files. A 77th "Migration note" is a section paragraph outside any task unit
(`2026-09-11-migrate-an-openspec-project/tasks.html:635`).

**76 task units carry a migration note, one each.**

### Classification

I read each of the 76 notes and asked one question: does the note say what was done, and does that
differ from the task's words? This is one reader's judgment, and the table at the end lists every
row so it can be judged again.

| Class | Notes | What the note says |
| --- | --- | --- |
| **A. Done, not as the words say** | **33** | what was done, which differs from the words |
| A: another way | 19 | built, run, or confirmed by another route or in another shape |
| A: only part | 11 | part of what the words ask was done |
| A: more | 3 | more than the words ask was done |
| B. No record | 20 | nothing records the task's result, so what was done is unknown |
| C. A correction or history | 21 | a figure, line number, or claim in the text corrected (12); later review findings (5); done as written under another task (2); where a carried document came from (2) |
| D. A state mapping | 2 | why a task is `todo` (8.4 of `pass-the-description-through`) or `dropped` (4.5 of `sync-a-change-into-the-accepted-contract`) |

**All 33 class-A tasks are `done`, and every count reads them exactly as it reads a task done as
written.** Eleven of them say so in words, by a mechanical match: seven notes contain "as built" and
four contain "differently". The remaining 22 describe the difference without either phrase.

An outcome would not help class B. It states what was done, and those notes exist because nobody
recorded that.

### Three of the notes

Quoted as the projection holds them, with markup removed.

`2026-08-18-install-the-full-workflow-set`, `task-1-4`:

> Migration note: this task was done differently from what it states. At commit 5f7e5cf, which
> archived this change, the test an_absent_subcommand_is_reported in
> crates/htmlspec-cli/src/invocations.rs asserts that htmlspec propose, htmlspec new spec, and
> htmlspec verify are reported, and htmlspec init <path> is not among them. […] The task keeps its
> source state.

`2026-09-02-state-a-changes-place-in-a-sequence`, `task-3-1`:

> Migration note: this was done differently from how the task states it. An html-spec change has no
> per-change declaration to add a field to, so the predecessor was built as a one-line FOLLOWS file
> in the change directory, on the precedent of VERSION, with an absent file and an empty one read as
> the same state. The gap and the choice are recorded in
> evidence/where-the-declaration-goes-2026-09-02.md.

`2026-08-31-sync-a-change-into-the-accepted-contract`, `task-4-4`:

> Migration note: this was done differently for one of the three refusals. At ee4301c, the commit
> that carried this change, sync returns the operational-failure code for an unresolvable change and
> for an absent specification, and the document-violation code for a specification that does not
> validate (crates/htmlspec-cli/src/commands.rs:1177), as archival does for the same refusal (:950).
> It still does at 6a591bd. Nothing in this change records the choice.

### The source authors met the same gap

Four task lines in `openspec/changes/archive/2026-09-06-distinguish-a-requirement-from-an-example/tasks.md`
(3.2, 3.4, 3.6, and 4.3) carry the author's own "**Amended during implementation.**" or "**Widened
during implementation.**" inside the task's text. Task 3.2 as archived:

> - [x] 3.2 Add the diagnostic under the existing `review/` namespace, following
>   `review/unverified-requirement`. **Amended during implementation.** This task said `kind/`, and
>   `kind/` cannot carry a warning: […]

The same four lines in the commit that proposed the change, `git show 8ca4b59:openspec/changes/distinguish-a-requirement-from-an-example/tasks.md`:

```
- [ ] 3.2 Add the diagnostic under the existing `kind/` namespace, following `kind/task-in-specification` and `kind/no-tasks`
- [ ] 3.4 Refuse in `htmlspec archive` while a change's specification carries an example identifier, and name the unit
- [ ] 3.6 Assert coverage moves with the templates: add an example unit to a template in a test fixture and confirm the check reports it with no other edit
- [ ] 4.3 Add it to all seven bodies with identical wording
```

**Three of the four rewrote the task's own words to match the work** (3.2, 3.4, and 4.3), and 3.6
kept its words and appended the account. No field held the account, so it went into the words, and
the words that were planned survive only in git.

### The drafting brief contradicts itself on this case

`~/projects/html-spec-migration/brief-draft.md`, convention 2 (lines 111-116):

> 2. **Records are carried as written, and so is disagreement with them.** A task marked done that
>    nothing records happening, that was done differently, or that was struck through keeps its
>    source state, and a note names the disagreement and links the evidence. A figure later
>    corrected stays as the author wrote it, with a migration note saying it was later corrected and
>    linking the correction. A bare link does not tell a reader it points at a correction. Nothing is silently
>    fixed.

The rule for tasks (lines 310-313):

> - `- [x]` is `done` and `- [ ]` is `todo`. A task the source struck through, or marks as dropped,
>   is `dropped` only if the source states why; that reason, verbatim, is its
>   `<p data-field="reason">`. If the source gives no reason, keep the checkbox's state and add a
>   migration note. A dropped task with no reason is invalid, and inventing one is worse.

For a struck-through task with a stated reason, convention 2 says it keeps its source state
(`done`), and the rule says it is `dropped`. The staged migration followed the rule for 4.5 of
`sync-a-change-into-the-accepted-contract`:

> Migration note: the source checks this task's box ([x]) and strikes its text through, then gives
> the reason above. It is carried as dropped, with that reason, because the source states why the
> action was not taken.

For a task that was done differently, convention 2 is the only instruction: keep `done` and add a
note. That is the class-A mapping above.

### Repeated single-valued fields

Scanning all 116 staged `.html` files for a task, risk, or verification unit holding more than one
`reason`, `mitigation`, or `method` paragraph, with a parser that tracks unit nesting: **0 units**.
The scan was checked against a scratch document holding two `reason` paragraphs, which it reports.

## 3. What html-spec can express today (task 1.3)

```
htmlspec init --tools none --root …/give-a-task-an-outcome/proj     # exit 0
cd …/proj && htmlspec new change measure-an-outcome                  # exit 0
```

The template's two example tasks were replaced by four:

```html
<li class="task" id="task-as-written" data-change="added" data-state="done">
  <h3>Add the subcommand <span class="hs-id">task-as-written</span></h3>
</li>
<li class="task" id="task-done-differently" data-change="added" data-state="done">
  <h3>Add the subcommand to the hand-written list <span class="hs-id">task-done-differently</span></h3>
  <p data-field="outcome">There is no hand-written list. The check resolves each invocation against the command tree, so the subcommand was covered the moment it existed.</p>
</li>
<li class="task" id="task-abandoned" data-change="added" data-state="dropped">
  <h3>Run the end-to-end check <span class="hs-id">task-abandoned</span></h3>
  <p data-field="reason">The command surface discards the description, so the run cannot observe this.</p>
</li>
<li class="task" id="task-open" data-change="added" data-state="todo">
  <h3>Record the version <span class="hs-id">task-open</span></h3>
</li>
```

`htmlspec validate htmlspec/changes/measure-an-outcome/tasks.html`, **exit 1**:

```
  error: schema/projection at task-done-differently: data-field="outcome" names no field of this unit; it accepts "reason", "note"

  1 error, 0 warnings: validation failed
```

With `--json`, the diagnostic is `"rule": "schema/projection"`, `"projectionRule": "4.0"`,
`"unitId": "task-done-differently"`.

`htmlspec list`, exit 0:

```
measure-an-outcome  2/4  in-progress
```

`htmlspec list --json`, exit 0:

```
{ "name": "measure-an-outcome", "state": "in-progress", "total": 4,
  "tasks": { "done": 2, "dropped": 1, "inProgress": 0, "todo": 1, "undeclared": 0 } }
```

`htmlspec status --change measure-an-outcome` reports `3 of 4 task(s) done or dropped`, and its
JSON `"tasks": {"open": 1, "openTasks": ["task-open"], "total": 4}`.

The projection, through `project-model`, holds `task-done-differently` as
`{"change": "added", "id": "task-done-differently", "state": "done", "title": "…", "unit": "task"}`.
**The outcome reaches no field, and the counts cannot see it.** The listing counts the task exactly
as it counts `task-as-written`.

### The two ways an author can write it today

The same document, with only the outcome paragraph changed:

| Written as | `validate` | `list` | `list --json` tasks | `status` |
| --- | --- | --- | --- | --- |
| `data-field="note"`, state `done` | exit 0, no diagnostics | `2/4  in-progress` | done 2, dropped 1, todo 1 | 3 of 4 done or dropped |
| `data-field="reason"`, state `dropped` | exit 0, no diagnostics | `1/4  in-progress` | done 1, dropped 2, todo 1 | 3 of 4 done or dropped |

As a note, the task is indistinguishable in every count from a task done as written. As dropped, a
task whose work happened is counted as abandoned, and the listing's numerator falls by one.

### A repeated reason is discarded silently

In the note variant above, `task-abandoned` given a second `<p data-field="reason">A second reason,
which the projection does not keep.</p>`: `validate` exits 0 with `no diagnostics: the document is accepted`, and the
projection holds the first reason only. `Paragraphs::single` in
`crates/htmlspec-parse/src/project.rs` says of this case that "the caller reports the repetition".
`project_task` does not, and nothing in the validator does.

## The 76 rows

Class letters as in the classification table. "Task" is the migrated identifier.

| # | Archived change | Task | State | Class |
| --- | --- | --- | --- | --- |
| 1 | `2026-08-10-html-spec` | `task-1-1` | done | B |
| 2 | `2026-08-10-html-spec` | `task-1-5` | done | C: corrects a claim |
| 3 | `2026-08-10-html-spec` | `task-4-6` | done | A: another way |
| 4 | `2026-08-10-html-spec` | `task-4-10` | done | A: another way |
| 5 | `2026-08-10-html-spec` | `task-6-1` | done | C: under another task |
| 6 | `2026-08-10-html-spec` | `task-6-2` | done | C: under another task |
| 7 | `2026-08-10-html-spec` | `task-6-9` | done | A: another way |
| 8 | `2026-08-10-html-spec` | `task-7-4` | done | A: another way |
| 9 | `2026-08-10-html-spec` | `task-8-6` | done | B |
| 10 | `2026-08-10-html-spec` | `task-9-1` | done | C: provenance |
| 11 | `2026-08-13-content-address-runtime-assets` | `task-2-4` | done | A: another way |
| 12 | `2026-08-13-content-address-runtime-assets` | `task-8-5` | done | A: another way |
| 13 | `2026-08-14-separate-artifact-kinds` | `task-4-2` | done | C: corrects a claim |
| 14 | `2026-08-14-separate-artifact-kinds` | `task-10-5` | done | A: only part |
| 15 | `2026-08-17-project-bootstrap-cli` | `task-3-1` | done | A: another way |
| 16 | `2026-08-17-project-bootstrap-cli` | `task-8-1` | done | A: only part |
| 17 | `2026-08-17-project-bootstrap-cli` | `task-8-2` | done | A: another way |
| 18 | `2026-08-17-project-bootstrap-cli` | `task-9-1` | done | A: another way |
| 19 | `2026-08-18-conform-generated-frontmatter-names` | `task-1-2` | done | C: corrects a claim |
| 20 | `2026-08-18-conform-generated-frontmatter-names` | `task-1-3` | done | C: corrects a claim |
| 21 | `2026-08-18-install-the-full-workflow-set` | `task-1-4` | done | A: another way |
| 22 | `2026-08-18-install-the-full-workflow-set` | `task-10-5` | done | A: only part |
| 23 | `2026-08-21-check-release-hygiene` | `task-1-3` | done | A: more |
| 24 | `2026-08-21-check-release-hygiene` | `task-1-4` | done | A: only part |
| 25 | `2026-08-21-check-release-hygiene` | `task-1-5` | done | A: only part |
| 26 | `2026-08-21-check-release-hygiene` | `task-5-2` | done | B |
| 27 | `2026-08-21-distinguish-command-names-from-skill-names` | `task-1-5` | done | A: only part |
| 28 | `2026-08-21-distinguish-command-names-from-skill-names` | `task-5-2` | done | A: more |
| 29 | `2026-08-22-accept-a-described-change` | `task-5-1` | done | A: another way |
| 30 | `2026-08-22-accept-a-described-change` | `task-5-3` | done | B |
| 31 | `2026-08-24-pass-the-description-through` | `task-6-1` | done | B |
| 32 | `2026-08-24-pass-the-description-through` | `task-6-3` | done | A: another way |
| 33 | `2026-08-24-pass-the-description-through` | `task-6-5` | done | A: only part |
| 34 | `2026-08-24-pass-the-description-through` | `task-8-4` | todo | D |
| 35 | `2026-08-30-finish-the-archival` | `task-2-11` | done | B |
| 36 | `2026-08-30-finish-the-archival` | `task-3-3` | done | A: another way |
| 37 | `2026-08-30-finish-the-archival` | `task-4-8` | done | A: another way |
| 38 | `2026-08-30-finish-the-archival` | `task-4-12` | done | A: only part |
| 39 | `2026-08-30-finish-the-archival` | `task-7-3` | done | A: only part |
| 40 | `2026-08-30-finish-the-archival` | `task-7-4` | done | A: only part |
| 41 | `2026-08-30-finish-the-archival` | `task-9-1` | done | B |
| 42 | `2026-08-30-finish-the-archival` | `task-9-2` | done | B |
| 43 | `2026-08-30-finish-the-archival` | `task-9-4` | done | B |
| 44 | `2026-08-31-sync-a-change-into-the-accepted-contract` | `task-4-4` | done | A: another way |
| 45 | `2026-08-31-sync-a-change-into-the-accepted-contract` | `task-4-5` | dropped | D |
| 46 | `2026-08-31-sync-a-change-into-the-accepted-contract` | `task-4-6` | done | B |
| 47 | `2026-08-31-sync-a-change-into-the-accepted-contract` | `task-5-4` | done | A: only part |
| 48 | `2026-08-31-sync-a-change-into-the-accepted-contract` | `task-6-1` | done | B |
| 49 | `2026-08-31-sync-a-change-into-the-accepted-contract` | `task-6-2` | done | B |
| 50 | `2026-08-31-sync-a-change-into-the-accepted-contract` | `task-6-3` | done | B |
| 51 | `2026-09-02-carry-task-progress-into-the-listing` | `task-1-1` | done | C: corrects a claim |
| 52 | `2026-09-02-carry-task-progress-into-the-listing` | `task-1-2` | done | C: corrects a claim |
| 53 | `2026-09-02-carry-task-progress-into-the-listing` | `task-5-1` | done | A: another way |
| 54 | `2026-09-02-state-a-changes-place-in-a-sequence` | `task-2-9` | done | B |
| 55 | `2026-09-02-state-a-changes-place-in-a-sequence` | `task-3-1` | done | A: another way |
| 56 | `2026-09-06-distinguish-a-requirement-from-an-example` | `task-3-6` | done | C: corrects a claim |
| 57 | `2026-09-06-distinguish-a-requirement-from-an-example` | `task-3-7` | done | A: another way |
| 58 | `2026-09-06-distinguish-a-requirement-from-an-example` | `task-4-5` | done | B |
| 59 | `2026-09-11-migrate-an-openspec-project` | `task-3-1` | done | C: later review |
| 60 | `2026-09-11-migrate-an-openspec-project` | `task-4-7` | done | C: later review |
| 61 | `2026-09-11-migrate-an-openspec-project` | `task-5-9` | done | C: later review |
| 62 | `2026-09-11-migrate-an-openspec-project` | `task-5-6` | done | C: later review |
| 63 | `2026-09-11-migrate-an-openspec-project` | `task-6-1` | done | C: later review |
| 64 | `2026-09-11-migrate-an-openspec-project` | `task-8-2` | done | C: corrects a claim |
| 65 | `2026-09-11-migrate-an-openspec-project` | `task-8-3` | done | C: corrects a claim |
| 66 | `2026-09-15-catch-a-templates-placeholder-prose` | `task-1-2` | done | C: provenance |
| 67 | `2026-09-15-catch-a-templates-placeholder-prose` | `task-6-5` | done | C: corrects a claim |
| 68 | `2026-09-15-catch-a-templates-placeholder-prose` | `task-7-4` | done | C: corrects a claim |
| 69 | `2026-09-26-carry-a-runbook-to-another-machine` | `task-7-3` | done | B |
| 70 | `2026-09-26-record-a-renamed-requirement` | `task-4-2` | done | C: corrects a claim |
| 71 | `2026-09-26-record-a-renamed-requirement` | `task-10-3` | done | A: another way |
| 72 | `2026-09-28-confine-the-asset-check-to-the-project` | `task-5-2` | done | B |
| 73 | `2026-09-29-keep-evidence-out-of-document-discovery` | `task-1-1` | done | A: more |
| 74 | `2026-09-29-keep-evidence-out-of-document-discovery` | `task-2-1` | done | B |
| 75 | `2026-09-29-keep-evidence-out-of-document-discovery` | `task-2-2` | done | B |
| 76 | `2026-09-29-keep-evidence-out-of-document-discovery` | `task-2-3` | done | B |
