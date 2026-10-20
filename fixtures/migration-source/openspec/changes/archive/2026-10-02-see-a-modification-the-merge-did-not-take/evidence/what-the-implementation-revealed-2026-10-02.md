# What the implementation revealed

**Date:** 2026-10-02
**Tasks:** 5.3 and 5.4

## 5.3: the checks

- `scripts/check` inside `nix develop`: the first nine stages pass, format and lint among them. The four
  Python stages fail there with `error: tool 'python3' not found`, as they did for the last two
  changes. Run outside the shell with the same Rust toolchain first on `PATH`, all four pass:
  - prompt: `all 4 prompt cases passed`
  - installed: `45 installed instruction file(s) match what init writes`
  - release-fixtures: `23 shared fixture case(s) read the same way both readers read them`
  - release: `declared version 0.20.0 has a changelog entry`
- The test stage ran 1,154 tests, and none failed.

## 5.4: what the work found

- **The planning text named the wrong contract version again.** The proposal, the design's Migration
  Plan, and task 6.2 said `CONTRACT_VERSION` stays `0.2.0`. It has been `0.3.0` since
  `reject-change-markup-inside-a-unit`. All three were corrected before the measurement, as
  `state-a-modified-requirement-whole`'s were.

- **The predecessor left five sentences that this change makes false, and no task named them.**
  `state-a-modified-requirement-whole` worked around a comparison that could not see a modification,
  and said so in five places: `sync.md`'s modified-requirement section and its "Finish by asking
  again" ("A modified requirement is in `reached` either way"), `sync.md`'s closing stop ("Archival's
  comparison reads identifiers, so it cannot see a held requirement"), `archive.md`'s step 5 stop
  ("`htmlspec archive` compares identifiers, so it counts that requirement as reached and archives
  without asking"), and step 6 ("Archival cannot read a modified requirement's text"). Task 1.2 found
  them, and each was corrected beside the sections tasks 4.1 and 4.2 name. **The two stops stay**:
  archival now reports a held requirement and asks, but a yes or `--yes` archives over it.
  `no_workflow_archives_a_change_while_a_modified_requirement_is_held` pins the stops and not their
  reasons, so it held through the edit.

- **Design decision 1 said two things that cannot both be true.** It derives the establishing states
  "by excluding the other two, as `rules::review::material_states` derives them", and it says a fourth
  state the schema adds "lands in carries forward". Found by exclusion, a fourth state lands in
  establishes. The implementation keeps the stated behavior, because it is the safe one: a state
  nobody has thought about is compared by identifier, as every requirement was before. So
  `establishing_states` finds `added` and `modified` by their stems, as `runs::is_withdrawn` finds
  `removed`. `every_change_state_is_judged_one_way` pins each schema state's case.

- **Design decision 7 asked for an action the predecessor's rule forbids.** Its bullet for the
  modified-requirement section says to "take out each scenario in `scenariosNotKept` that the change
  marks `removed` or does not state". A scenario the change does not state at all is the omitted
  scenario that `state-a-modified-requirement-whole` requires the agent to report and hold, and that
  decision keeps its reporting rules in the same breath. `sync.md` now reads `scenariosNotKept` as three
  kinds of entry: one the change marks `removed`, which comes out; the accepted text of a scenario the
  change states as altered under the same identifier, which comes out because the change's text
  replaces it; and one whose identifier the change never states, which is reported and holds the
  requirement. **The review should confirm this reading.** The review accepted it.

- **The comparison and the merging agent match scenarios differently, and agree on the verdict.**
  `sync.md` tells the agent to match a stated scenario to its accepted counterpart by identifier, which
  `state-a-modified-requirement-whole`'s review kept. The comparison matches by text. They meet in
  every case measured here. A scenario altered under one identifier is in both of the comparison's
  lists, under one identifier, which is what the agent reads as one scenario. A scenario marked
  `unchanged` whose text differs is in both lists too, and the agent holds the requirement. They part
  where an `added` scenario's identifier is already taken in the accepted document. The agent writes it
  under another identifier, and the comparison still counts it taken, because it compares text. That is
  design decision 2's reason for comparing text, and `sync.md` says so. No case in this change's tests
  or measurement had the comparison and the agent's reading disagree about whether the contract
  arrived.

- **`specification_with` was not extended.** Task 2.2 asked for it to state statements, scenarios,
  notes, and change states. Its nine callers build fixtures from a tuple, and widening the tuple would
  have rewritten all of them. A builder beside it, `Stated`, with `specification_stating`, writes the
  new fixtures, and the old ones are byte for byte as they were.

- **An entry that names no difference cannot be built, by any path.** Design decision 6 makes the
  comparison function the only builder. The implementation puts `UntakenContent` in a module of its own,
  with private fields and a constructor, `UntakenContent::new`, that returns `None` when nothing differs.
  Tests build entries through the same constructor. The identifier is public, as `UntakenRename`'s is,
  because no value of it names a difference or fails to.

- **A unit test cannot run `sync` against a scratch project.** `sync` resolves its project from the
  working directory or a registered store. So `the_payload_carries_where_content_differs_and_no_text`
  drives archival's payload from a real comparison, and the end-to-end tests in
  `crates/htmlspec-cli/tests/archive.rs` cover `sync`'s payload and its human report.

- **`sync.md` gained three things the design did not list.**
  - An `added` requirement in `untakenContent` means its identifier is already held. Where that is the
    change's own requirement merged incompletely, the agent finishes it. Where it is a different
    obligation that happens to carry the same identifier, the agent reports it.
  - An `added` scenario whose identifier the accepted document already uses goes in under another
    identifier, and the comparison still counts it.
  - A difference that is another change's later edit is the third case that holds a requirement.
    Design decision 7 asks for it, and `archive.md`'s count of the cases that hold a requirement went
    from two to three with it.

- **`init` wrote the archive instructions from a stale binary once.** `archive.md`'s "three cases" was
  edited after the last build, and `scripts/check-installed` reported the five archive copies as
  differing from what `init` writes. A rebuild and a second `init` fixed it. The check did its job.
