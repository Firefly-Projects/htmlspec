## 1. Measure what the listing discards before changing it

- [x] 1.1 Record as evidence what `list` emits today — names only, from `active_changes` at `crates/htmlspec-cli/src/commands.rs:1019`, with a payload of `{ "changes": [names] }` — against what `status` already computes per change at `:1088`, which is artifact presence and `{ total, open, openTasks }`
- [x] 1.2 Record that the counts come from `total_tasks` (`:1148`) and `open_tasks` (`:1269`), that archival's findings derive from the same two, and that the accepted requirement `The command surface reports a change's completion` obliges archival and the completion report to agree. This is the precedent the change extends, not a new rule
- [x] 1.3 Record the schema's task states — `todo`, `in-progress`, `done`, `dropped` — and that `open_tasks` collapses three into one. This is the measurement that decides whether a fraction is sufficient, and it is not
- [x] 1.4 Confirm against the accepted text that the enumeration's stated purpose is selection: `The command surface reports what a project holds` says so in its second paragraph. A change to what it carries is answerable from its own requirement rather than from preference

## 2. The contract states what an enumeration carries

- [x] 2.1 Modify `The command surface reports what a project holds` so the enumeration of active changes carries, for each change, how much of its work is complete
- [x] 2.2 State that the completion carried is a count per task state rather than a single fraction, with the reason: a fraction cannot separate an untouched change from one wholly in progress, nor a finished change from one with outstanding work
- [x] 2.3 State that the counts are derived from the same source as the completion operation and archival, so no two of the three can disagree about one change. This is the fault the version gate demonstrated and the reason it must be a requirement rather than a convention
- [x] 2.4 State that a change with no tasks document is enumerated with its completion absent rather than zero, because a zero fraction reads as finished work and an unwritten tasks document is the opposite
- [x] 2.5 State that the enumeration of accepted specifications is unaffected, so the requirement does not read as though a specification had tasks
- [x] 2.6 State that the order is unchanged. A listing that reorders itself as work progresses cannot be selected from twice
- [x] 2.7 Run `openspec validate --all`

## 3. One counter

- [x] 3.1 Lift the per-change completion `status` computes into a function the enumeration also calls, returning the count per state rather than the `open`/`total` pair the current callers use
- [x] 3.2 Derive `status`'s existing `{ total, open, openTasks }` from that function, so its reported numbers are unchanged and there is still one traversal of a change's tasks
- [x] 3.3 Leave archival's behavior unchanged; it keeps deriving findings from the same source
- [x] 3.4 Add a test asserting the enumeration and the completion operation report the same numbers for one change, in the shape of the accepted scenario `Archival and the report agree`

## 4. What the enumeration emits

- [x] 4.1 Carry the per-state counts into the structured payload for each active change, leaving the accepted-specification enumeration as it is
- [x] 4.2 Render the human report as name, `done`/`total`, and the change's overall state, derived from the breakdown rather than stored beside it
- [x] 4.3 Make the denominator every task including dropped ones, so a change's total does not shrink as work is abandoned. Add the test that pins it: dropping a task changes the numerator's meaning and not the denominator
- [x] 4.4 Render a change with no tasks document with its completion absent, and confirm the payload omits the counts rather than emitting zeros
- [x] 4.5 Keep the columns aligned across changes whose names differ in length, since the report is read as a column and a ragged one is read wrong
- [x] 4.6 Confirm the empty-project case still reports `no active changes` rather than an empty table

## 5. Prove it against this repository

- [x] 5.1 Run the enumeration against this project's active changes and confirm each fraction matches what `status` reports for that change, checked one by one rather than by eye
- [x] 5.2 Construct a change whose tasks are all `dropped` and confirm it reads as complete rather than as untouched
- [x] 5.3 Construct a change with no tasks document and confirm it is listed with its completion absent
- [x] 5.4 Confirm the workflow instructions that select a change still describe the listing accurately, and update the text where it implies the enumeration carries only names
- [x] 5.5 Add the changelog entry and advance the version

## 6. Archival names the sync it already checked

Requested during implementation and outside the original plan. Recorded here rather than done
silently, because the contract clause it adds is not derivable from sections 1–5.

- [x] 6.1 Establish that archival already computes the condition, from `merge_state` via `findings_for`, so nothing recomputes it. A second traversal would be the fault this change exists to avoid
- [x] 6.2 Modify `Archival assesses the merge and never performs it` so an unmerged contract is reported as an unsynced change, naming the operation as it is invoked, and state that archival still does not perform the merge
- [x] 6.3 State that the question asked about an unsynced change says so and says declining leaves the project unchanged, and that nothing about syncing is reported where the contract has reached the specifications
- [x] 6.4 Carry the condition into the structured payload as a value, since an agent cannot answer the question a contributor is asked
- [x] 6.5 Add tests for the predicate, both question wordings, the finding's text, and the payload field, with the question as a pure function so no terminal is needed
- [x] 6.6 Confirm against a real project that the warning appears unsynced, clears once the contract is applied, and agrees with what `htmlspec sync` reports
