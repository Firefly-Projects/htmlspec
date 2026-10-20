# The report a contributor received after the repair

Section 5, recorded after all seven bodies were amended. The pair to
`the-report-received-2026-08-24.md`, which holds the failing report from the same prompt.

## The run

`htmlspec 0.5.0`, installed from `refs/heads/report-in-the-contributors-terms` at `819d07a`,
in a Pi-only project at `~/projects/toy_project`. The installed bodies are stamped
`generatedBy: htmlspec 0.5.0`, which is task 5.1.

The prompt is the one that produced the failing report, reused verbatim so that the only
variable is the guidance:

```
/htmlspec-new I want to start a new app that is a tally mark creator.
It should have 2 tabs the first is one where you can add tally boxes that you can name
and click on, each is tied to a counter.
The second is a tab with stats.
```

## The report

```
All three documents validate clean. Here is what you're getting.

What working looks like

A person opens the app on the Counters tab, which is empty. They add a box and name it
"Cardinals"; an empty box labelled "Cardinals" showing 0 appears. They add a second box,
"Blue jays", showing 0. They click "Cardinals" three times and watch it read 3; they click
"Blue jays" once and it reads 1, while "Cardinals" stays at 3. They switch to the Stats tab
and see two rows — "Cardinals" 3 and "Blue jays" 1 — a total of 4, and a box count of 2. They
switch back to Counters and the two boxes are still there with their counts intact.

What it excludes

The boundary is: this is a counting surface and a read-back, nothing more. No backend,
accounts, or sync — it runs entirely in the browser. A click only ever raises a count: no
decrement, edit, rename, or delete of a box. No history or per-click timestamps. No charts on
Stats — it's a list plus totals.

What I decided for you, and what still needs you

Decided, and cheap to change:
- The name add-tally-counters for the change directory. Renamed by moving the directory, so
  say the word if you'd prefer another.
- Counts survive a page reload, stored in the browser's localStorage. Your description didn't
  say either way; I settled it in the design because in-memory-only would silently erase a
  session on an accidental reload — the exact failure of paper tallies. Easy to reverse if you
  want it purely ephemeral.
- Box names need not be unique — two boxes can share a name, kept distinct internally.
  Forbidding duplicates would add an error path for no gain.
- Vanilla JavaScript, no framework or build step. The state is one list and two render
  functions.

Scope I added that you did not ask for: none — I held to your two tabs.

What genuinely needs you: whether "click only ever increments" is right, or whether you'll
want to correct a miscount (decrement/edit). I left it out to settle the core act first, but
it's the most likely thing you actually want, and it changes the shape of the work if so.

Where it landed, and what's next

The change is at htmlspec/changes/add-tally-counters/, holding proposal.html, design.html,
and tasks.html. The tasks, in order: task-page-shell (the static page and tab switching,
first because everything renders into it), task-state-model (the counter list with add and
increment), task-counters-panel (add a named box, click to increment), task-stats-panel
(per-counter list and derived totals), and task-persistence (localStorage save and safe
restore). Next command when you're ready to build: htmlspec validate stays clean, and
implementation is driven by the htmlspec-apply-change skill.

Nothing runs yet. This step produced three specification documents and no code — there is no
app to open. The tasks describe the build; the build happens next.
```

## The register obligation, against the failing report

The first claim after the one-line preamble is a person naming a box "Cardinals" and watching
it read 3. The failing report's first claim was a numbered task order.

The quoted sequence is not composed for the report. `proposal.html` contains it already, and
the report reproduces it near-verbatim:

> What working looks like: a person opens the app on the Counters tab, which is empty. They
> add a box and name it "Cardinals"; an empty box labelled "Cardinals" showing 0 appears.

This is the surface-rather-than-summarize obligation doing its work. Note where that prose
sits: inside the proposal's `What changes` section rather than under a heading of its own. The
agent found it and quoted it regardless, so the obligation does not depend on the proposal
using a particular heading.

## Every pointer is glossed, and there are five

| Identifier in the report | Gloss given in the same sentence |
| --- | --- |
| `task-page-shell` | the static page and tab switching |
| `task-state-model` | the counter list with add and increment |
| `task-counters-panel` | add a named box, click to increment |
| `task-stats-panel` | per-counter list and derived totals |
| `task-persistence` | localStorage save and safe restore |

All five identifiers exist in `tasks.html` with those subjects, so no gloss is invented. Five
of five are named. The failing report named one of ten and restarted its numbering twice, so
items 6 through 10 were referenced and never shown.

`htmlspec validate` and `htmlspec-apply-change` appear unglossed, which is correct: a command
and a skill name name their own referents, and the contract states that a self-describing
reference owes nothing.

**The load was lighter here than in the failing case.** This run produced five tasks and zero
warnings; the earlier one produced ten tasks and eight warnings. So this transcript proves the
gloss rule holds, not that it holds under pressure. Task 5.6 exists for that, and `apply.md`
carried six artifact-subjected lines before the repair — more than the body that produced the
complaint.

## The boundary is stated plainly

> **Nothing runs yet.** This step produced three specification documents and no code — there
> is no app to open. The tasks describe the build; the build happens next.

This is task 5.4, and it is the check that guards the repair against its own failure mode. A
report that reads warmly while leaving a contributor to discover that nothing runs is worse
than the bookkeeping report under repair.

## No artifact fact was lost

The change directory, all three filenames, the task order with the reason the first task is
first, and the next step are all present. They appear in the closing section rather than the
opening one, which is the move the change intended: from subject to annotation. Task 5.5 asked
whether the reordering dropped anything, and it did not.

## One thing better than the instruction asked for

The bodies ask an agent to separate what it decided from what still needs the contributor.
This report added a line no instruction names:

> Scope I added that you did not ask for: none — I held to your two tabs.

Reporting the *absence* of unrequested scope is a stronger claim than silence, because silence
cannot be told apart from an agent that did not check. The `workflow-reporting` requirement on
settling decisions asks that added scope be named and its removal offered. An agent that added
none, and says so, satisfies the requirement's purpose rather than its letter.

The single open question is also the right kind. Whether a click can be corrected changes the
shape of the work, and the report says so. The failing report asked the contributor to
adjudicate a collapse threshold of 50 and a reset-per-box behavior they never mentioned.

## What this transcript does not establish

Whether the first claim reads in the contributor's vocabulary is the contributor's judgment,
not the agent's. This file records that the obligations were met in form. Scott read this
report and accepted it, which is the acceptance the register obligation admits, since task 4.3
already recorded that no mechanical check reaches it.
