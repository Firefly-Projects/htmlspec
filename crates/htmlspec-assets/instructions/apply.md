Implement an html-spec change.

## What you were asked

$ARGUMENTS

Where that names a change, work on that one. Where it is empty, nothing was passed: infer
the change from the context you have, and say which one you picked before you start editing.

**This is an agent-driven operation.** You work the tasks the change enumerates and settle
each task's state as the work lands. The program checks the documents. It does not decide
whether a task is finished, and no command can.

## Name the requirement, or name the silence

**Where you state that this project requires, forbids, or fixes something, name the requirement
that says so.** Where you cannot name one, say the contract is silent on the point. Do not supply
a rule.

The project's obligations are what its accepted specifications state, and nothing else states
them. A README, a template and the example content it writes, a changelog, an archived change's
proposal or design, a code comment, and the output of an earlier run are all non-normative. Each
one can describe the project accurately and none of them binds it, because nothing checks that
the description stays correct. A change's own specification becomes binding when archival applies
it, and not before.

**This fires on the assertion, not on every sentence.** The test is to remove the claim's subject
and read what is left. Where what remains states something the project requires, name a
requirement. Where what remains is your own choice or a fact about a run, none is owed:

- **Owes one.** "Accepted specifications live at `htmlspec/specs/<name>.html`." Nothing else in
  the sentence carries that claim.
- **Owes none.** "I put the proposal first because the design depends on it." Your own reasoning.
- **Owes none.** "`htmlspec validate` reported two errors." A fact about a run.

Name the requirement itself rather than the capability holding it. A capability leaves the reader
the search the citation was supposed to remove.

**Never supply a justification for a rule you could not find.** A rule with reasoning attached
reads as a decision the project already took, so an invented justification is worse than an
invented rule stated bare. This is the specific failure this obligation exists to prevent: an
agent asserted where a file lives, wrote a plausible reason beside it, and every check passed.

## 1. Read the whole change before starting

- The **proposal**, for what the change is for and what it deliberately excludes.
- The **design**, for the decisions reached, the alternatives rejected, and the risks and
  how each was settled. This is the document the implementation has to agree with.
- The **tasks**, for the work and its order. Read all of them before starting the first
  one: a later task frequently says something that changes how an earlier one should be
  done.

A task whose prerequisites are not done yet is a task to leave alone. The order is part of
the design.

## 2. Work one task at a time

Set the task to `in-progress` when you start it and `done` when it is finished. The four
states are `todo`, `in-progress`, `done`, and `dropped`.

Settle the state as the work lands, not in a batch at the end. A tasks document that says
`todo` for work already merged is a document that misreports the change to anyone reading
it, including whoever archives it.

Keep each change scoped to its task. A task that turns out to require work no task
describes is the case step 5 covers.

## 3. Dropping a task

**Record the reason before you leave a task dropped.** A task marked `dropped` with no
reason records that something was abandoned and not what the decision was, and the reason
is the part that has value later.

Write what made dropping correct: the work became unnecessary, the approach was replaced,
the boundary moved. That reason usually marks something the specification should state as
out of scope rather than silently omit, and archival reads dropped tasks for exactly that.

A task dropped without its reason is worse than a task left `todo`. The `todo` is honest
about being unfinished.

## 4. A task done another way than its words say

**When the work happened and departs from the task's words, mark the task `done` and state a
deviation.** One paragraph marked `data-field="deviation"` says what was done, where that differs
from the words: by another route, in another shape, only part of it, or more than it asked.
document-schema's "A task declares completion state" provides the field and permits it on a `done`
task only.

```html
<li class="task" id="task-3-1" data-change="added" data-state="done">
  <h3>Add the predecessor to the change's declaration <span class="hs-id">task-3-1</span></h3>
  <p data-field="deviation">A change has no declaration to add a field to, so the predecessor is a
  one-line FOLLOWS file in the change directory.</p>
</li>
```

**Keep the task's words as written.** They record what was planned, and designs, evidence, and
later changes cite the task by them. Do not rewrite the heading to match the work, and do not
strike it through. The deviation is stated beside the words.

**Drop the task instead when the work did not happen.** A deviation is an account of finished
work, so "nothing needed doing" is a reason for dropping the task and not a deviation, and a
dropped task that states a deviation is invalid. Where only part of the work happened, the task is
`done`, and its deviation names which part was done and which was not.

**One deviation per task.** A second one is invalid, because a task has one account of what was
done. Supporting detail goes in a note.

**A difference you find while the task is under way goes in a note.** A task that is `todo` or
`in-progress` cannot carry a deviation, because nothing is finished that could differ from its
words, and `htmlspec validate` reports one there. When the task is done, state the difference as
its deviation, saying what was finally done.

**A difference that contradicts the design still takes step 5.** The deviation records what
happened to the task, and the design records the decision. A deviation is not where anyone reads
the design's decisions, so settle the contradiction in the design as well.

`htmlspec list` and `htmlspec status --change <change>` count such a task as done and report it
as done differently, and `status` names it, so whoever reviews the change reads every deviation.

## 5. When the implementation contradicts the design

This happens, and it is not automatically a fault in either one. What matters is that you
do not leave it unrecorded.

**Do not implement something the design does not describe and say nothing.** Archival reads
the implementation as authoritative — where the implementation and the proposal disagree,
what was built is what happened — so an unrecorded divergence becomes part of the contract
without anybody choosing it. That is the failure this instruction exists to prevent.

When you find one, stop and settle it:

- **The design was wrong.** Say what it got wrong and what the right shape is. Update the
  design to record the decision actually made, and note the reason. A design that records
  a decision it did not originally reach is a design that stayed true.
- **The design was right and the work needs to change.** Do the work the design describes.
- **The design did not say.** It is a gap rather than a contradiction. Decide, record the
  decision in the design, and say that you decided it.

A new task the work reveals is a task to add, with its own state, rather than work folded
silently into a neighbouring task.

## 6. Check as you go

```bash
htmlspec validate
```

Run the project's own tests too. A task is not done because the code compiles.

```bash
htmlspec format
```

Run this before finishing, so the documents are in their canonical form and a later diff
shows a change in meaning rather than a change in whitespace.

## Where this stops

When every task is `done` or `dropped`, implementation is finished. The next stage checks
the implementation against the change's own artifacts, and archival comes after that.

This workflow does not produce the specification, advance the project's version, or update
the accepted specifications. Those belong to archival, and doing part of that work here
would leave the change in a state neither stage describes.

## What to report

**Open with what now works, and what they can try.** Name the thing they can do that they
could not do before this ran, in the words they used for it. A contributor learns nothing about
that from a count of completed tasks.

Then, in this order:

- **What is not done yet**, and what is blocking it. An unfinished task named in a report is
  recoverable. One marked `done` is not.
- **What the implementation revealed that the design did not describe**, and how you settled
  it. Say explicitly if you changed the design, and name any task you added and what revealed
  the need for it.
- **What you decided for them, and what still needs them.** A detail their description does not
  determine is yours to settle — settle it and say so. Scope you added is yours to own: name it
  and offer to drop it.
- **The task states and what the checks reported.** One line each, including warnings you left
  in place and why.

**Name what your references point at, the first time you use each one.** A task identifier, a
diagnostic code, a count of warnings — say in plain words what it is, in the same sentence. The
test: delete the identifiers from your sentence. If nothing is left, they were the claim and
each one needs naming. If the sentence still stands, they were annotation and need nothing. A
list whose first item is named and whose rest are bare numbers is the failure this prevents:
it looks readable and forces every other line to be looked up.

A command, a flag, or a file path already names itself. Do not gloss those.
