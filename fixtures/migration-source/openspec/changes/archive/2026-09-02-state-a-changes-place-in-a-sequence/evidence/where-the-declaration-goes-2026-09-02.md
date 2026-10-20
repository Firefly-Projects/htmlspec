# Where the declaration goes, since there is no `.openspec.yaml`

Recorded 2026-09-02 during implementation, against `main` at `595e38c`. This resolves a gap in
the design rather than working around it silently, and it is written down because the design
states a file that an HTMLSpec change does not have.

## The gap

The design's second section is titled *In the change's declaration, not in a document* and
opens:

> The ordering goes in `.openspec.yaml`, beside `schema` and `created`.

That is accurate about **OpenSpec** changes. This repository is still an OpenSpec project, so
`openspec/changes/state-a-changes-place-in-a-sequence/.openspec.yaml` exists and carries exactly
those two fields. But the contract this change writes lands in HTMLSpec's accepted
specifications, and the implementation is HTMLSpec's Rust command surface — and an HTMLSpec
change directory holds three documents and nothing else:

```console
$ htmlspec new change contract-first
$ ls -a htmlspec/changes/contract-first/
.
..
design.html
proposal.html
tasks.html
```

`crates/htmlspec-cli/src/commands.rs:825` is the whole of what a change is made of:

```rust
const CHANGE_DOCUMENTS: &[(&str, &str)] = &[
    ("proposal.html", "proposal"),
    ("design.html", "design"),
    ("tasks.html", "tasks"),
];
```

There is one declaration in an HTMLSpec project, `spec.nix` at the project root, and it
describes the project rather than a change. So "beside `schema` and `created`" names a location
that does not exist in the project being modified.

## What the contract requires of the answer

The delta specification is not silent about this, which is what makes the gap resolvable rather
than an obstacle. *A change states the change it follows* requires:

> The declaration SHALL live with the change's other whole-directory facts rather than inside
> one of its documents.

That rules out the cheap workaround — a `<meta>` tag or a unit in `proposal.html` — and it does
so for the reason the design gives at length: a reference addresses a unit, a change is not a
unit, and `refs/unresolved` reports a cross-document target as an error
(`a-reference-cannot-name-another-change-2026-09-02.md`).

## The resolution: a `FOLLOWS` file

One file in the change directory, holding one name:

```
htmlspec/changes/compare-the-guidance-solver-families/
├── proposal.html
├── design.html
├── tasks.html
└── FOLLOWS
```

```console
$ cat htmlspec/changes/compare-the-guidance-solver-families/FOLLOWS
contract-the-guidance-solver-comparison
```

**The precedent is `VERSION`, and it is close to exact.** `crates/htmlspec-cli/src/project.rs:46`:

```rust
/// The file an archived change records the version it shipped in.
pub const RECORDED_VERSION: &str = "VERSION";
```

Its doc comment states the standard it was chosen against — `toolchain-packaging` requires an
archived change to record its version "in a form the project's own suite can read without
version control history". A `VERSION` file is already a whole-directory fact about a change,
stored beside its documents, in a form anything can read. `follows` is the same kind of fact
about the same kind of subject.

Two properties fall out for free, and both are contract obligations rather than conveniences:

- **An absent file and an empty file are the same state.** The contract says a change stating no
  predecessor "SHALL be complete as it stands" and "SHALL NOT be reported". Reading a file that
  may not exist and trimming what it holds makes both cases one `None` at the type level, with
  no error path to get wrong.
- **No new configuration language.** `project-bootstrap` argues against exactly that: "A second
  configuration language for three fields is surface the project does not need." A YAML reader
  for one string would be that, and the Nix-subset reader in `declaration.rs` cannot be reused
  as-is — `parse` requires `name`, `version` and `htmlspec` and errors without all three
  (`declaration.rs:192`).

## What was considered and rejected

**`change.nix`.** Matches the language `spec.nix` uses and could carry more whole-directory
facts later. Rejected as premature: it needs a second entry point into the Nix-subset parser
for one string field, and the design's own closing argument applies — a richer model "would be
built on no evidence". A `FOLLOWS` file that later needs to become `change.nix` is a migration
of one file with one field in it.

**`.openspec.yaml`, as written.** Needs a YAML parser the toolchain does not have, for one
scalar, against the accepted argument quoted above.

## What this does not change

Nothing in the delta specification. The contract deliberately states *that* the declaration
lives with the change's whole-directory facts and not *which file* holds it — the same way it
states that an archived change records its version without naming `VERSION`. The requirement is
satisfied by this choice, and a project that later moves the fact into a `change.nix` would
satisfy it too.
