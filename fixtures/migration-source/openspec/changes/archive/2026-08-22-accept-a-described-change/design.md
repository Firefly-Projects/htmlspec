## Context

`crates/htmlspec-assets/instructions/new.md` is prose an agent reads before creating a
change. It is not code, and nothing tests it. Its only check is what an agent does after
reading it, which is why a defect here surfaces as a transcript rather than as a failing
build.

Two sentences produced the refusal. Line 35 narrows a need to a problem and illustrates it
with three examples that all presuppose an existing system misbehaving. Line 14 states the
naming rule without saying who may satisfy it. Neither sentence is wrong about the case it
was written for. Both are silent about new work, and an agent reading a silence as a
prohibition is behaving correctly given the text.

The transcript records something sharper than ambiguity. The agent inferred that an absence
of capability is a legitimate need, wrote that inference down, and then asked for a problem
anyway. The instruction did not fail to inform it. The instruction outweighed it.

## Goals

- A described change starts on the first exchange.
- A need may be an absence, stated in a form as concrete as a problem's.
- The agent derives a name and states it, rather than requiring one.
- No loss of the thing the current wording protects: a need must not be a category of
  concern.

## Non-Goals

- Changing `htmlspec new change` or its arguments. See the proposal.

## Decisions

### Two kinds of need, both concrete

The repair is not to delete the demand for concreteness. It is to give the second kind of
need its own concrete form, so an agent has something to write rather than only something
to avoid.

A problem-shaped need keeps its three examples: a measurement, an error message, a behavior
somebody hit. An absence-shaped need gets three of its own:

- who or what wants the capability,
- what they cannot do without it,
- what working looks like the first time you see it.

The third is doing the most work and is the least obvious. It is the input the tasks are
broken from, and at archival it is what the requirements are checked against. A proposal
that states it has given the rest of the workflow something to hold; one that stops at
"users want tally counters" has not.

Two alternatives rejected:

**Delete "state the problem concretely" and say only "state the need."** This removes the
refusal by removing the guidance, and the warning against a category of concern goes with
it. That warning is the correct part of the current sentence, and dropping it trades a
false refusal for accepted vagueness.

**Add a sentence permitting new work without stating its form.** "A need may also be an
absence of capability" would have unblocked the transcript. It leaves an agent with a
permission and no shape, which is how a proposal ends up arguing "we need better data
handling" — the failure the sentence already warns against.

### The agent derives the name

The naming rule stays exactly as it is. What changes is who may satisfy it: where a
description carries the work, the agent derives a kebab-case name, states it, and proceeds
without waiting.

Stating it matters more than deriving it. A derived name the contributor never sees is a
decision taken on their behalf; a derived name announced in the report is a decision they
can correct in one sentence. `add-tally-counters` follows from the transcript's request in
one step, and a contributor reading it either agrees or renames the directory.

Rejected: asking the contributor to approve the name before creating anything. That is the
behavior under repair, one step smaller. A round trip to confirm a name that follows from
the description already given is the same refusal in a politer form.

### Guidance is the fix, not a check

There is no test to add here, and that is a decision rather than an omission. These files
are prose read by a model. A test can assert that the string "absence" appears in
`new.md`, which pins the wording and proves nothing about the behavior — the behavior under
repair was produced by an agent that had already reasoned its way to the right answer.

What the project can check mechanically, it already does: `every_invocation_names_its_workflow`
resolves every invocation these files name against clap's command tree, so the instructions
cannot send an agent to a command that does not exist. The correctness of authoring advice
is not that kind of property. The evidence for this change is a transcript, and the check is
a second transcript.

Recorded so the next reader does not add a keyword assertion believing it closes the gap.

## Risks

**An agent derives a bad name and proceeds.** A change directory is renamed by moving it,
and the report states the derived name, so a contributor sees it immediately. Weighed
against a refusal that cost two exchanges and produced nothing, a name worth renaming is
the cheaper failure.

**"What working looks like" reads as a request for acceptance criteria.** It is not — a
proposal is not the contract, and `artifact-kinds` requires validation to shape a proposal
in no way at all. The guidance says "the first time you see it" to keep it observational
rather than formal. If proposals start carrying requirement-shaped lists, the wording needs
another pass; that would be a drift toward the specification's job, which the same file
already warns against twice.

**Accepting an absence weakens the bar for a proposal.** The mitigation is that the absence
form is three specific questions rather than a permission. An agent that cannot answer who
wants it and what they cannot do now still lacks a need, and the guidance still says so.
