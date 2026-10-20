# `htmlspec status --change` pointed at a runbook

The proposal corrects itself on this point: it first claimed completion counting comes free
with a runbook, then withdrew that. **The withdrawal is right, and it is also not the whole
truth.** Measured in a scratch project holding the cookie recipe at
`htmlspec/runbooks/chocolate-chip-cookies/tasks.html`.

## By name: refused, exit 2

```
$ htmlspec status --change chocolate-chip-cookies
htmlspec: no change named chocolate-chip-cookies in
  /private/tmp/rb-scratch/htmlspec/changes, and no directory at chocolate-chip-cookies
exit 2
```

Exactly what the proposal predicts. `resolve_change_in` (`commands.rs:706`) resolves a name
against `project.changes()` and nowhere else, so a runbook name matches nothing. The report
names `htmlspec/changes` as the directory it looked in, which is at least honest about why.

## By path: it works, and the proposal does not say so

```
$ htmlspec status --change htmlspec/runbooks/chocolate-chip-cookies
  absent  proposal.html
  absent  design.html
  present tasks.html
  absent  specification.html
  0 of 9 task(s) done or dropped
exit 0
```

**This is a correction to the proposal.** It says `status` "reads only the changes directory,
so it reports no change named `chocolate-chip-cookies` when pointed at a runbook". The first
clause is wrong about the path form. `resolve_change` accepts a directory that exists, and a
runbook directory exists, so the counting logic runs and counts correctly: 9 task units, 0
done or dropped, which is what the canonical procedure should read.

The proposal's conclusion survives the correction and is worth restating in its stronger form:

> The logic is reusable and it does not reach a runbook today.

Both halves now measured. It **is** reusable — pointed at a runbook by path it produces the
right numbers, and this change added nothing to make that true. What does not reach a runbook
is the **name** form, which is the one a contributor or a workflow would use.

## Why this change does not close it

Two artifacts of the report are wrong for a runbook and neither is a bug in `status`:

- `absent proposal.html`, `absent design.html`, `absent specification.html` — a runbook has
  two documents, not four, and `specification.html` is not one of them. `status` reports
  against `CHANGE_DOCUMENTS`, which is a change's shape.
- `0 of 9 task(s) done or dropped` is correct for the canonical procedure and would be the
  wrong subject for a run, whose copy is where states advance.

So making `status` accept a runbook by name means deciding what it should report about one,
and that decision belongs where run progress is reported. `log-a-runbook-run`'s Impact
section already claims it and names it open:

> Whether run progress belongs in `status` or in the `run` operation's own report is open,
> and `workflow-reporting` governs the answer.

This change therefore records the gap and does not narrow it. `htmlspec list --runbooks`
prints names and document counts, which is what listing owes; it deliberately prints no task
fraction, for the reason above — a fraction on the canonical procedure is always `0/N` and a
column that never moves teaches a reader to ignore it.
