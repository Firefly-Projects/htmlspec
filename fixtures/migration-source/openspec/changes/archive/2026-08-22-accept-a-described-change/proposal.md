## Why

A contributor described a change completely and the `new` workflow refused it twice.

```
/htmlspec-new We want to build an application that runs strictly in the web browser.
Basically, it would give you the ability to add tally counters.
Each counter could be named and each time you click, it is tracked.
It shoul run purely in frontend with no backend component.
```

That request states what the thing is, what it does, and what it excludes. The agent
created nothing, asked for four items — two of which were already in the request — and
closed the second exchange with "I still haven't created anything."

Three faults, and the third is the one that makes the other two hard to see.

**The instruction demands a problem, and new work has none.**
`crates/htmlspec-assets/instructions/new.md:35` says:

> State the problem concretely — a measurement, an error message, a behavior somebody hit
> — rather than as a category of concern.

All three examples presuppose an existing system misbehaving. A tally-counter application
that does not exist yet has no measurement, no error message, and nobody who hit a
behavior. The sentence offers no form for an absence, so an agent reading it faithfully
concludes the request is underspecified.

The contract does not ask for this. `openspec/specs/artifact-kinds/spec.md:36` gives a
proposal's intended content as **"Value, need, scope, alternatives"**. Need, not problem.
The narrowing is the instruction's own invention, and it is stricter than the thing it
serves.

**Nothing permits the agent to propose a name.** Line 14 states the naming rule —
kebab-case, naming the change rather than the work — and stops. It never says the agent
may derive one and offer it. A grep for `propose a name`, `derive`, or `suggest` across the
file returns nothing. So the agent treats the name as a required input it must be handed,
when `add-tally-counters` follows from the description in one step.

**The agent knew and still did not act.** Its own reasoning in the second exchange:

> a need for new functionality is legitimate even if it's not fixing a defect; it's an
> absence of capability rather than a broken system

It reached the correct reading, stated it to the contributor, and then asked the question a
fifth time. That is the diagnostic finding: the instruction is not merely unclear, it
overrides a correct inference the agent has already made. Guidance that loses an argument
against its own reader is guidance that must be rewritten rather than clarified.

## What Changes

- **A need may be an absence.** The proposal guidance states that a need is either a
  problem with an existing system or a capability the project does not have, and gives a
  concrete form for the second: who wants it, what they cannot do now, and what working
  looks like the first time you see it. The existing warning against stating a need as a
  category of concern survives, because that warning is correct and is the thing the
  problem-shaped examples were reaching for.

- **The agent proposes the name.** Where a contributor describes the work without naming
  it, the agent derives a kebab-case name from the description, states it, and proceeds.
  The naming rule is unchanged — it is a good rule, and it applies to a name the agent
  derives exactly as it applies to one a contributor supplies.

- **A description is enough to start.** The workflow does not require a contributor to
  answer a checklist when the description already carries the work. What is genuinely
  missing is asked for; what is inferable is inferred and stated so a contributor can
  correct it.

- **`explore` keeps its job and loses this one.** Unsettled work still belongs in
  `explore`. New work whose shape is described is not unsettled, and `ff.md` is amended so
  an absence of capability does not read as an unsettled idea.

- **The contract states it.** `artifact-kinds` already says "need". It gains the obligation
  that authoring guidance must not narrow a need to a defect, since that narrowing is what
  produced a refusal the contract never asked for.

## Non-Goals

- **Loosening what a proposal must argue.** A need still has to be concrete and specific to
  the change. Accepting an absence as a need is not accepting "we need better data
  handling" — that remains a category of concern, and the guidance still rejects it.

- **Making the agent invent a need.** Where a contributor's request genuinely carries no
  work — a bare `/htmlspec-new` with no description — asking is right. The fault is asking
  when the answer was supplied.

- **Changing `htmlspec new change` or its arguments.** `<NAME>` stays required and
  `--subject` stays optional. The defect is in the instructions the agent reads, not in the
  command's signature. A name inferred by an agent is passed to the same unchanged command.

- **Rewriting `explore`.** Its stance is right and this change does not touch its
  instructions beyond the one sentence in `ff.md` that routes an absence toward it.

- **Fixing the second-order behavior directly.** That the agent reached the correct reading
  and still refused is evidence about the instruction's force, not a separate defect to
  patch. Repair the instruction and the behavior follows.
