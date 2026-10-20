## Why

`establish-a-runbook` gives a procedure a place to live. It gives a run nowhere to go.

That is not an omission to tidy up later. **A runbook whose runs leave no record cannot answer the
question a runbook exists to answer.** The project's release runbook is the case: the reason it
exists at all is that the same lesson was learned at `0.10.0` and rediscovered at `0.12.0`, because
the fourth release's record was an archived `tasks.md` that binds nothing. A runbook with no run
records reproduces that exactly. The procedure improves and nobody can tell which releases ran
under which version of it.

**The harder half is that there is no single thing a run should record.** A cookie run records the
oven temperature it reached and how many cookies came out. A release run records the version it
advanced to, whether `scripts/check-release` passed, and how many installed files it regenerated. A
fixed set of log fields would be wrong for both — the release fields are meaningless to a baker and
the oven temperature is meaningless to a release.

So **the log is a contract each runbook states for itself**, and stating it belongs to the work of
authoring that runbook rather than to this program. That framing came from the contributor who
asked for run logging in the first place:

> I hadn't thought clearly enough about how different those logs might be per runbook so figuring
> that out should be part of the work that is done in the changespec for the runbook at creation
> time.

This project already has the vocabulary for "a stated obligation, and a record attesting to it."
A `verification` unit carries an `outcome` — `passed`, `failed`, `blocked`, `not-run` — a `method`
paragraph stating how the check was performed "precisely enough that someone else could repeat it",
and at least one reference. That is a log entry. `review/unverified-requirement` already walks a
document's requirements and reports each one no verification names. The machinery is here; nothing
points it at a runbook.

## What Changes

- **A runbook's specification states what its runs must record**, as ordinary requirement units
  with scenarios. This is the logging contract. It is written when the runbook is authored, because
  the person who knows a cookie run needs an oven temperature is the person writing the recipe.

- **A run record is a `tasks` document** under `htmlspec/runbooks/<name>/runs/`, carrying the
  copied procedure with its states advanced, plus one `verification` unit per log requirement. The
  `outcome` and the `method` paragraph are the recorded values. **This kind is forced rather than
  chosen:** with `kind=specification` the record fails `kind/task-in-specification` on every step,
  measured. A `tasks` document carrying requirements and verifications beside its tasks is
  accepted, because the tasks rule is completeness rather than placement.

- **A run record carries the requirements it verifies, copied from the runbook.** This looks like
  duplication and it is not a choice. **A reference resolves within one document only**, which I
  measured rather than assumed: a verification in `runs/batch-a.html` naming
  `req-record-oven-temp` in the runbook's `specification.html` fails with
  `refs/unresolved` — an error, not a warning, reading *"no unit in the document declares that
  identifier."* A run record that pointed outward would be invalid on the day it was written.

  Three further constraints came out of the same measurement, and the design has to satisfy all
  three: each copied requirement needs at least one `scenario` unit, its statement needs `SHALL` or
  `MUST` or `schema/weak-modal` reports it, and it needs `data-change="unchanged"` or
  `review/stale-verification` warns that the record attests to an obligation that has moved.

- **A project-level rule reports a run whose records do not satisfy its runbook's log contract.**
  Built the way `asset/duplicate` is built, in `crates/htmlspec-validate/src/rules/topology.rs`,
  whose module documentation states the reason a rule like this cannot be a per-document `Rule`:
  its subject is a project, and a per-document pass would report one violation once per document.
  A run's conformance is a fact about a runbook and its runs together, so nothing reading a single
  file can decide it.

- **The severity is `review/*`, so it never gates.** `crates/htmlspec-validate/src/diagnostic.rs`
  states the policy — `review/*` is "a warning that never gates work" and cannot be promoted under
  `--strict`. A run in progress legitimately has requirements nothing has verified yet, and a rule
  that failed the batch halfway through the bake is one an operator learns to skip.

**Not in scope:**

- **The aggregate.** "This runbook has been run 47 times" is derived from the run records, never
  stored, for the reason `project-bootstrap` refuses to read the last archived version out of a
  directory name. `record-a-runbooks-history`.
- **Scheduling or triggering.** Deciding *when* a runbook runs is a different question from
  recording that it did.
- **Judging whether a recorded value is correct.** A run stating it reached 200°F when the recipe
  says 375°F is a bad batch, not an invalid document. `artifact-kinds` reserves that reading for an
  agent.

## Capabilities

**New Capabilities:**

- `runbook-execution` — what a run is, where its record lives, what that record must carry, and
  that the record is history rather than contract. `contract-authority` already places "the output
  of any earlier run" among the non-normative, so this states where that output lives without
  reopening what it binds.

**Modified Capabilities:**

- `runbook-authority` — established by `establish-a-runbook`. Its subject gains the log contract:
  a runbook's specification states what its runs record, and a runbook that states nothing has runs
  nothing checks.

**Deliberately not modified:** `document-schema`, `artifact-kinds`, `document-validation`. No new
unit, no new kind, no schema change. The `verification` unit already carries an outcome, a method,
and its references, and this change points existing rules at a new subject rather than adding
vocabulary.

## Impact

- `crates/htmlspec-validate/src/rules/` — a new project-level module beside `topology.rs`, absent
  from `rules::ALL` for the reason that file already documents.
- `crates/htmlspec-cli/src/commands.rs` — the run operation, and where the project-level rule is
  invoked. `htmlspec assets` is the model: a project-subject rule needs a command that has a
  project in view.
- `crates/htmlspec-cli/src/main.rs` — the `run` subcommand.
- `fixtures/conformance/` — a conforming run record and one whose verification is missing, which is
  what makes this a contract rather than an implementation detail.
- **`htmlspec status` is a decision this change has to make rather than inherit.** It takes
  `--change` and reads only the changes directory, so pointed at a runbook it reports no change of
  that name. Whether run progress belongs in `status` or in the `run` operation's own report is
  open, and `workflow-reporting` governs the answer.

## The counter-argument

**A run record is a log file, and a log file is append-only text.** Every operational tool on the
machine writes runs to a line-oriented log, and this change proposes an HTML document per run with
copied requirements inside it. That is heavier, and it duplicates content the runbook already
states.

The weight is real and the answer is the one this project already gave when it rejected
`migration-plan.json`: the plan "is the document the contributor reviews, and a JSON file is not
reviewed, it is diffed." A run record is read when something went wrong, by somebody asking which
step was skipped and why. That is a reading task, and `schema/missing-reason` — an error, measured
— is what a line-oriented log cannot enforce: a skipped step with no stated reason is refused
rather than silently absent.

The duplication is the honest cost and it is worth stating twice. Forty runs means forty copies of
the procedure and of the log requirements. A runbook edited after run forty leaves the earlier forty
recording a contract that no longer exists — which is either an audit trail or dead weight
depending on facts this project does not have yet. `establish-a-runbook` names that as an open
finding and this change does not close it.

## Findings left open

- **The predecessor of this change is recorded in `.openspec.yaml`, not in a `FOLLOWS` file, and
  finding out why is worth recording.** `FOLLOWS` is what `crates/htmlspec-cli/src/project.rs`
  defines for an html-spec project. This repository's `openspec/` directory is an OpenSpec markdown
  mirror rather than an html-spec project, and writing `FOLLOWS` files into it fails
  `every_file_in_this_repository_classifies` with *"some files classify as no known kind"*. So the
  two mechanisms for one fact are not interchangeable, and the repository dogfooding OpenSpec cannot
  use the html-spec form. Named rather than solved; it is a fact about this repository's transition
  rather than about runbooks.

- **Whether a run can be resumed.** A batch abandoned at step 6 and picked up an hour later is one
  run or two, and the answer decides whether a run record is ever edited after it is written.
  Nothing here settles it.
- **Where run progress is reported.** Named under Impact above. `status` is the obvious home and
  its argument shape does not fit, so the choice needs an argument rather than a default.
- **The input gap `establish-a-runbook` records is adjacent to this change and not closed by it.**
  A run that consumed 1 cup of chocolate chips recorded a fact about its inputs, and until inputs
  exist as a notion there is nowhere for that to go but a `method` paragraph.
