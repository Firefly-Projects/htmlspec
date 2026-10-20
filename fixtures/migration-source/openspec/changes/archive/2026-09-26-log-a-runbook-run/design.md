## Context

The proposal settles what a run record *is*: a `tasks` document under
`htmlspec/runbooks/<name>/runs/`, carrying the copied procedure with its states advanced plus one
`verification` per log requirement, checked by a project-level rule at `review/*` severity. It
settles that shape by measurement rather than by preference, and re-measuring confirmed all four
claims with no correction — `evidence/the-shape-is-forced-2026-09-18.md`.

It leaves this design three decisions and names them: where run progress is reported, whether a run
can be resumed, and whether the `run` operation copies eagerly or lazily and what it names the
record. All three are settled below with their reasoning.

Four existing facts shape everything that follows.

- **`review/unverified-requirement` cannot see a run record.** Its rule body returns early unless
  the document's kind is `specification`, and measurement 2 forces a run record to be `tasks`. So
  the two forced facts point in opposite directions, and **a run record whose copied requirement no
  verification names reports nothing at all, at either strictness** — measured. That gap is the
  conformance rule's whole subject, and without it this change would state a contract nothing
  checks, which is the shape `project-bootstrap` names.
- **A reference resolves within one document only.** Measured again here: a verification in
  `runs/<record>.html` naming a requirement in the runbook's `specification.html` fails
  `refs/unresolved`, an *error*. The copying is forced.
- **`ProjectRoot` is the only thing that knows the layout**, and `RUNBOOKS` is already a private
  constant there with `runbooks()` and `runbook(name)` as its accessors. A fourth accessor joins
  them rather than any caller composing `runs`.
- **The guard covers `runs/`, verified with a record on disk.** Six routes and three known bypasses,
  all refused, runbook intact — `evidence/the-guard-covers-runs-2026-09-18.md`.

## Goals / Non-Goals

**Goals**

- A runbook's specification can state what its runs must record, as ordinary requirement units.
- A run record exists at `htmlspec/runbooks/<name>/runs/<record>.html`, created by an operation, and
  validates the moment it is written.
- A project-level rule reports a run record that does not satisfy its runbook's log contract, at
  `review/*`, absent from `rules::ALL`.
- `fixtures/conformance/` pins the conforming record and the one whose verification is missing.
- The three decisions the proposal leaves open are settled and stated.

**Non-Goals**

- Machine requirements and preflight. `carry-a-runbook-to-another-machine`.
- The aggregate — "this runbook has been run 47 times". `record-a-runbooks-history`.
- Scheduling, triggering, or judging whether a recorded value is *correct*.
- A workflow or a `/htmlspec:runbook` command. That fan-out takes the installed files from 45 to 50
  and is its own change; everything below is reachable from `run`, `validate`, and `list`.
- A new artifact kind, a new unit, or a schema change. The kind enum's guard test stays untouched.
- **Inputs.** Hit again, recorded again, not invented. See the last section.

## Decisions

### A run record is a `tasks` document at `htmlspec/runbooks/<name>/runs/<record>.html`

```
htmlspec/runbooks/<name>/
  specification.html       what it is for, what its history taught, and what a run must record
  tasks.html               the ordered procedure, states unstarted forever
  runs/                    the records, one file per run                             (new)
    <record>.html          kind=tasks: the procedure advanced, plus the log
```

A flat directory of files rather than a directory per run. A run is one document, and a directory
holding one file is a directory a reader opens twice. The `runs/` directory itself is named in
`project.rs`:

```rust
/// Where a runbook keeps the records of its runs.
const RUNS: &str = "runs";

impl ProjectRoot {
    /// Where the records of a runbook's runs live.
    pub fn runs(&self, runbook: &str) -> PathBuf { self.runbook(runbook).join(RUNS) }
}
```

Private constant, public accessor, exactly the shape the other four have. **Nothing composes
`"runs"` outside this module**, which is what makes the location one statement rather than several
that agree until the day it moves.

**`runs/` is created by the `run` operation rather than at bootstrap**, and this is the one place
this change departs from the predecessor's reasoning. `establish-a-runbook` argues that
`htmlspec/runbooks/` is created at bootstrap because *a place that does not exist until something is
in it is not a place a contributor can find* — and that argument is about the project's places, of
which there are four and which a contributor looks for. `runs/` is not a fifth place; it is
per-runbook, so creating it at bootstrap is impossible (there is no runbook yet) and creating it when
a runbook is created would mean the create-a-runbook operation this change does not have. The
operation that writes a record creates the directory, with `create_dir_all`, which is idempotent.

**Asset references resolve at this depth with nothing added**, measured: `htmlspec new document
tasks htmlspec/runbooks/release/runs/probe.html` writes `href="../../../assets/sha256-…"`, three
levels, correctly. The store is derived from the project, so a run record shares the one store and
`asset/duplicate` has nothing to report.

### Decision 1: run progress is the `run` operation's own report, and `status` is left alone

**`htmlspec status --change` keeps its subject. Run progress is reported by `htmlspec run`, which
has a runbook in view.** `htmlspec list --runbooks` gains nothing.

The proposal says this is a decision this change must make rather than inherit, and
`the-status-gap-2026-09-17.md` measured the gap precisely enough that the decision follows from the
measurement. What it found, in a scratch project:

- `status --change chocolate-chip-cookies` — refused, exit 2. A name resolves against
  `project.changes()` and nowhere else.
- `status --change htmlspec/runbooks/chocolate-chip-cookies` — **worked**, and the proposal was
  wrong to say it would not. `resolve_change` accepted the directory and the counting logic ran
  correctly: `0 of 9 task(s) done or dropped`.

That second reading is what decides it. The counting logic is reusable and reaches a runbook today;
what is wrong is the *shape of the report around it*. `status` reports against `CHANGE_DOCUMENTS`
plus `CHANGE_SPECIFICATION`, so it says `absent proposal.html` and `absent design.html` about
documents a runbook never has. Teaching it a second shape means a branch on what its argument turned
out to be, and then the operation's name — `--change` — is false for half its inputs.

**`workflow-reporting` governs, and it decides against the branch.** *A report answers the
contributor in their own terms*, and its scenario: a report whose first claim names a directory or a
document count is wrong, because none of that tells the contributor whether their request was
understood. A contributor who asks about a run asks "did the release run, and what did it record" —
not "which of four documents exist". The `run` operation's own report can answer in those terms
because it knows it is reporting on a run; `status` cannot, because it would have to discover which
question it was asked.

There is a second reason, and it is the stronger one. **The report a run needs is not a completion
fraction.** A run record's tasks *are* advanced, so the fraction is meaningful where the canonical
procedure's is not — but the fraction is not what a reader of a run wants. They want the outcomes:
which log requirements this run verified and what it recorded. That is a different report, produced
from different units, and it is the conformance rule's output. So the two reports are not one report
with a branch; they are two reports, and the run one does not exist yet.

`status` is therefore **unchanged**, and its refusal of a runbook path stands as the predecessor
built it. The guard already refuses `runs/` too, measured. Note what this leaves open and does not
close: **`status --change` pointed at a runbook by name is still a refusal**, and that is still the
form a contributor or a workflow would reach for. Recorded rather than narrowed, on the same terms
the predecessor recorded it.

### Decision 2: a run can be resumed, and a record is therefore edited after it is written

**A run is one record from start to finish, and picking it up an hour later continues that record
rather than starting a second one.**

The proposal names this open and says the answer decides whether a run record is ever edited after it
is written. The answer is yes, and the reasoning is that **the alternative contradicts the
document's own kind.** A `tasks` document's task units carry four states — `todo`, `in-progress`,
`done`, `dropped` — and `in-progress` exists to describe work that is under way and not finished. A
record written once and never touched could never carry `in-progress` on a step honestly: it would
mean "this step was in progress at the instant the file was written and nobody knows what happened
next". A model in which that state is unreachable is a model that should not have four states.

The batch abandoned at step 6 is exactly the case. **Two records would each be wrong about the
batch:** the first says six steps done and four `todo`, which reads as a run that stopped; the second
says four steps done and six what? It cannot say `done`, because this record did not do them, and it
cannot say `todo`, because they are finished. The step states in a copy of a procedure are a fact
about the work, and splitting one piece of work across two documents makes both of them lie.

**The cost, stated rather than discovered later.** A record that is edited is a record whose history
is its own file's history — git, here — rather than the record. Two runs a week apart look identical
on disk to one run interrupted for a week. Nothing in this change distinguishes them, and nothing
needs to yet: the question "when did this run happen" is `record-a-runbooks-history`'s, which is
also where the aggregate lives. **This change deliberately records no timestamp**, for
`project-bootstrap`'s reason about a version no gate compares: a recorded time nothing reads is
accuracy without authority, and the filesystem already has `mtime`.

The consequence for the conformance rule is what makes this load-bearing rather than philosophical:
**a run in progress legitimately has log requirements nothing has verified yet**, which is exactly
why the severity is `review/*` and cannot be promoted under `--strict`. If a run were immutable, an
incomplete record would be a permanent defect and an error would be arguable. Because it is resumed,
the report must never gate — an operator whose batch failed the rule halfway through the bake is an
operator who learns to skip the rule.

### Decision 3: `run` copies eagerly, and names the record from the contributor

**`htmlspec run <runbook> --as <record>` copies the whole procedure at the moment the run starts,
and the record's name is the contributor's to give.**

**Eagerly, because a lazy copy has no honest content for the steps it has not reached.** A record
that grew a task unit as each step was worked would, between steps 2 and 3, be a document stating a
three-step procedure — and a reader opening it would see a procedure, not a partial one, because
nothing in a `tasks` document says "there is more". The whole point of copying the procedure rather
than referencing it is that the record shows which version of the procedure ran; a partial copy
shows a version that never existed. Eager copying also means the record validates the moment it is
written, which is `document-template`'s standing requirement for anything the program creates, and
which `new_document` already enforces by validating what it wrote.

**The name is the contributor's, with no default.** Three candidates were available:

1. **A date.** `2026-09-18.html`. Rejected: two runs in one day collide, and `project-bootstrap`
   refuses to read a version out of a directory name for the same family of reason — a name that
   encodes a fact is a fact nothing enforces. It would also make the run record's name the one
   place a time is recorded, which decision 2 declines to do.
2. **A sequence number.** `run-7.html`. Rejected for the sharper form of the same objection:
   `runbook-authority` states that nothing SHALL derive an ordering, a version, or a history from a
   runbook's name, and deriving the *next* number means reading an ordering out of the directory's
   contents. That is the maintained-list shape the contract refuses.
3. **The contributor's own word.** Chosen. `--as 0.14.0` for a release run, `--as batch-a` for a
   batch of cookies. The name is what the run is *about*, which is the only thing a selector can use,
   and it is the same discipline a change's name already has: the name is the directory entry and
   nothing else records it.

`--as` is **required**, not defaulted. An operation that invented a name would be choosing on the
contributor's behalf, and `artifact-kinds` requires a derived name to be reported so a contributor
who disagrees can say so — which for a name means asking rather than reporting after the fact. The
name is checked by `ProjectRoot::is_a_name`, which is the constructor the predecessor moved this
invariant into after re-litigating it at three call sites; a `--as ../../sneaky` is refused there.

**`run` refuses to overwrite an existing record.** Measured, on `new document`, because it is the
same property: `already exists; new will not overwrite a document`, exit 2. A run that overwrote a
previous run's record would destroy history, which is the family of defect this whole line of work
exists around. **Resuming a run is therefore editing the file, not re-running the operation** — the
operation starts a run, and an agent or a contributor advances the states. That is the same division
as a change: `new change` writes the documents once and the work happens in them.

### The log contract is requirement units in the runbook's specification

Nothing new. A runbook's `specification.html` states what its runs must record as ordinary
requirement units with scenarios, alongside the hazard requirements
`establish-a-runbook` put there. **The two are not distinguished in the markup, and that is
deliberate:** a hazard the procedure met and a value a run must record are both "what this runbook
requires of a run", which is the sentence `runbook-authority` already uses. Distinguishing them
would need a new attribute or a new unit, and the proposal forbids both.

The consequence is that **every requirement in a runbook's specification is a log requirement**, so
the conformance rule's subject is all of them. That is the right reading rather than a simplification:
`req-version-before-regenerate` states that a run MUST advance the version before regenerating, and a
run record attesting to that is exactly what the archive's five copies of this procedure never had.

### A run record *is* a canonical document, so `unchanged` binds — and the reason is not filing

`runbook-authority` requires that **a unit in a runbook's canonical documents SHALL declare that it
is unchanged**, and the proposal instructs this design to decide whether a run record is one of
those documents. It is, and the measurement is what decides it rather than the word "canonical".

Re-measured here: a copied requirement declared `added` rather than `unchanged` draws
`review/stale-verification` — *"this verification records the outcome passed against
req-version-before-regenerate, which this change marks added"*. The rule is right. **A run record's
verification records an outcome against an obligation that existed before the run started**, which is
the same fact as the runbook's own documents arriving at the same attribute, and for the same reason:
a change state describes a unit's relationship to a change in progress, and a run is not a change.

So the delta states it once, for a runbook's documents *including* its run records, rather than
adding a second clause. The `run` operation writes `unchanged` because it knows what it is writing —
which is the thing `the-canonical-documents-change-state-2026-09-17.md` says `new document` cannot
know and a workflow would. **`run` is that operation for this one case**, so the finding is narrowed
by this change rather than closed: an author writing a run record by hand still meets the warnings,
and an author writing a *runbook* by hand still does too.

### The conformance rule is project-level, beside `topology.rs`, absent from `ALL`

A new module, `crates/htmlspec-validate/src/rules/runs.rs`, built the way `topology.rs` is built:
a plain function over what a caller gathered, returning `Vec<Diagnostic>`, not a
`crate::engine::Rule`.

```rust
/// A run record does not satisfy its runbook's log contract.
pub const UNRECORDED: RuleId = RuleId::new(Namespace::Review, "unrecorded-requirement");

/// What one runbook and its runs state, gathered by the caller.
pub struct RunbookUnderReview<'a> {
    /// The runbook's name, for the report.
    pub name: &'a str,
    /// Every requirement the runbook's specification states, by identifier.
    pub required: &'a [String],
    /// Each run record: its own name, and the requirements its verifications name.
    pub runs: &'a [RecordedRun],
}

pub fn conformance(subject: &RunbookUnderReview<'_>) -> Vec<Diagnostic>;
```

**The signature is the design.** It takes gathered facts rather than paths, so the walk stays in the
caller and the rule is a pure function — which is `topology::duplicates_across`'s exact shape, and
its doc comment states why: *"Takes `(location, digest)` pairs already gathered from every store, so
the walk stays in the caller and this stays a pure function over what the walk found."* `AGENTS.md`
§6 is the governing clause: the rule needs no filesystem capability, so it is not given one.

**Absent from `rules::ALL`** for the reason `rules/mod.rs` documents about `topology`: the subject is
a project rather than a document, and a per-document pass would report one violation once per
document. Here it is sharper than for assets — the rule's subject is *a runbook and its runs
together*, and no single file can decide it: the runbook's specification states the requirements and
the run records state the outcomes, and neither document alone knows whether the contract was met.
`every_rule_in_all_is_a_document_rule` keeps the two kinds from being confused, and the signature
above cannot be coerced into a `Rule` so the compiler refuses the mistake first.

**Severity is `review/*`, which is the namespace rather than a choice per rule.**
`diagnostic.rs` states the policy: `review/*` is a warning that never gates and `is_promotable`
returns false for it, so `--strict` cannot promote it. Decision 2 is why that is required rather
than convenient: a run is resumed, so a record legitimately has unverified requirements while the
work is under way.

**What it reports, and what it deliberately does not.** One diagnostic per (run, unverified
requirement) pair, naming both, because a reader told "this run is incomplete" has to find which
requirement. It does **not** report:

- a runbook with no runs. A procedure nobody has run yet violates nothing, and reporting it would
  fire on every runbook from the moment it is created — which is the `0/9` column the predecessor
  refused to print, in diagnostic form.
- a run record whose verification records `failed` or `blocked`. **The obligation is that the run
  recorded an outcome, not that the outcome was good.** A batch that reached 200°F when the recipe
  says 375°F is a bad batch and a perfectly valid record; the proposal reserves that reading for an
  agent, and `artifact-kinds` reserves judgment generally. A rule that reported `failed` would
  convert a run's honest record of a failure into a document defect, which teaches an operator to
  record `passed`.
- a run record verifying something the runbook does not require. That is a run that recorded more
  than it owed, which is not a fault.

### It is invoked by `htmlspec run --check`, which is where a project is in view

`htmlspec assets` is the model the proposal names, and it is the right one: a project-subject rule
needs a command that has a project in view. Rather than a second top-level command, the invocation is
a mode of the operation that owns the subject:

- `htmlspec run <runbook> --as <record>` — starts a run: copies the procedure, writes the log
  requirements and one `not-run` verification per requirement, reports where the record landed.
- `htmlspec run <runbook> --check` — writes nothing, gathers the runbook and its records, evaluates
  the conformance rule, and reports. Exit zero always, because the rule never gates.

**`--check` rather than a `runs` subcommand**, matching `archive --check` and
`migrate promote --check`, both of which mean "apply every condition and perform none of them". The
vocabulary already exists and a second spelling would invent a second one.

The reason it is not a flag on `validate`: `assets`'s own doc comment states it exactly — folding a
project-subject check into `validate` means `htmlspec validate one-document.html` either reports a
violation the named document did not cause, or silently skips a check depending on the argument, and
*a check whose presence depends on how the tool was invoked is the failure mode this change exists to
remove.*

## Risks / Trade-offs

- **Forty runs means forty copies of the procedure and of the log requirements.** The proposal states
  this as the honest cost and it is not closed here. A runbook edited after run forty leaves the
  earlier forty recording a contract that no longer exists, which is either an audit trail or dead
  weight depending on facts this project does not have. `establish-a-runbook` names it open and this
  change does not close it.
- **The conformance rule is reachable only from `run --check`.** So a project that never runs that
  command has the contract and no report of it — the same standing as `asset/duplicate`, which
  `scripts/check` runs and nothing else does. What closes it for *this* repository is a check stage,
  and this change does not add one, because this repository keeps its runbook at `runbooks/` rather
  than `htmlspec/runbooks/` and `run` resolves a runbook through the project. **Stated rather than
  discovered:** the rule is dogfooded in a scratch project and pinned by fixtures, not by this
  repository's suite.
- **`status --change` pointed at a runbook by name is still a refusal.** Decision 1 declines to
  change it and says why. A contributor who wants a run's progress uses `run --check`, and a
  contributor who guesses `status` gets a refusal naming `htmlspec list --runbooks`.
- **A resumed run is indistinguishable on disk from a run that happened all at once.** Decision 2
  accepts this and declines to record a timestamp. `record-a-runbooks-history` is where a time would
  earn itself.
- **Every requirement in a runbook's specification is a log requirement**, because nothing
  distinguishes a hazard from a recorded value. Accepted, and it is the contract's own sentence — but
  it means a runbook whose specification states ten hazards obliges every run to record ten
  outcomes, and a runbook author who wanted five obligations and five notes has no way to say so. The
  mechanism that would distinguish them is a new attribute, which the proposal forbids.
- **`run` writes a document from gathered units rather than from a template**, which is a shape
  `new_document` does not have. The mitigation is the one `new_document` already uses: validate what
  was written before reporting success, and fail loudly if the program produced an invalid document,
  because that is a defect in this program rather than in a contributor's project.

## Migration Plan

Additive throughout. No existing document changes meaning, no fixture changes state, no diagnostic is
reworded, no existing command's output changes. `status`, `sync`, `archive`, `list`, `validate`, and
`format` behave exactly as they did — `validate` and `format` descend into `runs/` because they
already descend into everything, which is measured and is the wanted behaviour.

`document.schema.json`, the artifact-kind enum, `TEMPLATES`, and `assets/vendor.lock.json` are
untouched. `rules::ALL` is untouched, which the proposal states as a checkable claim.

A project with no `runs/` directory is a project whose runbooks have not been run, and every
operation reports that as none rather than as a fault — the same standing `runbooks()` itself has for
a project bootstrapped before runbooks existed.

## The findings this change hit and did not close

**Inputs, again.** A run that consumed 1 cup of chocolate chips has nowhere to record that but a
`method` paragraph on a verification, which is prose. The proposal says so, `establish-a-runbook`
says so, and building the run record hit it a third time from a new angle: **the `method` paragraph is
where a recorded *value* goes, and it is the only place.** A release run recording "advanced to
0.14.0, regenerated 45 files, `check-release` passed" puts three values in one prose paragraph, and
nothing can read one of them out. That is the shape of the gap stated more precisely than before, and
it is still not this change's to fill: the answer needs its own argument, and a machine requirement
and an ingredient touch each other in a way `carry-a-runbook-to-another-machine` also records.

**The run record is invisible to `list --runbooks`.** The enumeration reads which of
`RUNBOOK_DOCUMENTS` a runbook holds, and `runs/` is neither of them, so a runbook with forty records
lists identically to one with none. Measured. Not closed, and the reason is decision 1: a count of
runs is the aggregate, which is `record-a-runbooks-history`'s. What `list --runbooks` owes is what a
selector needs to choose between runbooks, and "has been run" is not yet established as that.

**A run record that is never finished is never reported as abandoned.** A record whose steps sit at
`in-progress` forever is a run somebody walked away from, and nothing says so. Decision 2 is what
makes this reachable — an immutable record could not have the state — and closing it would need a
notion of when a run is over, which is the same missing notion as when it began.
