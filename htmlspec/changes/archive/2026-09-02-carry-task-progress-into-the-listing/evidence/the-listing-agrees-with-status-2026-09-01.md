# The listing agrees with `status`, change by change

Tasks 5.1, 5.2 and 5.3, recorded 2026-09-01 on branch `carry-task-progress-into-the-listing`.

## Why this ran against a scratch project rather than this repository

Task 5.1 says to run the enumeration against *this project's* active changes. It cannot be:
`htmlspec` resolves a project by walking upward for an `htmlspec/` directory, and this
repository keeps its specifications in `openspec/` as 100 markdown files. The counter reads
`tasks.html` through `htmlspec_parse`, and this change's own tasks document is `tasks.md`.

```console
$ htmlspec list
htmlspec: no htmlspec/ directory in . or any directory above it,
so there is no project here. Run `htmlspec init` to establish one.
```

That gap is the subject of the active change `migrate-an-openspec-project`, which opens by
naming it: "This repository is the argument against itself." Until it lands, the enumeration
cannot be pointed at the project that produces it.

So the check is made against a project `htmlspec init` established, holding five changes in
deliberately varied states — including the two cases tasks 5.2 and 5.3 name. The script is
`evidence/prove-listing.py`, kept beside this file so the measurement can be repeated.

## What was checked

Not "the two outputs look the same". For each change, the script reads what the enumeration
carries and what `status` reports, and compares the numbers:

- the listing's denominator against `status`'s `total`
- the listing's `done + dropped` against `status`'s `total - open`

`status` renders "N of M task(s) done or dropped", so `total - open` is the number it puts on
screen. If the enumeration counted separately, these would drift and this comparison is what
would catch it.

## The result

```
=== the human report ===
alpha                        2/2  done
beta                         1/3  in-progress
delta-with-a-long-name       1/4  in-progress
epsilon-no-tasks        no tasks
gamma                        0/2  done

=== each change's completion, listing against status ===
  alpha                    listing: 2/2 (+0 dropped, done)          status: total=2 open=0 resolved=2  -> AGREE
  beta                     listing: 1/3 (+0 dropped, in-progress)   status: total=3 open=2 resolved=1  -> AGREE
  delta-with-a-long-name   listing: 1/4 (+0 dropped, in-progress)   status: total=4 open=3 resolved=1  -> AGREE
  epsilon-no-tasks         listing: completion absent               status: total=0                    -> AGREE
  gamma                    listing: 0/2 (+2 dropped, done)          status: total=2 open=0 resolved=2  -> AGREE

all 5 changes agree
```

## The two cases 5.2 and 5.3 name

**`gamma`, every task `dropped` — reads as complete, not as untouched.** It renders `0/2 done`.
The fraction alone would read as work nobody has started; the state beside it says the change is
finished, and the payload carries `"dropped": 2` to say how. This is the reading the proposal
argued a bare fraction destroys, and it is the one place the human report would be actively
misleading without the state column.

**`epsilon-no-tasks`, no tasks document — completion absent, not zero.** The report says
`no tasks` in the column the fraction would occupy, and the payload omits `total` and `tasks`
entirely:

```json
{ "name": "epsilon-no-tasks" }
```

Note the asymmetry this exposes, which is deliberate and worth recording: `status` reports
`total=0` for the same change, because its payload shape has no way to say "absent". The
enumeration can, and does. A caller reading `0/0` from `status` learns less than one reading an
omitted field from `list` — the two do not disagree about a count, they differ in what they are
able to express, and the listing is the one selection is made from.

## Ordering

The report is in name order in both readings. `alpha` sorts first while finished and `gamma`
sorts last while also finished, so the order plainly does not follow progress —
`the_report_aligns_across_names_of_differing_length` pins this in the suite as well.
