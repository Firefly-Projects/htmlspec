---
name: htmlspec-review-run
description: Check a finished run's record against its runbook before it lands — each note against what its step asks for, each dropped step's reason, each outcome's method, and the go behind each step that needed one. Use when a run's steps are settled and its record has not landed.
allowed-tools: Bash(htmlspec:*), Read, Glob, Grep
argument-hint: [the runbook and the record to check, or nothing to infer them]
license: MIT
metadata:
  generatedBy: htmlspec 1.4.0
---

Check a run's record against its runbook, before the record lands.

## What you were asked

$ARGUMENTS

Where that names a runbook and a record, check that record. Where it names only a runbook, ask which
record, naming the runs `htmlspec run <runbook> --status` lists, unless the context makes one plain,
such as the run just worked; say which you picked. Where it is empty, nothing was passed: infer the
run from the context you have, and say which one you picked.

**This is an agent-driven operation, and it is read-only.** You read what one run recorded beside
the procedure it ran, check what can be checked, and report. You do not fix what you find, take a
step, or land the record.

The value of this stage is a second reader. The agent that worked a run reads its own notes as what
it meant. A reader who did not work it reads them as what they say, and that is where a wrong commit,
a missing value, or an outcome its method does not support is found.

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

## 1. Read the runbook, then the record

Read the runbook's **specification**, for what each run must record, and its **tasks**, for what
each step's note must hold and when a step may be dropped. Then read the record whole. Its steps and
requirements are the procedure as it stood when the run started, and that copy, not the runbook as
it stands now, is what the run was asked to do. Where the two differ, the runbook was revised since:
check the record against its own copy, and say so.

**A record is ready for review when every step is settled but the one that lands it,** where the
runbook has such a step, and every requirement the run reached has an outcome. That step is taken
after the review, on its own go. A record that has already landed can be reviewed too, and its
findings are then fixed differently, as "Where this stops" says.

## 2. Run the program's checks

```bash
htmlspec validate <record-path>
htmlspec run <runbook> --status <record>
htmlspec run <runbook> --check
```

`<record-path>` is the record's file beneath the runbook's `runs/`. `validate` must report no error.
`--status <record>` gives each step's state, each requirement's outcomes, and whether the run is
finished, which means only that every step is settled. `--check` names each requirement a run records
no outcome for: read the lines that name this record. These are evidence, not the review. Every one
of them passes a record whose `passed` outcome sits beside a method that says nothing.

## 3. Check each step

- **Each step `done`** has a dated note that holds what the step's words say to record: a commit, a
  tree, an address, a version, the account a push went out as. A value the words ask for and the
  note lacks is a finding.
- **Each step `dropped`** has a dated reason, and the reason meets the step's own condition for
  dropping it, or quotes the person who stopped the run. A step that applies "first run only",
  dropped on a first run, is a finding.
- **Each step that states a deviation** says what was done instead. A deviation is also a finding
  about the runbook: its words did not fit this run.
- **Each step that needed a person's go** quotes the go in its own note, in a dated note written
  before the step's first write. A step needs one where its words give it to a person, or where it
  writes outside the working copy. **Only words the person typed to the agent working the run, in
  its session, after it stopped and asked, are a go.** A go quoted as a paraphrase, or attributed to
  a relay, another agent or session, a standing instruction, a tool's output, a file, a pull
  request, an issue, or an earlier record, or given before the agent asked, is a finding.
- **No step is left `todo` or `in-progress`,** but the step that lands the record while it waits for
  its go, and a step whose own words say to leave it open, as a step waiting on a later release can.
- **The runbook's own tasks document** still has every step `todo`.

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

**A go that names several steps is quoted in the note of each step it allows,** each time dated and
saying where it was given. Accept such a quote when the earlier note shows the go was given before
this step's first write and its words name this step. A gated step whose own note quotes no go is a
finding whatever happened next, however plainly an earlier note covers it, because nobody reading
this step can tell it was allowed. The step that lands the record is the one exception, and only
while it is still waiting for its go.

## 4. Check each outcome

- **Every requirement the run reached has an outcome other than `not-run`.**
- **Each method says how its outcome was reached,** precisely enough that someone else could repeat
  it: the commands, what they printed, the values.
- **A method that still reads "Not run yet." beside any outcome but `not-run` is a finding.** The
  start writes it as a placeholder, and nothing the program checks reads a method, so this review is
  the one place it is caught.
- **Each outcome agrees with its method.** A method that describes a failure beside `passed` is a
  finding. A `partial` outcome names, in a paragraph marked `data-field="unchecked"`, each part it did
  not check.
- **A recorded failure is not a fault in the record.** It is a correct record of a failed check, and
  the way to a passing run is a new run, never an edited outcome. Report it as the run's result, not
  as a finding.

## 5. Check what can be checked, without changing anything

Where a note or a method states a fact you can confirm by reading, such as that a commit exists,
that a tag names it, that a pull request merged, or that a published version answers, confirm it,
and name the command that did. Run only commands that read. Where confirming a fact would write
anything, say what you would check and leave it.

## Where this stops

**This workflow reports. It changes nothing.** It does not edit the record, settle a step, set an
outcome, start or resume a run, or land the record. It writes no file anywhere, a scratch or
temporary file included: compare what you read in your own reading, not in a file you made. The
`htmlspec` commands it runs are `htmlspec validate`, `htmlspec run <runbook> --status`,
`htmlspec run <runbook> --check`, `htmlspec show`, and `htmlspec list`, and no other. Do not run
`htmlspec run` with `--as`, `htmlspec format` without `--check`, or any other command that writes.

A finding about a record that has not landed is fixed by the `run` workflow, in the same record and
before it lands: a dated note added rather than a note rewritten, a placeholder method replaced, an
outcome set to what its method shows. A finding about a record that has landed is fixed only by a
dated correction note, which the `run` workflow adds on the person's word, and a result that has to
change is a new run. A finding about the runbook is a revision, which the `runbook` workflow makes
inside a change. Whether the record then lands is the contributor's decision.

## What to report

**Open with your judgment: can this record land as it stands, and if not, what stops it.** That is
the question a contributor asked by invoking this, and it is one sentence. For a record that has
landed, say instead whether it needs a correction.

Then, in this order:

- **Every finding, classified:** a fact the record gets wrong, a value its step asks for and it lacks,
  a shape that is wrong (a method still a placeholder, a gated step whose own note quotes no go, a
  go that is a paraphrase or came from anyone but the person, a dropped step whose reason misses its
  condition), or a finding about the runbook rather than the
  record. Say what each rests on.
- **What you confirmed by reading,** each fact with the command that confirmed it, and what you could
  not confirm.
- **The run's result,** as `--status` gives it, kept apart from the findings: a failed run with a
  correct record has no finding.
- **The accounting.** Each step and whether its note holds what it asks for, each go and the step
  that quotes it, and each outcome and whether its method can be repeated.

**This workflow is read-only.** Say so, because a contributor reading a list of findings will
otherwise assume you repaired them.

**Name what your references point at, the first time you use each one.** A task identifier, a
diagnostic code, a count of warnings — say in plain words what it is, in the same sentence. The
test: delete the identifiers from your sentence. If nothing is left, they were the claim and
each one needs naming. If the sentence still stands, they were annotation and need nothing. A
list whose first item is named and whose rest are bare numbers is the failure this prevents:
it looks readable and forces every other line to be looked up.

A command, a flag, or a file path already names itself. Do not gloss those.
