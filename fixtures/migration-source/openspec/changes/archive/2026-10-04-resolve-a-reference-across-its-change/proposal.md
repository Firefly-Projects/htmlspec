## Why

**The tasks template gives advice the validator rejects.** `assets/template-tasks.html:40` tells an
author: "A task may reference the requirement it serves with `data-refs`, once that requirement
exists in this change's specification." Following it exactly, in a scratch project, produces an
error and exit status 1:

```
error: refs/unresolved at task-example: this task references req-example, and no unit in the
document declares that identifier.
```

A requirement that only an accepted specification holds fails the same way
(`evidence/the-measurement-2026-09-30.md`, steps 1 and 2). The rule in
`crates/htmlspec-validate/src/rules/refs.rs` resolves an identifier against the one document that
makes the reference, and against nothing else.

**The contract disagrees with itself about this.**

- document-schema's "Cross-references address units by identifier" says a reference "SHALL resolve
  to a unit that exists in the same document or in an accepted spec reachable from it". Nothing
  implements the second half, nothing defines "reachable", and the sentence says nothing about a
  change's own specification.
- runbook-execution's "A record carries the procedure it ran and the obligations it attests to"
  states the opposite as a universal rule: "This is forced rather than chosen. A reference addresses
  a unit within one document…". So does the help text of `htmlspec run`
  (`crates/htmlspec-cli/src/main.rs:237`), and so does the prose that `run` writes into every new
  record (`crates/htmlspec-cli/src/runs.rs`).

**The workaround is a link that nothing checks.** The migration's drafting brief says: "Never give a
unit `data-refs` naming a unit in another document: references resolve within one document only,
and it is an error." So its drafters used plain links. In the staged migration, 54 task units in 15
changes and 2 decision units link to a unit in another document, 68 links in all, and none of them
carries `data-refs` (step 3). A link to another document projects into neither the model's
references nor its links. The measurement shows a link to `specification.html#req-does-not-exist`
accepted with exit status 0.

**Why now.** This project's migration out of OpenSpec is staged and waiting. Without this change,
those 56 units go into the migrated documents as links nobody checks, or the gap is recorded there
as an open question. Either way the migrated project would carry the workaround and not the
relationship.

## What Changes

- **A `data-refs` entry resolves in three places, in order.** It is tried first in its own
  document. For one of a change's documents, it is tried next in that change's specification.
  Then it is tried in the project's accepted specifications. The first place that declares the
  identifier is the one the reference resolves in.
- **"Reachable" gets a definition, from where a document sits.** A document is loose (outside any
  project, or in a change's evidence), a change's document, a change's specification, an accepted
  specification, or a runbook's document. A change's documents, its specification, and an accepted
  specification reach beyond themselves. A loose document and a runbook's document do not. An
  archived change is still a change.
- **A change's proposal and design reach what its tasks reach.** Reach is decided by where a
  document sits, not by its kind.
- **Every accepted specification is reachable, and an accepted specification reaches the
  others.** A reference names a unit, not a capability, and a merge can split one change's units
  across two capabilities.
- **A reference to a withdrawn unit is still an error, wherever the unit is found.** The change's
  specification is tried before the accepted specifications. So a requirement the change withdraws
  is withdrawn for the change's own documents, even while the accepted specification still holds it.
- **An identifier that two accepted specifications declare still resolves.** The staged migration
  holds two such identifiers today.
- **Validating one document reads the documents it reaches, and reports only on the one named.**
  The scenario "Validating one document" says so.
- **A reachable document that cannot be read is not treated as empty.** A reference that resolves
  nowhere else is still reported, and the report names the unread document instead of claiming
  that no unit declares the identifier.
- **The template's advice becomes true, and the prose that stated the old rule is corrected.** The
  tasks template says where a task's reference can reach. The `run` help text and the record
  prose say that a runbook's documents resolve within themselves, which is still true.
- **Unchanged:**
  - A loose document, and so every conformance fixture, resolves within itself exactly as now.
  - A runbook's documents, including a run's record, resolve within themselves.
  - A fragment link, `href="#…"`, resolves only in its own document.
  - The `review/*` warnings still read one document.
  - No rule identifier, unit, field, attribute, or projection rule is added.
  - Every document that validates today still validates, because its own document is always
    tried first.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-schema`: "Cross-references address units by identifier" defines the places a reference
  can reach, their order, and what a fragment link, a withdrawn unit, and an identifier two accepted
  specifications declare each resolve to. It is modified rather than joined by a new requirement,
  because it already promises resolution beyond the document. A second requirement would leave
  two statements of one rule.
- `document-validation`: "Cross-references must resolve" gains the validation side of the rule.
  Validation reads the documents a reference can reach and reports nothing about them. A reference
  that resolves nowhere is reported with the places that were searched, and an unreadable document
  is named as unread. "The contract rules apply to every kind" is modified too. Its scenario
  "A design with a dangling reference" says "no unit in the document declares", which would become
  false, and the requirement gains one sentence: a kind does not decide what a reference reaches.
  Both are modified rather than added to, because each already states this rule and would
  otherwise contradict it.
- `runbook-execution`: in "A record carries the procedure it ran and the obligations it attests to",
  one sentence changes. The reason a record copies its requirements is restated as a fact about a
  runbook's documents, not as a rule about every reference. Every other word is kept.
- `toolchain-packaging`: in "The command surface names its operations as OpenSpec names them", only
  the scenario "Validating one document" changes. One document is still validated and reported on,
  and the documents it reaches are read only for their identifiers. The rest of the requirement is
  restated verbatim. Changes 6 and 9 modify the same requirement after this one, so each of them
  carries the re-base task. This change lands first and needs none.

## Impact

- `crates/htmlspec-validate/src/engine.rs`: `Subject` carries the document's placement and its
  resolution scope beside the store. `run` takes them, and so does every caller:
  - `validate_source` in the CLI;
  - `conformance/tests/conformance.rs` and `conformance/tests/runs.rs`;
  - `crates/htmlspec-validate/tests/template_conformance.rs`;
  - the `validate_file` example;
  - the unit tests in `engine.rs`, `output.rs`, and `rules/*`.

  A new module in the same crate holds the placement and scope types.
- `crates/htmlspec-validate/src/rules/refs.rs`: `references_resolve` resolves a `data-refs` entry
  through the scope, applies `REMOVED_TARGET` in the place that resolves it, and names the places
  searched in its message.
- `crates/htmlspec-cli/src/project.rs`: `ProjectRoot` derives a document's placement, positionally
  and on canonical paths, as `is_change_evidence` already does.
- `crates/htmlspec-cli/src/commands.rs`:
  - `validate_source` takes the document's path and derives both the store (`store_for`) and the
    scope from it.
  - What the other documents declare is read once per run.
  - "The accepted specifications" gets one definition, shared with `merge_state`.
- `crates/htmlspec-cli/src/main.rs` and `crates/htmlspec-cli/src/runs.rs`: the help text and the
  record prose that state the one-document rule as universal.
- `assets/template-tasks.html` and its sha256 pin in `assets/vendor.lock.json`.
- `crates/htmlspec-assets/instructions/new.md`, section "The tasks". Installed copies are
  regenerated.
- Tests:
  - CLI tests in `crates/htmlspec-cli/tests/cli.rs`;
  - a project-level conformance suite, `conformance/tests/references.rs`, built the way `runs.rs`
    and `topology.rs` build theirs;
  - unit tests in `refs.rs` and `project.rs`.
- **Not in this change**, and recorded so it is not assumed to follow:
  - `review/stale-reference` and `review/stale-verification` across documents. They still read one
    document. Of the 68 staged links, 36 point at a unit marked `added` or `modified`, so reaching
    across would warn on most of the references this change makes possible.
  - `review/unverified-requirement` being cleared by a verification in another document.
  - A reference to another change's documents, or from a change's tasks to its own design. Each
    has 2 task units in the staged migration.
  - A check that no two accepted specifications declare one identifier.
  - Checking a plain link to another document, such as `href="specification.html#req-x"`.
  - A runbook's documents reaching the accepted specifications.
  - Converting the staged migration's 68 links into `data-refs`. That is the migration's work, and
    it follows this change.
  - What an archived change's reference means after a later change withdraws the accepted
    requirement it names. The design records this as a risk.
