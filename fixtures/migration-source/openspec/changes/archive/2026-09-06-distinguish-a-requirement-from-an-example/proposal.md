## Why

An agent writing this project's own migration change asserted that an accepted specification
lives at `htmlspec/specs/<cap>.html`, put it in a mapping table, and wrote a justification for
it: *"a directory holding one file is a directory OpenSpec needs and HTMLSpec does not."*

No requirement says that. The claim came from one line in `README.md`, four lines below:

> **Status:** early design proposal. The name, commands, and file layout are working ideas.

The justification was invented, and it was also false about the format it described.
`<capability-path>` is a path rather than a wrapper — the installed workflow guidance gives
`identity/user-auth` as an example and directs an agent to preserve a capability's full path.
So the flattening would also have discarded hierarchy the source expressed.

**The agent was not careless.** It was directed to work within the project's rules, it went
looking for the rule, it found a statement of the layout, and it used it. Nothing in the
seven workflow bodies asks the question that would have stopped it: *is this a requirement,
or is it an example?*

**Nothing in the guidance draws the line.** `grep -ril "normative"` across all seven
instruction bodies at `crates/htmlspec-assets/instructions/` returns nothing. `archive.md`
directs an agent to apply a change's contract changes to the accepted specifications.
`new.md` directs an agent to work within the rules that apply to every document. `explore.md`
says the accepted specifications state what is already promised. Not one of them says a
README is not a contract, or says what to do when no requirement exists at all.

**The contract states the rule for exactly one case.** `artifact-kinds` says a requirement
unit inside a proposal is valid and is not part of the contract, "and it is a statement about
authority rather than about validity." That is the right rule. It was never generalized past
proposals, so it reaches a requirement written in the wrong artifact and does not reach a
layout written in a README.

**The failure is quiet, which is what makes it expensive.** An agent that cites an example
produces a document that validates, reads as authoritative, and carries a reason. Every check
passes. Nothing in the pipeline separates it from a document that cites a requirement, and a
convention invented this way arrives with as many instances as the change had files.

**The project ships the same hazard inside its own templates.** `assets/template-specification.html`
writes eight example units into every new specification, with identifiers a reader would take
for real ones:

| Template | Example identifiers it writes |
| --- | --- |
| `template-specification.html` | `req-example`, `scenario-example`, `decision-example`, `risk-example`, `evidence-example`, `verify-example`, `figure-example` |
| `template-tasks.html` | `task-example` |
| `template-proposal.html`, `template-design.html` | none |

`new.md` says to replace them and no check enforces it. Validation reports 31 diagnostics
across seven namespaces — `asset/`, `kind/`, `refs/`, `review/`, `safety/`, `schema/`, and
`viz/` — and not one of them is about example content. The nearest miss is
`safety/live-example`, which reports a code sample carrying live markup instead of inert
text, and has nothing to do with a template's example units. So `req-example` can survive
archival into an accepted specification and become an obligation nobody wrote, with every
check green. That is the same defect as the README case, arriving through the project's own
front door.

**What working looks like the first time you see it.** An agent needs a rule for where an
accepted specification lives. It searches the accepted specifications, finds none, and writes:
*"The contract states no layout beneath `htmlspec/specs/`. The only instance is a README
example, which is not normative. I preserved the source's shape and did not decide the
question."* A contributor reads that and knows two things at once — what the agent did, and
that a real question is open. Neither was visible in the report they actually got.

## What Changes

- **The project states what its contract is.** A new `contract-authority` capability says the
  project's obligations are what its accepted specifications state, and names what does not
  state them: a README, a template, generated example content, a changelog, an archived
  change's proposal or design, and a comment in the code.

- **An assertion about the contract names the requirement it rests on.** Where an agent
  states that the project requires something, it names the requirement. Where it cannot, it
  says the contract is silent. The obligation is on the assertion, not on every sentence —
  the same shape as the pointer-gloss rule, which fires on the claim and not on the
  annotation.

- **Silence is recorded rather than filled.** Where a decision needs a rule the contract does
  not state, an agent takes one of three routes: preserve what already exists, make the
  smallest reversible choice and mark it as its own, or name the change that would settle the
  question. What it must not do is present its own choice as the project's rule.

- **Example content is non-normative, and a document that keeps it is reported.** A new
  diagnostic reports a unit carrying a template's example identifier, and archival refuses
  while a change's specification carries one. The identifiers are read from the compiled
  templates rather than listed a second time, because a list beside the templates drifts from
  them.

- **The obligation is stated once and carried identically by every body.** A test pins the
  wording across all seven, the way the pointer rule's wording is already pinned. Seven
  independently-written statements of one rule is the drift this project measured and repaired
  one release ago.

## Non-Goals

- **Promoting the README.** The repair is not to make the README normative. Its status line
  is honest, and a design proposal that quietly became a contract is worse than one that says
  it is a working idea.

- **Re-opening `artifact-kinds`' authority statement.** The statement there is correct and
  stays as written. This change states the general rule that the specific one is an instance
  of, and it does not restate the specific one.

- **Requiring a requirement behind every decision.** Most decisions an agent takes are its
  own and always were: what a paragraph says, which alternative to reject, what to name a
  file. The obligation fires when an agent asserts the *project* requires something.

- **Checking that a cited requirement is the right one.** Whether a requirement supports the
  claim made from it is judgment, and no check reaches it. This change checks that a
  requirement was named, not that it was read well.

- **A citation format.** No footnote machinery, no identifier syntax, no reference block. An
  agent names the requirement in the sentence that makes the claim, in whatever words fit.

- **Fixing the layout question.** Whether an accepted specification lives at
  `htmlspec/specs/<cap>.html` or `htmlspec/specs/<capability-path>/spec.html` is a real open
  question and it is a different change. This one is about how an agent behaves when it finds
  the question open.

- **Reaching a contributor's own writing.** A person who says "I think specs should be flat"
  is stating a preference and needs no citation. The obligation is on an agent asserting what
  the project already requires.
