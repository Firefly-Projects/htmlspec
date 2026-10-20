## Why

**`htmlspec sync` and archival cannot see a modification nobody merged, and they report a withdrawal
the merge applied as work still to do.** toolchain-packaging's "The command surface reports whether a
change reached the accepted contract" fixes the method:

> The operation SHALL determine this by comparing requirement identifiers, and, for a renamed
> requirement, headings, and SHALL read neither the change's specification nor the accepted
> specifications for meaning.

A requirement a change modifies already carries an identifier the accepted specifications hold. So it
is reported as reached whether or not anyone merged the new word, paragraph, or scenario.
`evidence/the-measurement-2026-09-30.md` builds a scratch project and measures both halves:

- **A word added to a statement, not merged:** `sync` reports `req-x` as reached and says "the
  change's contract has been applied in full". `archive --check` says "ready: archival would archive
  this change", with `needsSync: false`. Adding an unmerged scenario as well changes nothing.
- **A requirement the change marks `removed`:** while the accepted specification still holds it, it is
  reached. Once the merge deletes it, which is what a correct merge does, it is `absent`. Archival then
  says "This change has not been synced" and asks whether to archive anyway.

`merge_state` in `crates/htmlspec-cli/src/commands.rs` never reads a requirement's change state. The
two defects meet in one change. Measured against the accepted specification before any merge, `sync`
says the contract "has been applied in full" and archival says the change is ready. Measured after
every part was merged correctly, archival says the change has not been synced. **Both answers are
the wrong way round.**

A text comparison could not have meant anything before `state-a-modified-requirement-whole`, because
a partial restatement always differs from the accepted text. That change adds artifact-kinds' "A
change's specification states a modified requirement whole", under which a change states every
statement and scenario the requirement will hold. Among its reasons: "it lets a later comparison tell
a modification the merge took from one it did not." This change is that comparison.

**Why now.** The project's migration out of OpenSpec is staged and waiting. Once it lands, this
project's own changes are merged through `sync`, and most of them modify requirements: the archive holds
34 modified requirements across 19 changes (`state-a-modified-requirement-whole`'s evidence, section 3).
Without this change, the only check the project has on those merges passes whether they happened or
not. The gap would be recorded in the migrated documents as an open question, or worked around.

## What Changes

- **Each requirement is judged by the change state the change declares on it.** Today every stated
  identifier is judged the same way, and the change state is never read.
- **A requirement marked `added` or `modified` reaches the contract only with its content.** Its
  statements are compared in order, and its scenarios as a collection. Both are compared as normalized
  projected text, which is the same kind of exact test the rename comparison already makes. Scenarios
  marked `removed` are left out, and the accepted requirement must no longer hold them. Notes and the
  `renamed-from` paragraph are not content. A requirement whose identifier is held but whose content
  differs goes into a new set, `untakenContent`.
- **A requirement marked `removed` reaches the contract when no accepted specification holds it.** It
  is never reported as absent. One still held goes into a second new set, `untakenRemovals`.
- **A requirement marked `unchanged` is compared by its identifier, as today.** Its text may differ
  from the accepted text because another change modified it since.
- **Both new sets count as not synced.** A change whose only unapplied work is a word, a scenario, or a
  withdrawal is reported as not synced, archival asks about it, and `needsSync` is true.
- **The report says where the content differs, never what it says.** An entry names the first statement
  paragraph that differs and each differing scenario by identifier. It carries no text from either
  side, so the operation still never proposes the text of a merge.
- **One source, and now one predicate.** Archival and `sync` already share `merge_state`. The test "has
  the contract arrived in full" is also stated twice today, once in `sync` and once in archival's
  `needs_sync`. It becomes one method on the comparison's result.
- **The instructions work every set.** `sync.md` says what to do with each set the payload carries, and
  a test holds that for any set added later. `archive.md` lists the new findings.
- **Unchanged:**
  - the document contract, so `CONTRACT_VERSION` stays `0.3.0`, the version
    `reject-change-markup-inside-a-unit` set. No unit, field, attribute, or projection rule is added.
  - validation, the refusals, and the exit codes.
  - the rename comparison.
  - the payload keys `stated`, `reached`, `absent`, and `untakenRenames`. What `reached` holds narrows,
    and `absent` no longer holds a requirement marked `removed`.
  - The operation still never writes an accepted specification.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `toolchain-packaging`:
  - **Modified: "The command surface reports whether a change reached the accepted contract".** The
    rule is folded in rather than added beside it, because it is the same comparison. A second
    requirement would give "reached" a second definition. `record-a-renamed-requirement` folded the
    heading comparison into this requirement for the same reason. The requirement now names a set by
    what it holds rather than by its number, because "three sets" and "a fourth set" no longer count
    anything. Its scenarios "The comparison is of identifiers" and "A structured report" keep their
    names and get new outcomes, and "A change whose contract has not been applied" gets a new premise,
    because its old one was false for a requirement marked `removed`. Every accepted scenario keeps its
    name, because OpenSpec 1.13.1 refuses a MODIFIED block that drops one.
  - **Modified: "Archival assesses the merge and never performs it".** Archival makes the same
    comparison from the same source. "Whether an absent requirement was withdrawn deliberately" is
    narrowed: a requirement marked `removed` is not in question, because its absence is the merge
    succeeding. The scenarios "A change whose contract has not been synced" and "A change whose
    contract was applied" get new premises. The second one's premise, that every stated requirement is
    held, was already false for an untaken rename.
  - **Modified: "Archival refuses on the contract and asks about the work".** The list of findings
    archival offers to proceed over becomes the list the comparison produces, and a withdrawal the
    merge applied is not reported.
  - **Modified: "The workflow instructions carry a stage for applying a change's contract".** It said
    to run the report again "to confirm the absent set is empty". That was already incomplete once
    renames were compared, and would be more so now. It also said a withdrawn requirement is "left
    absent", which is the opposite of what a requirement marked `removed` needs. It gains a rule that
    the instructions cover every set the payload carries.
  - **Checked and unchanged:** "Archival supports a mode that checks without performing" and
    "Archival is drivable without a person present". Both refer to what archival reports without
    listing it.
- **Checked and unchanged elsewhere:** artifact-kinds' "A rename reaches the accepted contract as a new
  heading on the same identifier", and `state-a-modified-requirement-whole`'s "A change's specification
  states a modified requirement whole". This change relies on both and alters neither.

## Impact

- `crates/htmlspec-cli/src/commands.rs`:
  - `MergeState` (3377-3393) gains `untaken_content` and `untaken_removals`, and a method stating
    whether the contract reached the accepted specifications in full.
  - `merge_state` (3545-3600) reads each requirement's change state, keeps each accepted requirement
    rather than only its heading, and compares content. `untaken_rename` (3608) is unchanged.
  - `Findings` (3241) carries the `MergeState` instead of copying two of its fields. `findings_for`
    (3332-3354), `needs_sync` (3294), `report_findings` (3630), `question_for` (3723), and
    `emit_archive_payload` change with it.
  - The `sync` payload (2439-2449) gains `untakenContent` and `untakenRemovals`, and the archival
    payload gains the same two keys. The human reports gain a line per requirement in each set.
  - The absent notes stop saying "withdrawn deliberately", because a requirement marked `removed` is no
    longer among the absent.
- `crates/htmlspec-cli/src/main.rs`: the `sync` help ("It compares requirement identifiers") and
  `archive --yes` help, which list what is compared and reported.
- Tests:
  - `commands.rs` unit tests for each new scenario.
  - The parity tests `archival_and_sync_name_the_same_untaken_renames` and
    `archival_and_sync_name_the_same_absent_requirements`, extended to the new sets.
  - `every_stated_requirement_is_in_exactly_one_set`, extended to state the partition over every set.
  - A test that every set the `sync` payload carries is named in the `sync` instructions.
  - `crates/htmlspec-cli/tests/archive.rs` gains end-to-end tests beside
    `a_change_whose_only_unapplied_work_is_a_rename_has_not_been_synced`.
- `crates/htmlspec-assets/instructions/sync.md` and `archive.md`, as `state-a-modified-requirement-whole`
  leaves them. Installed copies are regenerated, never edited: the ten `sync` and `archive` files under
  `.claude/`, `.codex/`, and `.pi/`.
- **Not in this change**, and recorded so it is not assumed to follow:
  - **A validation rule.** `state-a-modified-requirement-whole` rejected one in its design (decision
    5), because an archived change would become invalid when a later change modified the same
    requirement. This comparison runs in `sync` and archival, the two operations that run at the merge.
  - **A diff of what a modification changed.** That is `show-what-a-modification-changed`. This report
    names where the two sides differ, and never what they say.
  - **Words a modification withdraws.** `mark-the-words-a-modification-changed` adds `<ins>` and `<del>`.
    This comparison reads projected text, so that change only has to say its withdrawn words are not
    compared.
  - **A requirement's heading where no rename is stated.** `record-a-renamed-requirement` left this
    out on purpose (its design decision 4), and it stays out.
  - **An added requirement folded into a different accepted requirement.** Its identifier is never
    held, so it stays in `absent`, as today.
  - **The predecessor and cycle findings in "Archival refuses on the contract and asks about the
    work".** That list names neither today. artifact-kinds' "Archival produces the specification and
    then updates the accepted contract" states the predecessor finding, and this change leaves it
    there.
  - **The `review/unverified-requirement` warning on a requirement marked `removed`.** The measurement
    shows one, and it is validation's subject, not the merge's.
  - **Scenario identifiers across documents.** This comparison matches scenarios by text. It neither
    needs nor settles `state-a-modified-requirement-whole`'s decision 4.
