## Why

`htmlspec list` prints names. That is all it prints:

```
carry-task-progress-into-the-listing
distinguish-a-requirement-from-an-example
migrate-an-openspec-project
sync-a-change-into-the-accepted-contract
```

The accepted requirement that put it there says why it exists: "This is the operation that
lets a workflow select what it acts on. A workflow that cannot enumerate a project's changes
must have its subject named for it every time." The enumeration was built for selection, and
then given nothing to select on.

**A list of names cannot answer the question anyone actually asks it.** "Which change were we
working on?" is not answered by four names sorted alphabetically. Neither is "which of these
is nearly done", "which has been abandoned", or "which is ready to archive". Every one of
those is a question about state, the project already computes the state, and the enumeration
throws it away. An agent given the payload above has to run `status` once per change to
recover what the enumeration could have carried in one call — and an agent that does not think
to do that selects on name order, which is to say on nothing.

**The data exists and is already computed by the same functions.** `status` reports, for one
change, which artifacts are present and `{ total, open, openTasks }` from `total_tasks` and
`open_tasks` (`crates/htmlspec-cli/src/commands.rs:1148` and `:1269`). Archival derives its
incomplete-work finding from the identical functions, and the accepted contract requires that:
"Archival SHALL derive its report about incomplete work from this same operation, so that a
contributor who checks a change before archiving it and the archival that follows cannot
disagree about what the change holds." So the numbers are not new work to compute. They are
work already done, for one change at a time, that the enumeration does not ask for.

**Who is blocked and on what.** Every workflow that selects a change, which is most of them.
`archive` with no argument offers the list — the accepted contract says the enumeration is
what makes that possible — and it offers bare names, so a contributor picking from it is
picking blind at the moment they are about to move a directory and rewrite a contract. Agents
running `continue` or `apply` have the same problem one step earlier: they must choose a change
to resume, and the enumeration gives them no basis to choose.

**What working looks like the first time you see it.** A contributor runs `htmlspec list` and
sees each active change with how much of its work is done and what state that work is in:

```
carry-task-progress-into-the-listing        0/24  todo
distinguish-a-requirement-from-an-example  17/31  in-progress
migrate-an-openspec-project                 0/58  todo
sync-a-change-into-the-accepted-contract    0/27  todo
```

They can now answer "which were we working on" by reading. An agent asking for the structured
payload gets the same counts as fields, plus what `status` already reports about artifacts, and
selects without a second call per change.

**"m out of n" is the smaller half of this.** A count of done-over-total is the number a
contributor reads, and on its own it is ambiguous in a way the schema is not. The task state
enum is `todo`, `in-progress`, `done`, `dropped` — four states, and `open_tasks` collapses
three of them into one by filtering for "not done and not dropped". A change reading `0/24` is
therefore either untouched or entirely in progress, and those are different situations. A
change reading `20/24` with the remaining four dropped is finished; the same fraction with four
`todo` is not. So the enumeration carries the state breakdown, not only the fraction, and the
fraction is what the human report renders from it.

**The fault to avoid is the one the project has already been bitten by.** Version 0.6.0's entry
records a gate that never fired because it recomputed, from a convention it invented, a fact the
project recorded elsewhere: "the convention read and the convention written have one author and
cannot drift" was the repair. Counting tasks a second time inside `list` would be that fault
reintroduced, and it would be invisible in the same way — a listing that says `17/31` while
`status` says `18/31` for the same change is wrong in a way no test written against either one
alone would catch. There must be one counter, and the enumeration must call it.

**What this change does not do.** It does not change what `status` reports, and it does not
merge the two operations — `list` answers "what is here", `status` answers "what is in this
one", and the accepted contract names them separately. It does not add a task state, sort the
listing by progress, or filter it; the enumeration stays in name order, because a stable order
is what makes a list a thing you can select from twice.
