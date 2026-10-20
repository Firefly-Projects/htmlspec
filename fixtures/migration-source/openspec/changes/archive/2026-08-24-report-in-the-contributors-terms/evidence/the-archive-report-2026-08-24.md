# The archive report, on the body that closes the lifecycle

A fourth transcript, recorded because `archive` ran on the same change and its register had not
been read. Not required by any task in section 5, which asked for one other workflow beyond
`new`. Kept because `archive.md` carried four artifact-subjected lines before the repair, and
because its report is the one that describes a standing contract rather than work in progress.

## The run

`htmlspec 0.5.0` from `refs/heads/report-in-the-contributors-terms` at `819d07a`, in the
Pi-only project at `~/projects/toy_project`, archiving `add-tally-counters`.

## The report

```
The archival is complete. Here is the report.

What the project now guarantees

The project now ships the Tally Mark Creator, a single static browser page, and holds these
obligations as its standing contract in the new accepted specification
htmlspec/specs/tally-mark-creator.html:

- A person can add a named tally box, which appears at count zero; a blank or whitespace-only
  name creates nothing, and two boxes may share a name (kept distinct by a stable identifier).
- Clicking a box raises only its own count by one; there is no decrement, edit, rename, or
  delete.
- Two tabs, Counters and Stats, switch without reloading and counter state survives the switch.
- The Stats tab reports each box with its count plus a derived total and box count, computed at
  render time and never stored.
- Counts persist to localStorage and are restored exactly on reload, and an absent or
  unparseable store degrades to the empty first-run state rather than a broken page.

These carry three settled decisions (localStorage under one key, derived-not-stored totals,
vanilla JS static page) and one accepted risk (corrupt/absent store, mitigated by the
parse-guard).

What I could not resolve into a contract

Nothing was left unresolved. The implementation matches the proposal and design faithfully —
every done task has code behind it in app.js, and the one question the proposal deferred (reload
survival) the design settled and the code implements, so I stated it as req-persist-across-reload.
The proposal's scope exclusions (no backend/sync, no history/timestamps, no charts) I folded into
requirement wording where the code enforces them (the "no decrement/edit/rename/delete" clause)
rather than inventing standalone "shall not" requirements for capabilities that simply do not
exist.

One honesty note on verification: the five data-outcome="passed" verification units record
static inspection of app.js and index.html, not an interactive browser run. For an app this
small the code paths are decisive, but I did not exercise the UI in a browser, and the method
text says so.

What changed in the accepted specifications

htmlspec/specs was empty; this change founds it. I created one file,
htmlspec/specs/tally-mark-creator.html, carrying the five requirements, three decisions, one
risk, one evidence unit, and three verification units. I settled the data-change="added" deltas
there, since the accepted spec is the standing contract rather than a delta against a prior one
— that is also why it validates with no warnings while the change's own copy retains them. I
added nothing to, and removed nothing from, any pre-existing accepted requirement, because there
were none.

Where it landed

The change moved intact — proposal, design, tasks, and the new specification — to
htmlspec/changes/archive/add-tally-counters/; spec.nix version advanced 0.0.0 → 0.1.0.
htmlspec archive add-tally-counters reported 0 errors, 6 warnings, all of the advisory
review/stale-verification and review/stale-reference kind — each because a verification or
evidence unit references a requirement the change marks added — which I left in place on the
archived as-built record because that annotation truthfully records what this change introduced;
the never-gating warnings do not block archival.
```

## The register holds on a fourth subject

The first claim is what the project now guarantees, and the first bullet is a person adding a
named tally box. Not the specification file, though the file is named in the same sentence as
annotation. The three subjects read so far are different from each other — what you are getting,
what now works, what the project now guarantees — which is the register obligation stated as
register rather than as a fixed sequence of slots.

## Confirmed against the files

| Claim | Result |
| --- | --- |
| `htmlspec/specs/tally-mark-creator.html` created | confirmed, and `htmlspec/specs` held nothing before |
| five requirements, three decisions, one risk, one evidence unit | confirmed by unit count |
| three verification units | confirmed |
| the accepted spec carries no `data-change` deltas | confirmed — zero occurrences |
| the change's own copy retains them | confirmed — `data-change="added"` present in `specification.html` |
| change moved intact with its specification | confirmed — four documents under `htmlspec/changes/archive/add-tally-counters/` |
| `spec.nix` advanced to `0.1.0` | confirmed |
| 0 errors, 6 warnings | confirmed — five `review/stale-verification`, one `review/stale-reference` |

## One arithmetic slip

The report writes "the five `data-outcome="passed"` verification units". The file carries three
verification units and three `data-outcome="passed"` markers. Five is the requirement count, so
the wrong figure appears to be borrowed from the sentence above it.

Recorded because it is a factual error in a report, and this file is a record. It does not touch
the honesty note that sentence exists to make, and it is not a register or gloss failure — it is
a miscount inside an otherwise correct passage. No obligation in `workflow-reporting` addresses
arithmetic, and none should: a requirement that a report's numbers be right is a requirement no
instruction can enforce.

## The unprompted disclosure, for the third time

> One honesty note on verification: the five `data-outcome="passed"` verification units record
> static inspection of `app.js` and `index.html`, not an interactive browser run.

No instruction asks for this. A specification that claims verified behavior nobody exercised in a
browser is a hazard a contributor cannot see from the file, and the report surfaced it without
being asked.

This is the third run in a row to volunteer a limit rather than let silence stand. The `new` run
reported that it added no scope the contributor did not ask for. The `apply` run reported that it
declined a test suite and why. The `archive` run reports that its passing verifications are
static inspection. Three workflows, three agents, one habit — the guidance is producing a
disposition rather than three obedient outputs.

## The warnings are glossed

Six warnings, both kinds named, and the reason given in the same sentence: each is a verification
or evidence unit referencing a requirement the change marks `added`. The report also states why
they were left in place, which is that the annotation truthfully records what a founding change
introduced. A count with no kind attached is a pointer under the contract. This is not one.
