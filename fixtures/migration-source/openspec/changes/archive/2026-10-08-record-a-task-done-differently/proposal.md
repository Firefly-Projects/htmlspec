## Why

**A task can be marked done or dropped, and nothing can record that it was done another way than its words say.**
document-schema's "A task declares completion state" gives a task four states, `todo`,
`in-progress`, `done`, and `dropped`, and requires a dropped task to state why. When work lands
differently from a task's words, an author has two ways to write it, and both misreport.
`evidence/the-measurement-2026-09-30.md` section 3 measured each one against HEAD:

- **`done` with a note.** A note is prose that no field and no count reads. `list` shows `2/4`
  whether the note is there or not, so the task claims its words were carried out.
- **`dropped` with the difference as its reason.** The work happened, and `list` counts it as
  abandoned: `1/4`, with two tasks dropped instead of one.

The paragraph that would say it, `data-field="deviation"`, is refused. `validate` exits 1 with
`schema/projection` (section 4.0), because it "names no field of this unit".

OpenSpec, which html-spec comes from, is coarser still: a box is ticked or it is not. Section 1
measured `openspec` 1.13.1 on two of this project's archived changes. `accept-a-described-change`
reports 27/28, because its task 5.2 is marked `[~]` and will never be finished.
`sync-a-change-into-the-accepted-contract` reports "complete" at 31/31, although its task 4.5 is
struck through and was never done. html-spec already separates those two cases with `dropped`.
Migrated, they read `27/28 done` and `30/31 done`, each with one dropped task. The third case, done
but not as written, neither format can state.

**This project's own history shows how often that case occurs.** The staged migration of this
repository carries 76 task units with a migration note (section 2). By one reader's classification,
33 of those notes say the work was done another way than the task's words, or only part of it, or
more than it. All 33 tasks are `done`, and every count reads them as done as written. The source
authors met the same gap from the other side. Four task lines in
`2026-09-06-distinguish-a-requirement-from-an-example` carry "**Amended during implementation.**"
or "**Widened during implementation.**" inside the task's text. Three of them rewrote the task's own
words to match the work, because no field held the account. The words that were planned survive
only in git.

The migration's drafting brief contradicts itself on this case. Its convention 2 says a task "that
was done differently, or that was struck through keeps its source state, and a note names the
disagreement". Its task rule says a struck-through task "is `dropped` only if the source states
why". A migration that must choose between two wrong mappings needs a third one.

**Why now.** This repository's migration out of OpenSpec is staged and waiting. Without this change,
each of those 33 tasks is carried as `done` with a note, which every count reads as done as written.
The alternative is to record each one as open under project-migration's "A source distinction the
destination cannot express is recorded as open, not mapped to the nearest state".

## What Changes

- **A done task may state a deviation.** One paragraph marked `data-field="deviation"` states what
  was done, where that departs from the task's words. It projects into a new `deviation` field on
  the task. The task stays `done`, because the work happened, and its words stay as written. A
  `done` task with no deviation is read as done as its words say.
- **A deviation belongs only to a done task, and only once.** Validation rejects a deviation on a
  `todo`, `in-progress`, or `dropped` task, as a new error `schema/deviation-not-done`. It also
  rejects a second deviation paragraph in one task, as a projection fault. A dropped task keeps its
  `reason`, and a task whose work partly happened is `done`, with a deviation that says which part.
- **The listing and the completion report count it apart.** `list` and `status` report how many
  `done` tasks state a deviation, from the one counter they already share. `status` also names each
  one. Such a task still counts as `done`: the fraction keeps meaning every `done` task over every
  task.
- **The human listing shows it after the state, and only where there is one**:
  `measure-a-deviation  2/4  in-progress  1 done differently`. A change with no such task prints
  exactly what it prints today.
- **The structured payloads gain a field.** Each change in `list --json` carries `doneDifferently`
  beside `total`, outside `tasks`, so the per-state counts in `tasks` still sum to `total`.
  `status --json` carries `doneDifferently` and `doneDifferentlyTasks` under `tasks`, beside
  `open` and `openTasks`.
- **The template and the instructions teach it.** The tasks template gains a third example task,
  done with a deviation, beside the dropped one. The `apply` instructions say how to settle such a
  task. The `verify` and `archive` instructions say to read its deviation, not its words, as what
  was done.
- **The document contract moves one minor version**, because this adds a field.
- **Unchanged:**
  - the four states, and what each counts toward;
  - `dropped` and its required reason;
  - a change's total, its open tasks, and the state derived from them;
  - archival's refusals and questions;
  - a task's words carry no change markup, as decided for this whole set of changes. A deviation is
    stated beside the words and never by striking them through.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-schema`: "A task declares completion state" gains the deviation, and the three
  conditions that make one invalid. The requirement is modified rather than joined by a new one,
  because the deviation qualifies the `done` state this requirement defines. The rule that a
  deviation needs `done` mirrors the rule the requirement already states, that `dropped` needs a
  reason. Split into two
  requirements, the one question of what a task's state claims would be answered in two places.
- `toolchain-packaging`:
  - "The command surface reports what a project holds" carries the count of tasks done differently
    beside the per-state counts. It keeps the fraction's meaning, and it derives the new count from
    the same source as the rest.
  - "The command surface reports a change's completion" states the same count and names each task.
  - Both are modified rather than joined by a new requirement, because the count is one more
    projection of the counter they already share, and "the same source" is already stated in them.
    A third requirement would state the agreement a third time.
  - "Archival refuses on the contract and asks about the work" is **not** modified. It asks about
    incomplete tasks, and a `done` task with a deviation is complete. A report of it at archival
    would carry no question, and the `archive` instructions already direct the agent to read the
    tasks for what was done.

## Impact

- `crates/htmlspec-schema/schema/document.schema.json`: the task gains an optional `deviation`
  property, and `schemaVersion` gains the new version. The task's description says a deviation needs
  `done`, which the validator checks, as it does for a dropped task's reason.
- `crates/htmlspec-schema/doc/projection.md`: the section 4.0 field table, section 4.5, a new
  section 4.5.1 for a repeated deviation, section 10.1, and the declared version.
  `crates/htmlspec-schema/CHANGELOG.md`, `CONTRACT_VERSION` in `src/lib.rs`, and
  `tests/contract.rs` move with the version.
- `crates/htmlspec-parse/src/model.rs` (`Task` gains `deviation`) and `src/project.rs`
  (`project_task` accepts the field and reports a second one).
- `crates/htmlspec-validate/src/rules/relations.rs`: the rule `schema/deviation-not-done`, beside
  `dropped_tasks_state_a_reason`, reading `done` from the schema. `tests/schema_rust_split.rs` gains
  `done` as a literal rule code may not restate.
- `crates/htmlspec-cli/src/commands.rs`: `Completion`, `completion()`, the listing's human and
  structured output, and `status`.
- Tests and fixtures:
  - projection tests in `crates/htmlspec-parse/tests/projection.rs`;
  - mapping fixtures under `fixtures/projection/`;
  - conformance fixtures for `schema/deviation-not-done` through
    `conformance/src/bin/generate_fixtures.rs`, with the rule registered in
    `conformance/tests/conformance.rs`;
  - unit tests for the counter and both reports in `commands.rs`.
- `assets/template-tasks.html` gains the example task. Every template's declared contract version
  moves, with its sha256 pin in `assets/vendor.lock.json`.
- `crates/htmlspec-assets/instructions/apply.md`, `verify.md`, and `archive.md`, and the installed
  copies regenerated from them.
- **Not in this change**, and recorded so it is not assumed to follow:
  - Migrating the 33 tasks. `migrate-into-the-extended-vocabulary` maps the new
    vocabulary.
  - A repeated `reason`, `mitigation`, or `method` paragraph. Measured: two `reason` paragraphs
    validate, and the projection silently keeps the first. No unit in the 116 staged documents
    repeats one. Making that an error for every single-valued field is a change of its own.
  - A stylesheet label for the deviation paragraph. A `reason` has none either, and a new stylesheet
    revision moves every template's asset address.
  - The 20 migration notes saying nothing records a task's result. A deviation states what was done,
    and those notes exist because nobody recorded it.
  - An archival report of tasks done differently.
