# Nothing relates one change to another

Task 1.1, recorded 2026-09-02 against `main` at `595e38c`, with `target/release/htmlspec`
built from that commit.

The proposal's claim is that a change carries no relational fact and that no operation reads
one. This records the command output rather than the claim, because the whole change rests on
the absence being real.

## What a change directory holds

A change created by the toolchain, listed exhaustively:

```console
$ htmlspec new change contract-first
/private/tmp/seq-probe/htmlspec/changes/contract-first: created a change with 3 document(s).
The specification is produced at archival, from the completed change.

$ ls -a htmlspec/changes/contract-first/
.
..
design.html
proposal.html
tasks.html
```

Three documents and nothing else. **There is no per-change declaration file in an HTMLSpec
project at all** — the only declaration is `spec.nix` at the project root, which carries
`name`, `version`, and `htmlspec` (`crates/htmlspec-cli/src/declaration.rs:35`). The `schema`
and `created` fields the proposal quotes are OpenSpec's `.openspec.yaml`, which is what this
repository's own `openspec/changes/*/` directories carry while it is still an OpenSpec project:

```console
$ cat openspec/changes/state-a-changes-place-in-a-sequence/.openspec.yaml
schema: spec-driven
created: 2026-09-02
```

Two fields, both about the change in isolation. Neither names another change.

This is a fact the design has to account for and does not, and it is recorded here rather than
worked around. See `where-the-declaration-goes-2026-09-02.md`.

## What the enumeration returns

`htmlspec list --json` against a project holding two changes:

```json
{
  "changes": [
    { "name": "contract-first", "state": "in-progress", "total": 2,
      "tasks": { "done": 0, "dropped": 1, "inProgress": 0, "todo": 1, "undeclared": 0 } },
    { "name": "follows-it", "state": "in-progress", "total": 2,
      "tasks": { "done": 0, "dropped": 1, "inProgress": 0, "todo": 1, "undeclared": 0 } }
  ]
}
```

A name, a state, a total, and five per-state counts. Every field is a property of one change
read on its own. The human form is the same fact in columns:

```console
$ htmlspec list
contract-first  0/2  in-progress
follows-it      0/2  in-progress
```

Two changes, one of which could be a prerequisite of the other, and the listing has no column
in which that could appear. This is the enumeration the shipped archival instructions tell an
agent to select from (`crates/htmlspec-assets/instructions/archive.md:12`).

## What the completion operation returns

```console
$ htmlspec status --change contract-first --json
{
  "artifacts": [ {"name": "proposal.html", "present": true}, ... ],
  "change": "contract-first",
  "tasks": { "open": 1, "openTasks": ["task-example"], "total": 2 }
}
```

Artifact presence and task counts. Nothing about any other change.

## What validation checks

Validating a change validates its documents against the schema and the rule set:

```console
$ htmlspec validate htmlspec/changes/contract-first
validating htmlspec/changes/contract-first/tasks.html
htmlspec/changes/contract-first/tasks.html
  no diagnostics: the document is accepted

3 document(s): 0 error(s), 0 warning(s)
```

The subject of every rule is one document. `crates/htmlspec-validate/src/engine.rs` carries a
`Subject` holding one projection, and every rule module takes it — so a rule cannot see a
second document, let alone a second change. The one exception is `asset/duplicate`, whose
subject is a project rather than a document; see
`two-precedents-for-a-cross-change-check-2026-09-02.md`.

## What this settles

The proposal's factual claim holds in every particular: a change records nothing relational,
the enumeration carries nothing relational, and validation's unit of work is one document. An
ordering between changes is expressible nowhere and read by nothing.

It also surfaces one thing the proposal and the design both get wrong: they place the
declaration in `.openspec.yaml`, and an HTMLSpec change has no such file. That is a real gap
in the design, recorded separately rather than resolved silently.
