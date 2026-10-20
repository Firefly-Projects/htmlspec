---
description: Think through an idea before it becomes a change — investigate the problem, read the code, and clarify what the work is for. Use before a change exists, when the shape of the problem is not yet settled.
allowed-tools: Read, Glob, Grep
argument-hint: [the idea or question to think through]
generatedBy: htmlspec 1.0.0
---

Think through an idea before it becomes a change.

## What you were asked

$ARGUMENTS

Where that is empty, nothing was passed. Ask what to think through rather than choosing a
subject: exploring is for an idea somebody has, and one you supplied yourself is not that.

**This is a stance, not a sequence.** There are no steps to work through and no artifact
you owe at the end. You are a thinking partner: read the code, follow the question, and
help decide what the work actually is.

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

## Do not implement

While you are exploring, you **must not write code**. Read files, search the repository,
run read-only commands, and reason out loud. Do not edit an implementation file, and do
not create a change.

If the request turns into "go build it", say that exploring is finished and that the work
needs a change first. Then stop, so a contributor can decide.

**Why the prohibition is the whole value of this stage.** An agent asked to think about a
problem starts writing code, because writing code is the thing it is best at. The change
that results has no proposal, because the work happened before anybody decided what the
work was for. Later, at archival, somebody reads the implementation to produce a
specification and finds behavior nothing asked for. That is unrecoverable in the way a
half-finished conversation is not: the code exists, it works, and no one knows what
obligation it was meant to satisfy.

Nothing you produce here is wasted by refusing to build. A clear statement of the problem
is the input the proposal needs, and the proposal is what the next stage writes.

## Capturing thinking is not implementing

You may write down what the exploration concluded — in the conversation, or in a scratch
file a contributor names. That is a record of reasoning, not an implementation.

Producing the change's own artifacts is a different matter. If the exploration settles
what the work is, say so and name the workflow that starts a change. A contributor
decides when the thinking is finished, because that judgment is the point at which
exploration ends.

## What to do instead

Follow whichever of these the question wants. None is required.

- **Read the accepted specifications.** The project's contract states what is already
  promised. An idea that contradicts it is a different and larger change than it appears,
  and finding that out now is cheap.
- **Read the archive.** A change that was proposed and dropped states why in its tasks,
  and a question that has been settled before is usually settled the same way again.
- **Read the implementation.** What the code does is often not what anyone believes it
  does. Where they differ, the difference is frequently the actual subject.
- **State the problem back.** A problem you can state in two sentences is a problem with a
  scope. One that takes two paragraphs is usually several problems.
- **Look for the boundary.** What would this change deliberately not do? A non-goal found
  while exploring is worth more than one found halfway through implementation.
- **Name what you do not know.** An open question recorded now is a question the proposal
  can answer or exclude. One left unspoken becomes an assumption nobody checked.

## When exploring is finished

Exploring is finished when the work has a shape: what it is for, roughly what it touches,
and what it excludes. That is enough to write a proposal, and a proposal is where the
argument gets made properly.

It is also finished when the answer turns out to be "no change". An exploration that ends
in a decision not to build something has done its job, and that decision is worth
recording where the next person can find it.

## What to report

**Open with the question and your answer to it.** If the question changed while you looked, say
what it turned out to be. A contributor asked you to think about something, so the conclusion is
what they are owed first.

Then, in this order:

- **The shape of the work, if one emerged**: its purpose, its rough extent, and its boundaries.
- **What remains open.** Say which open questions would change the design if answered
  differently, because those are the ones a proposal must resolve rather than defer.
- **If no change is warranted**, say that plainly and say why.
- **What you read, and what it said that bears on the question.**

**Nothing was written and nothing changed.** That is this workflow's stance rather than a
shortfall, and a contributor expecting an artifact needs to hear it.

**Name what your references point at, the first time you use each one.** A task identifier, a
diagnostic code, a count of warnings — say in plain words what it is, in the same sentence. The
test: delete the identifiers from your sentence. If nothing is left, they were the claim and
each one needs naming. If the sentence still stands, they were annotation and need nothing. A
list whose first item is named and whose rest are bare numbers is the failure this prevents:
it looks readable and forces every other line to be looked up.

A command, a flag, or a file path already names itself. Do not gloss those.
