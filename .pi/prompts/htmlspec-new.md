---
description: Start an html-spec change — create it, then write its proposal, its design, and its tasks. Use when the work is decided and the change does not exist yet.
allowed-tools: Bash(htmlspec:*), Read, Write, Edit, Glob, Grep
argument-hint: [a description of the work]
generatedBy: htmlspec 1.1.0
---

Start an html-spec change.

## What you were asked

$ARGUMENTS

Read this as the description of the work, and take from it everything it carries — what the
thing is, what it does, what it excludes, and the name if one is stated. The guidance below
on deriving a name and on asking only for what is missing applies to exactly this text.

Where it is empty, nothing was passed rather than lost. Ask what the work is, which is the
one case where asking is right.

**This is an agent-driven operation.** The program creates the change and gives you three
documents in the shape their kinds expect. Deciding what each one says is your work.
There is no command that turns an idea into a proposal, and there is deliberately none:
those documents carry no imposed structure, so there is no grammar for a parser to read.

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

## Create the change

**Where the work is a procedure somebody runs again and again, it is a runbook, not a change.**
A release, a deployment, a restore: each is run many times, and a change ends when it is
archived, so a procedure written as a change is a procedure destroyed after its first run.
Create a runbook instead, and stop here:

```bash
htmlspec new runbook <name> --subject "<subject>"
```

It writes a specification of what every run must record, a tasks document holding the steps
and a figure of their order, a `VERSION`, and a changelog, every unit `unchanged` and every
step `todo`. `htmlspec run <name> --as <record>` starts a run of it. Work that changes what
the project does, including work that changes a runbook, is a change.

```bash
htmlspec new change <name> --subject "<subject>"
```

The name is the directory name, in kebab-case, naming the change rather than the work
(`separate-artifact-kinds`, not `edit-instructions-rs`). The command reports where the
change landed, and it writes three documents:

**Where a contributor describes the work and names nothing, derive the name yourself.** The
rule above is what a good name is, not who has to supply it — it applies identically to a
name you derive. "A browser application that adds named tally counters, incrementing on
click, with no backend" yields `add-tally-counters` in one step.

State the derived name in your report so a contributor who disagrees can say so. Do not stop
and ask them to approve it first: a change directory is renamed by moving it, so the name is
the cheapest thing in the change to correct, and a round trip to confirm one that follows
from the description already given is a refusal in a politer form.

**Ask only for what the description does not carry.** Asking again for a scope or an
exclusion the contributor already stated presents their description as incomplete, and
leaves them guessing which part fell short. Where a request carries no description of the
work at all, asking is right — a need nobody stated must not be invented.

| File | Kind | What it is |
| --- | --- | --- |
| `proposal.html` | `proposal` | The value, the need, and the scope. Not the contract. |
| `design.html` | `design` | Architecture and technical reasoning. |
| `tasks.html` | `tasks` | The work items and their states. |

Each arrives valid and each carries example content in the shape its kind expects.
Replace that content. Do not leave an example unit in a document you are handing to
someone.

**Where this change follows another, say so now.** Write the predecessor's name into a
`FOLLOWS` file in the change directory:

```bash
echo contract-the-guidance-solver-comparison > htmlspec/changes/<name>/FOLLOWS
```

One name, and at most one. This is the moment the fact is cheap to record — a change knows its
own predecessor when it is created, and a list of successors maintained by the change that comes
first goes stale silently. The listing then carries it, and archival reports a change being
archived before the change it follows.

Write it only where the ordering carries weight. Most changes follow nothing, and a change with
no `FOLLOWS` file is complete as it stands. What makes it worth recording is an ordering the work
depends on: a contract written before the changes it grades constrains them, and the same
contract written afterwards describes them. Say in the proposal why the order matters, and put
the name in the file so something other than a reader enforces it.

Write them in order. The proposal's argument is the design's input, and both are the
tasks' input. A tasks list written before the design is a guess at what the work will be.

## The proposal

**Intended content:** value, need, scope, alternatives.

Argue why the change is worth making. Say what the change deliberately excludes, because a
non-goal stated now is a boundary the specification can state later instead of silently
omitting.

**State the need concretely, and a need takes one of two forms.** Neither outranks the
other:

- **Something existing misbehaves.** State it as a measurement, an error message, or a
  behavior somebody hit.
- **The project cannot do something yet.** State who or what wants the capability, what
  they cannot do without it, and what working looks like the first time you see it.

The second form is not a lesser case. A project's first change has nothing to measure and
no error to quote, and neither does any change that adds a capability rather than repairing
one. If you find yourself asking a contributor for a problem when they have described
something new, the answer is that there is no problem and the description was already
enough.

"What working looks like" earns its place, and it is the one of the three most often left
out. It is the input the tasks are broken from, and at archival it is what the requirements
are checked against. A proposal that stops at "users want tally counters" has handed the
rest of the workflow nothing to hold.

**Neither form may be a category of concern.** "We need better data handling" is not a
need, whichever form it was reaching for: a category cannot be argued with while the
proposal is read, and cannot be checked against once the work is built.

**Validation does not shape a proposal, and you must not write as though it does.** A
proposal may use any structure its author finds effective. It may be pure prose with no
unit markup at all: headings, paragraphs, tables, and figures are a conforming proposal.
It may also state a prospective requirement while arguing for it, in prose or as a unit,
and validation reports nothing about that either.

The distinction the contract draws is between *permitted* and *authoritative*. A
requirement unit in a proposal is valid, and it is not part of the contract. The contract
is what a specification states. So write the proposal in whatever form argues best, and do
not reach for unit markup to make it look official.

Only the rules that apply to every document apply here: parseable markup, resolvable
references, and nothing remote or executable. Freedom of form is not freedom to ship a
document that will not open offline.

## The design

**Intended content:** architecture and technical reasoning.

Move the proposal's argument into how the thing will be built. Record the decisions
reached, the alternatives rejected and why, and the risks with how each is settled: what
holds it down, or that the change bears it as it stands and why. A rejected alternative is
worth as much as a chosen one, because the next person to have the same idea needs to know
it was considered.

A risk written as a unit declares how it was settled in `data-response`. A `mitigated` risk
states what holds it down in a `data-field="mitigation"` paragraph, and where the measure
does not reach all the way, that paragraph says how far it reaches. An `accepted` risk needs
no mitigation, and says in its statement why the change bears it; it may still carry a
mitigation paragraph for the part that something does hold down. A risk that declares no
response is `mitigated`, so validation asks it for a mitigation.

A design's reasoning is where an obligation is often stated most precisely, and it is
frequently stated in prose rather than as a unit. That is why archival reads the design
rather than only the proposal. Write the reasoning so it can be read that way: when a
decision creates an obligation the project will have to keep, say so in a sentence someone
can later check.

Structure it however the reasoning requires. Validation requires no unit here and reports
none, because architectural reasoning is continuous — the connections between
considerations carry as much as the considerations do. A design that marks its decisions
as units is valid. So is one that reasons the same ground in prose and marks nothing.

## The tasks

**Intended content:** workable implementation items.

Break the design into items someone can pick up and finish. Order them so that a task's
prerequisites come before it, and put the task that makes the others safe first — a check
that constrains later work belongs at the top, not at the end.

Each task is individually addressable, so that settling one is a change to one unit rather
than to a list. Give each a state. A task may name the requirement it serves in `data-refs`, which
document-schema's "Cross-references address units by identifier" resolves in this change's
specification or in the project's accepted specifications, so until archival produces the
specification, a task can name an accepted requirement and not one this change adds.

`kind/no-tasks` reports a tasks document that carries no task unit. That rule is
completeness rather than placement: the document exists to enumerate work, and one
enumerating none records nothing. A tasks document carrying other units alongside its
tasks is fine and nothing reports them.

**A dropped task states why it was dropped.** Without the reason it records that something
was abandoned and not what the decision was. Write the reason when you drop it, not later:
that reason usually marks a boundary the specification should state as out of scope rather
than silently omit, and archival reads it for exactly that.

## Do not write the specification

The change has three documents and the specification is not one of them.

A specification is produced **at archival**, from the completed change and from what was
actually built. Written now, it would state intent, and intent is what the proposal
already carries. A specification that describes the intent while the code does something
else is the failure archival exists to avoid, and writing it early is the surest way to
produce one.

## Check what you wrote

```bash
htmlspec validate
```

Reconcile every error. Read the warnings: a stale reference usually means a unit's change
state needs settling.

## What to report

**Open with what they are getting, in their words rather than yours.** You already wrote it:
the proposal's "what working looks like" is a concrete sequence, and quoting or compressing it
is the whole of this obligation. A contributor reads that and knows in one pass whether you
understood them. They cannot learn that from a document count.

Then, in this order:

- **What it excludes.** The boundary you drew, stated as a boundary rather than as an
  omission.
- **What you decided for them, and what still needs them.** Separate the two. A value that
  their description does not determine is yours to pick — pick it, say the choice is yours and
  cheap to change, and move on. Scope you added that they did not ask for is yours to own:
  name it and offer to drop it. What is left for them is what genuinely changes the shape of
  the work.
- **Where it landed, and what comes next.** One line. The change directory, the three
  documents, and the next command.

**Say plainly that nothing runs yet.** This workflow produces three documents and no code. A
contributor who described an application will assume otherwise unless you say so, and a
report that reads well while leaving them to discover it later is worse than one that reads
as bookkeeping.

**Name what your references point at, the first time you use each one.** A task identifier, a
diagnostic code, a count of warnings — say in plain words what it is, in the same sentence. The
test: delete the identifiers from your sentence. If nothing is left, they were the claim and
each one needs naming. If the sentence still stands, they were annotation and need nothing. A
list whose first item is named and whose rest are bare numbers is the failure this prevents:
it looks readable and forces every other line to be looked up.

A command, a flag, or a file path already names itself. Do not gloss those.
