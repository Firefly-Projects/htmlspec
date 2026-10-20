# What the enumeration carries

## One counter, called twice

The whole mechanism is: the per-change state `status` computes becomes a function the
enumeration can call once per change, and both operations call it.

The alternative — `list` counting tasks itself — is rejected on evidence rather than taste.
Version 0.6.0's changelog records a version gate that never fired for exactly this shape of
reason: a fact the project recorded in one place was recomputed elsewhere from an invented
convention, the two never met, and a green suite coexisted with a check that had never
refused anything. A listing that reports `17/31` while `status` reports `18/31` is the same
defect, and it is worse to find, because neither operation is wrong on its own terms.

The accepted contract already requires this shape between `status` and archival: "Archival
SHALL derive its report about incomplete work from this same operation, so that a contributor
who checks a change before archiving it and the archival that follows cannot disagree." This
change extends the same obligation to a third caller rather than inventing a rule for it.

## Four states, not one fraction

`total_tasks` returns the count; `open_tasks` returns everything not `done` and not `dropped`.
That filter is right for archival, which asks a yes-or-no question about outstanding work, and
it is lossy for a listing, which is read to compare changes against each other.

The schema's task state is `todo | in-progress | done | dropped`
(`crates/htmlspec-schema/schema/document.schema.json`). Two distinctions the fraction alone
destroys:

- `0/24` is *untouched* if all 24 are `todo` and *fully under way* if all 24 are
  `in-progress`. Opposite situations, one number.
- `20/24` is *finished* if the remaining four are `dropped` and *nearly finished* if they are
  `todo`. A contributor deciding what to archive needs these apart.

So the payload carries a count per state. The human report renders a fraction from it, because
a fraction is what reads well in a column, and names the change's overall state beside it —
derived from the breakdown rather than stored, since a change's state is a function of its
tasks and storing it separately would create a second answer.

**Done over total, not open over total.** The fraction's numerator is `done`, and its
denominator counts every task including `dropped`. Dropping the dropped tasks from the
denominator would make a change's total shrink as work is abandoned, so `18/20` could become
`18/18` without anything being completed. A denominator that moves is a denominator nobody can
compare two readings of.

## A change with no tasks document

`total_tasks` returns 0 when `tasks.html` is unreadable, and `open_tasks` returns an empty
vector with a comment saying archival does not gate on it. The listing inherits that: such a
change is enumerated, and its counts are absent rather than zero.

Absent rather than zero is the decision worth stating. `0/0` reads as a change with no work
left, which is the opposite of what an unwritten tasks document means. The payload omits the
counts; the human report says so in the column where the fraction would be.

## The reading cost

The enumeration currently reads one directory. It will now parse one `tasks.html` per active
change. That is bounded by the number of active changes — four in this repository, and a
project with enough active changes for this to matter has a problem the listing is not the
place to solve.

It is worth naming because the enumeration is on the path of every workflow that selects a
change, and because the honest answer is that the cost is real and small, not that there is
none.

## Rejected: a flag to opt into the counts

`list --status` was considered, to keep the cheap enumeration cheap. Rejected: the accepted
requirement says the enumeration exists so a workflow can select what it acts on, and a
selection basis available only behind a flag is one most callers will not pass. The
instructions would then have to name the flag everywhere, and an agent that forgot it would
select on name order with no indication anything was missing.

## Rejected: merging `list` and `status`

They answer different questions and the accepted contract names them as separate operations
with separate requirements. `list` describes a project, `status` describes one change and
reports artifact presence and the open task identifiers — detail that would make a listing
unreadable at four changes and unusable at twenty. The listing carries counts; `status` keeps
the names.
