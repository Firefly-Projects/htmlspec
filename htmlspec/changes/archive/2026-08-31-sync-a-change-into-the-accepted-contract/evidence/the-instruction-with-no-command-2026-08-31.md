# The instruction with no command behind it

Task 1.1, recorded 2026-08-31 against `main` at `8ca4b59`.

## What archival tells a contributor to do

`report_findings` in `crates/htmlspec-cli/src/commands.rs` emits this when a change states a
requirement no accepted specification holds:

```
note: N requirement(s) this change states appear in no accepted specification: <ids>.
Apply the change's contract before archiving, or confirm each one was withdrawn deliberately
```

"Apply the change's contract" names an operation. A contributor who reads that line and goes
looking for it finds nothing.

## The whole command surface, enumerated

From the `Command` enum in `crates/htmlspec-cli/src/main.rs`:

| Subcommand | Subject |
| --- | --- |
| `validate` | documents |
| `format` | documents |
| `assets` | a project's asset topology |
| `archive` | a completed change |
| `init` | a project |
| `list` | what a project holds |
| `status` | one change's artifacts and tasks |
| `store` | registered standalone projects |
| `instructions` | a workflow stage's text |
| `new` | a document or a change |

Ten subcommands. **None has the accepted specifications as its subject.** There is no `sync`,
no `apply`, and no `merge`:

```console
$ grep -n 'Sync\|sync' crates/htmlspec-cli/src/main.rs
$ echo $?
1
```

The workflow stages are the same story. `crates/htmlspec-assets/src/instructions.rs` defines
seven — `explore`, `new`, `continue`, `ff`, `apply`, `verify`, `archive` — and the one named
`apply` is about implementing a change's tasks, not about applying its contract. The collision
of names is worth noting: a contributor told to "apply the change's contract" who reaches for
`htmlspec instructions apply` gets the implementation stage.

## Why this is the 0.6.0 fault in the other direction

The 0.6.0 changelog entry opens: "`htmlspec archive` was named for an operation it did not
perform." That was repaired by making archival complete what it started.

This is the residue. Archival's exclusion of the merge is correct and is stated as an accepted
requirement. What no repair reached is that the excluded operation was never given anywhere
else to live. The program detects the gap, names the identifiers, tells the contributor what to
do, and offers no way to do it.
