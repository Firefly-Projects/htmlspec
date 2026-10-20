# What this change builds on, read as built

**Date:** 2026-10-07
**Tasks:** 1.1, which reads what the design rests on as built and corrects the design, and 1.4,
which reads the instructions the change edits
**Revision:** `main` at `bdbab7f`, after `mark-the-words-a-modification-changed` merged as #46.

## Summary

Both changes this one builds on are archived, and the comparison is built as its design planned in
every respect but four. Each of the four is a correction to this change's design, made before any
code. One of them changed what the view covers, and the contributor settled it.

| What the design assumed | As built | Corrected in |
| --- | --- | --- |
| The accepted index holds each requirement's heading name, content, and path, and chooses between two holders | `held` maps an identifier to every accepted requirement carrying it, as projected, in discovery order, with no path. The comparison chooses: a holder that holds the content, else the first | decision 3, Context |
| The comparison names scenarios by identifier | It names each by its identifier, else its heading name, else its position, as strings | decision 3, and task 3.2 |
| The view compares a requirement marked `modified`, and lists the rest | The comparison also compares a requirement marked `added`, and one marked `unchanged` that holds a changed scenario | decision 7, the delta specification, the proposal, and tasks 2.3, 2.4, 3.2, 7.1, 7.3, 9.1 |
| The marks reader walks a unit's own descendants | `own_descendants` drops what sits inside a `del`. The reader needs `own_nodes` | decision 5 |

## What has archived

- `see-a-modification-the-merge-did-not-take` archived as `2026-10-02-see-a-modification-the-merge-did-not-take`.
- `mark-the-words-a-modification-changed` archived as `2026-10-07-mark-the-words-a-modification-changed`.
- `openspec/specs/toolchain-packaging/spec.md` holds "The command surface reports whether a change
  reached the accepted contract" word for word as the former's delta states it. A script compared the
  accepted block with each archived delta that states the requirement: the 2026-10-02 delta is equal,
  and the two older ones, from `sync-a-change-into-the-accepted-contract` and
  `record-a-renamed-requirement`, are not, as expected.

## `crates/htmlspec-cli/src/commands.rs`

**The accepted index** is built inside `merge_state` (3919):

```rust
let mut held: std::collections::BTreeMap<String, Vec<Requirement>> =
    std::collections::BTreeMap::new();
for path in accepted_specifications(project) {
    …
    for requirement in projection.model.requirements {
        if let Some(id) = requirement.id.clone() {
            held.entry(id).or_default().push(requirement);
        }
    }
}
```

So it holds each holder's whole projected `Requirement`, and the path is read and dropped. The design
said the index chooses between two holders. It does not. `untaken_content` returns `None` where any
holder holds the content, and otherwise the difference from the first. `untaken_rename` returns
`None` where any holder carries the heading, and otherwise names the first holder's heading.

**The content comparison** is `untaken_content(requirement, id, accepted) -> Option<UntakenContent>`
(4003), which calls `content_difference` (4025) on each holder. `UntakenContent` is built only by
`UntakenContent::new`, which returns `None` where nothing differs, and its three fields are private:

- `first_differing_statement: Option<NonZeroUsize>`, from `first_differing_statement` (4072): the first
  position at which the two sequences differ when zipped, or one past the shorter where they agree up
  to its end, counted from one;
- `scenarios_not_held` and `scenarios_not_kept`, each a `Vec<String>`. `scenario_named` (4107) names a
  scenario by its identifier, by its heading name where it has none, and as `scenario N` where it has
  neither.

Scenarios are matched as a multiset of `ScenarioText` values: the heading name, then the given, when,
and then entries, each normalized. A scenario the change withdraws is skipped. The view pairs
scenarios by identifier and needs the scenarios themselves, so task 3.2 has the comparison give them,
and builds `UntakenContent` from the same result.

**The declared-state type** is `ContractEffect` (3617), private to the module:

| Case | Declared state |
| --- | --- |
| `Establishes` | `added` or `modified`, found by the stems `add` and `modif` in `establishing_states` (3670) |
| `Withdraws` | `removed`, found by `runs::is_withdrawn` |
| `CarriesForward` | `unchanged`, and any state the schema adds later |

`ContractEffect::of_requirement` (3637) reads a requirement whose own state carries it forward as
`Establishes` where one of its scenarios does not carry forward. `see-a-modification-the-merge-did-not-take`'s
review added that. `a_scenario_changed_inside_an_unchanged_requirement_is_compared` in `commands.rs`
and `a_scenario_added_inside_an_unchanged_requirement_has_not_been_synced` in `tests/archive.rs` pin it.

### The question this raised, and how it was settled

The design compared only a requirement marked `modified` and listed every other one. The comparison
compares two more kinds by content, and `sync` reports each in `untakenContent` when its text is not
taken:

1. a requirement marked `unchanged` that holds a scenario the change adds, modifies, or withdraws;
2. a requirement marked `added` whose identifier an accepted specification holds with other text.
   `sync.md` asks the merging agent to tell whether that requirement is this change's, merged
   incompletely, or another obligation under the same identifier.

For both, the view as designed would have listed the requirement as not compared while `sync`
reported it as differing. The delta specification's sentence "the view SHALL show a requirement as
differing exactly where that operation reports its content as not taken" would then be false.

**The contributor chose, on 2026-10-07, to have the view compare what the comparison compares**: a
requirement marked `added` or `modified`, and one marked `unchanged` that holds a changed scenario. A
requirement marked `removed`, and one marked `unchanged` whose every scenario is `unchanged`, is
listed. The two other options were to compare `modified` and the `unchanged` case while still listing
`added`, or to keep the design as written. Each would have narrowed the parity to the requirements the
view compares. Design decision 7 records all three.

## `crates/htmlspec-parse`

- The model holds `deletions: Vec<String>` on `Requirement` (`model.rs:171`) and on `Scenario` (292),
  skipped when empty, and holds no position of an `ins` or a `del`.
- The walk is `own_nodes(unit) -> Vec<(NodeRef, Standing)>` (`project.rs:911`), which tags each node
  `Standing::Current` or `Standing::Deleted`. `own_descendants` (942) keeps the current nodes, and
  every field reads it. `nested_units` (953) keeps every nested unit. Text is read by
  `collect_text(node, into, Reading)` (1068), where `Reading::Current` leaves out a `del` and
  `Reading::Whole` keeps it.
- Statements are `repeatable_primary(unit, "statement")` (477): each current `p` among the unit's own
  nodes, with no `data-field` or with `data-field="statement"`, whose current text is not empty.
  Scenario entries are the `dd` elements of the first `dl` among the scenario's current nodes, each
  under the label of the `dt` before it (`project_scenario_body`, 597).

So decision 5's reader reads `own_nodes`, not `own_descendants`, because it needs what a `del` holds.
A paragraph is wholly deleted in two shapes: `<del><p>…</p></del>`, which the walk tags `Deleted`, and
`<p><del>…</del></p>`, which is current with empty current text. Neither projects a statement. A
paragraph inside an `ins` is wholly inserted, and the walk records nothing about `ins`, so the reader
tracks it itself.

## Resolution and refusals

- `Refusal` (1052) has three variants, `Archival`, `Comparison`, and `Report`, each with its own
  `reason`. The view's is a fourth.
- `resolve_change(target, refusal)` (1103) resolves the project from the working directory first,
  takes a directory as the change wherever it is, refusing a runbook, and otherwise looks the name up
  under `changes/`. So an archived change is reached by its path.
- `resolve_change_in(store, target, refusal)` (1174) composes the name under the registered project's
  `changes/`, so `archive/<dated name>` reaches an archived change there.
- `sync` (2473) refuses in this order: a store that cannot be resolved, a change that cannot be
  resolved (`Refusal::Comparison`), no `specification.html`, a file it cannot read, a document that
  declares another kind, a specification that does not validate (exit 1), and a template's example
  content (exit 1). Decision 10 shares all but the last, as it says.
- `ProjectRoot` has no test for an archived change. `archive()` (`project.rs:237`) names the
  directory. `is_change_evidence` (366) and `placement` both accept a directory directly within
  `changes/archive/` as a change's, which is right for them and is why the view needs a test of its own.

## The instructions (task 1.4)

`grep -n "specification.html\|modified"` finds nothing in `new.md`, `continue.md`, `ff.md`,
`apply.md`, or `explore.md`. None of them writes or reads a specification, so each stays silent.
`migrate.md` belongs to `migrate-into-the-extended-vocabulary`.

| File | Edited or silent | Where, and why |
| --- | --- | --- |
| `verify.md` | edited | Section 1, "Read the change, then read the implementation", lists the proposal, the design, and the tasks, then says to read the implementation. The sentence of decision 11 goes after that list, as a fourth thing to read where the change already has a specification. |
| `archive.md` | edited | Step 2 ends its account of a modified requirement with "Marking the words the change added and deleted" and four bullets, then turns to verification and evidence. Decision 11's check goes after those bullets and before **Verification.**, because it is about the modified requirements just written. The literal end of step 2, after **Evidence.**, would separate it from them. |
| `sync.md` | silent | The reason in decision 11 holds. Its step 6, "Report, and do not revert, a difference that is another change's later edit", is the case `archive.md`'s new check catches before the merge. |
| `new.md`, `continue.md`, `ff.md`, `apply.md`, `explore.md` | silent | No hit. They write no specification. |
