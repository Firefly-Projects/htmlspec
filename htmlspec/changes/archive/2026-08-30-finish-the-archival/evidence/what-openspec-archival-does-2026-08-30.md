# What OpenSpec's archival does, read from its workflow rather than its help

Task 1.4, recorded 2026-08-30. The proposal claims this change does not diverge from OpenSpec on
merging. That claim is false if `openspec archive` merges programmatically, so it is checked
here against the workflow file rather than against the command help.

## The help, which is where the wrong impression comes from

```
archive [options] [change-name]    Archive a completed change and update main specs
```

Read alone, that says the program updates the accepted specifications. It is the line a
contributor sees, and it is why "make our archive do what OpenSpec's does" sounds like it
requires programmatic merging.

## The workflow, which is what actually runs

`.pi/skills/openspec-archive-change/SKILL.md`, step 4, routes the merge elsewhere:

> Then run the `openspec-sync-specs` workflow inline (agent-driven intelligent merge) for change
> '<name>', passing the delta spec analysis and the fetched specs-rule snapshot from above, and
> wait for it to finish.

The same step describes what the program contributes, and it is comparison rather than merging:

> Compare each delta spec with its corresponding main spec at
> `<planningHome.root>/openspec/specs/<capability-path>/spec.md` … Determine what changes would
> be applied (adds, modifications, removals, renames)

And `openspec-sync-specs`, the workflow that performs the merge, states the principle the
HTMLSpec archive workflow already quotes:

> This is an **agent-driven** operation… Unlike programmatic merging, you can apply partial
> updates. The delta represents *intent*, not a wholesale replacement.

## What follows

Both tools assess mechanically and merge by judgment. The division HTMLSpec states in
`artifact-kinds` — *every synthesis and every contract update is agent-driven* — is the division
OpenSpec operates under, whatever its help text advertises.

So the gap between the two archivals is not the merge. It is everything around the merge: the
completion checks, the delta assessment, the date-prefixed move, the collision refusal, and the
summary. All of those are mechanical, and none of them is something `artifact-kinds` reserves.

This is the finding that made the change writable without touching `artifact-kinds`. Had
OpenSpec's archive genuinely merged by parser, matching it would have required withdrawing that
capability's central requirement, and the honest recommendation would have been not to.
