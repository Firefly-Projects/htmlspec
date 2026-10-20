## Context

See proposal.md for why. The shape of the current system that the approach depends on:

- **Resolution reads one projection.** `refs::validate` in
  `crates/htmlspec-validate/src/rules/refs.rs` builds `addressable_identifiers` from the projection
  it is given. It reports `refs/unresolved` for a reference outside that set, and
  `refs/removed-target` for one naming a unit the same document marks `removed`. The message says
  "no unit in the document declares that identifier".
- **A rule sees a `Subject`, and nothing else.** `Subject` (`engine.rs:138`) holds the projection
  and the asset store. Its comment says why the store is handed to rules rather than reached for:
  "a rule that read the filesystem itself could depend on a path that exists on one machine and not
  another". This change follows the same arrangement. The CLI reads the other documents, and the rule
  reads sets of identifiers.
- **One CLI function validates.** `validate_source` (`crates/htmlspec-cli/src/commands.rs:209`) pairs
  a document with `store_for(path)`. Its callers are:
  - `validate`;
  - `new document` (`write_document`);
  - archival's gate on the specification;
  - `sync`;
  - promotion's `conformance`;
  - `run`, through `validate_source_at`, when it composes a record.

  `store_for` walks upward from the document's own directory. It does not start from where the
  command was pointed, so that naming one file checks what naming its directory would.
- **The layout is derived in one place.** `ProjectRoot` in `crates/htmlspec-cli/src/project.rs` owns
  the positions of `specs/`, `changes/`, `changes/archive/`, and `runbooks/`.
  `is_change_evidence` shows the pattern for a positional question: compare canonical paths
  against a location the root derives.
- **"The accepted specifications" already has one reader.** `merge_state` (`commands.rs:3545`) reads
  every document `discover::documents` finds under `specs/`. Its comment calls an identifier that
  two of them hold "a fault in those specifications", and it takes either.
- **The model records how a reference was written.** `ReferenceOrigin::Attribute` means `data-refs`,
  and `ReferenceOrigin::Link` means a same-document fragment (projection specification, section 5).
  Section 10.1 assigns reference resolution to the validator and not to the projection.
- **Measured** (`evidence/the-measurement-2026-09-30.md`). All 24 changes in the staged migration are
  archived. Their 56 linking units point at three kinds of target:

  | Target | Units |
  | --- | --- |
  | the unit's own change's specification | 31 |
  | an accepted specification | 23 |
  | another change's specification | 2 |

  Of the 68 links, 36 point at a unit marked `added` or `modified`, and 1 points at a removed unit.
  Two identifiers are each declared by two accepted specifications.

## Goals / Non-Goals

**Goals:**

- A `data-refs` entry in one of a change's documents resolves in its change's specification and
  in the accepted specifications, in a stated order.
- Where a document sits has one definition. It is derived once, and change 5 can read it.
- No rule reads a file. A rule is given the identifiers it needs, and nothing more.
- A loose document, and so every conformance fixture, is untouched. No document that validates
  today fails afterwards.

**Non-Goals:**

- `review/*` across documents. See decision 8.
- Checking a plain link to another document. It projects into nothing (projection section 5.1),
  and giving it meaning is a projection change.
- References between changes, and references to a change's proposal, design, or tasks.
- Adding `data-refs` to tasks at archival. The CLI says a change's specification "is produced at
  archival, from the completed change", so a task written at `new` usually has no change
  specification to reference yet. Whether archival should then add references is a question about
  the archival instructions.
- A project-level check that no two accepted specifications declare one identifier.

## Decisions

### 1. Three places, tried in order, and the first that declares the identifier wins

| Placement | Where the document sits | Reaches, after itself |
| --- | --- | --- |
| loose | outside any project; beneath a change's `evidence/`; anywhere else under `htmlspec/` the layout does not name | nothing |
| change document | directly within a change's directory, active or archived, and not its `specification.html` | the change's specification, then the accepted specifications |
| change specification | `specification.html` directly within a change's directory | the accepted specifications |
| accepted specification | beneath `specs/`, at any depth | the other accepted specifications |
| runbook document | beneath `runbooks/`, including a run's record | nothing |

A reference resolves in the first place that declares its identifier, and the unit found there is
the one it means. **The order matters when two places declare one identifier**, and that happens
in two ordinary cases.

- A change's specification restates an accepted requirement it modifies under the same identifier.
  The staged migration has 2 links to an accepted requirement whose identifier the linking change's
  own specification also declares, as `modified`.
- A change's specification declares a requirement it withdraws, marked `removed`, while the
  accepted specification still holds it unchanged until the merge.

In both cases the change's statement is the one its own documents mean, so the change's
specification comes before the accepted specifications.

**Considered: every document in the project is reachable.** This is rejected. A task could then name
another change's requirement. The contract relates changes only through "A change states the change
it follows", and that relation is a fact about a whole directory, not a reference. The staged tree
has 2 task units that link to another change's specification. Those links stay plain.

**Considered: the change's specification and nothing more.** This is rejected. 23 of the 56 staged
units link to an accepted specification, and the contract sentence being implemented names accepted
specifications already.

**Considered: the accepted specifications before the change's specification.** This is rejected,
because it gets the second case wrong. A task naming a requirement its change withdraws would resolve
to the accepted, unchanged copy and pass. Mutation 3 in task 5.4 checks this order.

### 2. Only a `data-refs` entry reaches beyond its document

A reference whose origin is `Link` still resolves in its own document only. The same anchor is also
in the model's `links`, where `refs/broken-link` checks it against the page's element identifiers.
So a fragment link to a unit that only another document declares keeps producing both diagnostics,
as `a_dangling_link_reference_is_reported_like_an_attribute_one` pins today.

**Considered: resolving a link reference in the same places.** This is rejected. The model would
record a relationship that resolves, while a reader following the link arrives nowhere. The
reference would be true and the hyperlink false, and only `refs/broken-link` would say so.

### 3. Where a document sits is a type: `Placement`

`Placement` has the five variants of the table, and none carries data. `ProjectRoot` in
`project.rs` derives it by position, on canonical paths, the way `is_change_evidence` does. A
document that has not been written yet is placed by its parent directory and its file name, because
`run` validates a record it has composed before it writes it, and refuses to write one that fails. The derivation happens
once per document, in the CLI, and every rule reads the result.

A loose document is one the project's layout does not place. That covers four cases:

- There is no project. This is every conformance fixture.
- The file is beneath a change's evidence. artifact-kinds' "A change's evidence is kept in the
  change and is not one of its documents" says evidence is not a document of the project, so a
  record that is checked by name reaches what it would reach anywhere.
- The file is directly within `changes/archive/`, which is not a change.
- The file is anywhere else under `htmlspec/` that none of the four places names, such as a
  subdirectory of a change other than its evidence.

**Change 5 reuses this type.** `mark-the-words-a-modification-changed` allows `<ins>` and `<del>`
only in a unit marked `modified` in a change's specification, and "a change's specification" is
`Placement::ChangeSpecification`. So the type is public in `htmlspec-validate`, and `Subject` gives
it on its own, not only as an input to resolution. A conformance suite can build a subject at any
placement without touching the filesystem (decision 4).

**Considered: take the placement from the declared kind.** This is rejected. A change's
specification, an accepted specification, and a runbook's specification all declare
`specification`. The kind says what a document is, not where it sits.

**Considered: a boolean for "inside a change".** This is rejected. There are five cases, and change
5 needs the specification apart from the change's other documents. `AGENTS.md` §3 prefers a sum
type to a flag.

**Considered: decide by file name alone.** This is rejected. A runbook also has a
`specification.html`, and so could a change's evidence.

### 4. Rules read sets, and the CLI reads files

The types go in a new module of `htmlspec-validate`, re-exported from `lib.rs`:

```rust
pub enum Placement { Loose, ChangeDocument, ChangeSpecification, AcceptedSpecification, RunbookDocument }

/// A place beyond a document that its references can reach.
pub enum Place { ChangeSpecification, AcceptedSpecifications }

impl Placement {
    /// The places this placement reaches after the document itself, in the order they are tried.
    pub const fn reaches(self) -> &'static [Place];
}

/// What one place declares, as far as resolving a reference needs to know.
pub struct Declared { units: BTreeSet<String>, withdrawn: BTreeSet<String>, unread: Vec<Unread> }
pub struct Unread { pub document: String, pub reason: String }

/// A document's placement, and what each place it reaches declares.
pub struct Scope { placement: Placement, places: Vec<(Place, Declared)> }

impl Scope {
    pub fn loose() -> Self;
    pub fn gather(placement: Placement, declared: impl FnMut(Place) -> Declared) -> Self;
    pub fn placement(&self) -> Placement;
}
```

- `Scope::gather` calls `declared` once for each place that `placement.reaches()` names, in that
  order. So the CLI cannot supply a place the placement does not reach, and it cannot leave one out.
  The order is stated once, in `reaches`, which is pure and is tested without I/O.
- `Declared` is built in the same crate, from projections the CLI parsed. It takes each unit's
  identifier, and it marks as withdrawn each unit whose change state is the schema's "removed"
  value. That value is found the way `refs::removed_state` finds it today. So the CLI never names
  a change state, and a figure identifier, which can never be a reference target, is never
  gathered.
- `Subject` gains `scope: &Scope` and a `placement()` accessor. `engine::run` takes the scope beside
  the store. Every loose caller passes `Scope::loose()`: the conformance harness, the template test,
  the `validate_file` example, and the rule tests.
- In the CLI, `validate_source` takes the document's path and derives both the store and the scope
  from it. No caller can then pair a document with another document's surroundings, or forget the
  scope. The documents it reads are cached for the run. A whole-project `validate` reads each
  accepted specification once, and each change's specification once, and does not read them again
  for every document.

**Considered: give rules a filesystem handle.** This is rejected, for the reason `Subject`'s own
comment gives for the store.

**Considered: one flat set of every identifier in the project.** This is rejected. It loses the
order and the withdrawn state of each place, and decision 1 needs both.

**Considered: carry the scope inside `Store`.** This is rejected. The store is about bytes, and the
asset rules are scoped by it alone. Putting references into it would let an asset rule see them.

### 5. Every accepted specification is reachable, from a change's documents and from each other

"The accepted specifications" means the documents `merge_state` already reads: every document that
discovery finds under `specs/`. One function answers this for both, so `sync` and validation cannot
disagree about which documents are the accepted contract. An accepted specification reaches the
others and not itself, because the first place already covers it. That also means validation reads
the document as it was given, not a copy on disk.

**Considered: an accepted specification reaches only itself.** This is rejected. A reference names a
unit and never a capability, and the merge puts each requirement in the accepted specification for
its capability. So a verification whose reference resolved in its change's specification could stop
resolving once the merge split the two units between two capabilities. The merge would then have to
avoid creating a fault it cannot see.

**Considered: only the accepted specification of the capability a change touches.** This is
rejected. A reference carries no capability, so the program would have to guess one.

### 6. An archived change reaches what an active one does

**Confirmed by the contributor on 2026-10-01**, with the risk below accepted.

A document directly within `changes/archive/<dated-name>/` is placed exactly as one directly within
`changes/<name>/`. It reaches its own change's specification, then the accepted specifications.

**Considered: an archived change does not reach the accepted specifications.** The archive is
history, and the accepted contract keeps moving after a change is archived. This is rejected all
the same. Archival moves a directory. Under this alternative, every task that referenced an accepted
requirement would be valid the moment before archival and invalid the moment after, and `validate`
would fail on the archive the day it was written. A rule that makes a valid change invalid by moving
it is worse than the risk it avoids, which is recorded below.

All 24 staged changes are archived, and 23 of their units link to an accepted specification.

### 7. A withdrawn unit is withdrawn wherever the reference finds it

`REMOVED_TARGET` is applied in the place that resolves the reference, not only in the document that
makes it. The accepted scenarios already describe this case. document-schema's "Reference to a
removed requirement" and document-validation's scenario of the same name each have a task
referencing a removed requirement. A task and a requirement rarely share a document, so until now
those scenarios mostly described a case the rule could not reach.

The cost is measured. One staged task links to a removed scenario: task 5.1 of
`2026-08-21-distinguish-command-names-from-skill-names`, "Withdraw the … scenario". That link cannot
become a `data-refs` entry.

**Considered: exempt a cross-document reference from the rule.** This is rejected. It contradicts
both scenarios, and a task that claims to implement a withdrawn requirement would pass.

**Considered: exempt a task that withdraws what it references.** This is rejected. A reference says
only "relates to". The model has no way to say "withdraws", and adding one is vocabulary.

### 8. Review warnings stay within one document

`review/stale-reference` and `review/stale-verification` keep reading `subject.projection` alone.
Of the 68 staged links, 36 point at a unit marked `added` or `modified`. What a change's tasks serve
is what the change adds and modifies, so reaching across documents would warn on nearly every
reference that follows the template's advice. The staged tree already carries 317
`review/stale-reference` warnings from references within one document.

The asymmetry this leaves is recorded as not in this change: a reference within one document to an
`added` unit warns, and the same reference from a task to its change's specification does not.

### 9. An identifier two accepted specifications declare still resolves

This follows `merge_state`, which takes either. The staged migration holds two such identifiers,
both scenarios. Reporting them per document would blame the referencing document for a fault it
does not contain and cannot fix, because a `data-refs` entry cannot name a capability. Where the
accepted place holds more than one unit with the identifier, the reference is withdrawn if any of
them is marked `removed`. All 726 staged accepted units are unchanged, so this does not arise today.

### 10. A document that cannot be read declares nothing a reference can rely on

A reachable document that cannot be read, or whose parse recovered, goes into `Declared::unread`
with its path and the reason. It is not treated as an empty set. When a reference resolves nowhere
and a place it reached has an unread document, the report names that document. A change with no
`specification.html` is not unread: that place declares nothing, which is what a change in progress
looks like.

**Considered: an empty set.** This is rejected. It is an in-band sentinel, and the report would say
that no unit declares the identifier, which sends the author to add a unit that may already exist.

**Considered: fail the whole run.** This is rejected. One unreadable specification would stop
`validate` from reporting on every other document in the project. The document itself is still
reported when it is validated.

### 11. The report says where it looked

The rule identifier stays `refs/unresolved`. The message names the places searched, taken from the
scope, for example: "no unit in this document, in its change's specification, or in the accepted
specifications declares that identifier". A loose document keeps today's message word for word.
The conformance fixtures pin rule identifiers, not messages, so none of them changes.

### 12. No document that validates today fails afterwards

The first place is always the document itself. A reference that resolves there resolves to the same
unit it did before, and meets the same withdrawal check. A reference that failed before either
resolves now or is reported as `refs/removed-target`, which is still an error. `review/*` is
unchanged. So validation only moves from failing to passing. Task 5.2 checks this on real data: the
staged tree must give exactly its current result, 109 documents, 0 errors, and 781 warnings.

### 13. The prose that stated the old rule

The following change:

- `assets/template-tasks.html:40` becomes: "A task in a change may reference the requirement it
  serves with `data-refs`, once that requirement exists in the change's specification or in the
  project's accepted specifications." "In a change" keeps the note true when the template is used for
  a runbook's tasks.
- `new.md`, section "The tasks", gains the same fact in one sentence.
- The `run` help text in `main.rs` changes to say that a reference from a runbook's documents
  resolves within that document.
- In `runs.rs`, the module comment changes, and so do the two sentences that it writes into every
  new record.

The following stay:

- The `FOLLOWS` comment in `project.rs` stays, because a change is not a unit and the comment is
  still true.
- Records already written keep their sentence. It is still true of a record, and a record is history.
- The note on the `valid-runbook-run-record` fixture in `generate_fixtures.rs` stays for the same
  reason. Leaving it means the generator, which deletes four hand-written fixtures when it runs,
  does not have to run.

## Risks / Trade-offs

- **[A later change withdraws an accepted requirement that an archived change's document
  references.]** → `validate` then reports that archived reference as unresolved, and the change
  doing the withdrawing has to edit the archived document. That is loud and not silent, and it
  happens before the withdrawing change can pass the project's checks. It is rare: one of this
  project's 24 archived changes withdrew a requirement. Whether an archived reference should resolve
  against the contract as it stood at archival is not in this change.
- **[Reading other documents costs time.]** → Each one is read once per run (decision 4). The staged
  tree takes 19.2 seconds to validate today. Task 5.2 measures it again, and an increase worth
  acting on is recorded there.
- **[A task that withdraws a requirement cannot reference it.]** → Measured at 1 of 68 staged links.
  It can link to the requirement or name it in prose, as it does today.
- **[Placement and the store walk disagree at a repository boundary.]** → `ProjectRoot::resolve`
  stops at `.git` and `store_for` does not. A document in a nested repository inside a project is
  therefore loose, while it still sits in the outer project's store. That is the boundary
  `ProjectRoot` exists to keep, and resolution staying inside it is the safer direction.
- **[Messages change.]** → Rule identifiers and severities do not. Fixtures and consumers that match
  on the rule are unaffected. A loose document's message is unchanged.
- **[Change 5 depends on the variant names.]** → They are this design's table. Renaming one is a
  change to both designs, and the compiler reports every use.

## Migration Plan

**The contract version stays at `0.3.0`**, the version `reject-change-markup-inside-a-unit` set. This design
first said `0.2.0`, because it was written before that change landed. The `crates/htmlspec-schema/CHANGELOG.md` table versions
the schema, the projection specification, and the fixtures that pin the mapping. It moves on a unit,
field, or projection rule. This change adds none of those. It adds no attribute and no model field,
and the projection is untouched. Section 10.1 of that specification names reference resolution as a
validator rule. The change goes in the compatible direction: a document that failed can now pass, and
no document that passed can fail (decision 12). That is the direction the table calls minor, but
nothing it versions has moved, so no entry is added. `keep-evidence-out-of-document-discovery` made
the same call when it changed what `validate` checks and left the schema alone.

**The release is expected to be `0.22.0`**, the next minor increment after changes 1 to 3. It
changes what `validate` accepts, and it adds no breaking command surface. The contributor confirms
the increment at the release task.

Rollback is a revert. A document that adopted a `data-refs` entry across documents fails
`refs/unresolved` on the older toolchain. The failure is loud, and it names the entry.
