## Why

Five of this project's eighteen archived changes carry a `## N. Release` section, and all five
state the same four steps in the same order: regenerate the installed instruction files, advance
the workspace version and relock offline, write the changelog entry, record the version in
`VERSION` at archival and confirm `scripts/check-release` passes.

The section number is different every time — 6, 8, 6, 9, 7 — because nothing is shared. The steps
are retyped into each new `tasks.md`.

**The procedure has been got wrong, and the same lesson was learned twice.** At 0.10.0, task 6.2
recorded that advancing the version restales every installed file, because `generatedBy` is
written from `CARGO_PKG_VERSION` at run time:

> **The version bump restales every installed file, and this is the first release where a check
> says so.** [...] the stage found, on its first release, the class of drift it was written for.

Three weeks later, at 0.12.0, task 7.1 predicted that regeneration would be a no-op and
rediscovered the same fact:

> **Not a no-op, and the task's own reasoning is what is slightly off.** No workflow *body*
> changed, as predicted — but each installed file carries `generatedBy: htmlspec <version>` in its
> frontmatter, so **advancing the version in 7.2 made all 45 files stale**.

That task then wrote down what it had learned, in as many words:

> The general form worth keeping: this check is coupled to the version, not only to workflow
> prose, so **every** release regenerates it.

**There is nowhere for it to be kept.** `contract-authority` states that the project's contract is
what its accepted specifications state, "and nothing else SHALL be treated as stating an
obligation of the project" — so a lesson recorded in an archived `tasks.md` binds nothing and is
read by nobody. `archive.md` is an instruction body regenerated from source, so a step added there
by hand is overwritten on the next `init`. The sixth release is therefore positioned to rediscover
what the fourth and fifth both wrote down.

The same task found a second trap the same way: running `htmlspec init` to regenerate inside a
repository that is already a project bootstraps a stray `htmlspec/` and a `spec.nix` at `0.0.0`,
which then fails `self-assets` with nine duplicate-asset errors. That is knowledge with a
half-life, and it currently has no home either.

**This is a shape the project has closed before.** `artifact-kinds` states the general form,
crediting the fault to `project-bootstrap`'s refusal of a gate derived from a directory name: "a
rule nothing checks is not a rule." Here the subject is not a rule but a procedure: an ordered
set of steps, run once per release, whose order carries constraints (version *before* regenerate)
and whose hazards are known and written down where nothing reads them.

## The worked example

A recipe for chocolate-chip cookies is the same shape, and it is not only the easier thing to
picture — **it is the example that tests whether this artifact is what it claims to be.** The
release procedure is made of commands, so a proposal arguing only from it invites the reading that
a runbook is a script with prose around it. A recipe settles that: nothing in a bowl of creamed
butter automates.

So I built one before writing this. Nine steps, from creaming the butter to moving the cookies to a
wire rack, with the source links the recipe came with. It needed no code change:

```
$ htmlspec validate htmlspec/runbooks/chocolate-chip-cookies/tasks.html
  no diagnostics: the document is accepted
$ htmlspec validate --strict …            # same
$ htmlspec format --check …               # is canonical
9 task units, 10 external links, all preserved through formatting
```

The ten citation links were the part I expected to fail, and the reason they do not is a line in
`crates/htmlspec-validate/src/rules/safety.rs`:

> `href` is deliberately absent for `a`: a hyperlink to an external page is a citation, not a
> dependency, and the document renders fine offline with it. It *is* checked on `link`, which
> loads.

Somebody drew the exact line a cited recipe needs. **Two findings came out of building it rather
than describing it**, and both appear below: the run record's shape is forced by reference
resolution, and a runbook has nowhere to put its ingredients.

## What Changes

- **A runbook becomes a place a project keeps a procedure**, at `htmlspec/runbooks/<name>/`, a
  fourth place beneath the project root beside the accepted specifications, the active changes,
  and the archive. A procedure that is run repeatedly is not a change and does not archive, so it
  needs somewhere to live that is not `changes/`.

- **A runbook is a directory of two documents, both of existing kinds.** A `specification` states
  what the runbook is for, what it requires of the machine it runs on, and what its history taught;
  a `tasks` document carries the ordered procedure. **The split is forced rather than tidy** —
  `artifact-kinds` states that a specification SHALL NOT carry task units and that this is "the
  **only** placement rule in the vocabulary", so the procedure cannot live inside the canonical
  document.

- **No fifth artifact kind.** The set of four stays closed and
  `the_schema_enumerates_exactly_the_four_artifact_kinds` stays untouched and passing. A kind
  answers *which stage of one change's argument is this*; a runbook is not a stage, it is a
  destination. The precedent is already here: `htmlspec/specs/<capability>/` is a top-level
  accepted artifact whose documents are of kind `specification`, the same kind a change produces —
  so "a document outside a change directory that changes flow into" is a shape that needed no kind
  of its own.

- **The procedure carries no completion state, because a run is a copy.** A run instantiates the
  runbook by copying its `tasks` document; the states advance in the copy exactly as they do in a
  change. The canonical procedure's tasks stay `todo` forever because nobody works them. Two
  One thing falls out at no cost, and it is the one worth having: `dropped` already requires a
  `data-field="reason"` paragraph, and the refusal is an **error** rather than a warning —
  `schema/missing-reason`, measured against a run record with the reason deleted. A batch that
  skips a step cannot record the skip silently. **The completion counting does not come free**,
  and this proposal earlier claimed it did. `htmlspec status` takes a `--change` argument and
  reads only the changes directory, so it reports no change named `chocolate-chip-cookies` when
  pointed at a runbook. The logic is reusable and it does not reach a runbook today.

- **`htmlspec list --runbooks`**, a flag beside the existing `--specs`. Listing what the project
  holds is `list`, and a new top-level command would invent a second vocabulary for an operation
  that already exists.

- **One real runbook ships with this change: the project's own release procedure**, extracted from
  the five archived `## N. Release` sections and from `archive.md`. A location with nothing in it
  proves nothing, and the release procedure is the cheapest possible subject because this project
  is the one that pays if the shape is wrong.

**Not in scope**, each deferred to a named successor rather than dropped:

- **The `run` operation and the logging contract.** Reading a runbook, instantiating a run, and
  recording what that run must record. **A runbook's log is per-runbook rather than one fixed
  format** — a cookie run records an oven temperature and a yield, a release run records a version
  and a check result — so the contract is stated when the runbook is authored.
  `log-a-runbook-run`.
- **A workflow for authoring one.** A tenth `WORKFLOWS` row and `/htmlspec:runbook` would take the
  installed instruction files from 45 to 50 and `check-installed` with them. That fan-out is the
  whole cost of that change, which is why it is its own —
  `author-a-runbook-through-the-change-lifecycle`. This change must be provable without a workflow.
- **Carrying a runbook to another machine.** Imported skills, self-containment, and the procedures
  for installing required software. These are **contracts a runbook declares** rather than a
  bundle it carries: the runbook states what it needs of a machine, and the tool reports what is
  missing before a run starts. `carry-a-runbook-to-another-machine`.
- **A recorded run history.** The aggregate — "this runbook has been run 47 times" — derived from
  the run records rather than stored, for the reason `project-bootstrap` refuses to read the last
  archived version out of a directory name: a value maintained beside the facts it summarizes
  depends on a convention nothing enforces. `record-a-runbooks-history`.
**A per-runbook version and changelog are not deferred — the spec analogy answers them.** The
draft this proposal comes from asked for both, and the answer is the one the accepted
specifications already give. **An accepted specification carries no version and no changelog.**
`openspec/specs/<capability>/spec.md` holds a `## Purpose` and its requirements, and nothing else.
A capability's version history is *derived*: it is the set of archived changes that touched it,
each recording the release it shipped in. For `contract-authority` that history reads
`0.10.0` and `0.12.0`, recovered from the archive rather than from a field anybody maintained.

A runbook inherits exactly that. It is changed by a change, that change records a project version,
and its history is the archived changes that touched it. Adding a second version axis with nothing
comparing it to the project's own is accuracy without authority, which `contract-authority` names
as the failure mode rather than an untidiness: "Accuracy SHALL NOT confer authority." Storing a
derivable fact is also how this project's version gate once came to compare against a value no
project ever had.

One case genuinely needs a version of its own, and it is the only one: a runbook carried to a
machine where the project is absent, so the project's version means nothing and derivation from
the archive is impossible. That is `carry-a-runbook-to-another-machine`, and its honest cost is
that `check-release` grows a fourth question and every runbook then owes a distinct version with a
heading.

## Capabilities

**New Capabilities:**

- `runbook-authority` — what a runbook is, where it lives, which of its documents carries
  authority, and that its run records do not. The last of those is the load-bearing part: a run
  record is history, and `contract-authority` already places "the output of any earlier run" among
  the non-normative.

**Modified Capabilities:**

- `project-bootstrap` — its requirement *"A project declares its name, version, and expected
  toolchain"* states that the `htmlspec/` directory "SHALL hold the project's accepted
  specifications, its active changes, and its archive, each in its own place." A runbook is a
  fourth place, so that sentence is modified rather than quietly extended.

  **That sentence is not declared closed, and the difference from the kind vocabulary is the
  argument.** The kind enum says its set is `CLOSED` and has a guard test whose failure message
  calls adding one "a decision with a compatibility cost". The layout sentence has neither, and
  `migrate-an-openspec-project` already established that the layout beneath the project root is
  only partly specified. So this is a modification with a stated reason, not a closed set being
  reopened.

**Deliberately not modified:** `artifact-kinds`, `document-schema`, `document-validation`. No new
kind, no new unit in the closed seven, no schema change, no projection change. If this proposal is
right, the diff touches none of them, and that claim is checkable.

## Impact

- `crates/htmlspec-cli/src/main.rs` — a `runbooks: bool` flag on `List`, beside the `specs: bool`
  at line 213.
- `crates/htmlspec-cli/src/commands.rs` — `list`, and the project-layout constants that name the
  places beneath `htmlspec/`.
- `htmlspec/runbooks/` — the new location, and this project's own release runbook inside it.
- `openspec/specs/project-bootstrap/spec.md` — the fourth place.
- No change to `crates/htmlspec-schema/schema/document.schema.json`, to `TEMPLATES`, to
  `assets/vendor.lock.json`, or to `crates/htmlspec-validate/src/rules/`.

**Verification that this worked is one observation:** the next change's `tasks.md` replaces four
hand-written release items with one that says to run the release runbook.

## The counter-argument

**A runbook is a shell script, and this project already has six.** `scripts/check-release` is a
procedure run hundreds of times that carries sixty lines of its own reasoning in a docstring, and
it is executed rather than read. The release procedure could be `scripts/release`, and then it is
code with tests, which is a stronger artifact than a document.

Half of that is true, and this change turns nothing in `scripts/` into prose: where a step is
exact it belongs in a script. What a script cannot hold is a step whose content is judgment, and
this project has already decided that the release contains one. `archive.md` says so:

> Deciding whether the change earned a patch, a minor, or a major increment is judgment, so it is
> yours; whether one version follows another is exact, so it is the program's.

`check-release`'s own docstring goes further and explains why the automated version is impossible
rather than merely unbuilt:

> "Fail if the version has not advanced" is therefore unusable: a change under way legitimately
> has not advanced the version, because the patch-versus-minor decision is best made once the
> work's shape is known. Such a stage would be red from a change's first commit to its last, and a
> suite that is always red is one a contributor learns to skip.

So the release procedure is provably not automatable end to end, by a decision this project
already took and wrote down. What is left is an ordered set of judgments with exact checks between
them, and a runbook whose step invokes `scripts/check-release` is the division `artifact-kinds`
already names: judgment where judgment is needed, mechanical checking where a rule is exact.

## Findings left open

- **A change that produces no deliverable has not been tested.** The
  proposal→design→tasks→specification rhythm assumes the work builds something. A change whose
  output is only a procedure is a shape that rhythm has not met. This change tests it exactly once,
  on the project's own release procedure, and the result is worth recording either way.

- **A runbook edited after its fortieth run means runs 1–40 record a procedure that no longer
  exists.** That is an audit feature and a storage cost at the same time, and which one it is
  depends on facts this project does not have yet. Named here rather than solved; it is the first
  question `record-a-runbooks-history` has to answer.

- **A runbook has nowhere to put its inputs, and the cookie recipe is what found it.** Building the
  recipe, the ingredients had no home, so "assemble the ingredients" became a task unit — and that
  is wrong. Ingredients are not a step somebody performs. They are what the procedure consumes.
  The release procedure has the same gap in a plainer form: it requires `cargo`, `python3`, and a
  clean working tree, and none of those is a step either. **A runbook wants a notion of inputs**,
  and neither this change nor its four successors names one. Recorded here because the shape of the
  answer is not obvious: an input is close to a machine requirement, which
  `carry-a-runbook-to-another-machine` settles, and close to a task's references, which already
  exist. Deciding which it is needs its own argument.

- **"Run books" was named as a gap before this draft.** `gpt_chats/html-spec-proposal-chats.org`
  lists it under "Things that are Hard for openspec", beside bug fixes, implementations, and mixed
  human/computer work. The other three are not addressed here and are not claimed to be.
