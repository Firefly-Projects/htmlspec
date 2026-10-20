# One identity, declared twice, in two files a tool loads

Observed 2026-08-20 against `htmlspec 0.1.0` from `main` at `4972234` — the commit that
merged `conform-generated-frontmatter-names`.

## The session load

A Pi session load lists both spellings for the same seven workflows:

```
- htmlspec-apply-change
- htmlspec-archive-change
…
- htmlspec:apply
- htmlspec:archive
…
```

Fourteen entries for seven workflows.

## What a fresh install writes

```
htmlspec init --tools claude,cursor,pi,codex
```

For each tool that receives both skills and commands, the skill file and the command file
declare the **same** `name:`:

| Tool | Skills dir | Commands dir | Names in common |
| --- | --- | --- | --- |
| Claude Code | `.claude/skills` | `.claude/commands/htmlspec` | all 7 |
| Cursor | `.cursor/skills` | `.cursor/commands` | all 7 |
| Pi | `.pi/skills` | `.pi/prompts` | all 7 |
| Codex | `.codex/skills` | — (skills-only) | n/a |

The seven in common, identical on both sides for all three tools:

```
htmlspec-apply-change  htmlspec-archive-change  htmlspec-continue-change
htmlspec-explore       htmlspec-ff-change       htmlspec-new-change
htmlspec-verify-change
```

Codex is unaffected: `Delivery::SkillsOnly`, so there is no second file to collide with.

## Where it came from

`conform-generated-frontmatter-names` changed `write_commands` to use `workflow.directory`
for `name:` — the same field `write_skills` already writes from `workflow.directory`. The
prior value was `tool.invocation(workflow.name)`, which was malformed but **distinct**. The
fix traded a malformed name for a duplicated one.

That change's delta states the duplication as an intended property:

> #### Scenario: One identity per workflow
> - **WHEN** a workflow produces both instructions and a command for one tool
> - **THEN** both declare the same name

It holds. It is also the defect: "one identity per workflow" is right for a workflow's
*meaning* and wrong for a `name:` field that a loader uses as a registry key.

## A command file does not need a name at all

The two vendored `opsx` command sets in this repository, installed by a different program,
disagree about whether the field belongs there:

```
.claude/commands/opsx/apply.md      name: "OPSX: Apply"
.pi/prompts/opsx-apply.md           (no name: field)
```

All ten `.pi/prompts/opsx-*.md` files omit `name:` entirely, and they load. That is the
proof that a command file's `name:` is not load-bearing for Pi — the file path is what
determines the invocation, which is exactly what the previous change established and
verified.

`.claude/commands/opsx/*.md` do carry a name, and it is neither kebab-case nor the
invocation: `name: "OPSX: Apply"` registers as `/opsx:apply`. So for the tools in play, a
command file's `name:` is either ignored or free-form — but it must not collide with a
skill's name in the same tool's namespace.

## Scope

Three tools × 7 workflows = 21 command files currently declaring a name that duplicates a
skill's. 14 of those are checked into this repository's own `.claude/` and `.pi/`.
