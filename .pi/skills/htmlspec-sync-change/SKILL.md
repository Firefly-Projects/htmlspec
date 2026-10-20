---
name: htmlspec-sync-change
description: Apply an html-spec change's contract to the project's accepted specifications — merge the requirements it states into the documents the project holds. Use when a change's work is done and its contract should reach the project before, or independently of, archiving it.
allowed-tools: Bash(htmlspec:*), Read, Write, Edit, Glob, Grep
argument-hint: [the change to sync, or nothing to infer it]
license: MIT
metadata:
  generatedBy: htmlspec 1.0.0
---

Apply a change's contract to the project's accepted specifications.

## What you were asked

$ARGUMENTS

Where that names a change, sync that one. Where it is empty, nothing was passed: infer the
change from the context you have, and say which one you picked before editing anything.

List the candidates rather than guessing between them:

```bash
htmlspec list --json
```

Each change comes back with how much of its work is done, so the selection has a basis other
than the order they happen to be listed in. A change that states a predecessor comes back with
it too, under `follows`: where `follows.resolved` is false, the change it follows is not
archived, which is worth knowing before you merge this one's contract into the project's
specifications.

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

## What this stage is for

A change's specification states the contract the change established. The project's accepted
specifications under `specs/` are the contract the project holds. Until someone merges the
first into the second, the change's requirements exist only inside the change.

Archival checks whether that merge happened and refuses to perform it. This stage is where it
happens.

**You are doing the merge. The program is doing the checking.** That division is stated in the
contract rather than chosen here: updating the accepted specifications is one of four
operations the project requires an agent to perform, and it is the only one of the four that is
a merge — it applies a change's contract into documents that already exist and hold content
this change never touched. The rules that direct it are these instructions. What the project
implements in code is the comparison.

## Start by asking what is left

```bash
htmlspec sync --change "<name>" --json
```

Six sets come back. The comparison judges each requirement by the change state the change
declares on it, as toolchain-packaging's "The command surface reports whether a change reached the
accepted contract" requires:

| Field | Meaning |
| --- | --- |
| `stated` | every requirement identifier the change's specification carries |
| `reached` | those that have reached the accepted specifications: held with the content the change states where it adds or modifies one, under the heading it states where it renames one, and no longer held where it withdraws one. One it carries forward `unchanged`, with no scenario in it the change adds, modifies, or withdraws, is reached when its identifier is held |
| `absent` | those the change adds, modifies, or carries forward that no accepted specification holds — your worklist |
| `untakenRenames` | those the change renames and an accepted specification still holds under another heading — also your worklist. Each names the identifier, the `heading` the change states, and the `acceptedHeading` the accepted specification holds |
| `untakenContent` | those the change adds or modifies, a requirement marked `unchanged` that holds a changed scenario included, whose identifier is held and whose content the accepted requirement does not hold — also your worklist. Each names the `id`; `firstDifferingStatement`, the first statement paragraph that differs, counted from one, or `null` where the statements agree; `scenariosNotHeld`, the scenarios the change keeps whose text the accepted requirement lacks, by the identifiers the change's specification gives them; and `scenariosNotKept`, the scenarios the accepted requirement holds whose text the change does not keep, by the identifiers the accepted specification gives them |
| `untakenRemovals` | those the change marks `removed` that an accepted specification still holds — also your worklist |

`absent`, `untakenRenames`, `untakenContent`, and `untakenRemovals` are what you work through.
Every stated identifier is in exactly one of `reached`, `absent`, `untakenRemovals`, or the untaken
pair, and within the pair it is in `untakenRenames`, `untakenContent`, or both: a requirement the
change renames and modifies carries two facts, and either can be untaken. A non-empty worklist is
not an error and the command succeeds with one: a change mid-implementation legitimately has
requirements that have not landed yet.

Content is compared as text, and never as meaning: statements in order, and scenarios, less those
the change marks `removed`, as a collection, each by its heading and its entries. Notes, the
`renamed-from` paragraph, a requirement's own heading, and every unit's change state are not
content. **The payload says where the two sides differ, and never what either says.** Read both
documents for the text.

If the command refuses, it is because the question cannot be answered rather than because the
answer is bad — the change does not resolve, the specification is absent, or the specification
does not validate. Fix that first; a comparison against an invalid contract means nothing.

## Merging one requirement

For each identifier in `absent`:

1. **Read the requirement in the change's specification.** What does it oblige? What scenarios
   express it?
2. **Read the accepted specifications** and decide which one should hold it. An existing
   capability usually should. A new capability is right only when no existing one governs the
   subject, and inventing one to avoid the reading is the failure this step exists to prevent.
3. **Check that no accepted requirement already states its obligation.** Where the change marks
   the requirement `added`, document-schema's "Semantic units declare a change state" gives that
   state one meaning: a requirement that does not exist in the accepted specifications. A change
   that sharpens or restates an accepted requirement and marks the result `added` stated a
   modification as an addition. **Do not fold it into the accepted requirement.** Its identifier
   would then never reach the contract, `absent` would never empty, and the only report left for
   it would call a merged obligation abandoned. Report it to the contributor instead, naming both
   identifiers. The fix belongs in the change's specification: the accepted requirement stated
   whole and marked `modified` under its own identifier, which "Merging a modified requirement"
   below then merges.
4. **Write it into the accepted specification** as a requirement of its own, under the identifier
   the change gives it, in that document's own arrangement.

**Leave every requirement the change does not state untouched.** The accepted specifications
hold requirements from every change before this one, and they are not yours to rewrite while you
are here. A requirement the change marks `modified` is the one you do rewrite, and only because
the change states it whole: artifact-kinds' "A change's specification states a modified
requirement whole" requires that, so you apply it as it is stated. "Merging a modified
requirement" below gives the procedure. A requirement the change marks `removed` is the one you
delete, under "Merging a withdrawal".

**Never copy a change's specification into an accepted specification wholesale.** The two are
different documents with different jobs. A change's specification states what one change
established; an accepted specification states what the project holds, in an arrangement built
up over many changes. **This is about the document, not about a modified requirement.** A
modified requirement's statements and scenarios do replace the accepted ones, because the change
states all of them. Its specification as a whole states only what one change touched, and
copying it over an accepted one deletes everything else.

## Merging a modified requirement

A requirement the change's specification marks `data-change="modified"` already has an accepted
counterpart with the same identifier. The comparison reads its content as well: where the accepted
requirement does not hold the statements and scenarios the change states, `untakenContent` names it,
with where to start reading. A modified requirement in `reached` has landed.

The contract makes this a step rather than a reading. artifact-kinds' "A change's specification
states a modified requirement whole" requires a change to state such a requirement whole: every
statement and every scenario it will hold once merged, each scenario in its own change state, a
withdrawn scenario stated as a unit marked `removed`, and each scenario carrying the identifier
of the accepted scenario it continues.

For each entry in `untakenContent`:

1. **Find the accepted requirement with that identifier.** Not the one whose heading matches.
   Where the change also renamed it, "Merging a rename" below gives it its new heading.
2. **Give it the statements the change states, reading from `firstDifferingStatement`.** The
   paragraphs before it already agree. A statement paragraph carries no change state, so the
   change's paragraphs are the requirement's paragraphs: one it leaves out was deleted, and one it
   rewrote is stated as rewritten.
3. **Bring in each scenario in `scenariosNotHeld`, and take out the scenarios in `scenariosNotKept`
   the change replaces or withdraws.** Match each to its accepted counterpart by identifier, never
   by heading, because a change may alter a heading. A scenario the change marks `modified` takes
   the change's text, and it appears in both lists under the identifier it carries in both
   documents. One marked `added` is new. One the change marks `removed` comes out. An `added`
   scenario whose identifier the accepted document already uses for another unit goes in under
   another identifier, and the comparison still counts it, because it compares text.
4. **Report, and do not apply, a scenario marked `unchanged` whose text differs from the accepted
   scenario's.** In the payload it is a scenario in `scenariosNotHeld` that the change marks
   `unchanged`. The change says it carries the scenario forward, and its text says otherwise.
   The usual cause is a change written before another change modified the same requirement, and
   applying its text would undo that change's edit without anyone deciding to. The same cause can
   leave the change's statements stale, and a statement carries no mark that would show it. So
   leave the whole requirement unmerged until the contributor has brought the change's
   specification up to date with the accepted text.
5. **Report an accepted scenario the change neither states nor marks `removed`, naming it.** In the
   payload it is a scenario in `scenariosNotKept` whose identifier the change's specification never
   states. Neither keep it nor delete it. Leaving it in place keeps it and dropping it deletes it,
   and the contract gives you neither reading: the text is the same whether the author meant to
   withdraw the scenario or forgot to restate it, and only the author knows which. Leave that
   requirement unmerged until the contributor answers. The answer belongs in the change's
   specification, as the scenario stated `unchanged` or marked `removed`, and then the
   requirement merges as stated.
6. **Report, and do not revert, a difference that is another change's later edit.** Where the
   accepted text holds words another change put there after this change's specification was
   written, merging this one's text would take them out. Leave the requirement unmerged, and say
   which words. The change's specification needs the accepted text, with its own edits made to it
   again.

**Write the current text, never the marks.** A change's specification may mark the words it added
with `<ins>` and the words it deleted with `<del>`, inside a requirement or a scenario it marks
`modified`. The accepted requirement takes the text as it now reads: keep each `ins` element's
content without the element, and drop each `del` element together with its content.
artifact-kinds' "A marked modification reaches the accepted contract as its current text" requires
this. The accepted specification states what the project requires now, and which words a change
moved is history that stays marked in the archived change. A mark left behind is refused under
`schema/change-markup` when you run `htmlspec validate specs` below, and the comparison reads the
same current text, so a merge that kept a deleted word reports the requirement in
`untakenContent` again.

An entry for a requirement the change marks `added` means the accepted specifications already hold
its identifier. Where that accepted requirement is this change's, merged incompletely, finish it as
above. Where it is a different obligation that happens to carry the same identifier, report it: two
obligations cannot share one identifier, and the change's specification needs another.

## Merging a withdrawal

For each identifier in `untakenRemovals`, **delete the accepted requirement that carries it, and
nothing else.** The change marks it `removed`, which is document-schema's "Semantic units declare a
change state" for a requirement the change withdraws. Its text stays in the change's specification,
where a reviewer can see what was withdrawn, and goes out of the accepted one.

A withdrawal is the one case where deleting is correct, and it must be deliberate rather than a side
effect of rewriting a file. Once it is deleted, `htmlspec sync` counts the requirement as reached,
because no longer being held is what the change asked for.

## Merging a rename

A requirement carrying a `<p data-field="renamed-from">` paragraph states the heading it
replaced; its own heading is its new name. The rename keeps the requirement's identifier, which
is why the comparison checks headings for it: its identifier is already held, and an identifier
comparison alone would call a rename nobody applied reached.

For each entry in `untakenRenames`:

1. **Find the accepted requirement with that identifier.** Not the one whose heading matches the
   `renamed-from` text. The identifier is what makes it the same requirement, and the payload's
   `acceptedHeading` is the heading it holds now.
2. **Give it the heading the change states, and keep its identifier.** Keep the identifier span
   beside the heading if the document writes one.
3. **Leave the `renamed-from` paragraph out of the accepted specification.** The accepted contract
   states what the project requires now, under the name it uses now. The history of a heading
   belongs to the archived change that renamed it, as OpenSpec's main specifications carry no
   RENAMED section.
4. **Where the change also modified the requirement** (`data-change="modified"`), apply the
   modification to that same accepted requirement as the change states it, under "Merging a
   modified requirement" above. The rename and the modification are two facts about one
   requirement.

**Never apply a rename as a removal followed by an addition.** That writes a second requirement
with a regenerated identifier, and every reference to the first stops resolving. A requirement's
identifier survives a rename, and that is the reason the contract keeps it stable.

A renamed requirement that is already in `reached` needs nothing: the accepted specification
already carries the heading the change states. That is a rename applied earlier and is not a
conflict.

A renamed requirement in `absent` is one whose identifier no accepted specification holds. Read
its `renamed-from` paragraph. If an accepted requirement carries that previous heading under a
different identifier, the two do not agree about which obligation this is. Report it to the
contributor, naming both identifiers, rather than merging a second copy of the obligation.

## A requirement you should not merge

A requirement the change marks `removed` never appears in `absent`: once deleted it is reached,
and while it is held it is in `untakenRemovals`. What remains in `absent` is a requirement the
change adds, modifies, or carries forward. One may be there because its work was abandoned — the
change stated it, the work went another way, and it is correctly absent forever.

Leave it absent. Say so to the contributor, by name, and say why you concluded it was
abandoned. Do not merge a requirement to make the report go quiet: a requirement written into
the contract because a list was non-empty is a requirement nobody decided on, sitting in the
document that governs the project.

## Finish by asking again

```bash
htmlspec sync --change "<name>" --json
htmlspec validate specs
```

`absent` should be empty, or hold only the identifiers you reported, as abandoned or as a
modification the change stated as an addition. `untakenRenames` and `untakenRemovals` should be
empty. `untakenContent` should hold only the requirements you left unmerged and reported. The
accepted specifications you edited must still validate — you have been writing into the project's
contract, and an invalid one is worse than an unmerged one.

## What to report

**Open with whether the change's contract now reaches the project, and what you left out of it.**
That is what a contributor asked by invoking this, and it is one sentence. The per-requirement
accounting is the evidence for it rather than a substitute.

Then, in this order:

- **Which accepted specification each requirement went into**, and each requirement the change
  marks `added` that you left unmerged because an accepted requirement already states its
  obligation, naming both identifiers.
- **Which requirements you renamed**, naming each by its identifier with the heading it had and
  the heading it has now.
- **Which modified requirements you merged**, and each you left unmerged because the change marked
  a scenario `unchanged` whose text differs from the accepted scenario's, or because the
  difference is another change's later edit. Name the scenario or the words, and the requirement
  they belong to.
- **Each accepted scenario a modified requirement neither stated nor marked `removed`**, named
  with its requirement, and the requirement you left unmerged until the contributor says whether
  the scenario was withdrawn. This is the question only the author can answer.
- **Which withdrawn requirements you deleted**, by identifier, from which accepted specification.
- **Which identifiers you left absent**, and why each was abandoned deliberately rather than
  missed. This is the part a contributor cannot reconstruct without you.
- **What `htmlspec sync` reports now**, and whether the accepted specifications still validate.

**Name what your references point at, the first time you use each one.** A task identifier, a
diagnostic code, a count of warnings — say in plain words what it is, in the same sentence. The
test: delete the identifiers from your sentence. If nothing is left, they were the claim and
each one needs naming. If the sentence still stands, they were annotation and need nothing. A
list whose first item is named and whose rest are bare numbers is the failure this prevents:
it looks readable and forces every other line to be looked up.

A command, a flag, or a file path already names itself. Do not gloss those.

The change stays active. Archiving it is a separate stage, and archival makes this same
comparison again — it should find nothing. **Do not archive a change while one of its modified
requirements is held.** A held requirement is in `untakenContent`, and archival reports it and asks,
but an answer of yes, or `--yes` where nothing can answer, archives the change with its edit never
merged, and nothing later brings an archived change back.
