# Omitting the name does not change what a contributor types

Tasks 4.1 through 4.6. Run against this repository after the `write_commands` change, with
`htmlspec 0.1.0` built from this tree.

## The regeneration

```
./target/debug/htmlspec init --tools claude,codex,pi
  Claude Code: 14 file(s) in .claude
  Codex:        7 file(s) in .codex
  Pi:          14 file(s) in .pi
```

`git status` reports **14 modified files**, all of them command files:

```
 M .claude/commands/htmlspec/{apply,archive,continue,explore,ff,new,verify}.md
 M .pi/prompts/htmlspec-{apply,archive,continue,explore,ff,new,verify}.md
```

14 here, not 21: this repository selects `claude,codex,pi`, and Codex is skills-only. 21 is
the all-tools figure, confirmed separately by `no_tool_receives_a_name_twice`.

No skill file changed. The 7 skills under `.claude/skills/` still declare their names, so the
omission is scoped to command files as intended.

## The frontmatter

```yaml
---
description: Implement an html-spec change — work its tasks, settle each task's state …
allowed-tools: Bash(htmlspec:*), Read, Write, Edit, Glob, Grep
generatedBy: htmlspec 0.1.0
---
```

`name:` is absent. `description`, `allowed-tools`, and `generatedBy` are retained —
`generatedBy` because an accepted requirement obliges instructions to record the generating
version, and dropping it alongside the name would have broken that.

## The closing message is unchanged

```
Start a change with your agent:
  /htmlspec:new  (Claude Code)
  $htmlspec-new  (Codex)
  /htmlspec-new  (Pi)
```

Three distinct spellings, grouped per spelling, from the single producer
`report_next_command`. `Tool::invocation` has exactly one non-test caller
(`commands.rs:1102`), unchanged by this work.

## The `opsx` comparison

The two vendored `opsx` command sets in this repository disagree about the field, and both
load:

| Set | `name:` |
| --- | --- |
| `.pi/prompts/opsx-*.md` | **0 of 10** carry it |
| `.claude/commands/opsx/*.md` | present, as `name: "OPSX: Apply"` |

Ten Pi prompt files omit `name:` entirely and are invocable. That is the reason to expect
omission to hold, and it is the same file kind html-spec now writes without a name.

The Claude Code side shows the field is also *permitted* to be free-form: `"OPSX: Apply"`
registers as `/opsx:apply`, which is neither the name nor derivable from it. Both directions
point at the file path as what determines the invocation.

## AGENTS.md

Untouched by the regeneration — `git diff --stat AGENTS.md` is empty. It carries no
frontmatter and this change does not reach it.

## What was NOT verified

**Task 4.2 and 4.3 are not closed by this evidence.** Whether the seven `/htmlspec:*` commands
still load, and whether a session load now lists seven html-spec entries rather than fourteen,
can only be observed by a harness loading the files — not by reading them. The regeneration is
on disk and a fresh session is what confirms it. The `opsx` Pi prompts above are the reason to
expect it, and they are an argument rather than the observation.

**Cursor's command loader was not tested** (task 4.5). This repository does not select Cursor,
so its 7 command files were not regenerated or loaded here. The design records this as an
accepted risk: if Cursor requires `name:` in a command file, the fallback is a distinct
per-tool value, and `no_tool_receives_a_name_twice` is what would keep that from re-colliding.

**No loader was shown to *reject* a duplicate name.** The reported symptom was a doubled
listing. Nothing here establishes that any tool errors on the repeat, and the change does not
depend on it doing so.

## The load, observed 2026-08-21

The section above recorded tasks 4.2 and 4.3 as open, because reading a file does not
establish that a tool loads it. Both are now closed by observation rather than by argument.

After #6 merged as `c864727` and the profile upgraded to it, a session load reports the
html-spec workflows **once each** rather than twice. The doubled listing — fourteen entries
for seven workflows — is gone, and the seven `/htmlspec:*` commands remain invocable under
their existing spellings.

That settles the one claim the `opsx` comparison could only argue for. Ten
`.pi/prompts/opsx-*.md` files omitting `name:` and loading was the reason to expect a command
file to need no name. This is the proof that html-spec's own command files do not.

Two limits from the section above still stand, and neither is closed by this load:

- **Cursor's command loader remains untested.** This repository selects `claude,codex,pi`, so
  Cursor's 7 command files were never loaded by Cursor. The design records the accepted risk
  and the fallback.
- **No loader was shown to reject a duplicate name.** The symptom was a doubled listing, and
  it is a doubled listing that went away.
