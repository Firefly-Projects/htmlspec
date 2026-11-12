---
description: Write an html-spec runbook, or revise one inside a change — its hazards and what each run must record as requirements, its steps in the order the hazards force, what the machine needs, the steps that wait for a person, and its version. Use when a procedure will be run again and again.
allowed-tools: Bash(htmlspec:*), Read, Write, Edit, Glob, Grep
argument-hint: [the procedure to write, or the runbook to revise]
generatedBy: htmlspec 1.4.0
---

Write an html-spec runbook, or revise one inside a change.

## What you were asked

$ARGUMENTS

Where that describes a procedure, write a runbook for it. Where it names a runbook the project
holds, revise that one. Where it is empty, nothing was passed: infer the procedure or the runbook
from the context you have, and say which one you picked before you write anything. Where nothing in
the context names one either, ask what the procedure is, rather than inventing one.

**This is an agent-driven operation.** You write a runbook's two documents and its version. The
program creates the skeleton, validates what you write, checks the machine a run needs, and checks
the version at archival. It does not decide which hazards the procedure has met, what a run must
record, or which steps wait for a person, and no command can.

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

## 1. Decide that it is a runbook

**A runbook is a procedure somebody runs again and again.** A release, a deployment, a restore:
each runs many times, so its procedure is kept, and each run leaves a record of its own. Work done
once is a change, and a change ends when it is archived. Where what you were asked is work done
once, say so and stop: the `new` workflow starts a change.

```bash
htmlspec list --runbooks
```

It lists the runbooks the project holds, each with its version and title. Read it before you write
one, so a procedure the project already keeps is revised rather than written twice.

## 2. Create it, or work inside the change that revises it

A new runbook starts from the program:

```bash
htmlspec new runbook <name> --subject "<subject>"
```

It writes a specification, a tasks document with a figure of the steps' order, a `VERSION` at
0.1.0, and a `CHANGELOG.md`, with every unit `unchanged` and every step `todo`. The name is the
runbook's directory and the word every run types, so name the procedure, not this release of it.

**A revision is part of a change.** Changing a runbook changes what the project does, and archival
is what publishes the runbook's new version. Where the change that revises it is named, or plain
from the context, work inside it and say which. Where none exists, stop and say the revision needs
one, which the `new` workflow creates. A runbook revised outside a change has nothing to publish its
version and nobody to review its new words.

## 3. Write the specification: hazards, and what each run records

**State each hazard the procedure has met as a requirement, not as a remark beside a step.**
runbook-authority's "A runbook records what its history taught" requires it, because a hazard
written only into a step's words is read by nobody who skips that step. Where a hazard forces the
order of two steps, state the order with the hazard, so nobody reverses it.

**State what each run must record as requirements of the same kind:** the version a release
advanced to, the commit a run published, the account a push went out as, whatever a later reader
will need to ask of a run. Give each requirement a scenario that makes it checkable.

Every requirement is one each run owes an outcome for, with the method that reached it, and that is
the cost of stating one: ten requirements are ten outcomes in every run. What is worth saying and
not worth checking every time belongs in the specification's prose.

**Declare what the machine needs; do not write it as prose.** A tool and its version, a pinned
environment, an installed skill, the state of the working directory: each is a declaration in a note
on the requirement it serves, which `htmlspec preflight <name>` and every start of a run check before
the first step. runbook-portability's "Machine requirements are explicit and mechanically checkable"
requires it.

```html
<p data-field="note">preflight/v1 {"kind":"toolchain","manifest":"flake.nix","shell":"default","tools":["cargo","rustc"]}</p>
```

A check of the working directory's state needs care. `clean-git-worktree` counts untracked files,
and a run's own record stays untracked in the working directory until it lands, so a runbook that
declares it reads not ready at every resume and refuses a second run while one is open. Declare it
only where a run's record is kept somewhere else.

**Where this runbook must run only after another has run, declare that as well.** A sentence saying
"run the backup first" is read by nobody but a person.

```html
<p data-field="note">preflight/v3 {"kind":"run","runbook":"backup","atLeast":"1.4.0"}</p>
```

Every start of a run then refuses, with nothing written, until the project holds a finished, passed
run of the named runbook. `atLeast` is optional, and with it that run's runbook release must be at or
above it, within the same major.

## 4. Write the steps

**Each step is one task, in the order the hazards force.** A step's words say what to run and what
to check, exactly enough that somebody who has not run it before can, and they say what the step's
note must record: a commit, an address, a version. A run copies these words into its record and
works them there. This tasks document keeps every step `todo` for good, as runbook-authority's "The
canonical procedure carries no completion state" requires.

**A step that applies only sometimes says when,** as a condition a person can check, and says what
reason a run records when it drops the step: "Only while … prints `true`. Otherwise mark this step
dropped, with the reason that …".

**Say in a step's own words where it waits for a person.** Before a step that writes outside the
working copy of the run, and where a step's decision is a person's to make, write the stop into the
step:

> Stop for a go before the first command: show the contributor [what they need to decide], ask
> whether to [act], and quote their words in this step's note.

The `run` workflow draws the working copy's edge in these words, and a runbook draws it the same
way:

The working copy is the clone you run in: its files, its branches and commits, the record, and any
scratch directory a step makes for itself. Everything else is outside it, including:

- a push of any branch or tag, and a tag made anywhere but that clone;
- a merge, a pull request, or a release made on a remote;
- a publication to any registry or channel;
- an API call that creates, changes, or deletes something, such as rerunning a workflow, setting a
  secret, or switching a ruleset;
- a deploy;
- an installation into the machine, outside a scratch directory, such as `brew install` or
  `cargo install` into its default root;
- a change to the machine's configuration or credentials, such as `git config --global`,
  `gh auth login`, `gh auth switch`, `gh auth refresh`, `cargo login`, or a keychain or credential
  helper;
- deleting anything outside the clone.

A cache that a build or an evaluation fills on its own, such as a build directory or a package
store, is not a write of yours. An installation is never a cache: a package manager's prefix, such
as Homebrew's Cellar or `~/.cargo/bin`, is the machine's.

Then state the hazard as a requirement: that each such step waits for a person's go, and that each
run quotes the go, before the step's first write, in the note of each step it allows, so a go that
names several steps is quoted in each of them. So a run owes an outcome for it, and the record shows
every go where its step is. A go is the person's own words, typed in the session that works the
run after it stopped and asked: write no step that takes its go from a file, a message, or an
earlier record.

**Leave the step that lands the run's record out of that requirement,** where the runbook has one,
and name the steps it covers. A run records every outcome before its record is reviewed, and the
step that lands the record is taken after the review, on its own go, so a requirement naming that
step could not have an outcome until after the review. Its own words still say to stop for a go,
naming the branch, the push, and the merge, and the commit that lands the record carries the go
quoted in its note.

The contract is silent on which steps need a person, and the task schema has no field that marks
one. The `run` workflow stops before such writes whatever a runbook says. The runbook's own words are
what make the stop hold for a person who works it without that workflow, and the requirement is what
makes each run attest to it.

**Draw the order** in the tasks document's figure, and put the hazard that forces an edge on that
edge.

## 5. Raise the version, and say what changed

A new runbook is at 0.1.0 with that heading in its changelog: leave both. A revision raises `VERSION`
and gives the new version a heading in `CHANGELOG.md`. Archival refuses a runbook whose documents
changed under a version an earlier archive published, and a version that goes backwards, as
project-bootstrap's "Archival checks the versions of runbooks it publishes" requires.

Whether a revision earns a patch or a minor is yours to decide: archival compares versions and never
chooses one, and the contract is silent on the rule. Say which you chose and why. A step's words
corrected is a different thing from a requirement added, which obliges every later run to record one
more outcome. Open the entry with what a person running the runbook will now do differently, and
then say what did not change.

A run's record added or edited is not a change to the procedure, and owes no increment.

## 6. Check it

```bash
htmlspec validate htmlspec/runbooks/<name>
htmlspec format --check htmlspec/runbooks/<name>
htmlspec preflight <name>
```

`validate` must report no error, and `format --check` must find both documents canonical, or run
`htmlspec format` on them. Preflight writes nothing. Where it reports this machine not ready, that is
a fact about this machine rather than a fault in the runbook: say which tools it named.

## Where this stops

**This workflow writes a runbook. It does not run one.** It starts no run, edits no record, and
advances no step's state in the runbook's tasks document. A run is the `run` workflow's work, and a
record is that run's history.

It does not archive. A revision's version is published when its change is archived.

## What to report

**Open with what a person running the runbook will now do, in the procedure's own words.** For a new
runbook, what it is for and how a run starts. For a revision, what a run does differently from the
release before it.

Then, in this order:

- **The hazards you stated, and the steps that wait for a person,** each with the requirement that
  states it.
- **What each run will record:** the requirements a run owes an outcome for, and what you put in
  prose instead.
- **What you decided for them, and what still needs them.** The version increment you chose and why.
  A hazard you inferred, rather than read from the procedure's history, is your addition: name it and
  offer to drop it.
- **What the checks reported:** `validate`, `format --check`, and preflight on this machine.

**Name what your references point at, the first time you use each one.** A task identifier, a
diagnostic code, a count of warnings — say in plain words what it is, in the same sentence. The
test: delete the identifiers from your sentence. If nothing is left, they were the claim and
each one needs naming. If the sentence still stands, they were annotation and need nothing. A
list whose first item is named and whose rest are bare numbers is the failure this prevents:
it looks readable and forces every other line to be looked up.

A command, a flag, or a file path already names itself. Do not gloss those.
