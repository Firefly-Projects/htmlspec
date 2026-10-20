# The guard covers `runs/`, verified with a record on disk

`the-guard-failed-open-2026-09-18.md` probed "a subdirectory inside a runbook (a future `runs/`)"
and recorded one word: **"Safe — `starts_with` covers descendants."** That claim was reasoned about
an empty directory that did not exist yet. This change creates that directory and puts documents in
it, so the claim was re-measured with a real run record on disk rather than trusted.

**The claim holds.** Every route refused, on the unmodified binary at `056df05`, with the runbook and
its run record intact afterwards.

## Setup: the runbook a project has once this change ships

`the-guard-failed-open` names the trap that made every cheap probe look safe: four unrelated gates
refuse before archival moves anything, so only a *complete* runbook reaches the move. So the probe
was set up to clear them — the project's declared version advanced from `0.0.0` to `0.1.0`, both of
the release runbook's real documents copied in, and a conforming run record written into `runs/`:

```
/private/tmp/rb-runs/htmlspec/runbooks/release/
  specification.html                  (5 requirements, real prose, real identifiers)
  tasks.html                          (4 task units)
  runs/
    conforming.html                   (2 tasks advanced, 1 copied requirement, 1 verification)
    missing-verification.html
```

## The six routes, all refused

| Route | Result |
| --- | --- |
| `archive htmlspec/runbooks/release/runs --yes` | refused, exit 2 |
| `archive htmlspec/changes/../runbooks/release/runs --yes` | refused, exit 2 |
| `archive /private/tmp/rb-runs/htmlspec/runbooks/release/runs --yes` | refused, exit 2 |
| `archive htmlspec/runbooks/release --yes` (the runbook, with `runs/` inside) | refused, exit 2 |
| `status --change htmlspec/runbooks/release/runs` | refused, exit 2 |
| `sync --change htmlspec/runbooks/release/runs` | refused, exit 2 |

Each refusal names the runbook and states the operation's own reason, which is the `Refusal` enum
working as designed:

```
htmlspec: htmlspec/runbooks/release/runs is a runbook, not a change. A runbook holds a
  procedure that is run repeatedly, so it is not archived — archival ends a change, and a
  procedure ended after its first run is a procedure destroyed. …
```

**The message says "is a runbook" about a directory that is a runbook's `runs/` subdirectory**, which
is imprecise and deliberately left alone. The refusal is correct, the reason is correct, and the
address it prints is the one the contributor typed. Making the sentence distinguish a runbook from a
directory inside one would mean the guard reasoning about depth, which is more structure for a
message rather than for a decision.

After all six: `htmlspec/runbooks/release` holds both documents and `runs/conforming.html`, and
`htmlspec/changes/archive/` is empty.

## The three known bypasses, re-applied to `runs/`

`the-guard-failed-open` found three ways past the previous guard. Each was re-applied with the target
being `runs/` rather than the runbook, because a nested project or repository inside a *run record's
directory* is the same shape one level down:

| # | Condition | Result |
| --- | --- | --- |
| A | `htmlspec/runbooks/` created inside `runs/` | **refused.** The project is a parameter now, so what is inside the argument cannot decide which project the guard judges against. |
| B | `.git` created inside `runs/` | **refused.** Same reason. |
| C | broken symlink at `htmlspec/runbooks`, `runs/` present | **refused**, and the message is the fail-closed one. |

C is worth quoting, because it is the one the predecessor found by probing its own fix and because
the message is what distinguishes a refusal from a near miss. With a *real change* present so that
the resolution reaches the guard at all:

```
htmlspec: cannot read /private/tmp/rb-runs/htmlspec/runbooks to determine whether
  /private/tmp/rb-runs/htmlspec/changes/a-real-change is a runbook: No such file or directory
  (os error 2). Refusing rather than proceeding, because a runbook archived is a procedure
  destroyed
```

**A broken `runbooks` symlink now refuses archival of an ordinary change.** That is the fail-closed
policy working as stated rather than a defect: `symlink_metadata` reports something is there, the
canonicalize then fails, and the guard refuses rather than reading an unreadable directory as
permission. The cost is that a project with a broken symlink at `htmlspec/runbooks` cannot archive
anything until the symlink is fixed, which is the correct side to err on for an irreversible
`rename`.

## Two further routes

| Probe | Result |
| --- | --- |
| `--store <id> archive ../runbooks/release/runs --yes` | refused. The store's project is passed, so the `--store` arm's only route is covered. |
| `--store <id> status --change ../runbooks/release/runs` | refused, with the report-specific reason. |
| `new change ../runbooks/release/runs/sneaky` | refused by `is_a_name` before anything is composed: *"is a path rather than a change name"*. |

The last one is the wedge `ProjectRoot::is_a_name` closes, and it closes for a `runs/` traversal
exactly as it does for a runbook one — the check is on the separator, not on the destination.

## The bypass hunt, from the question rather than from the list

`the-guard-failed-open`'s general form: **a guard against an irreversible operation should be tested
by asking what makes it answer "no" wrongly.** So the hunt started from
*what performs an irreversible filesystem operation on a path derived from user input?* rather than
from the previous round's table.

`fs::rename` and `fs::remove_dir_all` in non-test code: archival's, and two in `staging.rs`. Nothing
new was added by this change — the conformance rule reads and reports, and the run operation
**writes a new file and refuses to overwrite one**.

**The negative list, including where nothing was found:**

| Probe | Result |
| --- | --- |
| `archive`, all four path spellings, target `runs/` | Safe — table above. |
| `archive`, target the runbook holding `runs/` | Safe. |
| `sync` and `status` on `runs/` | Safe, each with its own reason. |
| Bypasses A, B, C re-applied at `runs/` depth | Safe — table above. |
| `--store` arm, both operations | Safe. |
| `new change` with a `runs/` traversal | Safe — refused at the constructor. |
| `new document` onto an existing run record | Safe — `already exists; new will not overwrite a document`, exit 2. **This is the property the run operation depends on**, so it was measured rather than assumed. |
| `migrate promote` over a project holding a runbook with runs | Safe, unchanged from the previous round: `Staged::resolve` refuses over an existing `htmlspec/`, and a runbook only exists inside a project. |
| `format` (which rewrites files) over `runs/` | Safe by design, and it is what `self-runbooks-format` asserts: formatting a run record is correct and canonical output is the property under check. Measured: 4 documents, all canonical. |
| `select_change` with no argument, `runs/` present | Safe — reads `changes/` only, and `active_changes` never sees `runbooks/`. |
| The conformance rule itself | **It performs no filesystem write at all.** It reads the runbook's specification and each run record and returns diagnostics, which is the same shape as `topology::duplicates`. There is no path from it to a `rename` or a `remove_dir_all`. |
| The run operation | Writes one new file and refuses an existing one, so the irreversible case is not reachable. It creates `runs/` with `create_dir_all`, which is idempotent. |

## What this does not establish

**That `starts_with` on a canonicalized path covers descendants is now measured for one level of
descent.** Nothing here establishes it for arbitrary depth, and nothing needs to: `starts_with`
compares whole path components from the front, so a path under `runs/` is also under
`runbooks/`. The reason to measure rather than reason was that the predecessor's two data losses were
both a guard reasoning correctly about a path it had derived wrongly — and the thing worth checking
was the derivation, which this did, on disk, with the record present.
