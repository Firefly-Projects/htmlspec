## Context

See proposal.md for why. The shape of the current system that the approach depends on:

- **A task's fields come from marked paragraphs.** `project_task` in
  `crates/htmlspec-parse/src/project.rs` accepts `reason` and `note`. Any other `data-field` value is
  projection fault 4.0, reported as `schema/projection`. The measurement confirms this for the name
  the field was first proposed under (decision 7). The fault does not depend on the name.
- **A single-valued field takes its first paragraph.** `reason` is read through `Paragraphs::single`,
  whose comment says "the caller reports the repetition". No caller reports it for `reason`. Two
  `reason` paragraphs validate with no diagnostic, and the second is dropped from the model
  (`evidence/the-measurement-2026-09-30.md`, section 3).
- **A rule that depends on a task's state lives in Rust.** The schema keeps to a subset with no
  conditional composition, so "a dropped task states a reason" is `dropped_tasks_state_a_reason` in
  `crates/htmlspec-validate/src/rules/relations.rs`. It reads the value `dropped` from the schema by
  a stem, through `status_named`. `tests/schema_rust_split.rs` forbids rule code from restating a
  schema value as a literal.
- **One counter feeds every report of completion.** `Completion` in
  `crates/htmlspec-cli/src/commands.rs` counts `todo`, `in_progress`, `done`, `dropped`, and
  `undeclared`, and carries the identifiers of the open tasks. `completion()` builds it from the
  projection of an unvalidated tasks document. Three reports use it:
  - `list` renders `{done}/{total}`, then the change's state, then its predecessor. Its JSON puts
    the per-state counts under `tasks`, with `total` and `state` beside them.
  - `status` renders `N of T task(s) done or dropped`. Its JSON is
    `tasks: {total, open, openTasks}`.
  - Archival's `open_tasks` is a projection of the same counter.
- **The tasks template has two example tasks**, one `todo` and one `dropped` with its reason.
  `review/example-content` derives the example identifiers it reports from the templates, so a new
  example is covered without a code change.
- **The contract is `0.4.0`.** A minor bump adds its version to the `schemaVersion` enumeration and
  keeps the earlier ones, under the bump table in `crates/htmlspec-schema/CHANGELOG.md`. Every
  template declares the current version, `0.4.0`, in its `htmlspec-version` meta and its masthead.
  *Corrected on 2026-10-08:* this said `0.2.0` when it was written. `reject-change-markup-inside-a-unit`
  moved the contract to `0.3.0` and `mark-the-words-a-modification-changed` to `0.4.0` before this
  change was implemented (`evidence/what-it-builds-on-2026-10-08.md`).
- **"Outcome" already names something in the contract**: a verification unit's `data-outcome`
  attribute, which is `passed`, `failed`, `blocked`, or `not-run`, and which the stylesheet labels.
  Decision 7 names the task's field so that the two do not share a word.

## Goals / Non-Goals

**Goals:**

- A `done` task can state what was done where that departs from its words, readably and in the
  model.
- The counts tell a task done as written from one done another way, and no existing count changes
  meaning.
- A deviation cannot be stated where it cannot be true: on a task that is not done, or twice in one
  task.
- The instructions an agent settles tasks by say when a deviation is the right record.

**Non-Goals:**

- **Migrating this repository's tasks.** `migrate-into-the-extended-vocabulary` maps the new vocabulary.
- **Repeated `reason`, `mitigation`, or `method` paragraphs.** See decision 3.
- **A stylesheet label for the paragraph.** A `reason` has none either. A new stylesheet revision
  moves every template's asset address, which `record-a-renamed-requirement` showed is a piece of
  work of its own. Without a label, the deviation is a paragraph of prose under a heading marked
  `done`.
- **A deviation on any unit but a task.** A decision already has a status and a rationale, and a
  verification already has its own outcome.
- **Detecting a task done differently.** The program cannot know it. The author states it, and the
  `verify` stage checks it.

## Decisions

### 1. A deviation is a field of a `done` task, not a fifth state

A task states a deviation in one `<p data-field="deviation">`: what was done, where it departs from
the task's words. The paragraph projects into the task's `deviation` field as normalized text, typed
`nonEmptyText` like `reason`. The task keeps `data-state="done"`, and its heading keeps its words.

**Considered: a fifth state, such as `amended`.** Rejected, for two reasons. First, every consumer
of the four states would have to learn that it counts as done: `Completion::resolved`, the open
tasks, the change's derived state, archival's question, the stylesheet's state label, and
migration's mapping. Each one that did not learn it would count finished work as outstanding.
Second, a state carries no text, and the difference has to be said.

**Considered: a note, which is today's workaround.** Rejected. A note is "prose inside the unit
that carries no field of its own", in the schema's words. The measurement shows that `list`,
`list --json`, and `status` count a `done` task with a note exactly as they count one without.

**Considered: rewriting the task's words**, as three source task lines did beside "**Amended
during implementation.**" or "**Widened during implementation.**" (evidence, section 2).
Rejected. The words are what designs and evidence cite a task by. Rewriting them loses the plan,
and change markup inside a task is ruled out for this whole set of changes, so the old words would
survive only in git.

**Considered: an attribute on the task.** Rejected. An attribute is invisible without a
stylesheet, and a deviation is prose a reader has to see. `record-a-renamed-requirement`
(decision 1) chose a field paragraph over an attribute for that readability reason.

### 2. A deviation only on a `done` task, reported as an error

The rule is `schema/deviation-not-done`: the constant `DEVIATION_NOT_DONE` and the function
`deviations_are_on_done_tasks` in `relations.rs`, beside `dropped_tasks_state_a_reason`. It reads
`done` with `status_named("don", "definitions/task/properties/state")`. The stem matches only
`done` among the four states. `every_conditional_rule_finds_its_state` gains that assertion, and
`schema_rust_split.rs` gains `done` as a literal rule code may not restate. No rule code contains
`"done"` today. The one occurrence in `relations.rs` is in its test module.

*Corrected on 2026-10-08:* the stem was `"done"`, the whole value. `schema_rust_split.rs` matches
each owned literal as a whole quoted string on every line of rule code, so the stem `"done"` is the
literal the same decision forbids, and the test would fail on the rule. The stem is a fragment of
the state's name, as `dropp` and `supersed` already are, which is what `status_named` asks for
(`evidence/what-it-builds-on-2026-10-08.md`).

Each state is rejected for its own reason:

- **`todo` and `in-progress`.** Nothing is finished, so no account of finished work can be true
  yet. A difference found midway belongs in a note, and the `apply` instructions say to state it as
  the deviation when the task is done. A task reopened from `done` keeps a deviation that no longer
  describes finished work, and the rule names it.
- **`dropped`.** The work did not happen, and the reason says why. Allowing both would let one task
  claim that the work happened and that it did not. A task whose work partly happened is `done`, and
  its deviation says which part. The migration's own records draw the same line. Task 5.2 of
  `accept-a-described-change` ran, but the confirmation it asks for could not be made, so it is
  `dropped` with its reason. Task 4.4 of `sync-a-change-into-the-accepted-contract` was done, with
  one of three refusals done another way, so it is `done` with a deviation.

A runbook's canonical procedure is `todo` at every moment, under runbook-authority's "The canonical
procedure carries no completion state", so it can never carry a deviation. The record of a run
advances its states, so a step done another way can say so.

**Considered: allowing a deviation on `in-progress`**, to record a difference as soon as it is
known. Rejected. The work is still changing, and a note already records a difference found along
the way.

**Considered: a review warning.** Rejected. This is a contradiction inside one document, like a
dropped task with no reason, and that is an error.

### 3. One deviation per task; a second is a projection fault

The projection specification gains section 4.5.1: "A task containing more than one `deviation`
paragraph is invalid. The projection does not choose between two accounts of what was done."
`project_task` reports fault 4.5.1 and keeps the first paragraph's text in `deviation`. It keeps the
first because the counter reads the projections of unvalidated documents. A task whose author
stated two deviations was done differently, and dropping the field would count it as done as
written.

**Considered: a repeatable field, like a decision's rationale.** Rejected. The count asks one
question of each task, and one paragraph answers it. Supporting detail goes in a note, which a task
already accepts.

**Considered: a validator rule with its own code, as `schema/rename-repeated` is.** Rejected. The
rename rules need the model to keep paragraphs that project nowhere (section 4.1.1). A repeated
deviation is the projection's own ambiguity, and section 4.0.1 already treats that ambiguity as a
projection fault for a single-valued primary field. This takes one sentence in section 4.5, one
fault, and no change to the model's shape.

**Considered: making every repeated single-valued field invalid.** This would cover `reason`,
`mitigation`, `method`, and `deviation` together. Deferred, and not in this change. It changes the
contract of three existing fields, and it deserves its own measurement. None of the 116 staged
documents repeats one. This change corrects the comment on `Paragraphs::single`, so it no longer
says that every caller reports a repetition.

An empty `deviation` paragraph is discarded, as every empty paragraph except `renamed-from` is, so
the task reads as done as written. That is what its text supports, because it states no difference.

### 4. The one counter carries the tasks done differently

`Completion` gains `done_differently: Vec<String>`. It holds the identifiers of the `done` tasks
whose projection carries a `deviation`, in document order, falling back to the title as `open`
does. The count is its length. `completion()` fills it in the same loop that counts states, so there
is no second traversal. A task in any other state that carries a deviation is counted under its own
state only. That document is invalid, and this is the reading the counter already takes for an
undeclared task.

`total()`, `resolved()`, `open`, and `state()` are unchanged. A `done` task with a deviation is
resolved.

**`status` reports on a resolved change through a function of its own.** *Decided on 2026-10-08,
because the design did not reach it:* `status` resolves its project from the working directory, so a
unit test cannot call it, and the tests of its payload belong beside the counter's. It is split as
`list` already is into `list` and `list_in`: `status` resolves, and `status_of` reports on the
change directory it resolved. Nothing it prints or exits with changes
(`evidence/what-it-builds-on-2026-10-08.md`).

**Considered: a bare count.** Rejected. `status` names each task, and names computed anywhere else
would be the second traversal that the one-counter design exists to prevent. `open` sets the
precedent of carrying identifiers.

### 5. The listing keeps its fraction and adds a column after the state

The intended rows, for the measured document and for a change whose every task was done as
written:

```
a-change-done-as-written  3/3  done
measure-a-deviation       2/4  in-progress  1 done differently
```

The count sits in its own column after the state and before the predecessor. It is widened to its
widest member like the other columns, blank where the count is zero, and trimmed at the end of a
line. The reasons:

- **The fraction keeps its meaning.** Every `done` task is in the numerator, so a change whose every
  task is done, some of them differently, still reads `n/n  done`.
- **The columns a reader compares stay where they are.** The fraction and the state do not move,
  and `the_report_aligns_across_names_of_differing_length` keeps pinning the fraction's column.
- **Nothing is shown for zero.** A listing of changes done as written is byte-identical to today's.
- **Words, not a symbol.** A mark needs a legend.

**Considered: subtracting the tasks from the numerator** (`1/4`). Rejected. It reads as work
outstanding, and it is exactly what dropping the task does today.

**Considered: a suffix in the fraction's column**, such as `2/4 (1 differently)`. Rejected. That
column is right-aligned, so a suffix moves the slash on the rows that carry one, and the fractions
stop lining up.

**Considered: a column between the fraction and the state.** Rejected. It would move the state
column on every row whenever any change has a task done differently.

**Considered: a mark on the fraction**, such as `2/4*` or `2/4~`. Rejected. A mark needs a legend,
and `[~]` already means "unfinished" in the OpenSpec archive this project keeps.

`status` keeps its line `3 of 4 task(s) done or dropped`. Where the count is not zero, it adds a
second line, `1 done differently: task-done-differently`, naming each task.

### 6. The structured payloads

- **`list --json`.** Each change gains `doneDifferently` beside `total` and `state`. It is present
  wherever `tasks` is, and omitted with it for a change that has no tasks document. It is not
  inside `tasks`, because `tasks` holds one count per state and those sum to `total`. A subset
  count among them would make a consumer's sum exceed the total. It is present when zero, as
  `undeclared: 0` already is, because a payload states the fact and a caller should not have to
  test for a key.
- **`status --json`.** `tasks` gains `doneDifferently`, a count, and `doneDifferentlyTasks`, the
  identifiers. They sit beside `open` and `openTasks`, which they mirror.

Both are additive, and no existing key changes meaning.

**Considered: naming the keys after the field, as `deviations`.** Rejected. The keys name the count,
not the field: how many tasks were done differently, in the words the listing prints. The field is
what a task states, and the count is how many tasks state one.

### 7. The field is called `deviation`

A task states a deviation: what was done, where it departs from the task's words. The rule is
`schema/deviation-not-done`. The listing's words "done differently" and the keys `doneDifferently`
and `doneDifferentlyTasks` keep their names, because they name the count (decision 6).

The deviation is from the task's words, not from what the work should have been. A task that did
more than its words ask departs from them as much as one that did less or did it another way, so
the three of the 33 tasks that did more state a deviation too.

**The name is free.** `grep -rni deviation openspec/specs crates assets/template-*.html` finds
nothing on 2026-10-01, so no requirement, code, or template uses the word. Elsewhere in the
repository it appears only as ordinary prose, in the installed OpenSpec verify skill and two
archived evidence files, and inside vendored libraries.

**Considered: `outcome`.** Rejected by the contributor on 2026-10-01. A verification unit already
carries `data-outcome`, with the values `passed`, `failed`, `blocked`, and `not-run`. One word with
two meanings breaks the project doctrine's "Use domain terminology consistently". The name would
also have put two `outcome` fields of different types into the model: a task's paragraph of prose,
and a verification's closed set of values. The markup would have kept the two apart, since the
selector `[data-outcome]` does not match `[data-field="outcome"]`. The model, its schema, and the
instructions that teach both would not have.

**Considered: `as-built`**, the phrase seven migration notes use. Rejected. Many tasks are runs,
measurements, or confirmations rather than building. Task 6.9 of `html-spec`, for one, verified
offline behavior another way.

### 8. The template and the instructions teach it

**The template.** `assets/template-tasks.html` gains a third example task,
`task-deviation-example`. It is `done`, and its deviation paragraph says what a deviation is for and
that the words stay as written. It sits beside the dropped example, which set the precedent: an
author copies a working example. `review/example-content` covers the new identifier without a code
change. The template's sha256 pin in `assets/vendor.lock.json` moves.

**Considered: explaining the deviation in the first example's note.** Rejected, for the same reason.

**The instructions.**

- `apply.md` gains a section after "Dropping a task". It says when to use a deviation and when to
  drop a task instead. It says to keep the words, to state which part was done when only part
  was, and to move a difference noted while the task was under way into the deviation when it is
  done. It says that a difference contradicting the design still takes the step "When the
  implementation contradicts the design": the deviation records the task, and the design records
  the decision. That step was 4 when this was written, and the new section, inserted before it as
  section 4, makes it step 5.
- `verify.md` says that for a `done` task with a deviation, the work to find is what the deviation
  states. It adds each such task to the accounting.
- `archive.md` names `doneDifferently` where it describes `list --json`. It also says to read a
  deviation, not the task's words, as what was done.

## Risks / Trade-offs

- **[A deviation becomes a softer `dropped`.]** An agent could write "nothing needed doing" as a
  deviation to keep a count high. → `apply.md` says that work which did not happen is dropped with
  its reason. `verify` checks each deviation against the work, and `status` names every one, so a
  reviewer reads them all.
- **[A deviation absorbs a design decision.]** A difference that contradicts a design decision could
  be recorded on the task and nowhere else. → `apply.md`'s step on a contradiction with the design,
  step 5 once the new section is in, still requires the design to record it, and the new section
  says a deviation does not replace that.
- **[A reopened task keeps a stale deviation.]** → `schema/deviation-not-done` names it. The author
  moves it into a note or removes it.
- **[The new column pushes the predecessor right.]** On a listing where any change has a task done
  differently, `follows …` moves right on every row. → The column is widened consistently, so the
  rows stay aligned. This is the same shift the listing accepted when it added the predecessor
  after the state.
- **[A document with a deviation meets an older toolchain.]** → It fails there with the 4.0 fault
  the measurement recorded, so the failure is loud.

## Migration Plan

This is additive for documents. On HEAD every `data-field="deviation"` paragraph is an error, so no
valid document carries one, and none changes meaning.

**The document contract moves one minor version**, because a field is added. That is the bump
table's "A unit, field, or projection rule is added". The version is `0.5.0`, the minor version
after the `0.4.0` HEAD declared when this was implemented. The new version joins the
`schemaVersion` enumeration, and every earlier version stays in it. `CONTRACT_VERSION`, the schema's
`$id`, the projection specification's header, every template, and the schema changelog move with
it. *Corrected on 2026-10-08:* this named `0.3.0` if nothing moved the contract first, and two
changes did (`evidence/what-it-builds-on-2026-10-08.md`).

The structured payloads gain keys and change none.

**Release.** This is expected to ship as the next minor version after the change before it: 0.25.0,
if changes 1 to 6 ship as 0.19.0 to 0.24.0. The contributor confirms the increment at the release
task.

**Rollback** is a revert. A document that adopted the field fails validation on the older toolchain
with `schema/projection` (section 4.0), as measured, and a document that declares the new version
fails on its version. So a downgrade is loud, not silent.
