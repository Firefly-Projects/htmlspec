---
name: htmlspec-migrate-project
description: Bring an existing OpenSpec project into html-spec — survey what it holds, read what each document does, plan every entity's destination, then stage and promote the whole project. Use once, on a project that is not html-spec yet.
allowed-tools: Bash(htmlspec:*), Read, Write, Edit, Glob, Grep
argument-hint: [the OpenSpec project to migrate, or nothing to infer it]
license: MIT
metadata:
  generatedBy: htmlspec 1.0.0
---

Bring an existing OpenSpec project into html-spec.

## What you were asked

$ARGUMENTS

Where that names a directory, migrate the OpenSpec project there. Where it is empty, nothing
was passed: find the source yourself and **say which one you picked before producing
anything**. A migration reads a whole project and writes a whole project, so a wrong subject
is expensive to discover late.

```bash
htmlspec migrate survey <path-or-nothing>
```

That enumerates what the source holds and writes nothing at all. Run it first, whatever else
you do. If the path is not an OpenSpec project it says so and names what it looked for, which
answers the question of which directory you are standing in.

**The judgment is yours and the mechanics are the program's.** Two commands are exact —
`htmlspec migrate survey` lists what a source project holds, and `htmlspec migrate promote`
checks that your plan accounts for all of it and moves the result into place. Everything
between them is reading: what each document argues, which of its paragraphs states an
obligation, which comparison deserves a table and which deserves a diagram, and what a
proposal's argument was actually for.

That division is the same one archival already uses. Moving a directory is not judgment.
Deciding which requirement a scenario belongs under is.

The project deliberately ships **no** command that reads a proposal, a design, or a tasks
document and emits a document in html-spec's format, and you must not write one. Those kinds
carry no imposed structure in either format — an author may arrange them any way they found
effective — so there is no grammar to parse. A converter would emit documents that satisfy
every check and lose the reasoning that made the migration worth doing, which is worse than
producing nothing because it looks finished.

**That is the whole failure this workflow exists to prevent.** Nothing mechanically
distinguishes a thoughtfully migrated document from a mechanically converted one. Both
validate. Both pass `format --check`. The only thing that catches a conversion is a person
reading the plan you wrote, which is why the plan comes before any document exists.

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

**A migration is the worst possible place to break this rule**, and the reason is arithmetic. A
convention you invent here arrives with a hundred instances behind it and no requirement
anywhere, inside a change whose subject is something else. A later reader cannot tell your
decision from an accident. Where the contract is silent about where something goes, preserve the
shape the source used and record that you did.

## What a source project holds, and where each part goes

| OpenSpec | html-spec | What decides it |
| --- | --- | --- |
| `specs/<capability-path>/spec.md` | `htmlspec/specs/<capability-path>/spec.html` | **Silent.** No requirement states a layout beneath `htmlspec/specs/`, so the source's nested path is preserved rather than flattened |
| N delta files in one change | one `specification.html` | A change directory holds at most one specification artifact |
| `## ADDED Requirements` | `data-change="added"` on the unit | The change state is expressed per unit rather than per file |
| `## MODIFIED Requirements` | `data-change="modified"` on the requirement, stated whole: every statement and every scenario it held once the change was merged, each scenario `unchanged`, `modified`, `added`, or `removed`, with its accepted counterpart's identifier, and the words the change added and deleted marked with `ins` and `del` against the text it replaced | project-migration's "A change's delta specifications collapse into that change's one specification". A block that left out a scenario is completed from the source's own merge. See "A modified requirement" below |
| `## REMOVED Requirements` | `data-change="removed"` on the requirement, holding the statements and scenarios it held before, each scenario `removed`, with the delta's Reason and Migration as `<p data-field="note">` paragraphs | project-migration's "A change's delta specifications collapse into that change's one specification" records the state on the requirement, and document-schema's "Semantic units declare a change state" keeps a withdrawn unit present so a reader sees what was withdrawn |
| `## RENAMED Requirements` | a `<p data-field="renamed-from">` paragraph stating the `FROM:` heading, on the requirement headed with the `TO:` heading | A rename is a fact about the heading and separate from the change state. **Take the state from the rest of the delta:** `modified` where the delta also restates the requirement under `## MODIFIED Requirements` with the new heading, and `unchanged` where it renames the requirement and says nothing else, with the statements and scenarios it held before. The requirement keeps the identifier its `FROM:` heading already resolved to. See "Record every identifier derivation" below |
| a task's checkbox | `- [x]` is `done` and `- [ ]` is `todo`. `- [~]`, or a task struck through, is `dropped` with the source's reason, whichever box is ticked. A task done otherwise than its words say is `done`, with a `<p data-field="deviation">` saying what was done | project-migration's "A source task is carried in the state its record supports". See "Tasks" below |
| `VERSION` | `VERSION`, carried unchanged | The archival gate reads the version a change records and never derives one from a directory name |
| `.openspec.yaml` | `follows:` becomes the change's `FOLLOWS`, holding the value as the source wrote it. The file is a plan entity like any other. `created:` maps to no field, and the date crosses only inside the file, where the plan carries the file | project-migration's "A source change's own configuration is surveyed, and its predecessor becomes its declaration", and artifact-kinds' "A change states the change it follows". The destination records no creation date, because no command depends on one. See "A change's own configuration" below |
| `config.yaml` context, rules | *nothing* | The declaration carries no authoring rules or project context, so there is nowhere for this to go |
| `evidence/**` | `evidence/**`, byte-identical | Evidence is worth having because nobody edited it. artifact-kinds' "A change's evidence is kept in the change and is not one of its documents" makes it a record rather than a document, so `validate`, `format`, and promotion's checks pass over it, HTML at an earlier contract version included |
| generated agent instructions | *not migrated* | `htmlspec init` writes its own from the compiled workflow set |

**Every row where the contract is silent is a row you must not decide permanently.** Preserve
the source's shape, record the choice in the plan as yours, and say that a later change whose
subject is that layout can settle it.

## Carry each distinction the destination can express

**Where the destination has a value for a distinction the source makes, map it. Do not record it
as open.** project-migration's "A source distinction the destination cannot express is recorded as
open, not mapped to the nearest state" says both halves: a distinction with no destination value is
recorded as open, and one the destination expresses is mapped, judged against the contract you are
staging against. A plan written before the destination gained a value is revised before the
documents it affects are staged. You still add no value of your own.

The destination expresses a renamed heading, a task done otherwise than its words say, the words a
modification changed, a check that reached part of what it names, and how a risk was settled. Each
has its own requirement. The rename's is project-migration's "A source rename is carried as the
heading the requirement replaced", and each other's is named in the part below that covers it.

### A modified requirement

project-migration's "A change's delta specifications collapse into that change's one
specification" governs this.

- **State it whole.** Write every statement and every scenario the requirement held once the
  change's contract was merged, as artifact-kinds' "A change's specification states a modified
  requirement whole" requires of every change's specification. Each scenario carries its own change
  state, and the identifier its accepted counterpart carries: the scenario the migrated accepted
  specification holds under the same requirement with the same heading.
- **Complete a block that left a scenario out, from the source's own merge.** An OpenSpec delta
  cannot say whether a scenario it leaves out was withdrawn or kept. The accepted specification at the
  commit that applied the change's contract can. A scenario it still held is `unchanged`, with the
  text it held. A scenario it no longer held is `removed`, with the text it held before. Where the
  history does not hold that commit, put the omission to the plan's review, naming the requirement
  and the scenario, and do not decide it either way.
- **Mark the words the change added with `ins` and the words it deleted with `del`,** against the
  accepted text the requirement or scenario held before the change. A migration must mark them,
  although an author may leave a modification unmarked: you compared the two texts to state each
  scenario's change state, so you hold the changed words, and no later reader can recover them.
  document-validation's "Change markup in or around a semantic unit is rejected" permits the marks
  only in a requirement or a scenario marked `modified`, in a change's specification.
- **The marks are your reading.** A comparison of the two texts is evidence you read, not markup a
  program writes. Which words form one edit is a judgment. Where a paragraph was rewritten
  throughout, delete it whole and add it whole, rather than marking it phrase by phrase until it
  cannot be read.
- **A summary does not narrate which words moved.** The marks show it.

### Tasks

project-migration's "A source task is carried in the state its record supports" governs this.

- `- [x]` is `done`. `- [ ]` is `todo`.
- `- [~]`, or a task whose words are struck through, is `dropped`, whichever box is ticked. Its
  reason is the source's own words, verbatim, in `<p data-field="reason">`. Do not wrap the words in
  `del`: a task never carries change markup, and `dropped` already says its words were given up. The
  strike may be carried as `s`.
- A `done` task whose record says the work was done another way, only in part, or beyond its words
  states what was done in one `<p data-field="deviation">`, beside its words, which stay as written.
  Where the task's own text carries that account, as "**Amended during implementation.**" does, the
  account is the deviation, in the author's words. Where the account is elsewhere, the deviation says
  what was done and names where that is shown: a record the change carries, which it links, or the
  source at a named commit.
- A `done` task whose result nothing records stays `done`, with no deviation, because a deviation
  states what was done. A note may say that nothing records it.
- **Never write a reason the source did not state.** Where the source abandons a task and says
  nothing of why, put the task to the plan's review. If the review supplies no reason, the task takes
  the state its checkbox gives, with a note, and the plan records what you did.

### References

project-migration's "A unit the author's words name is referenced wherever the migrated project
resolves it" governs this.

- Where a task or a decision names a requirement or a scenario in the author's words, and
  document-schema's "Cross-references address units by identifier" resolves a reference from its
  document to that unit, give the task or decision the unit's identifier in `data-refs`. Keep the
  link around the author's words beside it: a reference is not a hyperlink, and the page is read by
  people too.
- The author's words must name the unit, by its heading or by words that plainly are its heading.
  Do not add a reference from a capability's name, a task's subject, or a unit the work touched.
- Where the document cannot reach the unit, the link alone names it: a unit only another change
  declares, a unit in the change's own proposal, design, or tasks, and a unit the place resolving the
  reference marks `removed`.
- A staged reference is checked in the staging location as it will be after promotion, so a
  reference that resolves nowhere holds promotion back.

### Verifications

project-migration's "A check that reached part of what it names is carried as a partial
verification" governs this.

- Write a verification only where a carried record states the outcome of a check.
- Where the record shows that the check reached part of the requirement or scenario and not the
  rest, record the outcome `partial`, as `data-outcome="partial"`, with one `unchecked` paragraph,
  `<p data-field="unchecked">`, for each part it did not reach, in the words of the requirement or
  scenario. Do not record it as `passed`, and do not leave it out for being partial.
- A check whose condition, subject, or trigger differs from the scenario's did not reach that
  scenario. Write no verification of it.

### Risks

project-migration's "A source risk's stated settlement is carried as its response" governs this.

- A source risk becomes a risk unit only where the source gave it a heading, and stated it and any
  measure against it in paragraphs of their own. Do not split a paragraph or write a heading to
  make one.
- Declare a risk's `data-response` only where the source states its settlement. A risk the source
  marks accepted, or says is not mitigated, is `data-response="accepted"`, and needs no mitigation;
  a paragraph of its own stating a partial measure is its mitigation. A risk the source marks
  mitigated is `data-response="mitigated"`. A risk that states a measure and no settlement takes the
  default, which is `mitigated`.
- A headed risk that states neither a settlement nor a measure stays in prose.
- Call it the risk's response. In this workflow a disposition is what the plan decides about an
  entity, and the two are not the same thing.

### A change's own configuration

project-migration's "A source change's own configuration is surveyed, and its predecessor becomes
its declaration" governs this.

- The survey reports, for each change, whether it holds a `.openspec.yaml`, every key the file
  sets, and its `follows:` value. Carry what the survey reports. Do not read the file again.
- Where the survey reports a `follows:` value, the migrated change states it in `FOLLOWS`, exactly
  as written, under artifact-kinds' "A change states the change it follows". Where it reports none,
  the change has no `FOLLOWS`. Never infer a predecessor from a change's name, dates, or
  requirements.
- The file is a surveyed entity and owes a disposition like any other. A key with no destination,
  such as `schema:`, is accounted for in the plan.
- **A creation date is neither mapped nor an open question.** The destination records no creation
  date, because no command depends on one, so `created:` has no destination. The date crosses only inside
  the file, where the plan carries the file. Where the plan folds or drops a file that states
  `created:`, its reason says the date goes with it.

## The stages, in order

1. **Survey.** Enumerate. Write nothing.
2. **Read.** Open every source document and record what each one *does in the project*.
3. **Gather evidence.** The evidence files, the recorded versions, the archive dates, the
   release record.
4. **Plan.** Write the plan and **stop**. Report it.
5. **Stage.** Assemble the whole migrated project outside the destination.
6. **Check.** Run the project's checks over the complete staged tree.
7. **Promote.** Move it into place, in one step.

Stages 1 through 4 produce no html-spec document at all. That is deliberate: the artifact of
the first four stages is a plan a person reads, and reading it is the only check that catches a
conversion.

### 1. Survey

```bash
htmlspec migrate survey <source> --json > /tmp/survey.json
```

Keep that file. The coverage check in stage 4 compares your plan against **this** payload
rather than against a fresh enumeration, so the plan is checked against the list it was
written from.

Read the human form too. It names the things your plan will find hardest: the changes
recording no version, the changes whose configuration states a predecessor, the versions several
changes share, the delta sections in use, and the configuration keys that are set.

### 2. Read every document, and record what it does

**Open every file the survey listed. All of them, before writing anything.**

For each one, record one line: **what it does in the project**. Not what it says — what it is
doing there. Those differ, and the difference is the whole of this stage.

- "Argues that asset duplication is a topology problem rather than a size problem" is what a
  document does.
- "Proposal for content-addressed assets" is what a file listing already told you.

**The failure to avoid:** a plan whose per-entity lines could have been written from the file
listing alone. If every line reads like `proposal.md — the proposal`, you skipped this stage,
and the plan you produce will be reviewed and approved because it looks complete. Nothing
downstream will catch it. This is the one stage with no mechanical check at all.

This is also the pass that finds what a bulk conversion cannot: the three proposals arguing the
same defect class, the design whose middle section is a state machine written as prose because
Markdown had nothing better, the tasks document whose dropped task marks a boundary the
specification should state.

### 3. Gather the evidence, before any document exists

A change's evidence is the record of why the project has the shape it does, and **nothing in a
source document points at it**. So it is the first thing a bulk conversion loses, and that is
why this stage comes before any document is produced rather than after.

Collect the evidence files, each change's recorded version, the archive dates, and the release
record. Carry evidence **byte-identical** — converting a transcript is rewriting a record, and a
rewritten record cannot serve the purpose evidence serves.

**Gather the text each modification replaced, too.** For each requirement a source delta restates
under `## MODIFIED Requirements`, read the accepted specification before and after the change's
merge from the source's version history, at the commit that applied the change's contract. The
text before is what you mark the change's words against, and the text after decides each scenario
the delta left out (see "A modified requirement" above). Nothing in the source points at these
commits either, which is why they are gathered here.

**Evidence already in this format crosses the same way.** A record that declares an earlier
contract version, or names a stylesheet the project no longer stores, is not a document of the
project. Nothing checks it or rewrites it, and it does not hold promotion back. The same applies
to a source document already in this format that no longer conforms: "A source entity already in
the destination format is not assumed usable" requires you to preserve it rather than carry it as
a document. Where you preserve one in its change's `evidence/`, it sits there unchanged. Say in
the plan that the placement is your choice.

Then make sure something points at it. A migrated document that makes a claim its change
measured should reference the carried evidence file, so a reader reaches it from the document
that cites it rather than by finding it in a directory.

### 4. Write the plan, then stop

The plan is a `tasks` document: one task unit per surveyed entity.

```bash
htmlspec new document tasks /tmp/migration/plan.html --subject "Migration plan"
```

Each entity's facts go in note paragraphs, each opening with its name and a colon:

```html
<li class="task" id="task-openspec-specs-project-bootstrap-spec-md"
    data-change="added" data-state="todo">
  <h3>The project-bootstrap capability's accepted specification</h3>
  <p data-field="note">source: openspec/specs/project-bootstrap/spec.md</p>
  <p data-field="note">destination: htmlspec/specs/project-bootstrap/spec.html</p>
  <p data-field="note">disposition: converted</p>
  <p data-field="note">derives: A project has a declared root holding one asset store -&gt; req-a-project-has-a-declared-root-holding-one-asset-store</p>
</li>
```

A disposition is one of four words:

| Disposition | Means | Owes a reason |
| --- | --- | --- |
| `converted` | it becomes an html-spec document | no |
| `folded` | it becomes part of another artifact | **yes** |
| `carried` | it crosses unchanged | no |
| `dropped` | it does not cross | **yes** |

A reason may be a `reason:` note or, on a task whose `data-state` is `dropped`, the task's own
`data-field="reason"` paragraph. Either is read.

**Distinguish empty from dropped.** "This field carried nothing in the source" and "this field
carried something and it is gone" are different facts about the project a contributor is
receiving, and they need to know which one they have.

**Record every identifier derivation on the entity that stated the heading**, with a `derives:`
line. Derive from the **full** heading. A rule that truncates or normalizes maps distinct
obligations onto one identifier and manufactures a collision the source never contained.

**A renamed requirement keeps the identifier it already held.** A heading named as a `TO:` in a
source `## RENAMED Requirements` section does not derive an identifier of its own. It takes the
identifier its `FROM:` heading resolved to. So a chain `A → B → C` resolves to the identifier
derived from `A` in every change in the chain and in the accepted specification, because one
obligation holds one identifier. **A later change that restates `C` without renaming it** resolves
to that identifier too. It is modifying the same obligation, and deriving from `C` there would give
the one obligation a second identifier in exactly one document. Record it on the renaming change's entity, where it says the
heading was introduced:

```html
<p data-field="note">renamed: A -&gt; B</p>
<p data-field="note">derives: B -&gt; req-a</p>
```

The `derives:` line stays one per stated heading, and it names the identifier the heading
**resolves** to, which here is not the one `B` would derive by itself. The `renamed:` line is what
tells a reader why.

- **Carry the author's reason.** Where the RENAMED section holds prose beside its pairs, such as an
  explanation of why the heading changed, carry it into the migrated specification. No field holds
  it, and it is the author's reason for the rename.
- **A chain inside one delta** (`A → B` and `B → C` in one RENAMED section) is one requirement
  renamed once in that change: carry one `renamed-from` paragraph stating `A`. The heading between
  was never accepted.
- **A heading changed without a RENAMED section**, by removing one requirement and adding another,
  derives each identifier independently. The source did not say the two are one obligation, so do
  not infer it from their wording.
- **A rename is not an open question.** The destination expresses it, so a plan that records a
  RENAMED section as open has mapped it to nothing.

One heading several changes each restate lands in several archived specifications with the same
identifier, and that is correct too, as is a renamed obligation's one identifier across the
changes that renamed it. A collision is two **different** headings deriving one identifier inside
**one** destination document.

Then check it, and stop:

```bash
htmlspec migrate promote /tmp/migration/plan.html --survey /tmp/survey.json
```

With no `--staged`, that runs the coverage check alone. It names every entity your plan does not
account for, every disposition it cannot read, every folded or dropped entity with no reason,
and every identifier collision. It does **not** judge whether a disposition is right — that is
what a contributor reviews the plan for, and it is the reason you stop here and report.

**Report the plan and stop.** Do not stage in the same step.

### 5. Stage the whole project, outside the destination

```bash
mkdir -p /tmp/migration/staged && cd /tmp/migration/staged
htmlspec init --tools none --name <project-name>
```

**Outside the destination, not inside it.** A project holds exactly one asset store, and a
staging tree beneath the destination puts a second one there — so `htmlspec assets` would fire
on your own working directory and a contributor could not tell that from a real defect.

**Stage each document at the path it will occupy after promotion.** A document references the
project's asset store by a path relative to its own directory, so `htmlspec/specs/x.html`
reaches `../assets/` and `htmlspec/changes/y/proposal.html` reaches `../../assets/`. Stage at
the wrong depth and every reference needs rewriting after the checks that would have caught a
mistake in the rewriting.

Use `htmlspec new document` for each one, then replace its example units. That gives you a
document that validates and references the store correctly before you have written a word.

**Version history is carried, never composed.** Copy each archived change's `VERSION` file
across unchanged and name the archived directory `<date>-<name>` from the source. Do not put a
version in the directory name: the archival gate reads the recorded file, so a version in the
name is both a naming violation and a version nothing reads.

Set the declaration's `version` to the **greatest** version among the migrated archives.
Compare numerically — `0.10.0` is ordered after `0.9.0`, and text comparison says otherwise. Get
that backwards and the migrated project's next archival passes without advancing past its own
history.

**Carry the release record.** Where several source changes record one version, carry that
version onto each of them and carry the changelog entry that explains it. Do not invent distinct
versions to separate them: that records releases that never existed. A migrated project whose
release record lost the entry fails its own release check on arrival, reporting a collision it
cannot explain.

### 6. Check the complete tree, not each document

Check documents while you draft them — that is the cheapest feedback available and nothing
prohibits it. What is prohibited is **ending** at the first failure.

When the tree is complete, `htmlspec migrate promote --staged` runs `validate`,
`format --check`, and `assets` over all of it and reports every occurrence together:

```bash
htmlspec migrate promote /tmp/migration/plan.html \
  --survey /tmp/survey.json --staged /tmp/migration/staged --into <destination> --check
```

**Read the shape of what it reports, not the first line.** A mapping fault is systematic: a rule
that is wrong for one scenario is wrong for every scenario. "Every scenario in every
specification reports the same diagnostic" is a different finding from "specification 3 is
broken", and it is the finding that points at the repair. If you find yourself fixing the
twelfth document by hand, stop and fix the rule.

`--check` performs nothing. Use it here.

### 7. Promote

```bash
htmlspec migrate promote /tmp/migration/plan.html \
  --survey /tmp/survey.json --staged /tmp/migration/staged --into <destination>
```

One step. The destination holds either the whole migrated project or none of it, and it refuses
while the destination already holds anything it would place rather than overwriting or merging.

**The source project survives.** Nothing in this operation modifies or deletes it, and removing
it is a separate act a contributor takes after reading the result. Every fault a migration can
have is a judgment fault found by reading the output, so keeping the input is what makes those
faults recoverable.

## Use the format's expressive capacity

A migrated document whose content is the source's paragraphs with markup around them has moved
the bytes and delivered none of the reason for moving. The source used prose in places where it
had nothing better; html-spec does have something better, and using it is part of the work
rather than a flourish on top of it.

Three cases, from this project's own history:

**A comparison that was a table already.** The change that rewrote report guidance counted
artifact-subjected lines in each of seven instruction bodies, and its proposal put them in a
Markdown table — 7 rows, 2 columns, one number each. That is a table in either format and it
carries across unchanged. **Not everything needs upgrading.** Recognizing what is already right
is as much of this stage as recognizing what is not.

**A mapping the source expressed as prose but meant as a table.** The same change's design
states, per workflow, what a contributor's own terms are for that stage — "what the question
turned out to be" for `explore`, "what the project now guarantees" for `archive`. Seven
workflows, one phrase each. Where a source states that as a paragraph naming each in turn, it is
a two-column table, and the table is what a reader can scan for their own stage.

**A sequence that was a numbered list because Markdown had nothing better.** The accepted
`artifact-kinds` capability states archival's order as four numbered steps: produce the
specification, validate it, apply its contract, archive the directory. The numbering is real —
step 2 gates step 3 — and a numbered list is the only thing Markdown offers for it. In html-spec
this is a Mermaid diagram with a gate on the second node, and the gate becomes visible rather
than something a reader has to notice in the prose.

The rule to apply: **where the source's format was the limitation, use the destination's
capacity. Where the source's shape was the right shape, keep it.** Both halves matter, and an
agent that upgrades everything produces a document as wrong as one that upgrades nothing.

`<pre class="mermaid">` for a flow or a state machine, `<pre class="vega-lite">` for a
measurement, `<math>` for an expression, an ordinary `<table>` for a comparison. All of them
render offline with no network, which is the contract every document in this format owes.

## What to report

**Open with how much of their project came across, and what did not.** That is what a
contributor asked for and it is the only question this operation answers. The entity counts and
the staging path are annotation on that sentence rather than the sentence itself.

Then, in this order:

- **What you could not carry, and why.** Each entity you dropped or folded, in plain words. This
  is the part a contributor is deciding about, so it goes before anything that went smoothly.
- **What the destination has no way to express**, where you found such a case. Say what the
  source stated, that the destination has no value for it, what you did instead, and what change
  would settle it. Do not present a distinction you mapped onto the nearest available value as
  though it survived.
- **Which distinctions you mapped**, and to what: renamed headings, tasks done differently, marked
  words, partial checks, risks' responses, and predecessors. A mapped distinction is not open, so do
  not report it as open, and do not report an open one as mapped.
- **Where you used the format's capacity, and where you deliberately did not.** A reader who
  sees three diagrams wants to know they were decisions.
- **Where the checks stand.** One line: what `htmlspec migrate promote` reported, including
  anything you left in place and why.

**Name the boundary plainly, and name it whether or not you promoted.** The migration is staged
and checked. The source is untouched. Promotion is the step that changes their repository, and it
has either happened or it has not — say which. A contributor who cannot tell whether their
repository changed has to go and look, which is the one thing a report exists to prevent.

**Name what your references point at, the first time you use each one.** A task identifier, a
diagnostic code, a count of warnings — say in plain words what it is, in the same sentence. The
test: delete the identifiers from your sentence. If nothing is left, they were the claim and
each one needs naming. If the sentence still stands, they were annotation and need nothing. A
list whose first item is named and whose rest are bare numbers is the failure this prevents:
it looks readable and forces every other line to be looked up.

A command, a flag, or a file path already names itself. Do not gloss those.
