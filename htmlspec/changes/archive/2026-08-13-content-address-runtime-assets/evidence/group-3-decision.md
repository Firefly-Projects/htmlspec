# Group 3: the rule-context decision, and what it cost

Written while implementing group 3. The handoff left one design decision open — a
`Rule` receives `(&Projection, &mut Diagnostics)` and so cannot reach a store — and
listed three options. This records which was taken, why, and what the alternatives
would have broken.

## The decision: option 3, the hybrid

Reference resolution lives in the per-document engine, given store context.
Duplication becomes a project-level pass wired into `scripts/check` (group 4).

The handoff suggested weighing option 2 (both rules as a project-level pass) first,
"since the specs describe both rules as properties of a project". Reading the spec
text closely, that is true of only one of them:

- **Requirement 1** (reference ↔ bytes) is per-document throughout. Its scenarios are
  outcomes for *a document*: "reports the document, the reference, and that the bytes
  are absent"; "GIVEN a document that is not part of a project with an asset store …
  WHEN **it is validated** THEN its asset references are not resolved".
- **Requirement 2** (duplication) says the opposite, explicitly: "This rule takes a
  project as its subject rather than a document, so it SHALL be evaluated where a
  project is in view."

So the split is not a compromise between options 1 and 2. It is what the two
requirements separately say.

### Why not pure option 2

Two reasons, the first decisive.

1. **Conformance coverage would have been lost.** If reference resolution leaves the
   engine, no per-document fixture can pin it, and tasks 3.7/3.8 would have had to
   become `UNFIXTURABLE` entries in `conformance/tests/conformance.rs`. That list is
   for rules *no document can violate* — a schema that failed to compile, a vendored
   schema that failed to preprocess. A document can absolutely violate this rule; that
   is the entire defect class the change exists for. Registering it as unfixturable
   would be the coverage rotting in exactly the way that list's own comment warns
   about.
2. **The store's `Absent`/`Present` distinction already solved the scoping**, and it
   solves the fixture problem the same way. A fixture directory carrying its own
   `assets/` is `Present`; the existing 37, which carry none, are `Absent`. So the
   rule fires on the new fixtures and cannot fire on the old ones — their bytes did
   not need to change, and the scoping is pinned by the same mechanism it is
   implemented with, rather than by a harness flag.

## Mechanism

`Rule` became `fn(&Subject<'_>, &mut Diagnostics)`, where `Subject { projection,
store }`. `engine::run` gained one `&Store` parameter and wraps it into the `Subject`.

Chosen over keeping `Rule` intact and adding a second `ProjectRule` type with its own
registration list, because:

- `run` has to change either way, so the "no change to any caller" advantage of
  option 2 was already spent.
- One rule type and one `rules::ALL` keeps `every_namespace_has_a_live_rule`
  meaningful. Two lists would let a rule be live in one sense and dead in the other,
  with nothing noticing.

`Store::Absent` is the no-context value rather than `Option<&Store>`, so there is
exactly one way to say "no store" instead of two that a caller could confuse.

Cost: a one-line signature change in each of the 8 existing rule modules, none of
which reads the store. That is the price of one rule type, and it was worth paying.

## Where the store comes from

`store_for(path)` in `crates/htmlspec-cli/src/commands.rs` walks **upward** from the
document's own directory to the nearest ancestor holding `assets/`.

Upward rather than "the directory the command was pointed at", because a document is
no less inside its project when a contributor names it directly. Anchoring on the given
root would mean `htmlspec validate doc.html` skipped a check that `htmlspec validate .`
applied — so whether a stale reference was caught would depend on how the tool was
invoked, which is how the original defect survived. Three tests pin the walk:
`a_nested_document_finds_the_store_above_it`,
`a_document_with_no_store_above_it_has_none`, `the_nearest_store_is_the_one_found`.

A visible consequence: the existing conformance fixtures *are* in scope when validated
from the repository, because the repository root has an `assets/` directory. They pass
anyway, because their references are unaddressed and therefore claim no hash. That is
the correct outcome and not an accident of scoping — `an_unaddressed_reference_is_not_
this_rules_business` pins it.

## Severity

`Namespace::Asset` was added and joins `Safety` in refusing downgrade:
`is_downgradable` is now `!matches!(self, Self::Safety | Self::Asset)`.

The existing `only_safety_refuses_to_be_downgraded` test asserted the old invariant, so
it was rewritten as `only_safety_and_asset_refuse_to_be_downgraded` — an exhaustive
partition over `Namespace::ALL` rather than two assertions, so adding a namespace now
forces a decision about downgradability instead of defaulting to "yes".

## One thing found on the way, and the decision it forced

`generate_fixtures`' base is `render_template`, which group 2 changed to emit
**addressed** paths. The 37 committed fixtures hold unaddressed ones. The generator and
its own output had silently diverged: running it would have rewritten all 37 inputs.

Inert for validation — with no store beside a fixture both forms resolve to
`NotAddressed` — but it collided with the "37 fixtures unchanged" constraint, and task
3.7 requires new fixtures go *through* that generator.

Resolved (user's call, offered as three options) by making the generator additive:

- `base()` now rewrites addresses back to the legacy unaddressed form, derived from the
  asset table rather than hardcoded, so it cannot go stale when bytes change. The 37
  reproduce byte-for-byte — verified by running the generator and observing a clean
  `git status`.
- A new `assets: &[StoredAsset]` field lets a fixture carry its own store. Only the
  four `asset/*` fixtures use it.
- Three harness tests keep the arrangement honest: `only_the_asset_fixtures_carry_a_
  store`, `only_the_asset_fixtures_reference_an_addressed_path`, and
  `the_two_asset_faults_are_pinned_separately`. The first two are keyed on an explicit
  named list, not a substring of the directory name — `valid-local-asset-reference`
  predates this change, contains "asset", and carries no store.

## State at the end of group 3

- Tasks 3.1–3.8 complete. 20 of 43.
- `scripts/check` passes all 6 steps.
- 41 conformance fixtures, up from 37. The original 37 are byte-identical.
- New rule identifiers: `asset/unresolved`, `asset/mismatch`.
- 14 unit tests in `crates/htmlspec-validate/src/rules/assets.rs`, 3 in the CLI, 3 in
  the conformance harness.

The duplication signals the handoff predicted are still pending and still expected:
`Store::discover(".").duplicates()` reports 8 sets, and wiring the topology rule into
`scripts/check` (task 4.3) will make the suite fail until group 7. Nothing in group 3
touched that.
