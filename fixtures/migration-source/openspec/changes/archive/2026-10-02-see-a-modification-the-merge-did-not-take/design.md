## Context

See proposal.md for why. The shape of the current system that the approach depends on:

- **One function makes the comparison.** `merge_state` in `crates/htmlspec-cli/src/commands.rs`
  (3545-3600) serves both `sync` and archival's `findings_for`. It collects the identifiers the change's
  specification states, and maps every accepted requirement's identifier to its heading name. It then
  puts each stated identifier into `reached`, `absent`, or `untaken_renames`. **It never reads a
  requirement's change state.** `grep -c '\.change'` over the function's lines prints `0`
  (`evidence/the-measurement-2026-09-30.md`, section 7).
- **"Has the contract arrived" is stated twice.** `sync` prints "the change's contract has been applied
  in full" when `state.absent.is_empty() && state.untaken_renames.is_empty()` (2475). Archival asks
  `Findings::needs_sync` (3294), which restates the same two clauses over `Findings`' own copies of
  the two sets. The parity tests pin the sets, not the two predicates.
- **The model already holds the text to compare.** A projected `Requirement`
  (`crates/htmlspec-parse/src/model.rs`) carries:
  - `change`;
  - `statements`, as normalized text from every unmarked and `statement` paragraph among its
    descendants, list items included (projection specification, sections 4.1 and 6.1);
  - `scenarios`, each with `title`, `given`, `when`, `then`, `notes`, `change`, and `id`
    (section 4.2);
  - `notes`;
  - `renamed_from`, which never supplies a statement (section 4.1.1).

  `HeadingName` (section 4.1.2) is defined for any unit, so a scenario has a heading name too. No new
  field and no new projection rule is needed.
- **Schema values are not written as literals.** `crates/htmlspec-validate/tests/schema_rust_split.rs`
  forbids `"removed"` and `"unchanged"` in rule code. The CLI follows the same discipline:
  `runs::is_withdrawn` finds the withdrawn state by a stem from `schema_facts::enum_values`, and
  `rules::review::material_states` finds `added` and `modified` by excluding the other two.
- **The instructions, as `state-a-modified-requirement-whole` leaves them.** `sync.md` works through
  `absent` and `untakenRenames`. That change adds a section on merging a modified requirement, which
  works from the change's specification because the comparison cannot see one. `archive.md` step 5 says
  "A requirement the change withdrew is removed". Step 4 then says that after the merge, each identifier
  left in `absent` "should be a requirement the change deliberately withdrew". So archival expects a
  requirement the merge deleted to be reported as not arrived, and that is exactly what it does
  (evidence, section 4).

## Goals / Non-Goals

**Goals:**

- `sync` and archival report a modification, an added requirement, or a withdrawal that the accepted
  specifications have not taken.
- A withdrawal the merge applied counts as reached, so a correct merge leaves nothing to report.
- The comparison stays exact. It reads text and never meaning, and it never proposes merge text.
- One definition of "reached", and one predicate for "reached in full", both shared by the two
  operations.
- Instructions that tell an agent what to do with every set the payload carries.

**Non-Goals:**

- **A diff.** Where the two sides differ is reported, and what they say is not. A diff view is
  `show-what-a-modification-changed`'s.
- **Checking that a change's specification states a modified requirement whole.** An accepted scenario
  the change neither states nor marks `removed` shows up here only if the merge kept it. That is enough
  for this operation's question. Whether the change's specification is complete is
  `state-a-modified-requirement-whole`'s rule, and it is carried out by the merging agent.
- **A requirement's heading where no rename is stated**, and notes. Both are left out on purpose. See
  decision 2.
- **Validation.** Nothing here runs in `validate`.

## Decisions

### 1. The change state decides what "reached" means

| The change marks the requirement | It has reached the contract when | Otherwise it is in |
| --- | --- | --- |
| `added` or `modified` | an accepted specification holds its identifier with the content the change states | `absent` if the identifier is not held, and `untakenContent` if it is |
| `removed` | no accepted specification holds its identifier | `untakenRemovals` |
| `unchanged` | an accepted specification holds its identifier | `absent` |

A stated rename is compared as it is today, for `unchanged` and `modified` requirements alike.
Validation already rejects a `renamed-from` paragraph on an `added` or `removed` requirement.

**A requirement marked `unchanged` that holds a changed scenario is judged as `modified`.** The code
review of this change's pull request found the case: a scenario marked `added`, `modified`, or
`removed` inside a requirement marked `unchanged`. Nothing in validation or the contract rejects it,
and judged by the requirement's own mark, the added scenario reached the contract unmerged while
`sync` said "applied in full". The change states content inside such a requirement, so the state
read into the type below is the requirement's own, except that one carrying the requirement forward
gives way to "establishes" when a scenario in it is changed. **Considered: a validation rule rejecting
a changed scenario inside an `unchanged` requirement.** Rejected here. It changes what every document
may say, which is document-validation's subject and not this comparison's.

The state is read once per requirement into a closed type with three cases:

- **establishes**, for the material states, derived from the schema's `changeState` values by
  excluding the other two, as `rules::review::material_states` derives them in the validator;
- **withdraws**, for the state the CLI's `runs::is_withdrawn` already finds by its stem;
- **carries forward**, for every other value.

The last case is the schema's default state, `unchanged`. An unknown value cannot reach this function,
because both callers refuse a specification that does not validate before they compare it. The match
over the three cases is exhaustive, so a fourth state added to the schema lands in "carries forward"
until someone decides otherwise. That is the behavior every requirement had before this change.

**Considered: compare `modified` requirements only.** Rejected. An `added` requirement's content is
also text the change establishes and the merge writes. An added requirement merged without one of its
scenarios is reported as reached today, and so is an added identifier that happens to equal the
identifier of an unrelated accepted requirement. Both are this change's defect.

**Considered: compare every requirement's text, `unchanged` included.** Rejected. The text of a
requirement carried forward untouched moves whenever another change modifies it. A difference then
reports that other change's edit as this change's unapplied work. The merge must not act on it,
because acting on it would revert the other change. A finding nobody can act on is noise.

**Considered: leave a withdrawal as it is, and explain in the instructions that a withdrawn requirement
belongs in `absent`.** Rejected. That is today's behavior, and the evidence shows its cost. Against the
unmerged accepted specification, `sync` says the change's contract "has been applied in full" and
archival says "ready". After a correct merge, archival says "This change has not been synced" and asks
(evidence, sections 4 and 6). `sync`'s note for an absent identifier opens "Merge each into the
specification that should hold it". An agent that follows that clause puts back the requirement it
just deleted.

### 2. Content is statements in order and scenarios as a collection, compared as projected text

A requirement's content is its `statements`, in order, and its scenarios less those the change marks
`removed`. Each scenario is compared as its heading name and its `given`, `when`, and `then` entries,
each list in order. The scenarios are compared as a multiset: the accepted requirement holds exactly
these scenario texts, in any order, and no others. So a scenario the change marks `removed` that the
merge kept is reported, because it is a text the accepted requirement holds beyond the change's.

Every text is compared after whitespace normalization, which is the heading comparison's test. The
projection already normalizes, except inside `code` (section 6.2). The comparison reads the projection,
not the markup, so emphasis, links, and attributes are not compared, and neither is a unit's
`data-change`, which differs between a change's specification and an accepted one by design.

**Statement order is compared.** Statements are prose read in sequence. Moving a paragraph changes what
"That division" or "The reason" in the next one refers to, and the accepted requirement is read in its
order.

**Scenario order is not.** A scenario is a unit that is checked on its own, and nothing in the contract
gives its position a meaning. OpenSpec 1.13.1's check that a MODIFIED block keeps every scenario matches
scenarios by name, not position. The merge at `9c3a3c7` appended the two scenarios it kept after the
delta's five, and the result reads correctly. Both facts are recorded in
`state-a-modified-requirement-whole`'s evidence, sections 1 and 2.

**Considered: compare scenario order too**, reading "apply it as stated" literally. Rejected. A reorder
changes no obligation. A report that fires where nothing is wrong is one a reader learns to pass over,
which is the reason "Archival assesses the merge and never performs it" gives for staying silent when the
contract has arrived.

**Considered: compare statements as a collection.** Rejected, for the reason above: a statement's
meaning depends on the one before it.

**Considered: match each stated scenario to an accepted one by identifier, then compare the pair.**
Rejected. It answers a different question: which scenario this is, not whether the accepted requirement
says it. It also adds a failure the text question does not have. Identifiers are unique only within one
document, and an accepted specification holds many requirements. toolchain-packaging already holds two
scenarios named "An invalid specification", under two requirements
(`openspec/specs/toolchain-packaging/spec.md` lines 385 and 651). So the identifier a change's
specification gives a scenario it adds can be taken in the accepted document, and the merge has to
write it under another. Matched by identifier, that correct merge would be reported as not taken. The
identifiers still appear in the report, because they are how a reader finds a scenario in each document
(decision 5).

This is also the answer to `state-a-modified-requirement-whole`'s decision 4. That decision kept the
sentence that a stated scenario carries its accepted counterpart's identifier, and said this change's
comparison needs it. This comparison does not. That change's review settled the question on 2026-10-02
and kept the sentence, because its `sync` instructions match each stated scenario to its accepted
counterpart by identifier. The sentence serves the merging agent, and this comparison still compares
text. Nothing here moves.

**Considered: compare the requirement's markup.** Rejected. Formatting, attributes, and `data-change`
would all differ where nothing normative does. And `mark-the-words-a-modification-changed` is about to
put `<ins>` and `<del>` into a modified unit.

**Notes and the scenario's change state are not content.** A note is "prose that is not a field"
(projection specification, section 4.0), and `state-a-modified-requirement-whole` records that the
contract is silent both on notes and on the change states an accepted specification carries after a
merge. Comparing either would report an accepted note, or a leftover `data-change`, that the merge was
right to leave alone. The `renamed-from` paragraph is not a statement, and the requirement's own heading
is the rename comparison's, as `record-a-renamed-requirement`'s design decision 4 settled.

**Forward note.** `mark-the-words-a-modification-changed` will let a unit marked `modified` carry `<ins>`
and `<del>`, and its projection moves deleted text out of `statements` into a list of withdrawn words.
Because this comparison is defined on the projected statements and scenario entries, and not on raw
HTML, that change only has to say that withdrawn words are not content for this comparison. Inserted
words are already in the statement. That change's behavior is not specified here.

### 3. A withdrawal the merge has not taken is a set of its own

`untakenRemovals` lists identifiers only. There is no content to compare, because
`state-a-modified-requirement-whole` does not ask a withdrawn requirement to be stated whole.

**Considered: one set for everything held that should not be, withdrawals and content together.**
Rejected. The work differs. A withdrawal is finished by deleting a requirement, and content by editing
one. The entries differ too: a withdrawal has no statement position and no scenarios to name. The rename
set was given its own note for the same reason. `report_findings` says so in its comment: "Its own note
rather than a clause of the one above, because the work differs".

### 4. A requirement renamed and modified can be in both of those sets

Every stated identifier is in exactly one of four places: `reached`, `absent`, `untakenRemovals`, or the
untaken pair. Within the pair, an identifier is in `untakenRenames`, in `untakenContent`, or in both.
Where the identifier is not held, `absent` still takes precedence, because there is nothing to compare.
`every_stated_requirement_is_in_exactly_one_set` is extended to state exactly this partition.

**Considered: one finding per requirement, with the rename first.** Rejected. It hides a fact the
comparison already computed, and costs the agent a second round to find it. artifact-kinds' "A rename
reaches the accepted contract as a new heading on the same identifier" calls the rename and the
modification "two facts about one requirement", and the report says both.

### 5. The report names where the content differs, never what it says

Each `untakenContent` entry in both payloads:

```json
{
  "id": "req-x",
  "firstDifferingStatement": 1,
  "scenariosNotHeld": ["scenario-x-c"],
  "scenariosNotKept": []
}
```

- `firstDifferingStatement` is the first paragraph position, counted from one, at which the change's
  statements and the accepted requirement's differ. Where one side has fewer paragraphs and they agree
  up to that point, it is the first position the shorter side lacks. It is `null` where the statements
  agree.
- `scenariosNotHeld` lists the scenarios the change keeps (states, and does not mark `removed`) whose
  text the accepted requirement does not hold, by the identifier each carries in the change's
  specification.
- `scenariosNotKept` lists the scenarios the accepted requirement holds whose text the change does not
  keep, by the identifier each carries in the accepted specification, or by heading name where it has
  none.

A scenario the change altered and the merge did not take appears in both lists. Under
`state-a-modified-requirement-whole`'s identifier sentence, it carries the same identifier in both, which
tells the reader it is one scenario.

`untakenRemovals` is a list of identifiers, like `absent`.

The human report adds one line per entry, after the `renamed` lines:

```
  differs  req-x (statements from paragraph 1; not held: scenario-x-c)
  held     req-y (this change marks it removed)
```

Archival's `report_findings` gains one note per set. Each note names the requirements and says that the
change has not been synced, as the rename note does. The content note names the work as "give each
accepted requirement the statements and scenarios this change states". The removal note names it as
"delete each from the accepted specification that holds it".

**Considered: report only that the content differs.** Rejected. `2026-08-31-sync-a-change-into-the-accepted-contract`
changed toolchain-packaging's "The command surface names its operations as OpenSpec names them" by
one word, `sync`, in the third of its ten statement paragraphs (evidence, section 9). Without a
position, the agent reads both sides of all ten to find it.

**Considered: carry both texts, or a diff.** Rejected. This report is a worklist an agent executes,
and in a worklist a text reads as the text to write, which is the text of a merge the operation must
never propose. A view with no worklist, labelled by source, reads as a quotation instead, and that
view is `show-what-a-modification-changed`'s subject. The contributor accepted that reading on
2026-10-01, so the two designs agree: the ban governs this report, and the diff view stays apart from it.

**Considered: every differing paragraph, found by aligning the two sequences.** Rejected here. That is
a diff algorithm, and it belongs with the diff view. The first position is exact, cheap, and enough to
start reading.

### 6. One predicate, and `Findings` carries the comparison instead of copying it

`MergeState` gains `untaken_content: Vec<UntakenContent>`, `untaken_removals: Vec<String>`, and
`fn reached_in_full(&self) -> bool`, which is true exactly when `absent` and all three untaken sets are
empty. `sync`'s "applied in full" line and `Findings::needs_sync` both call it. `Findings` replaces its
`unmerged` and `untaken_renames` fields with `merge: MergeState`, so a set added to the comparison is in
archival's findings without a second edit. The archival payload's key `unmergedRequirements` is kept,
read from `merge.absent`.

`UntakenContent` is built only by the function that compares one stated requirement with the accepted
ones. That function returns `Option<UntakenContent>`, and it returns `None` when nothing differs. So an
entry that names no difference cannot be built. The accepted side of `merge_state`'s map changes from
heading names to the accepted requirements themselves, or a view holding their heading name and
content. The rename check keeps reading the heading name from it.

**Considered: add the two sets to `Findings` and a clause to each predicate, as the rename change
did.** Rejected. That is how the second statement of the predicate arose, and the parity tests only
cover the sets they name. The next set would need the same edits in the same places, and nothing
would report a missed one.

### 7. The instructions, and a test that they name every set

`sync.md`:

- The table of what comes back gains `untakenContent` and `untakenRemovals`. The partition sentence
  becomes decision 4's.
- `state-a-modified-requirement-whole`'s "Merging a modified requirement" section stops working around
  the comparison and works through `untakenContent`:
  - find the accepted requirement by identifier;
  - start reading at `firstDifferingStatement`;
  - bring in each scenario in `scenariosNotHeld`;
  - take out each scenario in `scenariosNotKept` that the change marks `removed` or does not state.

  Its reporting rules stay. It still reports a scenario marked `unchanged` whose text differs, and an
  accepted scenario the change neither states nor marks `removed`. It gains one more: where the
  difference is another change's later edit, the agent reports it and does not revert it.
- A new section, "Merging a withdrawal": for each identifier in `untakenRemovals`, delete the accepted
  requirement that carries it, and nothing else. It then counts as reached.
- "A requirement you should not merge" says that a requirement marked `removed` never appears in
  `absent`. What remains there is a requirement the change states as added, modified, or carried
  forward.
- "Finish by asking again" and "What to report" cover all four worklists.

`archive.md`:

- Step 4 gains the two findings.
- Step 4's sentence that after the merge each remaining absent identifier "should be a requirement the
  change deliberately withdrew" is corrected.
- Step 5's withdrawal bullet says archival then counts the requirement as reached.
- Step 6's list of what archival still asks about gains the two sets.

The absent notes in `sync` and `report_findings` stop saying "withdrawn deliberately". They say
"abandoned", because a requirement the change withdraws is no longer among the absent.

A test in `crates/htmlspec-cli/tests/archive.rs`, `every_set_the_sync_payload_carries_is_named_in_its_instructions`,
runs `sync --json` on a scratch project, takes every key of the payload whose value is an array, and
asserts that each appears in backticks in the body `htmlspec_assets::instructions::workflow_for("sync")`
returns. **Considered: the same test in `htmlspec-assets`, over a list of keys.** Rejected. That list
would be a second statement of the payload, which drifts. The CLI's tests can run the payload and read
the instructions both.

## Risks / Trade-offs

- **[A change written before another change modified the same requirement is reported as not
  taken.]** → The report is right: merged as stated, the older restatement reverts the newer edit. The
  `sync` instructions say to report such a difference and not to apply it, and this project's overlap
  convention is a "1.0 Re-base" task that refreshes the older change first.
- **[A merge that adapts the wording to its destination is reported as not taken.]** → The change's
  specification is the as-built record under artifact-kinds' "A specification states the settled
  contract as built". The fix is to make the two agree before archival, in whichever document is wrong.
- **[Text the projection does not carry is not compared.]** → The comparison reads what the model
  holds. That includes paragraphs inside list items, because statements come from every paragraph among
  a requirement's descendants. A list item with no paragraph, a table, or a figure inside a requirement
  projects no statement, so a change to it goes unseen. Every consumer of the model has the same limit,
  and fixing it belongs to the projection.
- **[Agents that read the payloads see different members in two existing sets.]** → A requirement
  marked `removed` leaves `absent` and `unmergedRequirements`. `reached` no longer means "held". The new
  keys are additive, `stated` is unchanged, and the release note says both. The same shift happened when
  untaken renames moved out of `reached`.
- **[`sync` named with an archived change's path reports later changes' edits.]** → The operation
  answers for a change being merged. An archived change's modified requirement matches the accepted text
  only until a later change modifies it, which is why `state-a-modified-requirement-whole` kept this out
  of validation. The instructions run it on the active change, and nothing else is added.
- **[Whitespace inside inline `code` is normalized away.]** → Two statements that differ only there
  compare equal. The heading comparison accepts the same limit.
- **[An identifier held by two accepted specifications.]** → That is a fault in those specifications.
  The content is taken if either holds it, as a rename is. Otherwise the entry reports against the first
  one found in discovery order.
- **[`merge_state` keeps whole accepted requirements rather than headings.]** → It already reads and
  projects every accepted specification. Only what it keeps grows.

## Migration Plan

No document changes meaning, and none stops validating. No unit, field, attribute, or projection rule is
added or reshaped, so under the bump table in `crates/htmlspec-schema/CHANGELOG.md` the contract version
stays `0.3.0`, the version `reject-change-markup-inside-a-unit` set. This design first said `0.2.0`,
because it was written before that change landed. What changes is the behavior of two commands and the shape of their payloads. Two keys are
added, and `absent`, `unmergedRequirements`, and `reached` change members as described under Risks.

This change follows `state-a-modified-requirement-whole`. It relies on that change's requirement, and it
edits the instruction sections that change adds, so it lands after that change is archived.

The release is expected to be `0.21.0`, the minor increment after `state-a-modified-requirement-whole`'s
`0.20.0`. A minor increment fits because `sync` and archival report findings they did not report before,
and four accepted requirements change. The contributor confirms the increment at the release task.

Rollback is a revert. It touches no document and no stored data.
