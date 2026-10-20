# Two precedents for a check whose subject is not one document

Task 1.4, recorded 2026-09-02 against `main` at `595e38c`.

A resolution that reads one change and looks for another is a new *shape* of check in this
project only if nothing like it exists. Two things like it exist, and both are load-bearing.

## `asset/duplicate`: a rule whose subject is a project

`crates/htmlspec-validate/src/rules/topology.rs:26` defines the rule, and its module comment
states the arrangement:

> This is an asset rule like the two in [`super::assets`], and it is **not** a
> [`crate::engine::Rule`]. Its subject is a project rather than a document, which the specs
> state directly: the rule "takes a project as its subject rather than a document, so it SHALL
> be evaluated where a project is in view".

It is therefore **absent from the per-document rule set**. `rules/mod.rs:34` lists `ALL`, the
ten rules the validator runs over each document, and `topology` is not among them —
`rules/mod.rs:7` records why:

> Adding it to `ALL` would report one duplicated library once per document — the same two paths
> named twenty times, none of it the fault of the document being validated.

Its signature is the evidence: `pub fn duplicates(store: &Store) -> Vec<Diagnostic>` takes a
store and returns diagnostics directly, where every rule in `ALL` takes a `&Subject<'_>`
holding one projection. `every_rule_in_all_is_a_document_rule` pins the separation.

**What this establishes.** A check whose subject is wider than one document is already part of
the model, already excluded from the per-document pass for a stated reason, and already
evaluated by a command that has the wider thing in view (`htmlspec assets`). A predecessor
resolution is the same shape: its subject is a project, because the answer depends on what
other directories exist.

## `merge_state`: archival already compares one change against the project

`crates/htmlspec-cli/src/commands.rs:1750`. It takes the change's specification *and* the
project, reads every accepted specification, and partitions the change's requirement
identifiers by whether the project holds them:

```rust
fn merge_state(project: &ProjectRoot, specification: &htmlspec_parse::Projection) -> MergeState
```

Two callers — archival's findings and `htmlspec sync` — and one function, deliberately:

> One function rather than one per caller. A contributor who runs `sync`, sees an empty absent
> set, and then meets archival reporting unmerged requirements has met a defect that no test
> written against either operation alone would catch.

Its result reaches a contributor exactly the way this change's predecessor finding will: as a
`Findings` entry that archival reports, asks about, and archives over on `--yes`. The accepted
requirement *Archival produces the specification and then updates the accepted contract* is
where the reported-not-refused arrangement is already stated.

**What this establishes.** Archival already reads outside the change it is archiving, already
reports what it finds there as a finding rather than a refusal, and already carries the
condition into the structured payload as a value (`needsSync`, `:1993`). A predecessor finding
needs no new mechanism — the tasks document says so, and this is why that is true rather than
hoped for.

## What is genuinely new

Neither precedent reads *another change*. `asset/duplicate` reads a store; `merge_state` reads
the accepted specifications. This change reads sibling and archived change directories, which
no existing check does.

So the new thing is the *subject* — one change looking at another — and not the *shape*: a
project-scoped check reported through archival's existing findings path. That distinction is
worth keeping straight, because it is what makes the implementation small while the contract
clause is genuinely additive.
