# Every obligation about what a command may write

Task 1.5, recorded 2026-08-30. This change withdraws one sentence on the argument that it is the
only thing stopping archival from completing. That argument fails if a second requirement states
the same obligation somewhere else, so the search is recorded rather than asserted.

## What was searched

Every capability under `openspec/specs/`, with two expressions:

```
grep -rn "SHALL NOT [a-z ]*\(write\|modif\|edit\|creat\|move\|delet\)" openspec/specs/*/spec.md
grep -rni "does not write\|never write\|read-only\|writes nothing\|without modifying" openspec/specs/*/spec.md
```

## What they returned, and what each one governs

| Location | Obligation | Governs | Bears on archival |
| --- | --- | --- | --- |
| `project-bootstrap:58` | Archival SHALL NOT modify the declaration | Archival | **Preserved verbatim** |
| `project-bootstrap:60` | A command whose stated job is checking SHALL NOT write | Every command | **The sentence withdrawn** |
| `project-bootstrap:147` | The operation SHALL NOT write a name where none is needed | Bootstrap's installed files | No |
| `project-bootstrap:285` | SHALL NOT overwrite configuration, SHALL NOT delete a file it did not write | Bootstrap | No |
| `project-bootstrap:274`, `:325` | Scenarios in which bootstrap writes nothing on error | Bootstrap | No |
| `deterministic-formatting:59` | Formatting SHALL NOT widen a diff | The formatter's output | No |
| `deterministic-formatting:93` | Canonical formatting determinable without modifying | Format | No |
| `toolchain-packaging:56` | The format subcommand SHALL support a non-modifying mode | Format | No |
| `workflow-reporting:13`, `:32` | A workflow may deliberately write nothing | Reporting register | No |

## The finding

Two sentences in the whole contract constrain what archival writes, and they are adjacent lines
of one requirement. One is preserved word for word in the modification. The other is the
withdrawal.

Nothing outside `project-bootstrap` restricts archival from writing to the project. The three
hits in `deterministic-formatting` and `toolchain-packaging` are about the formatter, and the
remaining `project-bootstrap` hits are about bootstrap.

## The precedent this search surfaced, which the design did not account for

`toolchain-packaging:56` requires the format subcommand to *support a mode that reports whether
documents are canonically formatted without modifying any file*. `deterministic-formatting:93`
states the same obligation from the document's side.

That is a precedent for how this project lets one command both perform and check: a mode, not a
second command. Archival is about to change from checking to performing, and the design as
written provides no mode that checks without performing. The gap is recorded here rather than
resolved, because resolving it is a contract decision.
