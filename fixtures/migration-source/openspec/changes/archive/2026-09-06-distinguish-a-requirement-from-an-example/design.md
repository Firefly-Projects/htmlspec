## Context

An agent looking for the project's rules meets four kinds of text, and only one of them is
the contract:

| Where it lives | What it is | Binding |
| --- | --- | --- |
| `openspec/specs/<cap>/spec.md` | Accepted specifications, 79 requirements across 8 capabilities | **Yes** |
| A change's `specs/<cap>/spec.md` | What that change will establish, until archival applies it | Not yet |
| `README.md`, `CHANGELOG.md`, an archived proposal or design | Description, history, argument | No |
| A template's example units, a doc comment | Shape and explanation | No |

Nothing states this table. `artifact-kinds` states one row of it — a requirement unit in a
proposal is not part of the contract — and the seven installed workflow bodies state none of
it. An agent therefore decides, per encounter, which text is authoritative, and it decides
without being asked to.

The defect is not that an agent guesses wrong sometimes. It is that guessing is invisible.
A cited example and a cited requirement produce identical artifacts, and both pass every
check, so the project learns about the guess when someone asks the question this change
exists to make routine.

## Goals

- An agent that asserts what the project requires can be checked against the requirement.
- An agent that finds no requirement says so, rather than supplying one.
- The one hazard the project ships itself — example content inside the authoritative kind —
  is caught mechanically.

## Non-Goals

- Promoting the README, re-opening `artifact-kinds`, a citation format, requiring a
  requirement behind every decision. See the proposal.

## Decisions

### A new capability, because the subject is authority rather than kinds

`artifact-kinds` governs the four documents a change holds: what each argues, what validation
enforces, what archival does with them. Its authority statement is scoped to that subject —
a requirement unit is valid in a proposal and not binding there.

This change reaches text `artifact-kinds` has no business governing. A README is not an
artifact. A template's example unit is not a document. A changelog entry is neither. And the
central obligation here is on an **agent's assertion**, which is not a document at all — it
is the same kind of ephemeral output that `workflow-reporting` was created to govern, for the
same reason: an obligation about prose nobody archives does not belong inside the capability
that defines archived documents.

So `contract-authority` is a new capability. `artifact-kinds`' statement stays exactly as
written and remains true. This change does not restate it, because a rule stated in two
capabilities is the drift the last release measured and repaired.

### The obligation fires on the assertion, not on the sentence

The rule is *an assertion about what the project requires names the requirement*. It is not
*every claim carries a citation*.

That boundary matters, and the project already has the machinery for drawing it. The
pointer-gloss rule in `workflow-reporting` is bounded by a deletion test: remove the pointers,
and if the claim still stands they were annotation and owe nothing. The same test works here.

- **Owes a citation.** "Accepted specifications live at `htmlspec/specs/<cap>.html`." This
  states what the project requires. Nothing else in the sentence carries the claim.
- **Owes none.** "I put the proposal first because the design depends on it." This is the
  agent's reasoning about its own work. No project obligation is asserted.
- **Owes none.** "`htmlspec validate` reported two errors." A fact about a run.

Without the boundary the rule fires on everything an agent writes, which produces documents
padded with citations and an agent that learns to ignore the rule. Worked against the failure
that prompted this change: the mapping table's destination column asserts where an artifact
lives, so every row owes either a requirement or a statement of silence. The design's prose
about which paragraph becomes a diagram owes neither.

### Silence has three answers, and inventing a rule is not one of them

An agent that finds no requirement has three routes, and stating them is what keeps "the
contract is silent" from becoming a way to stop working:

1. **Preserve what exists.** Where something already has a shape, keep it. This is what the
   migration change now does with capability paths.
2. **Choose, reversibly, and own it.** Where nothing exists and something must be picked, pick
   the cheapest-to-change option and say the choice is the agent's. `new.md` already directs
   this for a derived change name, so the pattern is established.
3. **Name the change that would settle it.** Where the question is large enough that a
   reversible choice would mislead, record it as an open question with the change that closes
   it.

What is excluded is the fourth route, which is what happened: choose, then write a
justification that reads as though the project had already decided.

### The example-content check is the one mechanical part, and it earns its place

Four of this project's last five releases ended with a central obligation no test could reach,
and each wrote that down. This change has one obligation a test *can* reach, so it takes it.

`assets/template-specification.html` writes seven example units and `template-tasks.html`
writes one. Their identifiers — `req-example`, `scenario-example`, `decision-example`,
`risk-example`, `evidence-example`, `verify-example`, `figure-example`, `task-example` — are
a closed set, they are compiled into the binary, and no legitimate document wants one.

The check therefore reads the identifiers **out of the templates** rather than from a list
beside them. A hardcoded list is a second statement of the same fact, and it goes stale the
first time a template gains a unit. This is the same reasoning that put the workflow set in
one table and the pointer rule in one constant.

**Warning at validation, error at archival.** A new document legitimately carries example
content the moment it is created — `htmlspec new document` writes the template and the
contract requires the result to be valid, so an error at creation would make the program
report a defect it created itself. But a change's specification reaching archival with
`req-example` in it is an obligation nobody wrote entering the accepted contract, and that is
exactly what archival exists to stop. So `htmlspec validate` warns, and `htmlspec archive`
refuses.

The alternative considered was a check on the example *text* rather than the identifier. It
was rejected: text is edited while an identifier survives, so a half-replaced example — new
prose under `req-example` — passes a text check and fails an identifier check. The
half-replaced case is the likely one.

### Every body carries the wording, and a test pins it

An agent reads one instruction file and no other, so each body must be self-contained. That
forces the rule to be repeated seven times, and repetition is what produced the report-section
divergence one release ago: `apply.md` at six artifact-subjected lines against `explore.md` at
two.

The existing repair is the precedent. `workflow-reporting`'s pointer obligation lives in one
constant, `POINTER_RULE`, and a test asserts its wording is identical across all seven bodies.
This change adds a second constant the same way. Repetition stays, drift does not.

### Risk: an agent that cites a requirement that does not support the claim

**Accepted, and out of scope by design.** A check can see that a requirement was named. It
cannot see whether the requirement supports the claim, because that is reading. An agent that
cites `project-bootstrap`'s statement about "each in its own place" as authority for a flat
filename has satisfied the letter and misread the requirement.

The mitigation is that a named requirement is *checkable by a person in seconds*, where an
invented rule with a confident justification is not checkable at all. That is the whole gain,
and it is worth stating as a partial repair rather than overclaiming.

### Risk: "the contract is silent" becomes an excuse to stop

**Mitigated by the three routes.** An agent that reports silence and does nothing has not
finished. The requirement names what follows silence, so reporting it is a step rather than
an exit.

### Risk: the assertion rule cannot be checked mechanically

**Accepted, and stated for the fifth time in five releases.** Whether an agent's sentence
asserts a project obligation is a judgment, and no test reads an agent's prose. What this
change adds that the previous four did not is a bounded test an agent can apply to its own
sentence — delete the claim's subject and see whether a project obligation was stated — plus
one obligation in the set that *is* mechanically checked. Neither makes the first rule
checkable. Both make the release something other than a fifth honest note about the same
limit.

## Risks

- **A cited requirement that does not support the claim.** Accepted above. Partial repair,
  stated as partial.
- **Silence used as an exit.** Mitigated by naming the three routes that follow it.
- **The assertion rule is not checkable.** Accepted above.
- **An eighth workflow added later without the wording.** Mitigated by the pinning test, which
  fails on a body that lacks it.
