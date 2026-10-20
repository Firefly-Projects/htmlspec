# The description reaches the agent, run on Pi

Tasks 6.2, 6.3, 6.6. Run in `~/projects/toy_project` against the installed
`htmlspec 0.4.0` from `~/.nix-profile/bin`, so the binary and the instructions came from the
same version.

The invocation, in Pi:

```
/htmlspec-new I want to start a new app that is a tally mark creator.
It should have 2 tabs the first is one where you can add tally boxes that you can name
and click on, each is tied to a counter.
The second is a tab with stats.
```

## A change was created on the first exchange

```
htmlspec/changes/add-tally-counters/
  proposal.html
  design.html
  tasks.html
```

No exchange was spent asking what the work was. This is what task 5.2 of
`accept-a-described-change` could not observe and was dropped for.

## Pi substitutes `$ARGUMENTS` — the accepted risk is retired for Pi

The design accepted not knowing whether Cursor and Pi substitute, and recorded that a
non-substituting tool would show a literal `$ARGUMENTS` in the body. Pi shows no such
token: the description arrived and was acted on. **Pi substitutes.** Cursor remains
unobserved.

This is the more useful of the two confirmations, because Pi is the tool actually in use
here, and it was the one the design could not vouch for.

## The content came from the description, not from the worked example

This is task 6.3, and it is the distinction the earlier run could not draw. The example
`new.md` ships is:

> "A browser application that adds named tally counters, incrementing on click, with no
> backend" yields `add-tally-counters` in one step.

That example has **no stats tab and no tab structure at all**. The proposal has both:

```
"two tabs"   2 mentions
"Stats tab"  7 mentions
"stats"      8 mentions
```

Its prose describes "The Stats tab reads the same counters. It reports, per box, the count
and that box's share of the total, and reports the overall total, the number of boxes, and
which box leads." Nothing in the worked example could produce that. The content is derived
from the contributor's own text.

**One caveat, stated because the name alone is not evidence.** The derived name
`add-tally-counters` is identical to the example's. From the name by itself there is no way
to tell derivation from copying — which is exactly the ambiguity that made the earlier run
inconclusive. The document body is what settles it here, and a future run of this check
should read the body rather than the name.

## The need is stated as an absence, in the form 0.3.0 introduced

Unplanned confirmation of the previous change, from the proposal's own Why:

> The project cannot count anything yet. There is no application in this repository — only
> the specification toolchain — so this change adds a capability rather than repairing one,
> and there is no measurement or error message to quote.
>
> Who wants it: anyone counting several kinds of thing at once during an activity where
> stopping to write is awkward. [...] What they cannot do without it: keep more than one
> running count where each count is one tap away, and read a total without recounting the
> marks.

All three questions of the absence form answered, and the sentence about having no
measurement to quote is the guidance being applied deliberately rather than incidentally.
`accept-a-described-change`'s task 5.2 stays dropped — this does not reopen it — but the
prose repair it shipped is now observed working end to end.

## A separate defect this run surfaced, filed rather than folded in

The contributor's reaction to the report: *"If I was a user I asked to make a tally book app
and I got a bunch of nothing back."*

That is a fair reading of the report and it is not this change's subject. `new.md`'s "What to
report" section asks for five things and every one of them is about the artifacts: where the
change landed, what the proposal argues, the design's decisions, the task order and why the
first task is first, and what could not be settled. None asks the agent to say what the
contributor will get or when they will see it working.

So the report closed on a validation warning count and two design minutiae — a collapse
threshold of 50, a reset-per-box behavior — presented as questions for the contributor to
adjudicate, about a thing they had described in three sentences.

The workflow behaved correctly otherwise: `new` writes documents and no code by design. The
defect is the report's register, not its refusal to build. It belongs to authoring guidance
rather than to the invocation surface, and it is filed separately so that this change stays
a verified one.
