---
description: Archive an html-spec change — produce its specification artifact from the completed change, advance the project version, validate, apply its contract changes to the project's accepted specifications, and archive the directory intact. Use when a change's tasks are complete and it is ready to be archived.
allowed-tools: Bash(htmlspec:*), Read, Write, Edit, Glob, Grep
argument-hint: [the change to archive, or nothing to infer it]
generatedBy: htmlspec 1.2.1
---

Archive an html-spec change.

## What you were asked

$ARGUMENTS

Where that names a change, archive that one. Where it is empty, nothing was passed: infer
the change from the context you have, and say which one you picked before producing
anything.

List the candidates rather than guessing between them:

```bash
htmlspec list --json
```

Each change comes back with how much of its work is done: a count per task state, and the
change's overall state derived from it. Select on that rather than on the order — a change
still `in-progress` is not the one to archive, and a change reading `0/24` is untouched or
wholly under way depending on which states those tasks are in. Say what you selected on.
`doneDifferently`, beside the counts, is how many of the `done` tasks state a deviation: they were
done another way than their words say. They are counted as `done` too, so the number does not
make a change less finished; it tells you how many deviations there are to read.

**A change that states a predecessor comes back with it, under `follows`.** Where
`follows.resolved` is false, the change it follows is not archived, and that is the strongest
reason not to select this one — the ordering between changes is frequently the argument of the
work, and a contract archived after the changes it grades describes them instead of constraining
them. Archive the predecessor first. `follows.resolution` says which case you have:

| `resolution` | What it means | What to do |
| --- | --- | --- |
| `archived` | the predecessor's work is done | nothing; this change is free to archive |
| `active` | the predecessor is still being worked on | archive that one first |
| `unmatched` | no active or archived change carries that name | the name is wrong or the change was renamed — correct the declaration |

A change that follows nothing carries no `follows` field at all, which is the ordinary case.

The payload also carries `followsCycles`: sets of changes that each follow one another. That is a
contradiction in the recorded facts rather than an ordering to work out, and the program chooses
no order within one. Report it and ask which declaration is wrong.

`htmlspec archive` with no change also selects: one active change is chosen and announced,
and several are reported rather than picked between. Archival moves a directory and rewrites
a contract, so an ambiguous subject is a question rather than a coin toss.

**The judgment is yours and the mechanics are the program's.** You produce the change's
specification by reading its proposal, design, tasks, and implemented result, and deciding
what the change established. You apply its contract to the accepted specifications. `htmlspec
archive` checks what you wrote, tells you what it could not find, and then performs the
archival itself: it moves the change into the archive, names it for the date, and records the
version it shipped in.

That division is the whole arrangement: judgment where judgment is needed, mechanical work
where no judgment is required. Moving a directory is not judgment. Deciding which accepted
requirement a scenario belongs under is.

The project deliberately ships **no** command that reads a proposal or a design and
emits requirements, and you must not write one. Those kinds carry no imposed
structure — an author may arrange them any way they found effective — so there is no
grammar to parse. Deciding which paragraph states an obligation, what its identifier
is, and which scenarios express it is reading rather than parsing. An extractor that
guessed would produce a specification that passes every check and misstates what the
change established, which is worse than producing nothing because it looks
authoritative.

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

## What a change directory holds

Four kinds of document, and each declares its kind in
`<meta name="htmlspec-kind" content="...">`:

| File | Kind | What it is |
| --- | --- | --- |
| `proposal.html` | `proposal` | The value, need, and scope. Not the contract. |
| `design.html` | `design` | Architecture and technical reasoning. |
| `tasks.html` | `tasks` | The work items and their states. |
| `specification.html` | `specification` | The settled contract, as built. |

A change directory holds **at most one** specification artifact, and it is also that
change's as-built record — the same file, not two. Two documents are in play at
archival and there are exactly two: this specification, and the project's accepted
specifications, which live outside the change directory.

## Steps

### 1. Read the whole change

Read every document the change has. All of them, before writing anything:

- The **proposal** for what the change was for and what it deliberately excluded.
- The **design** for the decisions reached, the alternatives rejected, and the risks and
  how each was settled. A design's reasoning is often where an obligation is stated most
  precisely, and it is frequently stated in prose rather than as a unit.
- The **tasks** for what was actually done, and for what was **dropped** — a dropped
  task states why, and that reason usually marks a boundary the specification should
  state as out of scope rather than silently omit. For a task that states a **deviation**,
  read the deviation, not the task's words, as what was done: the words are what was
  planned. `htmlspec status --change <change-name>` names each such task.
- The **implemented result**: the code, the tests, and the fixtures. This is what
  makes the specification an as-built record rather than a restatement of intent.

Where the implementation and the proposal disagree, **the implementation is what
happened**. Say what was built. A specification that describes the intent while the
code does something else is the failure this operation exists to avoid.

### 2. Produce the specification

Create `specification.html` in the change directory, or update it if the change
already has one.

Start from the shape the toolchain provides:

```bash
htmlspec new document specification <change-dir>/specification.html --subject "<subject>"
```

That writes a document declaring the specification kind, already valid, with the
units the kind expects. Then replace the example units with the change's own.

**What a specification carries.** Requirement, scenario, evidence, and verification
units. These four are retrospective: evidence and verification can only be recorded
after building, and a requirement reaches its final wording once the built shape is
known. Decisions and risks may appear where they survived the change as part of the
contract.

**What it must not carry.** A task unit. `kind/task-in-specification` reports one as
an error, and the reason is worth holding onto while you write: a specification
records the obligation a task satisfied rather than the work that satisfied it. A
completed task is history. When a task did something the contract depends on, state
the obligation as a requirement and leave the task in the tasks document.

**Requirements.** One obligation each, stated with SHALL or MUST, in a sentence a
reader can check. Every requirement carries at least one scenario, because a
requirement with no scenario is not checkable. Give each a `req-` identifier that
names the obligation rather than the work.

**A requirement the change renamed.** Where the change gave an accepted requirement a new
heading, state it under the new heading with **the identifier the accepted requirement already
carries**, and put the heading it replaced in one paragraph immediately after the heading:

```html
<section class="requirement" id="req-the-accepted-identifier" data-change="modified">
  <h3>The new heading <span class="hs-id">req-the-accepted-identifier</span></h3>
  <p data-field="renamed-from">The heading the accepted specification holds</p>
  <p>The system SHALL …</p>
  …
</section>
```

- The rename is separate from the change state. `data-change="modified"` says the statements or
  scenarios changed as well. A requirement renamed and otherwise untouched is
  `data-change="unchanged"`, and carries the statements and scenarios the accepted requirement
  holds.
- The paragraph states the heading as a reader reads it, without the identifier.
- Never on an `added` requirement, which had no previous heading, or a `removed` one, which is
  withdrawn rather than renamed. Validation rejects both.
- One paragraph per requirement. If the change renamed a requirement twice, state the heading
  the accepted specification holds, because the ones between were never accepted.
- Never regenerate the identifier from the new heading. A renamed requirement is the same
  obligation, and its identifier is what every reference to it resolves through.

**A requirement the change modified.** State it whole, as artifact-kinds' "A change's
specification states a modified requirement whole" requires: every statement and every scenario
the requirement will hold once merged, not only the part the change altered. Start from the
accepted requirement's text and edit it, so the scenarios you carry forward match it:

```html
<section class="requirement" id="req-the-accepted-identifier" data-change="modified">
  <h3>The heading <span class="hs-id">req-the-accepted-identifier</span></h3>
  <p>The system SHALL state every obligation the requirement holds once merged.</p>
  <article class="scenario" id="scenario-carried-forward" data-change="unchanged">
    <h4>A scenario carried forward as the accepted specification states it</h4>
    <dl><dt>When</dt><dd>The accepted text, unedited.</dd><dt>Then</dt><dd>The accepted text, unedited.</dd></dl>
  </article>
  <article class="scenario" id="scenario-altered" data-change="modified">
    <h4>A scenario whose text the change altered</h4>
    <dl><dt>When</dt><dd>The text as the change leaves it.</dd><dt>Then</dt><dd>The outcome as the change leaves it.</dd></dl>
  </article>
  <article class="scenario" id="scenario-introduced" data-change="added">
    <h4>A scenario the change introduced</h4>
    <dl><dt>When</dt><dd>A case no earlier change described.</dd><dt>Then</dt><dd>The outcome the change established.</dd></dl>
  </article>
  <article class="scenario" id="scenario-withdrawn" data-change="removed">
    <h4>A scenario the change withdrew</h4>
    <dl><dt>When</dt><dd>The accepted text, kept so a reviewer sees what goes.</dd><dt>Then</dt><dd>The accepted text.</dd></dl>
  </article>
</section>
```

- **Every scenario carries its own change state:** `unchanged` for one carried forward as the
  accepted specification states it, `modified` for one whose text the change altered, and `added`
  for one it introduced.
- **A scenario the change withdrew is stated and marked `removed`, never left out.** The merge
  deletes it. A scenario left out is a defect the merge reports rather than a withdrawal, because
  the text is the same whether the change meant to withdraw it or forgot to restate it.
- **Each scenario keeps the identifier its accepted counterpart carries**, whatever its heading.
  The identifier is what makes it the same scenario, so a heading the change altered does not read
  as one scenario withdrawn and another added.
- **The statements are the requirement's statements once merged.** A statement paragraph carries
  no change state, so a paragraph the change deleted is either left out or kept inside a `<del>`,
  as below. Either way it supplies no statement. State a paragraph it rewrote as rewritten.
- The requirement keeps the accepted identifier. Where the change also renamed it, add the
  `renamed-from` paragraph above.

**Marking the words the change added and deleted.** A reviewer reading a modified requirement
whole cannot see which of its words moved. Mark them in place: `<ins>` around words the change
added, and `<del>` around words it deleted, kept where they stood. The edit that added `sync` to a
list of spellings is one inserted word:

```html
<p>The spellings are therefore fixed rather than left to an implementer: <code>validate</code>, <code>archive</code>, <code>init</code>, <code>new</code>, <code>list</code>, <code>status</code>, <ins><code>sync</code>, </ins><code>instructions</code>, and <code>store</code> name the operations OpenSpec also provides.</p>
```

and an edit that deleted a word and added a phrase is both:

```html
<p>That directory SHALL hold the project's accepted specifications, its active changes, <del>and </del>its archive, <ins>and its runbooks, </ins>each in its own place.</p>
```

The model reads the current text: the statement above is "… its active changes, its archive, and
its runbooks, each in its own place.", and the deleted "and" is recorded apart from it.

- **Mark only in a requirement or a scenario marked `modified`**, and only in this specification.
  document-validation's "Change markup in or around a semantic unit is rejected" refuses the marks
  anywhere else: in a unit marked `added`, `removed`, or `unchanged`, in a decision, evidence,
  risk, verification, or task, and in any other document, the accepted specifications included.
  A scenario carries its own state, so a scenario marked `unchanged` inside a modified
  requirement takes no marks.
- **Mark statements, notes, and scenario entries.** Never a heading, which is a name: a renamed
  requirement states its old name in the `renamed-from` paragraph, which takes no marks either.
  Never around a unit: a unit's change state says it is new or withdrawn. Never in a figure's
  diagram source, where a deleted line would leave the diagram unreadable.
- **A paragraph deleted whole goes in `<del><p>…</p></del>`.** A paragraph rewritten throughout
  is deleted and added whole, `<del><p>…</p></del><ins><p>…</p></ins>`, rather than marked phrase
  by phrase, because a sentence dense with marks is one nobody can read.
- **Marking is permitted, not required.** A modified requirement with no marks is valid. A mark
  claims only the words it marks, so mark every word the change moved or none.

**Read what you restated beside the accepted text, before step 5.** Run
`htmlspec show <change-name> --diff`. For each requirement you marked `modified`, it shows your text
beside the accepted text with the words that differ marked, as toolchain-packaging's "The command
surface shows a modified requirement beside the accepted one" provides. Every difference it shows
should be one this change made. A difference this change did not make is another change's edit that
reached the accepted specification after this change was written, and merging your restatement would
revert it: restate that paragraph or scenario from the accepted text, mark your own words again, and
run the view once more. Where the view says the words are marked as you marked them, your marks
account for the whole difference. Where it marks them by its own comparison instead, your marks leave
part of the difference unmarked.

**Verification.** Record how each requirement was checked, precisely enough that
someone else could repeat it, and name the requirement it covers with `data-refs`. A
requirement no verification names produces a `review/unverified-requirement` warning,
which is **advisory and never gates**: verification arrives as work completes, so a
specification in progress legitimately has requirements not yet verified. Do not
invent a verification to silence the warning. Record the ones that ran.

A check that reached part of what its verification names records
`data-outcome="partial"`, with one `<p data-field="unchecked">` paragraph for each
part it did not check, each a sentence saying what went unchecked. A `partial`
verification without one is invalid, and an unmarked paragraph does not count,
because it is a note (document-schema's "A partial verification states what it did
not check"). A check whose condition, subject, or trigger differs from the
scenario's did not reach that scenario at all: it is not a partial check of it, and
it gets no verification. Like any outcome, `partial` clears the unverified warning,
so record it only for a check that ran.

**Evidence.** Where the change measured something, record what was measured and what
it showed. Evidence is what makes a claim checkable by someone who was not there.

### 3. Advance the project's version

`htmlspec archive` compares the `version` in `spec.nix` against the last archived
version and refuses unless yours is ordered after it. **Advancing it is your work.**
The program only compares — the same division as the specification itself, where you
decide what the change established and the program checks the result.

Edit `spec.nix` and raise `version`. Deciding whether the change earned a patch, a
minor, or a major increment is judgment, so it is yours; whether one version follows
another is exact, so it is the program's.

### 4. Check, and reconcile what it reports

```bash
htmlspec archive <change-name-or-directory> --check
```

A change can be named rather than pathed: `htmlspec archive add-dark-mode` resolves it
against the project, from any directory inside it.

**`--check` performs nothing.** It applies every condition archival applies — the version
comparison, the specification's presence, its declared kind, and its validation — and reports
what it finds without moving anything. Use it here, because the merge in step 5 has not
happened yet and you are asking a question rather than finishing.

It **refuses while the specification reports an error**. A change archived with an invalid
specification would leave the accepted contract in a state the project's own tools reject.

Reconcile every error it reports, then run it again. Read the warnings too: an
unverified requirement is legitimate, and a stale reference usually means a unit's
change state needs settling now that the work is done.

It also reports things it does not refuse on, and each is for you to judge:

- **Tasks that are neither done nor dropped.** A dropped task is a finished decision. A task
  still open at archival is either work left undone or a task nobody updated.
- **Requirements your specification states that no accepted specification holds.** Before step
  5 this is expected — you have not merged yet. After step 5, each remaining one should be a
  requirement whose work the change abandoned, and you should be able to say which. A requirement
  the change marks `removed` is never among them: once the merge deletes it, archival counts it as
  reached.
- **Renames the accepted specifications have not taken.** A requirement your specification
  renames, whose identifier an accepted specification holds under another heading. Before step 5
  this is expected too. After it there should be none, because a rename is never withdrawn: it
  was applied or it was forgotten. `untakenRenames` in the payload names each one with both
  headings, and `needsSync` is true while any remain.
- **Content the accepted specifications have not taken.** A requirement your specification adds or
  modifies whose identifier an accepted specification holds with other statements or scenarios.
  Before step 5 this is expected for every one. After it there should be none, except a
  requirement step 5 held. `untakenContent` in the payload names each one with where it differs:
  the first statement paragraph that differs, and the scenarios each side holds that the other
  does not, by identifier. It never carries either side's text.
- **Withdrawals the accepted specifications have not taken.** A requirement your specification
  marks `removed` that an accepted specification still holds. Before step 5 this is expected.
  After it there should be none. `untakenRemovals` in the payload names each one.
- **A predecessor that is not archived**, where the change states one it follows. Archival is
  the last moment the ordering can be observed, so this is where it is reported. Archive the
  predecessor first, or state that this change no longer follows it. `outOfOrder` in the payload
  says whether this fired, and `unresolvedPredecessor` names the change and how it resolved.
  This does **not** refuse: an abandoned sequence whose later change was completed on its own
  merits is legitimate, and you are the authority on whether that is what happened.
- **A cycle of `follows` declarations.** Two or more changes each following the other is a
  contradiction, and no ordering holds among them until one declaration changes.

Do not proceed to step 5 until the errors are gone.

### 5. Apply the contract changes to the accepted specifications

This is the one **merge** in the operation. The three earlier movements — proposal to
design, proposal and design to tasks, completed change to specification — are
*synthesis*: each produces a new artifact from artifacts that remain intact and
separate. This one applies a change's contract into specifications that already exist
and hold content this change did not touch.

**Preserve what the change does not state, and apply what it modifies as stated.** The
accepted specifications hold the contract of every change before this one, and a requirement this
change does not state is one it did not decide to alter. A requirement it modified is stated
whole, as artifact-kinds' "A change's specification states a modified requirement whole"
requires, so the merge applies it as it is stated rather than reconstructing it from what the
change left out.

Concretely:

- Naming two of a capability's requirements leaves every other requirement in that
  capability untouched.
- A requirement the change modified takes the statements and the scenarios the change states,
  less the scenarios it marks `removed`, and keeps its identifier. The `sync` instructions'
  "Merging a modified requirement" gives the procedure, including the three cases that hold the
  whole requirement unmerged until the contributor answers: a scenario marked `unchanged` whose
  text differs from the accepted one, an accepted scenario the change neither states nor marks
  `removed`, and a difference that is another change's later edit.
- Where the change's specification marks words with `<ins>` and `<del>`, the accepted requirement
  takes its current text: each `ins` element's content without the element, and nothing of each
  `del`. artifact-kinds' "A marked modification reaches the accepted contract as its current text"
  requires it, and the `sync` instructions' "Merging a modified requirement" says how. The
  accepted specification never carries the marks, and validating it refuses one left behind.
- A requirement the change renamed gets the new heading and keeps its identifier. The
  `renamed-from` paragraph stays in the change and does not go into the accepted specification,
  which states the name the project uses now. Where the change also modified it, apply the
  modification to the same requirement. Never apply a rename as a removal and an addition,
  because the addition would carry a new identifier and every reference to the old one would
  stop resolving. The `sync` instructions state the procedure in full.
- A requirement the change withdrew is removed. A withdrawal is the one case where
  deleting is correct, and it must be deliberate rather than a side effect of
  rewriting a file. Archival then counts the requirement as reached, because no longer being
  held is what the change asked for.

Never regenerate an accepted specification from the change's specification alone. The
accepted specification is the project's contract across every change that came
before, and the change in hand knows nothing about most of it.

After the merge, validate what you wrote:

```bash
htmlspec validate <accepted-specs-dir>
```

**Do not proceed to step 6 while a modified requirement is held.** A requirement you left
unmerged for one of the three cases above is content the accepted specifications have not taken, so
`htmlspec archive` reports it and asks. An answer of yes, or `--yes` where nothing can answer,
archives the change anyway, and the change's edit never reaches the contract. Nothing later brings
an archived change back. Stop, and report the held requirement and the scenario that held it.
Once the contributor has corrected the change's specification, merge the requirement as stated,
and then archive.

### 6. Archive it

```bash
htmlspec archive <change-name-or-directory>
```

**Now, not earlier.** Run this after the merge in step 5, so the requirements your
specification states are already in the accepted specifications and archival has nothing to
report about them. Running it before the merge means answering a question you caused. Archival
reports a held requirement and asks, but it cannot tell a requirement you held on purpose from one
you missed, and `--yes` archives over both. That is why step 5 stops on a held requirement rather
than leaving it for archival to find.

This moves the change into the archive, names it `<YYYY-MM-DD>-<change-name>`, and records the
declared version in a `VERSION` file. A change whose name already begins with a date keeps the
one it has. If the target name is taken, archival refuses and names what is there rather than
picking a different name — a collision means the change was already archived, or that two
changes share a name on one day, and both want you.

**It also publishes the project's runbooks, and this is a refusal rather than a question.** Each
runbook the project holds is compared against the release it last published: its `VERSION`, its
`CHANGELOG.md` heading for that version, and the fingerprints of its two canonical documents. A
runbook whose documents changed without its version advancing is refused, and so is a version that
goes backwards — a version identifies one released procedure, and a copy of a runbook on another
machine has only its version to say which one it is. The archive then carries a `runbooks.json`
inventory of what was published, and the first time this happens the project gains a
`runbook-releases.json` marker naming the release publication was adopted at. Neither file is
yours to write, and the marker never moves afterwards.

Advancing a runbook's version *is* yours, for the same reason the project's increment is: whether a
changed procedure earned a patch or a minor is judgment. Archival compares and reports; it never
chooses. A run whose record was added is not a change to the procedure and owes no increment.

If it still reports open tasks, unmerged requirements, untaken renames, untaken content, untaken
removals, or a predecessor that is not archived, it asks before proceeding. Answer it yourself where you can. Where nothing can answer — an agent
loop, a continuous-integration job — it stops rather than archiving silently, and `--yes` is how
you say to go ahead anyway:

```bash
htmlspec archive <change-name-or-directory> --yes
```

Use `--yes` when you have read the findings and decided they are acceptable, not to make the
report go away.

**All of the change's documents move with it.** Archival could keep only the specification,
since that is what carries forward as contract, and it deliberately does not: the whole
argument for separating the stages is that a proposal's argument and a design's reasoning are
worth having later. Discarding them at archival would make the separation a filing convention
after all.

A reader opening an archived change must be able to find the argument that motivated
it, the reasoning that shaped it, the work that delivered it, and the contract it
established.

Archival does not advance the version, and never has. That is step 3, and it is yours.

## Idempotence

Producing the specification twice against an unchanged change must not alter its
**meaning**. That is weaker than byte-identity, and byte-identity is unavailable here:
two agents synthesizing one change can write differently-worded specifications that
both validate, because the operation is judgment. Meaning-stability is the strongest
property available, and validation plus review at archival is what bounds it.

If a specification already exists and the change has not moved, do not rewrite it for
style. Update what changed.

## What to report

**Open with what the project now guarantees that it did not guarantee before.** That is what
archiving accomplished, stated as an obligation a reader can hold the project to. The
specification file is where it lives rather than what it is.

Then, in this order:

- **What you could not resolve into a contract** — an obligation the design implies but does not
  state, or a behavior the code has that nothing asked for. Say so rather than guessing. An
  unresolved question in a report is recoverable, and a guess written into the accepted
  contract is not.
- **What changed in the accepted specifications**: which ones you merged into, and what you
  added, modified, or removed in each. Where step 5 held a modified requirement, name it and the
  scenario that held it, and say that you stopped before archiving.
- **Where it landed, and what `htmlspec archive` reported.** One line, including warnings you
  left in place and why.

**Name what your references point at, the first time you use each one.** A task identifier, a
diagnostic code, a count of warnings — say in plain words what it is, in the same sentence. The
test: delete the identifiers from your sentence. If nothing is left, they were the claim and
each one needs naming. If the sentence still stands, they were annotation and need nothing. A
list whose first item is named and whose rest are bare numbers is the failure this prevents:
it looks readable and forces every other line to be looked up.

A command, a flag, or a file path already names itself. Do not gloss those.
