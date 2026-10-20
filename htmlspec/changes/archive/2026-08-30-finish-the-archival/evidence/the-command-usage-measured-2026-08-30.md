# What the vendored workflow actually invokes

Task 1.3, measured on 2026-08-30 against the OpenSpec workflow files vendored in this
repository. The proposal's claim that four commands are load-bearing rests on this count, so it
is recorded as a command anyone can re-run rather than as a figure.

## The measurement

```
grep -rho "openspec [a-z][a-z-]*" .pi/prompts/opsx-*.md .pi/skills/openspec-*/SKILL.md \
  | sort | uniq -c | sort -rn
```

```
  52 openspec status
  48 openspec instructions
  22 openspec store
  14 openspec new
  14 openspec list
  10 openspec archive
   6 openspec schemas
   4 openspec context
```

Total, from the same expression piped to `wc -l`: **170** invocations across 8 distinct
commands.

## The correction this evidence produced

The first report of this measurement, made in conversation and carried into the proposal, gave
the total as 156. The per-command counts were right and the sum was not: 52+48+22+14+14+10+6+4
is 170. The proposal and task 1.3 were corrected before this file was written.

The failure is worth recording because of where it came from. The eight counts were measured and
the total was added up by hand, so the one figure nobody could check against a command was the
one that was wrong. It is also the figure most likely to be quoted, being the headline. Task 1.3
says *measured, not estimated*; the lesson is narrower than that and worth stating: a derived
figure is not measured merely because its inputs were.

## What it supports

Of the five most-invoked commands, four have no HTMLSpec counterpart — `status`, `instructions`,
`store`, and `list`. Only `new` does. `status` and `instructions` together account for 100 of
the 170 invocations, which is why the proposal treats them as structural rather than as
conveniences.

The HTMLSpec workflow files invoke a different and smaller set:

```
grep -rho "htmlspec [a-z][a-z-]*" .pi/prompts/htmlspec-*.md .pi/skills/htmlspec-*/SKILL.md \
  | sort | uniq -c | sort -rn
```

```
  14 htmlspec validate
  10 htmlspec archive
   8 htmlspec new
   2 htmlspec format
```

Searching those same files for a change-selection step returns nothing:

```
grep -rn -i "ambiguous\|available changes\|active changes" .pi/prompts/htmlspec-*.md
```

No output. The OpenSpec prompts resolve an ambiguous change by running `openspec list --json`,
in 14 places. The HTMLSpec prompts have no such step, and the reason is that there is no command
to put in it.
