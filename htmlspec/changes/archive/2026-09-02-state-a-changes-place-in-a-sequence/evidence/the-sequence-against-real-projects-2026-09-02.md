# The stated sequence, against real projects

Section 5, recorded 2026-09-02. Produced by `prove-sequence.py` in this directory, run against
`target/release/htmlspec` built from the working tree. **31 checks, all passing.**

```console
$ uv run openspec/changes/state-a-changes-place-in-a-sequence/evidence/prove-sequence.py
...
every check passed
$ echo $?
0
```

Each scene builds a project with `htmlspec init`, creates its changes with `htmlspec new
change`, and drives the real binary. Nothing is stubbed and no output is read by eye — every
claim below is an assertion in the script.

## 5.1 The middle change of a sequence, archived first

Three changes: `the-contract`, then `the-graded-work` following it, then `the-device-work`
following that. `the-graded-work` is archived first.

```console
$ htmlspec archive the-graded-work --check
note: this change follows the-contract, which is not archived. Archive that one first, or
state that this change no longer follows it
ready with questions: this change follows a change that is not archived, and archival would
report the findings above and ask
```

**That is the wording the proposal promised**, before any of this was built:

> ```
> note: this change follows contract-the-guidance-solver-comparison, which is not archived.
> Archive that one first, or state that this change no longer follows it
> ```

Checked: the payload states `outOfOrder: true`, names `the-contract` under
`unresolvedPredecessor.change`, and reports its resolution as `active` rather than `unmatched`.
`--check` exits `0` — it is a question about the work, not a refusal.

The three ways archival can end are each checked:

| Run | Outcome |
| --- | --- |
| `--check` | reports, exits `0`, changes nothing |
| no terminal, no `--yes` | exits non-zero, the change is **still active**, and the predecessor is named |
| `--yes` | archives, having named the predecessor first |

The `--yes` row is the one that matters most. An abandoned sequence must remain archivable, and
a gate here would leave a contributor editing a declaration to state a history that did not
happen.

## 5.2 The same sequence, archived in order

The same three changes, archived in the order they state.

```console
$ ls htmlspec/changes/archive/
2026-09-03-the-contract
2026-09-03-the-device-work
2026-09-03-the-graded-work
```

Every archival exited `0` with no question asked, and **nothing about ordering was reported at
any step** — no "this change follows", no contradiction. `the-device-work` follows
`the-graded-work`, which by then was archived under a dated directory, and it resolved as
archived. That is the case a naive equality match gets wrong
(`equality-fails-exactly-when-the-predecessor-is-done-2026-09-02.md`), and it is silent here.

One correction to how this is checked: an early version grepped stderr for `follows` and failed
on all three, because the **version gate** prints `version 0.2.0 follows 0.1.0`. The same word,
a different fact. The check names `this change follows` instead.

## 5.3 A predecessor renamed after its successor recorded it

`the-contract` is renamed by moving its directory, which is how `artifact-kinds` says a change
is renamed.

```console
$ htmlspec archive the-graded-work --check
note: this change follows the-contract, and no active or archived change carries that name.
It was renamed, or the name is wrong — correct the declaration rather than waiting on it
```

Checked: the resolution is `unmatched`, the report does **not** contain "which is not archived",
and it says to correct the declaration. The listing separates the two the same way —
`follows.resolution` is `unmatched`, not `active`.

This is the distinction the contract requires be reported distinctly: a contributor told to wait
on `the-contract` would be waiting on a directory that no longer exists.

## 5.4 Two changes that each follow the other

```console
$ htmlspec list
first-change   1/1  done  follows second-change (active)
second-change  1/1  done  follows first-change (active)
note: first-change, second-change each follow one another, so the ordering they state is a
contradiction. No order holds among them until one of those declarations changes
```

Checked:

- `followsCycles` in the payload is `[["first-change", "second-change"]]` — **one** finding, not
  one per member.
- The report names it a contradiction.
- **No ordering is chosen.** The table is still in name order, which is the check that the
  program did not try to resolve what it just called irresolvable.
- Archival carries `followsCycle` as a value and calls it a contradiction too.

## 5.5 The motivating case

The four changes from the missile-launcher project, with the contract first, as
`the-motivating-sequence-is-live-2026-09-02.md` records them.

```console
$ htmlspec list
compare-the-guidance-solver-families     1/1  done  follows contract-the-guidance-solver-comparison (active)
contract-the-guidance-solver-comparison  1/1  done
measure-the-cost-of-decimation           1/1  done  follows contract-the-guidance-solver-comparison (active)
settle-sensors-and-precision             1/1  done  follows compare-the-guidance-solver-families (active)
```

**This is the listing that carried nothing.** Four changes, all reading `1/1 done`, and three of
them now say what they are waiting on. An agent asked to archive "the change we were working on"
selects from this rather than from four names at the same fraction.

The contract itself carries no predecessor field at all, which is the silence the contract
requires for a change that follows nothing.

Then the archival that is silent today:

```console
$ htmlspec archive measure-the-cost-of-decimation --check
note: this change follows contract-the-guidance-solver-comparison, which is not archived.
Archive that one first, or state that this change no longer follows it
ready with questions: this change follows a change that is not archived, and archival would
report the findings above and ask
```

Reported, named, and not refused. Exit `0`.

## What is not proven here

`settle-sensors-and-precision` follows `compare-the-guidance-solver-families` in this scene, and
the source proposal places it after **all three** of its predecessors. One relation is carried,
so the other two are unstated and unreported. That is the design's accepted limit — "if a project
later needs a change with two independent predecessors, that is a change to make then, with the
case in hand" — and it is recorded here so a reader of this evidence is not left thinking the
four-change sequence is fully expressed. It is not; the one edge that would have caught the
motivating failure is.
