# Installing seven workflows, measured

Task 10.1 through 10.3, run against `htmlspec 0.1.0` built from this change, in an empty
scratch directory at `/tmp/g10` with `--tools all`.

## What each tool received

`htmlspec init --tools all` in an empty directory:

| Tool | Files | Shape |
| --- | --- | --- |
| Claude Code | 14 | 7 skills + 7 commands |
| Codex | 7 | 7 skills, no commands |
| Cursor | 14 | 7 skills + 7 commands |
| Pi | 14 | 7 skills + 7 commands |
| AGENTS.md | 1 | one marked region carrying all seven |

49 generated `.md` files across the four skill-bearing tools. The skills-only tool receives
no command files, which is the `Delivery::SkillsOnly` property re-checked at seven rows.

## The shared region

This is the measurement the design named as a risk with a stated fallback: the region
inlines every body, so a shared-file tool receives all seven in one file.

| Measure | Value |
| --- | --- |
| Region | 36,386 bytes, 779 lines |
| Whole `AGENTS.md` | 36,408 bytes, 780 lines |
| Sum of the seven bodies | 36,143 bytes |

Per body, the seven sources:

| Body | Bytes |
| --- | --- |
| `continue.md` | 3,400 |
| `explore.md` | 3,940 |
| `ff.md` | 4,271 |
| `apply.md` | 4,425 |
| `verify.md` | 4,535 |
| `new.md` | 5,861 |
| `archive.md` | 9,711 |

**The fallback is not needed.** 36 KB of Markdown is a large file to read start to finish and
an ordinary one for a tool to load — comparable to the repository's own `AGENTS.md` at 16 KB,
and well inside what an assistant reads as context. The design's fallback (a summary per
workflow, with detail left in the skill files) stays available and unexercised. Recording the
number is what makes that a decision rather than an omission.

Two observations bound the claim. `archive.md` is 27% of the region on its own, so a later
workflow of that size would push the region past 40 KB. And a `SingleFile` tool is the only
delivery with this property: the other four split the same content across seven files, where
a tool loads one skill at a time.

## Section order

The seven sections appear in the region in lifecycle order, which is the property
`the_table_is_in_lifecycle_order` pins and this confirms against real output:

```
4:## htmlspec: explore
80:## htmlspec: new
206:## htmlspec: continue
292:## htmlspec: ff
384:## htmlspec: apply
485:## htmlspec: verify
581:## htmlspec: archive
```

## Version stamping and idempotence

All 49 generated files carry `generatedBy: htmlspec 0.1.0`. Checked by grep across every
file rather than by sampling, so a delivery that skipped the frontmatter would be caught.

A second `init --tools all` run over the established project changed **no file**. Compared by
MD5 across `AGENTS.md` and all 49 skill and command files, before and after. The region was
replaced rather than appended, and `AGENTS.md` stayed at 780 lines.

## The closing message

```
Start a change with your agent:
  /htmlspec:new  (Claude Code)
  $htmlspec-new  (Codex)
  /htmlspec-new  (Cursor, Pi)
  ask your assistant to run the new workflow  (AGENTS.md)

7 workflow(s) were installed, covering the lifecycle from exploring an idea to archiving the result.
```

The entry point is named, archival is not, and Cursor and Pi share a line because they share
a spelling. That is the per-spelling grouping preserved at seven rows.

## One defect this run found

The first run printed:

```
  ask your assistant to new  (AGENTS.md)
```

`Tool::invocation` interpolated the workflow name into a verb slot — `ask your assistant to
{workflow}`. That read correctly for the one workflow that shipped before this change,
because `archive` is a verb. `new`, `ff`, and `continue` are not, and "ask your assistant to
new" is not English.

Fixed by naming the workflow as a noun: `ask your assistant to run the {workflow} workflow`.

This is the class of defect no test in the change would have caught.
`every_invocation_names_its_workflow` asserts the workflow name appears in the rendered
string and that the placeholder is filled, and both held. The string was grammatical
nonsense that satisfied every stated property, and only reading real output surfaced it —
which is the argument task 10.4 rests on.
