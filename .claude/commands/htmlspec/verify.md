---
description: Check an implemented html-spec change against its own artifacts before archival — what was built against what the tasks claim, and the code against the design's decisions. Use when the tasks are complete and archival has not started.
allowed-tools: Bash(htmlspec:*), Read, Glob, Grep
argument-hint: [the change to check, or nothing to infer it]
generatedBy: htmlspec 1.0.0
---

Check an implemented html-spec change against its own artifacts, before archival.

## What you were asked

$ARGUMENTS

Where that names a change, check that one. Where it is empty, nothing was passed: infer the
change from the context you have, and say which one you picked.

**This is an agent-driven operation, and it is read-only.** You compare what was built
against what the change said it would build, and you report. You do not fix what you find,
and you do not archive.

The value of this stage is entirely in the judgment. A command can report that a document
is invalid. Only a reader can say whether the code does what the design decided.

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

## 1. Read the change, then read the implementation

Read the change's documents first, so you know what was promised before you look at what
exists:

- The **proposal**, for what the change was for and what it excluded.
- The **design**, for the decisions reached, and the risks and how each was settled.
- The **tasks**, for what the change claims was done and what was dropped.
- The **specification**, where the change already has one. For each requirement it adds or
  modifies, `htmlspec show <change> --diff` shows the change's text beside the accepted text, with
  the words that differ marked, as toolchain-packaging's "The command surface shows a modified
  requirement beside the accepted one" provides. A modification the design did not decide, or a
  decided one the view does not show, is a divergence to classify like any other.

Then read the implementation: the code, the tests, and the fixtures. Read the actual diff
rather than the file as it stands, where you can — a change is what moved, and a file read
in isolation does not distinguish this change's work from what was already there.

## 2. Compare, on three axes

**The tasks against what was built.** For each task marked `done`, find the work. A task
claiming completion with nothing to show for it is the most common finding at this stage,
and it is usually a state settled optimistically rather than a fabrication. For each task
marked `dropped`, check that the reason is recorded.

**For a `done` task that states a deviation, the work to find is what the deviation states.** The
task's words are what was planned, and the deviation is the claim about what was done, as
document-schema's "A task declares completion state" provides. `htmlspec status --change <change>`
names every such task. Check each deviation against the work, and check that it describes work
that happened: a deviation saying nothing needed doing is a dropped task recorded as done.

**The design's decisions against what the code does.** Take each decision the design
records and find where the code embodies it. A decision the code contradicts is the finding
this stage exists for, because archival will read the implementation as authoritative and
the contradiction becomes contract silently.

**The proposal's scope against what the change touches.** Work outside the stated scope is
not automatically wrong, and it is always worth naming. Something the proposal excluded but
the change did, or something the proposal promised and the change omitted, is a scope
question a contributor should answer before the contract is written.

Run the project's checks as evidence rather than as conclusions:

```bash
htmlspec validate
```

## 3. Name what a divergence is

**A divergence is not automatically a defect, and saying which it is is the whole point of
this stage.**

Three cases, and they need different answers:

- **The implementation is wrong.** It does something the design decided against, or fails
  to do something the design decided on. The code needs to change.
- **The design is stale.** Implementation revealed the design's decision was wrong, the
  right thing was built, and nobody updated the document. The design needs to change, and
  the built shape is correct.
- **Neither is wrong and the design was silent.** The code makes a decision the design
  never addressed. Nothing is in conflict, and something undecided is now decided by
  default. The design should record it.

A report that lists divergences without classifying them hands the reader the work this
stage was supposed to do. Say which case each one is, and say what you based that on.

Where you cannot tell, say that too. "The design says the store is derived from the project
root, the code derives it from the document's directory, and I cannot tell which was
intended" is a useful finding. A guess in its place is not.

## Where this stops

**This workflow reports. It changes nothing.**

It does not produce the specification, advance the project's version, apply contract
changes to the accepted specifications, or move the change to the archive. Those four are
archival's work, in that order.

Do not perform part of them here. A change with a specification produced but no version
advanced and nothing merged is in a state neither this workflow nor archival describes, and
the next person to touch it has to work out how it got there.

If the change is ready, say so and stop. Archival is a separate decision.

## What to report

**Open with your judgment: is this change sound, and if not, what is wrong with it.** That is
the question a contributor asked by invoking this, and it is one sentence. A per-task
accounting is the evidence for it rather than a substitute.

Then, in this order:

- **Every divergence, classified**: implementation wrong, design stale, or design silent. Say
  what your classification rests on.
- **What the change does that no artifact describes**, and what an artifact describes that the
  change does not do.
- **What has to be settled before archival**, separated from what you settled yourself.
- **The accounting.** Each task marked `done` and whether you found the work, each task done
  differently and whether you found the work its deviation states, each task you could not
  account for, each dropped task and whether its reason is recorded, and each design decision and
  whether the code embodies it.

**This workflow is read-only.** Say so, because a contributor reading a list of divergences
will otherwise assume you repaired them.

**Name what your references point at, the first time you use each one.** A task identifier, a
diagnostic code, a count of warnings — say in plain words what it is, in the same sentence. The
test: delete the identifiers from your sentence. If nothing is left, they were the claim and
each one needs naming. If the sentence still stands, they were annotation and need nothing. A
list whose first item is named and whose rest are bare numbers is the failure this prevents:
it looks readable and forces every other line to be looked up.

A command, a flag, or a file path already names itself. Do not gloss those.
