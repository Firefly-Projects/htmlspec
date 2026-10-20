## Why

A session load prints this:

```
[Skill conflicts]
  auto (user) ~/.agents/skills/_driver_add_schemas/SKILL.md
    name contains invalid characters (must be lowercase a-z, 0-9, hyphens only)
  … seven more …
```

Those eight files belong to another repository. The reason they matter here is that
**`htmlspec init` writes the same field, for five tools, with no rule about what may go in
it.** Every project a contributor bootstraps inherits whatever this program decides, and
right now it decides two different things for no stated reason.

`crates/htmlspec-cli/src/install.rs` has two writers of the same frontmatter key:

| Writer | Source of `name:` | Produces |
| --- | --- | --- |
| `write_skills` (`:107`) | `workflow.directory` | `htmlspec-apply-change` |
| `write_commands` (`:149`) | `tool.invocation(workflow.name)` | `/htmlspec:apply` |

The command form carries a `/` and a `:`. The skill form conforms — and it conforms *by
coincidence*, because a skill directory happens to be spelled in kebab-case. Nothing asserts
it, so nothing would notice if a later workflow directory were named
`htmlspec_bulk_archive`. That is one underscore away from the eight warnings above, in a
field this program writes 35 times per project across five tools.

**The `/htmlspec:apply` value does not warn today, and that is not the same as being
right.** I checked: the loader applies the character rule to skills, not to commands. The
vendored `opsx` commands in this repository declare `name: "OPSX: Apply"` and have loaded
without complaint since `2989654`. So the current output works.

It works for one harness. `init` installs for five, and the tolerance being relied on is one
loader's treatment of one file kind. A `name:` field holding `/htmlspec:apply` is not a name
in the first place — it is the invocation, which the table already knows how to compute and
the closing message already prints from `Tool::invocation`. Putting it in the name field
states the same fact twice and states it in the one place a stricter reader will reject it.

**The precedent is the argument.** The change that installed the full workflow set replaced a
rule held in a code comment with a test over clap's command tree, because a rule a human
remembers is a rule that ships broken. This is the same shape one layer out: the frontmatter
this program generates has no stated contract, and the other repository demonstrates what
that costs — six malformed markers passed review, shipped, and surfaced from a session load
rather than from a diff.

## What Changes

- **A generated `name:` is a name, and it must be loadable.** Both writers produce a value
  of lowercase letters, digits, and hyphens. The command writer stops putting the invocation
  there and uses the workflow's identity, the way the skill writer already does.

- **The invocation keeps its one home.** `Tool::invocation` exists to answer "what does a
  contributor type", and the closing message is where that answer is printed. Removing it
  from the frontmatter leaves one producer of the spelling instead of two, which is the same
  reasoning the table's `entry_point` flag rests on: one fact, one place.

- **A test over every generated file.** `htmlspec init` writes 35 files across five tools,
  and no test reads a `name:` value. The check walks every skill and command file a
  bootstrap produces and fails on a value outside the character set — so the skill form's
  accidental conformance becomes a guaranteed property, and a workflow directory added later
  with an underscore fails the build rather than a contributor's session.

- **The contract states the rule.** `project-bootstrap` requires that generated instructions
  record their version and says nothing else about their frontmatter. It gains the obligation
  that a generated name is loadable, and that the invocation is not a name.

## Non-Goals

- **Changing how a contributor invokes anything.** `/htmlspec:apply` remains what you type.
  Only the frontmatter key changes, and the closing message that teaches the spelling is
  untouched.

- **Renaming a workflow, a skill directory, or a command file.** The seven directories and
  their file names already conform. This is about a field's contents.

- **Fixing the eight warnings.** They come from `~/skills`, whose `_`-prefixed helper markers
  are Python package directories imported by 75 files — a constraint that repository's own
  contract records and that has nothing to do with this program. Two of the eight are not
  even a defect there: they are a synced destination running behind commit `d27e6fd`. This
  change prevents html-spec from ever producing that class of file; it does not reach into
  another repository to repair one.

- **Auditing the five tools' loaders.** The character rule is inferred from one warning's
  text rather than read from five published specifications. Conforming to the strictest rule
  observed is the cheap and safe direction, and this change does that rather than pretending
  to a survey it has not done.
