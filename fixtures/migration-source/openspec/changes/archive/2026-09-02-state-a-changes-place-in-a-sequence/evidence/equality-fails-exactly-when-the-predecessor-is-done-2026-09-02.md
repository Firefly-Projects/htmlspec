# Equality fails exactly when the predecessor is done

Task 1.3, recorded 2026-09-02 against `main` at `595e38c`.

The obvious implementation of "is my predecessor archived?" is string equality against the
archive's directory names. It is wrong, and it is wrong in the one case the resolution exists
to detect. This demonstrates it against this repository's real archive rather than reasoning
about it.

## What archival does to the name

`crates/htmlspec-cli/src/commands.rs:1923`:

```rust
fn archived_name(name: &str, today: &str) -> String {
    if leads_with_a_date(name) {
        return name.to_owned();
    }
    format!("{today}-{name}")
}
```

`leads_with_a_date` (`:1931`) tests for `YYYY-MM-DD-`: ten characters and a trailing hyphen,
each digit position checked. The accepted contract requires this shape —
`toolchain-packaging`, *An archived change is named for its date and refuses to collide*:
"Archival SHALL name the archived directory for the date of archival followed by the change's
name."

So a change's identity while active and its directory name once archived are different strings.

## The failure, against the real archive

A successor written a week ago recorded its predecessor under the name the predecessor had
then: `carry-task-progress-into-the-listing`. That change is now archived.

```
A successor recorded its predecessor as: carry-task-progress-into-the-listing

Archived directories (last four):
   2026-08-24-report-in-the-contributors-terms
   2026-08-30-finish-the-archival
   2026-08-31-sync-a-change-into-the-accepted-contract
   2026-09-02-carry-task-progress-into-the-listing

string equality against every archived directory: False
equality with the leading YYYY-MM-DD- removed:    True
the directory it matches: ['2026-09-02-carry-task-progress-into-the-listing']
```

**Equality finds nothing.** The predecessor is archived — the successor is free to proceed and
there is nothing to report — and an equality match concludes the opposite. It cannot even
report "unfinished", because the predecessor is not among the active changes either; it
concludes *unmatched*, which is the report reserved for a name that is wrong.

The failure direction is what makes this worth a contract clause rather than an
implementation note:

| The truth | Equality concludes | What a contributor is told |
| --- | --- | --- |
| predecessor archived | no such change | "the name is wrong, or the change was renamed" |

A contributor is sent to correct a declaration that is correct, about a predecessor whose work
is already done. Worse, the finding fires on **every** archival of **every** successor whose
predecessor was archived — which is the normal, in-order case. A report that fires on the
correct path is one a reader learns to skip, which the tasks document names as the standard.

## Why the date test is reused rather than rewritten

`leads_with_a_date` already encodes what a leading date is, and archival's own behaviour
depends on that definition: a change already carrying a date keeps it, so
`2026-08-10-html-spec` in this repository's archive is a directory whose name was already
dated before archival ran. A second copy of the test could drift from the first, and the two
would disagree about exactly those directories. The contract states the reuse for that reason:
"The date SHALL be recognised by the same test archival uses when it decides whether a name
already carries one."

## Not the prohibition `project-bootstrap` states

`project-bootstrap` forbids deriving the last archived version from a directory name, and that
prohibition's reason is that a version is a fact the project records elsewhere — in a `VERSION`
file — so reading it out of a name invents a convention. This is a different act. A change's
name *is* its identity, and matching a recorded name against a directory name is the only
operation that can resolve a named predecessor at all. Nothing else records which change a
directory holds.
