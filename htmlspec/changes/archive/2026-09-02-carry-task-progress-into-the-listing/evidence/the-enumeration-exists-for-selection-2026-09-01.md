# The enumeration exists for selection

Task 1.4, recorded 2026-09-01 against `main` at `ee4301c`.

The point of this task is that the change is answerable from the accepted contract rather than
from anyone's preference about what a listing ought to show.

## The accepted text

`openspec/specs/toolchain-packaging/spec.md:434`, second paragraph:

> ### Requirement: The command surface reports what a project holds
>
> The command surface SHALL provide an operation that enumerates a project's active changes, and
> SHALL be able to enumerate its accepted specifications instead. The enumeration SHALL be
> available as a structured payload.
>
> **This is the operation that lets a workflow select what it acts on.** A workflow that cannot
> enumerate a project's changes must have its subject named for it every time, and instructions
> written against such a surface omit the selection step rather than performing it.

The requirement states its own purpose, and the purpose is selection. That makes "does the
enumeration carry enough to select on" a question about whether the implementation satisfies its
requirement — not a feature request.

## The three scenarios it currently holds

At `:440`–`:450`, and each is satisfied by names alone:

- **Enumerating active changes** — each active change is named, archived ones are not
- **Enumerating accepted specifications** — each is named
- **A structured enumeration** — emitted as a payload a workflow can select from

The third is the one worth reading closely. "A payload a workflow can select from" is satisfied
literally by a list of names — a workflow *can* select from it — while the paragraph above says
the operation exists so a workflow can select *what it acts on*. The scenarios check the
mechanism and not the purpose, which is how an enumeration built for selection ended up carrying
no basis for one and passing every check.

## The callers that select, and what they get

Three places in the shipped instructions tell an agent to select a change from this enumeration:

```console
$ grep -rn 'htmlspec list' crates/htmlspec-assets/instructions/
crates/htmlspec-assets/instructions/archive.md:14:htmlspec list --json
crates/htmlspec-assets/instructions/sync.md:13:htmlspec list --json
```

Both under the heading "List the candidates rather than guessing between them". `archive.md`
follows it with the reason ambiguity matters there: "Archival moves a directory and rewrites a
contract, so an ambiguous subject is a question rather than a coin toss." The candidates offered
to answer that question are bare names.

`archive` itself is the third: `commands.rs:688` calls `active_changes` to select when no change
was named, and reports at `:691` when there are none. So the enumeration is on the path of the
one operation the contract describes as irreversible, and it is the operation with the least to
go on.

## What this settles

The requirement says the enumeration exists for selection; the enumeration carries names; names
order alphabetically and nothing else. Modifying the requirement to state what it carries is
therefore a repair to a requirement whose purpose its own scenarios do not check — the same shape
as the two faults the project has already recorded, where `archive` was named for an operation it
did not perform and `sync` was an instruction with no command behind it.
