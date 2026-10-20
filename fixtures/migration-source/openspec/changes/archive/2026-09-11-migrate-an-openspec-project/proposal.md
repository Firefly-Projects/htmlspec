## Why

This repository is the argument against itself. It ships a program whose whole claim is that
HTML is the better format for a specification, and it keeps its own specifications in 161
markdown files under `openspec/`. Every requirement this project has ever accepted — 97 of
them, carrying 401 scenarios across 9 capabilities — is written in the format the project
exists to replace.

There is no way out. `htmlspec init` establishes an empty project, and `htmlspec new change`
creates a change with three empty documents. Neither has anything to say about the 16
archived changes, the 28 delta specifications inside them, or the 75 evidence files that
record why this project is shaped the way it is. A contributor who wants to adopt HTMLSpec
for an existing OpenSpec project has one option today, and it is to retype it.

Those counts are measured rather than estimated, and recording that is not a formality: an
earlier draft of this proposal carried counts from a snapshot four weeks old, and every one of
the seven was wrong. The project kept working while the change sat in planning, which is what a
count in a planning artifact always risks. The measurement is in
`evidence/the-source-inventory-2026-09-07.md`, and the numbers the change is checked against are
the ones there.

**Who wants this and what they cannot do.** Any project already using OpenSpec — including
this one — cannot adopt HTMLSpec without abandoning its history. That is the whole
population of plausible adopters: HTMLSpec is a spinoff of OpenSpec, and its README says so
in the second paragraph. A tool whose stated lineage is another tool, and which cannot read
that tool's projects, asks every prospective user to start over. Most will not.

**What working looks like the first time you see it.** A contributor stands in this
repository and asks an agent to migrate. The agent reads all 161 files first and writes
nothing. It reports back: 9 capabilities become 9 specifications; 16 archived changes keep
their proposals, designs and tasks, and each one's delta specifications collapse into the
single `specification.html` that HTMLSpec's contract allows — five of them for
`2026-08-10-html-spec`, one or two apiece for the rest; the 75 evidence files are
carried across untouched, because a transcript's value is that it is verbatim; and
`openspec/config.yaml`'s project-context field has no destination at all, because a
declaration is forbidden from carrying authoring rules, so it is named as dropped rather
than dropped quietly — and in this repository it is named as *absent*, which is a different
fact a contributor needs told apart from the first.

The report also names what the destination format cannot hold. Two changes recorded a
requirement **rename**, and the destination's change-state vocabulary has no `renamed` — so the
plan states that as an open question and names the change that would settle it, rather than
quietly recording a rename as a modification and losing the one thing the rename said.

That report is the plan, and it is a document the contributor reads and argues with before
one HTML file exists. When they accept it, the migration builds the whole project somewhere
else, checks it there, and moves it into place in one step. At no point does the repository
contain a half-migrated project.

**Why a straight conversion is the wrong thing to build, and the tempting thing.** Markdown
to HTML is a solved problem; a converter is an afternoon's work and would produce 161 valid
documents. It would also produce a project that has lost the entire reason for migrating.
The README's argument is that a serious specification needs architecture diagrams, state
transitions, comparisons, and exact requirements, and that Markdown holds approximations of
those things while HTML expresses them directly. The design for
`report-in-the-contributors-terms` contains a seven-row table comparing what each workflow
owes its reader, and the proposal for it contains another comparing artifact-subjected lines
per instruction body. Those are already tables. A converter would emit them as `<table>` and
stop, and everything that was reasoned in prose because Markdown had no better offer — the
data flow through a migration, the ordering of an archival — would arrive as `<p>` and stay
there.

So the interesting content is not the markup. It is the judgment about which paragraph was a
diagram all along. That judgment is the same kind the project already refuses to automate:
`artifact-kinds` states that every synthesis and every contract update is agent-driven,
because a proposal and a design carry no imposed structure and therefore no grammar a parser
can read. A migration reads exactly those documents. It is the same problem, at the scale of
a whole project.

## What Changes

- **A migration operation exists, and it is a workflow rather than a command.** `htmlspec
  migrate` — what a contributor will type as `html-migrate` — carries the two parts that are
  exact. Everything between them is an agent reading, deciding, and writing, following
  stated instructions, exactly as archival already works.

- **The first pass reads and writes nothing.** The migration surveys every entity in the
  source project and records what each one is *for* before deciding what it becomes. A
  survey is mechanical: OpenSpec's layout is fixed, so enumerating it is enumeration. What
  the survey cannot say is which of those files carries an argument worth preserving as an
  argument, and that is why a reading pass follows it and precedes everything else.

- **Evidence is carried, not converted.** The 75 evidence files are transcripts, measured
  outputs, and recorded complaints. Their worth is that nobody edited them. They cross into
  the migrated project byte-for-byte and are referenced by evidence units in the documents
  that cite them.

- **The migration produces a plan, and the plan is a `tasks` document.** One task per source
  entity, each naming where it goes and what happens to it, each individually addressable so
  a contributor can reject one without rejecting the migration. The set of artifact kinds
  stays closed at four: a plan is a list of work items with states, which is what a `tasks`
  document is.

- **Everything is built in a staging location outside the destination.** The migrated
  project is assembled as a complete, resolvable HTMLSpec project somewhere else — its own
  declaration, its own asset store, its documents at the same relative depth they will
  occupy — and moved into place only once. A project resolves by ascending to an `htmlspec/`
  directory, and every document references the asset store by a relative path computed from
  its own depth, so a staging tree that mirrors the destination promotes by moving and one
  that does not promotes by rewriting every reference.

- **Nothing is dropped silently.** Every entity the survey found is accounted for in the
  plan: converted, folded into another artifact, carried verbatim, or dropped with a stated
  reason. Promotion compares the plan against the survey and refuses while any entity is
  unaccounted for. This is the division archival already uses — the agent decides what each
  entity becomes, and the program checks that it decided about all of them.

- **The HTML checks run last, over the whole staged project, and they gate promotion rather
  than drafting.** Validation, formatting, and asset topology are checked once the staged
  tree is complete. A migration that stopped at the first invalid document would interleave
  judgment with mechanical repair and leave a partial tree behind; worse, a defect in the
  mapping shows up as the *same* failure across dozens of documents, and that is a pattern
  only visible once they all exist.

- **The migration lands as a change in the project it created.** Its plan and its proposal
  become that project's first change directory. A migration whose own record was thrown away
  would be the one act in the project's history with no artifact behind it.

## Non-Goals

- **A markdown-to-HTML converter.** Stated above as the thing this deliberately is not. No
  command in this change reads a proposal and emits a document, and the project already
  forbids writing one.

- **Migrating anything but an OpenSpec project.** The survey knows one layout. Supporting a
  second source format is a second survey and a second set of mappings, and there is no
  second format anybody has asked for.

- **Deleting the source.** `openspec/` survives migration untouched. Removing it is the
  contributor's act, taken after they have read the result, and a migration that deleted its
  input would make its own defects unrecoverable.

- **Migrating installed agent instructions.** `openspec/AGENTS.md` and the tool command files
  under `.claude/`, `.codex/`, and `.pi/` are generated. `htmlspec init` writes their
  replacements from the compiled workflow table, and a migration that converted the old ones
  would produce instructions naming commands that do not exist.

- **Round-tripping.** Nothing converts an HTMLSpec project back to OpenSpec. A migration is a
  move, and a project that maintained both would have two sources of truth for one contract.

- **Migrating this repository as part of this change.** The change builds the operation and
  proves it against this repository's 161 files in a scratch copy. Whether and when
  `html-spec` itself moves off OpenSpec is a decision with its own consequences for every
  open branch, and it is made separately.

- **A fifth artifact kind.** The plan is a `tasks` document. `artifact-kinds` states the set
  of four is closed and that a document declaring a kind outside it is invalid, and one
  operation's working record is not the reason to reopen that.
