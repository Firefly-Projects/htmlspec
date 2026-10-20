# The motivating sequence, in the project that stated it

Task 1.5, recorded 2026-09-02 against `main` at `595e38c`. The project is
`~/projects/missile-launcher`
at `8669687`, an HTMLSpec project at version `0.3.0`.

## The proposal, quoted from the document

`htmlspec/changes/contract-the-guidance-solver-comparison/proposal.html`, under the heading
*The four changes this makes gradeable*:

> The contract governs three changes that follow it, and one that follows them. They are stated
> here because the ordering is the argument, and because htmlspec expresses no dependency
> between changes — the convention is enforced by contributors reading it and by nothing else.

The proposal then states the ordering as a table — in prose, in a document, where only a person
reads it:

| Change | Why it sits here |
| --- | --- |
| This one | "A contract written after the models exist is a description of the models" |
| Coarsening cost, to be created | "Solver only, no model. Both numbers decide what a later failure means" |
| The comparison, to be created | "The cheapest dynamics in the project, and the resolution machinery already exists there" |
| The device work, to be created | "Last, so a whole-solver port never blocks the cheap experiment" |

The first row is the load-bearing one, and the proposal argues it twice more elsewhere: "The
test of whether this document works is a contributor reading it and being unable to find a way
to pass the gates without solving the problem. That is what a contract is for, and it is why
this one is written first rather than alongside the models it constrains."

## The four changes as they stand today

```console
$ htmlspec list
add-the-booster-and-cycle-the-rail           0/12  todo
build-numeric-burn-solver                     2/6  in-progress
calibrate-against-named-site             no tasks
close-the-control-system                      0/9  todo
compare-the-guidance-solver-families     no tasks
contract-the-guidance-solver-comparison     13/13  done
couple-fuel-tank-and-casing-boundaries        0/9  todo
infer-rail-state-from-radar              no tasks
measure-the-cost-of-decimation           no tasks
settle-sensors-and-precision             no tasks
```

The four the proposal names, in the order it names them:

| Change | Directory contents | Completion |
| --- | --- | --- |
| `contract-the-guidance-solver-comparison` | proposal, design, tasks | `13/13 done` |
| `measure-the-cost-of-decimation` | proposal only | `no tasks` |
| `compare-the-guidance-solver-families` | proposal only | `no tasks` |
| `settle-sensors-and-precision` | proposal only | `no tasks` |

**The contract is the change that is done and unarchived.** Its 13 tasks are all resolved and
it has no `specification.html` yet, which is the state immediately before archival. Its three
successors exist as proposals. So the sequence the proposal describes is not hypothetical or
finished — it is at the exact moment the ordering matters, right now.

## Which changes would archive out of order with nothing reported

**All three successors.** Each of `measure-the-cost-of-decimation`,
`compare-the-guidance-solver-families`, and `settle-sensors-and-precision` could be carried to
completion and archived before `contract-the-guidance-solver-comparison`, and every operation
in the toolchain would be silent about it:

- `htmlspec list` would show the candidate beside the contract with no indication of the
  relation — the ten-row listing above is the whole of what a selecting agent sees.
- `htmlspec validate` checks each change against the schema and never against a sibling.
- `htmlspec archive --check` applies the version gate, the specification's presence, its kind,
  and its validation, then reports open tasks and unmerged requirements. None of the five reads
  another change directory.

`settle-sensors-and-precision` is worth naming separately, because it is the one the proposal
places fourth — "one that follows them" — so it has three predecessors in prose and would be
reported against at most one under this change's one-relation model. That is a real limit of
what is being built, and it is the limit the design accepts deliberately: one relation covers
the observed problem, and a change with two independent predecessors "is a change to make then,
with the case in hand."

## What the failure costs, in this project's own terms

The proposal states the consequence directly: a contract written after the models exist is a
description of the models. Archive `compare-the-guidance-solver-families` before the contract
and the twelve forbidden shortcuts the contract enumerates — training one model per resolution,
fine-tuning on the target, resampling from a canonical grid, changing the spectral mode count
with the grid — become criteria fitted to whatever the models turned out to do. Every document
still validates. The archive still reads as a clean sequence of versioned changes. The method
is the only thing that broke, and nothing in the toolchain is positioned to notice.
