## Context

Seven instruction bodies under `crates/htmlspec-assets/instructions/` each end with a
`## What to report` section. They were written one at a time, and they diverged: `explore.md`
carries two artifact-subjected lines and `apply.md` carries six. Nothing in the accepted
contract governs a report at all — `project-bootstrap` governs how instruction files are
installed, and `artifact-kinds` governs what each document must contain. A report is neither,
which is why no requirement reaches it and why seven independently-written sections were
never reconciled.

The failure this repairs is not an agent behaving badly. The received report follows
`new.md`'s five bullets faithfully. It is the instruction that is wrong, in the same shape as
the two changes before it: guidance that produces a bad outcome when followed exactly.

## Goals

- A contributor reads a report and knows whether the agent understood them.
- A pointer in a report names what it points at, so a report is readable without opening the
  artifacts.
- One statement of both obligations, applying to every workflow.

## Non-Goals

- A report template. See the proposal.
- Importing the explainer's sentence caps or modal ladder.

## Decisions

### A new capability rather than an extension of `artifact-kinds`

`artifact-kinds` governs documents: what a proposal must argue, what a specification may
carry, what validation reports about each kind. A report is not a document. It is not
written to a file, not validated, not archived, and not read by the next agent. Putting an
obligation about ephemeral prose inside the capability that defines persistent artifacts
would make that capability about two unrelated things.

`workflow-reporting` is the name, and it holds both obligations plus what follows from them.
The seven bodies then conform to one statement rather than restating it seven ways — and the
divergence measured in the proposal is the argument: the same rule written seven times
already drifted, before anyone wrote it down at all.

### Register, not sequence

The obligation is that a report's **first claim is stated in the contributor's vocabulary**.
It is deliberately not "state the outcome, then the artifacts", which was the alternative
considered and rejected.

That alternative fails on `explore`. Explore produces no artifact and changes no file — its
whole stance is that work done before deciding what the work is gets thrown away. A two-slot
shape of "what changed, then where it lives" has nothing to put in either slot for a
workflow whose output is a conclusion. Forcing one would produce "nothing changed, nothing
was written", which is worse than the current wording.

Register generalizes where sequence does not:

| Workflow | The contributor's terms |
| --- | --- |
| `explore` | What the question turned out to be, and what the answer is |
| `new`, `ff` | What they are getting, as the working sequence describes it |
| `continue` | What the change can now do that it could not before |
| `apply` | What now works, and what a contributor can try |
| `verify` | Whether the change is sound, and what is wrong if not |
| `archive` | What the project now guarantees |

Each is that workflow's own subject. None is a file.

### The gloss rule is imported with its test, not without it

The pointer obligation comes from this project's explainer standard, which states it as:
gloss every pointer on first use, naming in plain words what it points at, in the same
sentence. It arrives with the **deletion test** that bounds it — delete every pointer from
the sentence, and if the claim still stands, no gloss is owed.

Importing the rule without the test would be the error the explainer standard explicitly
guards against. A rule that fires on every identifier makes an agent gloss
`htmlspec validate` and `--json`, which name their own referents. The test is what keeps the
obligation on the references a reader genuinely cannot resolve.

Worked against the received report:

- **Owes a gloss.** "1. `task-test-harness` — first because every later correctness claim
  rests on it. 2. state module → 3. mark grouping → 4. statistics." Delete the pointers and
  nothing survives. Each identifier is the claim.
- **Owes none.** "`htmlspec validate`: 0 errors, 8 warnings." Delete `htmlspec validate` and
  the sentence collapses, but the pointer is self-describing — it names the command that
  produced the count. The `8 warnings` figure is where the gloss is owed, because a count
  without a kind tells a reader nothing they can act on.

That second case is the useful one. The received report did gloss its warnings
(`review/stale-reference`, with an explanation), and that part was correct. The rule must
not fire on prose that already satisfies it.

### Surfacing existing prose, rather than writing new prose

The cheapest part of the repair. `artifact-kinds` already requires a proposal to state what
working looks like, and the tally proposal contains a concrete sequence naming Robin,
Sparrow, and Crow. A report that quotes or compresses that sentence answers "what am I
getting" at no additional cost in judgment.

So the obligation is written as *surface what the artifacts already state in the
contributor's terms* rather than *write a summary*. An agent asked to summarize invents; an
agent asked to surface quotes. The material is already there because an earlier change
required it, and this change connects the two.

### Risk: an agent that pads a report to satisfy the register rule

**Accepted, and mitigated by one sentence in the contract.** An obligation to lead in the
contributor's terms can be read as license to add warmth, and a report that opens with
"Great news! Your tally app is coming along nicely!" satisfies the letter and fails the
purpose.

The contract states that a report's first claim is a **fact about what the contributor
asked for**, and that persuasion and reassurance are not reports. The existing bodies are
already written in a plain register throughout, so the amendment is a reordering of subjects
rather than a change of tone, and nothing in this change asks for a different voice.

### Risk: the register rule cannot be checked mechanically

**Accepted; stated because the last three changes each turned on what a check could see.**
Whether a report's first claim is in the contributor's vocabulary is a judgment. No test
asserts it, and the check is a transcript, exactly as it was for the guidance repairs in
0.3.0 and 0.4.0.

The gloss obligation is nearer to checkable — an unglossed bare identifier in a report is
recognizable — but a report is not a file this program writes, so there is nothing to assert
against. This is a defect class whose only verification is reading a real report, and the
tasks say so rather than implying a test could cover it.

## Risks

- **An agent that pads a report.** Mitigated above by stating that a report's first claim is
  a fact.
- **No mechanical check.** Accepted above. Verification is a transcript.
- **Seven bodies drifting again.** Mitigated by stating the obligations in one capability
  rather than seven times, which is the same reasoning that put the workflow set in one
  table.
