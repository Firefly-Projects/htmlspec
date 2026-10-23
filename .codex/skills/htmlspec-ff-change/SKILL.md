---
name: htmlspec-ff-change
description: Create an html-spec change and carry it through every artifact in one pass, checking each before the next. Use when the work is well understood and the shape of the change is settled.
allowed-tools: Bash(htmlspec:*), Read, Write, Edit, Glob, Grep
argument-hint: [a description of the work]
license: MIT
metadata:
  generatedBy: htmlspec 1.1.0
---

Create an html-spec change and carry it through every artifact in one pass.

## What you were asked

$ARGUMENTS

Read this as the description of the work, and take from it everything it carries — what the
thing is, what it does, what it excludes, and the name if one is stated.

Where it is empty, nothing was passed rather than lost. Ask what the work is before creating
anything, because this workflow writes every artifact in one pass and a wrong premise
propagates through all of them.

**This is an agent-driven operation**, and it is the same work the `new` workflow does with
fewer pauses for a contributor. Fewer pauses is not fewer checks. Every artifact is checked
before the next one starts.

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

**Recording silence is a step, not an exit.** Having said the contract is silent, take one of
exactly three routes:

1. **Preserve what exists.** Where the thing being decided already has a shape, keep it.
2. **Choose reversibly and own the choice.** Where nothing exists and something must be chosen,
   take the option that is cheapest to change and say the choice is yours rather than the
   project's.
3. **Name the change that would settle it.** Where the question is large enough that a reversible
   choice would mislead, record it as open and name the change that closes it.

Where a decision under silence applies at scale, across many artifacts at once, prefer the first.
A choice made once and applied to a hundred artifacts acquires a hundred instances and no
requirement behind any of them.

## When this is the wrong workflow

Decide this before creating anything.

**A change whose shape is not settled wants `explore` first.** Carrying an unsettled idea
through three artifacts produces three artifacts to rewrite, because the design reasons
about the wrong problem and the tasks enumerate the wrong work. The cost of exploring first
is a conversation. The cost of skipping it is a change that has to be started over.

Use this workflow when the work is understood: you can state what it is for, roughly what
it touches, and what it excludes. If you cannot state those three, stop and say so.

**New is not the same as unsettled.** A capability the project does not have yet has no
problem to describe and no existing behavior to measure, and neither absence makes its shape
unclear. If a contributor can say what the thing is and what it excludes, the work is
understood and this workflow fits. Route to `explore` when the shape is genuinely open, not
when the need happens to be an absence.

**Stop partway when the work turns out to be different than it looked.** A design that
reveals the proposal argued for the wrong thing is a reason to fix the proposal, not a
reason to write tasks against a design you no longer believe. Stopping with two good
artifacts is a better outcome than finishing with three that disagree, and a contributor
can resume from a change that stopped honestly.

## The sequence

```bash
htmlspec new change <name> --subject "<subject>"
```

That creates the change with its proposal, its design, and its tasks, each valid and each
carrying example content in the shape its kind expects. Then work them in order, because
each artifact is the next one's input.

1. **The proposal.** Write the argument: the value, the need, the scope, and what the
   change excludes.
2. **Checkpoint.** Validate. Read what it reports.
3. **The design.** Move the argument into architecture: the decisions, the alternatives
   rejected, the risks and how each was settled.
4. **Checkpoint.** Validate. Then re-read the proposal — if the design contradicts it,
   settle which one is right before continuing.
5. **The tasks.** Break the design into items someone can finish, ordered so a task's
   prerequisites come first.
6. **Checkpoint.** Validate the whole change.

**What each artifact carries is stated in the `new` workflow's instructions, and this
workflow does not restate it.** Read that workflow for the proposal's freedom of form, the
design's continuous reasoning, and the tasks document's completeness rule. Read the
`continue` workflow for how a change's state is established when you are resuming rather
than starting. This workflow is the sequence and the checkpoints; those workflows are the
content.

## The checkpoint

```bash
htmlspec validate <change-dir>
```

After each artifact, not once at the end.

**Do not carry an invalid artifact into the next stage.** A design reasoning from a
proposal that does not hold is reasoning from a document nobody can read, and the error
then has to be fixed in two documents instead of one. The whole saving of this workflow is
in the pauses it removes, and none of that saving comes from deferring the checks.

Reconcile every error before continuing. Read the warnings rather than dismissing them: a
stale reference usually means a unit's change state needs settling now that the shape is
clear.

## What this does not do

It does not implement the change, and it does not write the specification.

The specification is produced at archival, from what was actually built. Producing one here
would state intent, which the proposal already carries. Once the tasks are written the
change is ready to implement, and implementing is a separate stage with its own
instructions.

## What to report

**Open with what they are getting, in their words rather than yours.** The proposal's "what
working looks like" is a concrete sequence, and quoting or compressing it is the whole of this
obligation. A contributor reads that and knows in one pass whether you understood them.

Then, in this order:

- **What it excludes**, stated as a boundary rather than as an omission.
- **What you decided for them, and what still needs them.** Separate the two. This workflow
  moves fast, so a guess written into a design at speed is the one nobody catches — name yours
  rather than leaving it to be found.
- **Where you stopped, if you stopped early**, and what made stopping correct.
- **Where it landed, and what comes next.** One line: the change directory, the artifacts, the
  next command, and what each checkpoint reported.

**Say plainly that nothing runs yet.** This workflow produces documents and no code. A
contributor who described an application will assume otherwise unless you say so.

**Name what your references point at, the first time you use each one.** A task identifier, a
diagnostic code, a count of warnings — say in plain words what it is, in the same sentence. The
test: delete the identifiers from your sentence. If nothing is left, they were the claim and
each one needs naming. If the sentence still stands, they were annotation and need nothing. A
list whose first item is named and whose rest are bare numbers is the failure this prevents:
it looks readable and forces every other line to be looked up.

A command, a flag, or a file path already names itself. Do not gloss those.
