## Context

See proposal.md for why. The shape of the current system that the approach depends on:

- **The projection reads `ins` and `del` as presentational markup.** Projection specification
  section 8.2 makes every element without a reserved class presentational, and its text
  contributes to the field around it. In `crates/htmlspec-parse/src/project.rs`, `own_descendants`
  walks through any such element and `collect_text` keeps its text. Only `script`, `style`,
  `template`, and `noscript` give no text (section 8.5). So an inserted word is right in the model
  and a deleted one is wrong (`evidence/the-measurement-2026-09-30.md`, section 2).
- **`reject-change-markup-inside-a-unit` rejects both elements in and around every unit.** Its rule,
  `schema/change-markup` in `crates/htmlspec-validate/src/rules/change_markup.rs`, reads the parse
  tree and recognizes a unit with `UnitKind::from_class`. It applies in every kind and every
  declared version, and it says nothing about markup in prose outside every unit. It rejected `ins`
  with `del` on purpose, so that this change only relaxes it (its design, decision 4). Its review,
  archived with it, added two cases the rule as built decides first: an element whose own class
  names a unit is that unit (`Reach::Itself` in the rule), and an element named `ins` or `del`
  inside SVG or MathML is not change markup.
- **`resolve-a-reference-across-its-change` says where a document sits.** Its `Placement` has five
  variants: `Loose`, `ChangeDocument`, `ChangeSpecification`, `AcceptedSpecification`, and
  `RunbookDocument`. `ProjectRoot` derives it once, by position, in the CLI. `validate_source`
  passes it to the engine, and `Subject` exposes it through `placement()`. Every conformance fixture
  is `Loose`. That design names this change as the reader of `ChangeSpecification` (its decision 3).
- **`state-a-modified-requirement-whole` makes a modified requirement a whole statement.** Every
  statement and every scenario is stated, and each scenario carries its own change state. The marks
  this change allows sit inside that whole statement.
- **`see-a-modification-the-merge-did-not-take` compares text.** It compares a change's added and
  modified requirements with the accepted ones as normalized projected text: `content_difference`
  in `crates/htmlspec-cli/src/commands.rs` compares the projected `statements`, and each scenario's
  projected title and entries. Task 1.1 read it as built.
- **Where `modified` is used.** In the staged migration, every unit marked `modified` is a
  requirement (34) or a scenario (13) in a change's specification. The staged accepted
  specifications hold 138 requirements, 590 scenarios, and no unit of any other kind (evidence,
  section 4).
- **Only `project.rs` builds the model's unit types**, so a new field on `Requirement` and
  `Scenario` is set in one module.
- **The stylesheet says nothing about either element.** The browser draws `ins` underlined and `del`
  struck through (HTML Standard, section 15.3.4). The project's screen stylesheet also underlines
  every link, and tells a link from other text by its accent color (evidence, section 5).
- **The formatter treats both as unknown, and so inline.** Inline markup is canonical as written. A
  block-level `<del><p>` is laid out with the `p` on its own line, and a second run changes nothing
  (evidence, section 3).
- **artifact-kinds calls the task rule "the only placement rule in the vocabulary"**, in "A
  specification states the settled contract as built". The schema's `kind` description says the
  same. In both places "placement" means which units a kind may carry. `kind/*` holds exactly two
  rules, and `the_kind_namespace_has_exactly_two_rules_and_constrains_no_upstream_kind` pins that.

## Goals / Non-Goals

**Goals:**

- The model holds the current text in every field, whatever markup a document uses.
- A modified requirement or scenario in a change's specification can show which words the change
  added and which it deleted, readably, with scripts off.
- The marks stay in the archived change and never reach the accepted contract, and validation
  catches a merge that leaves them there.
- The rule only relaxes. Every document `reject-change-markup-inside-a-unit` accepts is still
  accepted.
- One place decides where a document sits, and the rule reads it rather than working it out again.

**Non-Goals:**

- A diff view. That is `show-what-a-modification-changed`, and it reads the accepted text, which
  one document does not hold.
- Marking the archive or the staged migration. That is `migrate-into-the-extended-vocabulary`.
- Tasks. A task never carries change markup.
- A completeness check for the marks. It needs the accepted text.
- When a word changed. `datetime` stays ignored (projection section 2.2).

## Decisions

### 1. The projection gives the two elements one meaning, wherever they sit

`ins` stays presentational under sections 8.2 and 8.3, because its text is current text. `del`
gets a rule of its own: its content supplies no field, anywhere. `collect_text` gives no text for a
`del`, so every normalized field is current text: a statement, a scenario entry, a title, a note, the
summary, and a caption. A deletion's own text is read by a walk that does not skip it. The walk
over a unit's own descendants marks what sits inside a `del`, and no field rule reads from there.
So a paragraph inside one is no statement, a `dd` inside one is no scenario entry, a heading
inside one is no title, and an anchor inside one is no reference. The anchor still projects into
`links`, because the page still carries the hyperlink and
`refs/broken-link` should still check where it goes. A unit inside a `del` is still found and still
projects, and validation rejects the `del` around it.

Every rule that reads the model then reads current text. For example, `schema/weak-modal` judges a
statement without its deleted words, so a modified requirement whose only SHALL was deleted states
no obligation, which is true.

**Considered: letting the projection depend on where the document sits.** Rejected. The projection
reads one parsed tree and knows nothing about a project (section 10.1). Placement is derived by the
CLI and handed to validation. If the model depended on it, the same bytes would project differently
by path, and so would every mapping fixture and every `project-model` run. Where the markup is not
permitted, validation rejects the document anyway.

**Considered: applying the rule inside units only.** Rejected. `reject-change-markup-inside-a-unit`
measured the summary reading "no new spellingone spelling, store" and a caption reading
"beforeafter" (its evidence, section 5). A rule for units only would leave both. It would also make
the text walk ask where it is. Applied everywhere, the rule settles that change's open question
about the summary, as far as the model goes, with no validation rule.

**Considered: making `del` give no text and recording nothing, as for `script`.** Rejected. The model
would then say less than the page: a consumer reading the model could not tell a unit that deleted
a word from one that never had it.

### 2. `deletions`: one list of text on a requirement and on a scenario

The field is a list of strings, in document order, one entry for each `del` that no other `del`
encloses. The entry is the normalized text of all the element's content, an `ins` inside it
included. An empty `del` records nothing. The list is serialized only when it holds an entry, so the
model of every document without a `del` is byte for byte what it is today.

A sentence with an insertion and a deletion is represented as two facts. The field holds the
sentence as it now reads. `deletions` holds the deleted phrase. Nothing pairs the two. The markup
keeps the positions, and `show-what-a-modification-changed` can show them against the accepted text.

**The name is `deletions`.** It is the plural noun for what `del` marks, in the form the model already
uses for lists: `statements`, `notes`, `references`.

- **Considered: `withdrawn`, the working name.** Rejected. It already names a unit marked `removed`,
  in document-schema's scenario "Requirement being withdrawn", in `sync.md`, in 63 lines of
  `crates/`, and in this set's other changes: `resolve-a-reference-across-its-change`'s `Declared`
  has a field `withdrawn` holding the identifiers of removed units (evidence, section 6). A
  requirement's `withdrawn` words and a withdrawn requirement would be two meanings of one word in
  one model.
- **Considered: `removed` or `removedText`.** Rejected. `removed` is a change state.
- **Considered: `deleted`.** Rejected. It is an adjective, and every list in the model is a noun.

**Considered: one list for each field, such as `statementDeletions` and `whenDeletions`.** Rejected.
A deleted paragraph belongs to no field, so its list would have to be guessed. It would add a
field for every text field of two units, and nothing reads which field a deletion came from.

**Considered: statements as runs of text marked current or deleted.** Rejected. It reshapes
`statements` from a list of strings, which is a major change to the contract, and it breaks every
reader of the model to serve a view that `show-what-a-modification-changed` provides.

**Considered: an `insertions` list as well.** Rejected. Inserted text is current text and is already
in its field. A second copy states it twice.

**Considered: `deletions` on all seven units.** Rejected. An accepted specification holds only
requirements and scenarios (evidence, section 4), and decision 3 permits the markup in those two
only. A field on the other five could hold something only in a document that validation rejects.
`renamedFrom` sets the precedent: it exists on a requirement alone, a misplaced `renamed-from`
paragraph projects nowhere, and validation reports it (projection section 4.1.1). A `del` in another
unit likewise projects nowhere, and the rule reads the tree. Adding the field later, if the
permission widens, is a minor change.

### 3. The markup is permitted in one place, decided by the nearest unit

The permission holds when all of these do:

1. the document is a change's specification (decision 4);
2. the nearest unit enclosing the element is a requirement or a scenario;
3. that unit is marked `modified`;
4. the element does not sit in, or enclose, the unit's heading or a `renamed-from` paragraph;
5. the element encloses no unit, and is not one because its own class names a unit.

The cases at the edges, each decided:

| Case | Verdict | Why |
| --- | --- | --- |
| a `modified` scenario inside a `modified` requirement | permitted | Each scenario carries its own state, as `state-a-modified-requirement-whole` requires, and this one says its text was edited. |
| an `unchanged` scenario inside a `modified` requirement | rejected | It states that its text is the accepted text. A mark inside it contradicts that. `sync.md` is to report an `unchanged` scenario whose text differs, and this would be one. |
| a unit marked `added` | rejected | Every word is new, so there is no accepted text to mark against. |
| a unit marked `removed` | rejected | It is withdrawn whole, and its state says so. |
| markup in a heading, or in a `renamed-from` paragraph | rejected | A heading is a name. A requirement's new name has its own paragraph, which the merge comparison reads. A heading written `The <del>old</del> name` would give `renamedFrom` and the heading name two readings. A scenario keeps its identifier when its heading changes, so its old heading needs no mark. |
| a block-level `<del><p>…</p></del>` | permitted | It deletes a whole paragraph, and its text is one entry of `deletions`. `<ins><p>…</p></ins>` likewise adds one. |
| `del` or `ins` enclosing a unit | rejected | A unit's change state is how a document says a unit is new or withdrawn. |
| `del` or `ins` whose own class names a unit | rejected | It is that unit, and the same reason holds. `reject-change-markup-inside-a-unit`'s review added the case. |
| a decision, evidence, risk, or verification unit marked `modified` | rejected | No accepted specification holds one, so no accepted text exists for the marks. No staged unit of these kinds is `modified`. |
| a task | rejected | Settled for this set of changes: `dropped` with a reason already says that a task's words were given up. A task in a specification is also `kind/task-in-specification`. |
| a document outside any project | rejected | Decision 4. |

**Marking is permitted, not required.** Whether a unit's marks are complete can only be checked
against the accepted text. The 34 staged modified requirements carry none, and requiring them would
make every one of them invalid.

**Considered: judging by every enclosing unit, or by the outermost.** Rejected. A scenario's state is
its own. Judged by the requirement, an `unchanged` scenario inside a `modified` requirement could
carry marks it denies having.

**Considered: permitting the markup in any unit marked `modified`, wherever the document sits.**
Rejected. An accepted specification could then keep its marks after a merge, and no check would say
so. That is the establish-a-runbook bold again (evidence, section 4).

**Considered: permitting it in any of a change's documents.** Rejected. A proposal's and a design's
units are not merged, so their marks would be claims about nothing the merge applies. A proposal
that wants to show an edit already can, in prose outside every unit. Tasks are settled.

**This is not a second "placement rule" in artifact-kinds' sense.** That sentence is about which
units a kind may carry, and this rule is not in `kind/*`. A change's specification and an accepted
specification are the same kind, and this rule turns on where a document sits and on a unit's
state. The contract text of the modified requirement avoids the word "placement" for that reason,
and says "where the document sits".

### 4. Where a document sits comes from `Placement`, and a loose document gets no permission

The rule reads `subject.placement()` and permits the markup only at
`Placement::ChangeSpecification`. That variant is a `specification.html` directly within a change's
directory, active or archived (`resolve-a-reference-across-its-change`, decision 1). Every caller
that validates in the CLI goes through `validate_source`, which derives the placement: `validate`,
archival's gate, `sync`, and promotion's conformance check. So the permission is the same in all of
them, and none can forget it. Callers outside the CLI, such as the conformance harness, pass
`Loose`.

A loose document gets no permission. It sits in no change, so the rule has nothing to grant it on,
and the alternative is a document claiming the permission by what it says about itself. The
stricter answer is also the loud one: a document wrongly refused says so, and one wrongly permitted
does not.

**The valid case cannot be a conformance fixture, because a fixture is loose.** So it is tested the
way `resolve-a-reference-across-its-change` tests resolution, and the way `topology.rs` and `runs.rs`
test rules whose subject is more than one document. A suite, `conformance/tests/change_markup.rs`,
validates documents through the public API at each placement. It takes its input from a new
committed fixture, `invalid-schema-change-markup-in-a-modified-unit`: the generator's base with its
requirement marked `modified` and a `del` and an `ins` in its statement. Loose, that fixture pins for
an independent implementation that a modified unit outside a change is still refused. At
`ChangeSpecification`, the suite asserts that the same bytes report no `schema/change-markup`, and at
every other placement that they do.

`resolve-a-reference-across-its-change` adds a property case to `conformance/tests/references.rs`:
every committed fixture reports the same rules at every placement as it does loose. This change
makes a second rule depend on placement on purpose, so the case is restated, not weakened. It names
its one exception: `invalid-schema-change-markup-in-a-modified-unit` at `ChangeSpecification`
reports what it reports loose, less exactly its `schema/change-markup` diagnostics. Every other
fixture, and that one at every other placement, reports the same as loose. A named exception cannot
widen without an edit to the test, and a fixture that later differs by placement fails it.

**Considered: a `placement` key in a fixture's `expected.json`.** Rejected. Placement is where a
document sits, and a fixture sits nowhere. A key would be the fixture stating its own place, which
`resolve-a-reference-across-its-change` rejected for documents. The precedent for a rule that needs
surroundings is a suite of its own.

CLI tests in `crates/htmlspec-cli/tests/cli.rs` check the derivation end to end, in a real project:
the same specification bytes pass as an active or archived change's `specification.html`, and fail
as an accepted specification, as a change's `proposal.html`, as a named file under a change's
`evidence/`, and outside any project.

### 5. One rule, the same identifier, and a reason in every message

The rule stays `schema/change-markup`, as `reject-change-markup-inside-a-unit` chose it: a name for
the fault that does not change when the place changes. The requirement keeps its heading.

**Considered: renaming the requirement**, to something like "Change markup is permitted only where it
marks a modification". Rejected. The heading states the rule's default, which still governs every
placement but one. The requirement already states one exception in its body, for prose outside every
unit, and this is a second. A rename would also make the re-base carry a RENAMED pair for a
requirement no document yet cites.

The decision is one pure function, so each case is a unit test with no tree and no project:

```rust
enum Verdict { Permitted, Refused(Refusal) }

enum Refusal {
    IsAUnit,                              // put the unit on an element that is not change
                                          // markup, and write data-change on it
    EnclosesAUnit,                        // write data-change on the unit instead
    NotMarkable(UnitKind),                // only a requirement or a scenario carries marks;
                                          // a task is dropped with a reason instead
    NotAChangeSpecification(Placement),   // write the text as it now reads
    InAName,                              // a requirement's new name goes in renamed-from
    NotModified,                          // the nearest unit is marked otherwise
}

fn verdict(placement: Placement, markup: &MarkupContext) -> Verdict;
```

`MarkupContext` holds what the walk found for one element: whether the element is itself a unit,
the nearest unit's kind and whether its state is the modified one, whether the element sits in or
encloses a name, and whether it encloses a unit. The first two refusals keep the messages
`reject-change-markup-inside-a-unit` gave them, which name the unit and the change state to write. Refusals are checked in the order above, so the message names the fix that applies. Markup
in an accepted specification is told to state the text as it now reads, not to use `renamed-from`
or to mark the unit `modified`, because neither belongs there. The modified value is found in the
schema by its stem, as `refs::removed_state` finds the removed one, because
`crates/htmlspec-validate/tests/schema_rust_split.rs` forbids the literal in rule code.

### 6. The merge writes the current text, in a requirement of its own

artifact-kinds gains "A marked modification reaches the accepted contract as its current text". The
merge unwraps each `ins`, drops each `del` with its content, and leaves no change markup in the
accepted specification. The archived change keeps its marks. The program performs none of this:
the merge is an agent's work, and the program never writes an accepted specification. What the
program does is refuse leftover marks, because an accepted specification is
`Placement::AcceptedSpecification` and the rule gives it no permission.

**Considered: modifying "Every synthesis and every contract update is agent-driven".** Rejected.
`state-a-modified-requirement-whole` modifies it, so this change would need a re-base onto that
edit. That requirement is also about who performs four operations, not about what a merge leaves
behind. "A rename reaches the accepted contract as a new heading on the same identifier" is the
precedent for a requirement of its own.

**Considered: having `sync` or archival strip the marks.** Rejected. toolchain-packaging's "The
command surface reports whether a change reached the accepted contract" says the operation "SHALL
NOT modify an accepted specification", and "Archival assesses the merge and never performs it".

### 7. The comparison reads current text, so it needs nothing new

`see-a-modification-the-merge-did-not-take` compares a modified requirement's projected text with
the accepted text. Under decision 1 the change's projected statements are the current
text. A merge that wrote the current text compares equal, and one that kept a deleted word or lost an
inserted one does not. So no requirement of that change is modified here. The scenario that states
this sits in decision 6's requirement, beside the fact it rests on. Task 1.1 checked that the
comparison, as built, reads projected text, and it does.

### 8. The screen stylesheet draws both marks by shape and announces them

A new revision of `htmlspec.css`:

- draws `ins` with a double underline, so that an inserted word does not look like a link. The
  stylesheet underlines every link and tells the two apart by hue alone otherwise (evidence, section
  5). document-template's "The visual system favors comprehension over decoration" says color SHALL
  NOT be the only means of conveying meaning;
- states `del`'s line-through, so the stylesheet does not depend on the browser's default for it;
- gives both a start and an end marker in generated content that a screen reader announces and the
  page does not draw. Most screen readers do not announce `del` on their own (evidence, section 5).
  Without the markers, a listener hears the deleted phrase as current text, which is the defect this
  change removes from the model. The markers are limited to these two elements, as the reference
  that recommends them advises.

The revision is added through `assets/vendor.lock.json`'s procedure: the new bytes at their own
address, `own` moved to them, and the revision `own` names now, `aac8967609f80d9f`, moved to
`retained`. The templates name the
stylesheet by `{{ASSET:htmlspec.css}}`, so no template changes for it. An existing document keeps the
revision it names and gets the browser's defaults, which still draw both marks. The print
stylesheet is unchanged. The screen stylesheet applies on paper too, and links are not underlined
there.

**Considered: no stylesheet change.** Rejected, for the link and the screen reader above.

**Considered: a background or text color.** Rejected. It is hue alone.

**Considered: a visible word beside every mark.** Rejected. It would put labels inside normative
sentences, and the shape already gives a sighted reader the fact.

**Considered: hiding `del`.** Rejected. document-template says styling "SHALL NOT hide or reorder
semantic content", and a reviewer needs to see what was deleted.

### 9. The formatter stays as it is

Both elements stay unknown, and so inline. The measurement shows inline marks canonical as written,
and a block-level `<del><p>` or `<ins><p>` laid out with the paragraph on its own line, stable on a
second run, with the same statements before and after (evidence, section 3).

**Considered: adding both to `BLOCK`.** Rejected. They are used inside sentences, and a block element
is written on its own line. In `<ins><code>sync</code></ins>,` a line break before the comma would
render as a space. The element table's own comment names this as the reason an unknown element is
inline. A formatter test pins both measured forms, so a later edit to the table cannot change them
unseen.

### 10. The instructions

- `archive.md`, step 2, after the paragraph `state-a-modified-requirement-whole` adds for a modified
  requirement: how to mark the words, with the one-word example. `ins` for added words and `del` for
  deleted ones, in place, only in a requirement or scenario marked `modified`, and only in its
  statements, notes, and scenario entries, never in a figure's source. A paragraph deleted
  whole goes in `<del><p>…</p></del>`, and a paragraph rewritten throughout is deleted and added
  whole rather than marked phrase by phrase. Never in a heading, a `renamed-from` paragraph, a unit
  marked otherwise, or around a unit.
- `sync.md`, in the section on merging a modified requirement that `state-a-modified-requirement-whole`
  adds: write the current text, keeping each `ins`'s content without the element and dropping each
  `del` with its content. Its closing `htmlspec validate specs` already catches a mark left behind.
- `archive.md`, step 5: the same, in one bullet that points to `sync.md`.

No other instruction describes writing a change's specification or merging one:
`state-a-modified-requirement-whole` recorded that `new.md`, `continue.md`, `ff.md`, `apply.md`,
`verify.md`, and `explore.md` leave both to archival. `migrate.md` is
`migrate-into-the-extended-vocabulary`'s.

**Considered: guidance in `assets/template-specification.html`.** Rejected, for the reason
`state-a-modified-requirement-whole` gave. The template's example units are placeholders that
`review/example-content` asks an author to replace, and its one example requirement is `added`. Its
declared version changes in this change anyway, with its pin.

## Risks / Trade-offs

- **[`record-a-partial-verification` also makes a new revision of `htmlspec.css`.]** → Its task 5.1
  retires `aac8967609f80d9f`. If this change lands first, that task must retire this change's
  revision instead, and carry both edits. Task 6.1 here retires whatever revision `own` records at
  the time, so the order of landing does not matter to this change. The other change's task needs
  the same wording, and the report on this change says so.
- **[A `del` inside a figure's source is drawn by the diagram renderer and missing from the model's
  `source`.]** → Section 6.2 still applies to preserved text, and the rule in decision 1 applies
  inside it too. The measured count of change markup anywhere is zero, a figure states no
  obligation, and the instructions tell an agent to mark statements and entries only. A later
  change can refuse change markup inside `pre` if the case arises.
- **[A figure whose whole caption, or whole diagram source, is in a `del` stops validating.]** →
  The caption or the source is then empty, and the existing rule for each names it: projection
  fault 7.2 for the caption, and `viz/diagram-empty` for the source, which reads the model's
  `source`. A deletion that removes only a diagram's first keyword meets `viz/diagram-unrecognized`
  the same way. These are the constructs that
  `reject-change-markup-inside-a-unit` accepts outside every unit and this change refuses. No
  document carries either.
- **[An author marks some words and not others.]** → A mark claims only what it marks. The
  comparison in `sync` and archival, and the diff in `show-what-a-modification-changed`, read the
  accepted text, and they are where incompleteness shows.
- **[A paragraph dense with marks is hard to read.]** → The instructions say to delete and add a
  rewritten paragraph whole.
- **[A document with marks is read by an older toolchain.]** → A document written for this contract
  declares its version, which an older toolchain rejects. One that declares an older version is
  rejected by `schema/change-markup` on a toolchain with `reject-change-markup-inside-a-unit`. On a
  toolchain before both, its model keeps the deleted words, which is today's defect.
- **[Screen-reader users who have turned verbosity down hear two more markers per mark.]** → The
  markers are on these two elements only, where missing a deletion changes what a sentence obliges.
- **[`resolve-a-reference-across-its-change` builds `Placement` differently from its design.]** → Task
  1.1 reads the type as built, and decisions 4 and 5 are corrected to match. The compiler reports
  every use of a renamed variant.
- **[`see-a-modification-the-merge-did-not-take` compares something other than projected text.]** →
  Task 1.1 checks it before anything is built, per decision 7.

## Migration Plan

**The document contract moves one minor version**, from whatever version task 1.1 finds. That is
`0.4.0` if `reject-change-markup-inside-a-unit`'s `0.3.0` is the only move before it. The change adds
a field, `deletions`, and a projection rule, 8.6, which the table in
`crates/htmlspec-schema/CHANGELOG.md` puts under a minor bump. It also reshapes how one element
projects: `del` stops being presentational under section 8.2. The table calls a reshaped rule major,
and its `0.x` clause permits it in a minor, as `reject-change-markup-inside-a-unit` used it. No
staged document carries a `del` (evidence, section 7). Two tracked documents do, both conformance
fixtures that `reject-change-markup-inside-a-unit` generated: `invalid-schema-change-markup`, whose
statement now projects as the current text, and `valid-change-markup-outside-a-unit`, whose markup
sits outside every unit and projects nowhere. Neither verdict changes, and the invalid fixture's note
is reworded, because the reason it gave was that the projection ignores the markup
(`evidence/the-measurement-repeated-2026-10-05.md`). No other document's model changes, and the only
constructs that can stop validating are the figure's caption and source in the risks above. The changelog entry
says both in its first lines. Every earlier version stays in `schemaVersion`'s enumeration, and a
document declaring one gets the new projection and the new permission, because the vocabulary
accepted is the newest version's.

**The release is expected to be `0.23.0`**, the next minor after the four changes before it in this
set. The contract moves, and `validate` accepts documents it refused. The contributor confirms the
increment at the release task.

Rollback is a revert. A document that adopted the marks then fails `schema/change-markup`, which is
loud, and names each mark.
