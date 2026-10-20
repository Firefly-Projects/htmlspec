---
description: Produce the next artifact an html-spec change is missing. Use on a change that exists and is not yet ready to implement.
allowed-tools: Bash(htmlspec:*), Read, Write, Edit, Glob, Grep
argument-hint: [the change to continue, or nothing to infer it]
generatedBy: htmlspec 1.0.0
---

Produce the next artifact an html-spec change is missing.

## What you were asked

$ARGUMENTS

Where that names a change, work on that one. Where it is empty, nothing was passed: infer
the change from the context you have, and say which one you picked.

**This is an agent-driven operation.** Establishing what the change has is mechanical.
Writing what it lacks is judgment, and there is no command that does it.

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

## 1. Establish what the change has

Look at the change directory. Each document declares its kind in
`<meta name="htmlspec-kind" content="...">`, so the kind is the file's own claim rather
than an inference from its name.

```bash
htmlspec validate <change-dir>
```

That reports what is there and whether it holds. Read the documents themselves too — a
document that exists is not the same as a document that is finished, and a proposal still
carrying the example content the template shipped is a document with nothing in it.

## 2. Decide what is next

The order is fixed, because each artifact is the next one's input:

| The change has | The next artifact is |
| --- | --- |
| nothing | a proposal |
| a proposal | a design |
| a proposal and a design | a tasks document |
| all three | **nothing — the work is implementation** |

Read everything the change already has before writing. A design written without reading
the proposal restates the proposal's intent instead of moving it into architecture, and a
tasks list written without the design is a guess at what the work will be.

If a document exists but is unfinished, finish it rather than moving on. An unfinished
proposal produces a design that reasons about the wrong thing.

## 3. Write it

```bash
htmlspec new document <kind> <change-dir>/<kind>.html --subject "<subject>"
```

That writes a document of the kind, already valid, in the shape the kind expects. Then
replace the example content with the change's own.

**What each kind carries is stated in the `new` workflow's instructions, and this
workflow does not restate it.** Read that workflow for the proposal's freedom of form, the
design's continuous reasoning, the tasks document's completeness rule, and why a dropped
task records its reason. One description of a proposal is the arrangement the project
wants; three descriptions drift, and the one that drifts is the one somebody reads.

## 4. Check it

```bash
htmlspec validate <change-dir>
```

Reconcile every error before stopping. Read the warnings.

## Where this stops

**When the change has its proposal, its design, and its tasks, the next step is
implementation.** Not another document.

There is no fourth artifact to write here. A change holds four kinds and the specification
is produced at archival, from what was actually built — so writing one now would state
intent rather than record a result. If the change has all three documents, say so and say
that the work is ready to implement. Do not invent an artifact to have something to
produce.

The same applies when the tasks are already being worked. A change under implementation
whose tasks are settling is a change past this stage.

## What to report

**Open with what the change can now do that it could not before you ran.** A contributor knows
their change was missing something; what they want is what it now has. Name the capability the
new artifact gives it, not the file name.

Then, in this order:

- **Whether the change is ready to implement**, or what it still lacks.
- **What decided the content**, particularly anything in the earlier artifacts that settled it
  for you.
- **What you could not resolve from the artifacts you had.** An obligation the design implies
  but does not state is worth reporting rather than guessing at.
- **Which artifact you produced, and where it landed.** One line.

**Name what your references point at, the first time you use each one.** A task identifier, a
diagnostic code, a count of warnings — say in plain words what it is, in the same sentence. The
test: delete the identifiers from your sentence. If nothing is left, they were the claim and
each one needs naming. If the sentence still stands, they were annotation and need nothing. A
list whose first item is named and whose rest are bare numbers is the failure this prevents:
it looks readable and forces every other line to be looked up.

A command, a flag, or a file path already names itself. Do not gloss those.
