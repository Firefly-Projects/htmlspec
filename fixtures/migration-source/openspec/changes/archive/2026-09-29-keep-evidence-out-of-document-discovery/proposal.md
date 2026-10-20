## Why

**A project cannot hold evidence written as HTML and still pass its own checks.** `validate` and
`format`, given a directory or no path at all, operate on every `.html` file beneath it.
`crates/htmlspec-cli/src/discover.rs` collects those files, and nothing in it knows that a change's
`evidence/` directory is not a place a project keeps documents. The conformance check promotion
runs over a staged tree uses the same walk.

Two requirements put HTML files there:

- project-migration's "Evidence crosses unchanged" carries a change's evidence byte-identical,
  whatever its format.
- "A source entity already in the destination format is not assumed usable" requires a document
  that no longer conforms to be *preserved* rather than carried as a document.

Both routes end in a file under `evidence/` that declares a superseded contract version or names
asset bytes the project never stored. `2026-09-11-migrate-an-openspec-project/evidence/migrating-this-repository-2026-09-09.md`
measured what happens next:

- `validate` reports `asset/unresolved` errors against files nobody should be resolving.
- `format` **rewrote one of them**, destroying the byte-identity that evidence exists for.
- Promotion's "Checks gate promotion" scenario then refuses the whole migration, because the
  staged tree cannot pass `validate` while those files are in it.

That record named the fix and declined to make it: "Excluding `evidence/` from discovery is the
right shape and it is a contract question about what a project's documents are." It left the
question to a change whose subject is document discovery. This is that change.

**Why now.** This repository's own migration is blocked on it. Seven HTML files are affected:

- three evidence files in `migrate-an-openspec-project`;
- the four documents converted by hand in August, which the plan preserves as evidence because
  they reference a deleted store.

Every other obstacle to that migration has been removed: renames by
`record-a-renamed-requirement`, and the asset topology by `confine-the-asset-check-to-the-project`.

## What Changes

- **A change's evidence has a stated place.** A change directory, active or archived, may hold an
  `evidence/` directory. Every file beneath it, at any depth, is that change's evidence: a record
  kept as it was written. Today only the non-normative `migrate` instruction names this location,
  so the contract states it for the first time.
- **Evidence is not a document of the project.** When `validate`, `format`, or promotion's
  conformance check walks a directory, it does not descend into a change's evidence. The same
  holds for any other command that collects a project's documents by walking.
- **Naming evidence explicitly still reaches it.** A contributor who passes an evidence file, or an
  evidence directory, as the path argument gets that path checked. The existing rule that "a file
  means that file" is extended to a directory named on purpose. The exclusion applies to what a
  walk finds, never to what a contributor names.
- **The exclusion is positional, not by name.** A directory called `evidence` is excluded only when
  it sits directly inside a change directory. A capability whose path is `evidence`, at
  `htmlspec/specs/evidence/spec.html`, is still a specification and is still validated.
- **Unchanged:** outside a project, where no change directories exist, a walk collects every
  `.html` file as it does now. That covers the conformance fixtures and a loose directory of
  documents. The asset topology is also unchanged, because no evidence in this repository holds a
  store and the duplicate check reads stores rather than documents.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `artifact-kinds`: a new requirement, "A change's evidence is kept in the change and is not one of
  its documents". It states where a change's evidence lives and that evidence is not a document of
  the project. It is added rather than folded into "A change is a directory of kinded documents",
  because that requirement's closed set of kinds is about documents, and evidence is exactly what
  is not one.
- `toolchain-packaging`: a new requirement, "An operation over a directory collects documents and
  not a change's evidence". Validating a project is specified today only as scenarios inside "The
  command surface names its operations as OpenSpec names them", and that requirement is about
  spelling. Putting the exclusion there would bury a rule about what gets checked inside a rule
  about what commands are called.
- `project-migration`: "Evidence crosses unchanged" states that carried evidence does not
  participate in the conformance checks run over the staged project. It is neither validated nor
  formatted there, so a record in the destination format at a superseded version does not gate
  promotion.

## Impact

- `crates/htmlspec-cli/src/project.rs`: the project layout gains the evidence location. It lives
  beside `changes()` and `archive()`, so the one module that derives the layout also answers
  whether a directory is a change's evidence.
- `crates/htmlspec-cli/src/discover.rs`: `documents` stops descending into a change's evidence
  when the walked root is inside a project. The root itself is never excluded.
- `crates/htmlspec-cli/src/commands.rs`: `validate`, `format`, the specification listings, and
  promotion's `conformance` all reach the exclusion through the one discovery function. None of
  them filters on its own.
- Tests:
  - unit tests for the positional rule;
  - CLI tests showing that `validate` and `format --check` pass a project holding non-conforming
    HTML evidence, and that `format` leaves such a file byte-identical;
  - a promotion test showing that a staged tree holding such evidence promotes.
- `crates/htmlspec-assets/instructions/migrate.md`: the `evidence/**` row cites the requirement
  instead of standing on its own.
- **Not in this change**, and recorded so it is not assumed to follow:
  - Stores beneath evidence. None exist, and the topology check reads stores, not documents.
  - Migrating this repository, which this change unblocks.
