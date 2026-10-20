## Why

`/htmlspec-new I want to start a new app that is a tally mark creator...` discards
everything after the command name. The agent receives the instruction body alone and asks
what the work is, because that is the correct behavior when nothing was described.

This is the same outcome `accept-a-described-change` was written to repair — a contributor
states the work and is asked for it again — reached by a different mechanism. That change
fixed the prose: a need may now be an absence, and an agent derives an unstated name rather
than demanding one. The prose repair works. Its own verification run
(`openspec/changes/archive/2026-08-22-accept-a-described-change/evidence/described-change-run-2026-08-22.md`)
records an agent naming the absence form, declining to invent a need, and disclaiming the
round trip to approve a name — and then creating nothing, because the description never
arrived.

**No generated file carries an argument placeholder.**

```
grep -rc 'ARGUMENTS' .claude .codex .pi          # 0 in every file
```

`write_commands` at `crates/htmlspec-cli/src/install.rs:124` emits `description`,
`allowed-tools`, and `generatedBy`. `write_skills` at `:88` emits `name`, `description`,
`allowed-tools`, `license`, and `generatedBy`. Neither emits `argument-hint`, and no
instruction body in `crates/htmlspec-assets/instructions/` contains `$ARGUMENTS`. The
tools that load these files substitute the invocation's arguments into that token, so with
no token there is nothing to substitute into and the text a contributor typed is dropped
before an agent sees it.

The convention is not in doubt. Claude Code's own example plugin ships
`argument-hint: <required-arg> [optional-arge]` with `The user invoked this command with:
$ARGUMENTS` in both the command and the skill form, and states that the two layouts "are
loaded identically". A Codex plugin command branches on `If "$ARGUMENTS" contains "prod"`.
The mechanism is ordinary, documented, and this program does not use it.

**Why this outranks an ordinary omission.** Seven workflows are installed and every one of
them takes a subject. `new` takes a description of the work, `apply` and `verify` take a
change to act on, `explore` takes the idea to think through. All seven silently discard it.
The failure is invisible from inside the program: `htmlspec init` writes the files, every
one is valid, every generated-file test passes, and the loss happens in a tool this
program does not control. Only a transcript shows it, which is why it survived two changes
that were looking directly at the `new` workflow's text.

**Why the prose repair could not have caught it.** `accept-a-described-change` amended
instruction bodies. A body is what an agent reads *after* invocation, so no amendment to
one can recover an argument the invocation already dropped. That change's task 5.2 is
dropped for exactly this reason, and its dropped-task note is the boundary this change
crosses: "a prose repair to authoring guidance cannot be verified end-to-end while the
invocation withholds the contributor's input."

## What Changes

- **Generated files declare and consume their arguments.** Each command file and each
  skill file gains an `argument-hint` in its frontmatter and a placeholder its body
  substitutes into, so the text after the invocation reaches the agent.

- **The hint is a property of the workflow.** What `new` takes (a description of the work)
  differs from what `apply` takes (a change to work on), so the hint belongs in the
  `WORKFLOWS` table beside the summary rather than being one string for all seven. The
  table is already the single place a workflow's properties live and the contract already
  requires that.

- **A test asserts the placeholder survives to the file.** `install.rs` already checks
  `generatedBy` and name-loadability against produced files rather than against the table,
  for a stated reason: a value assembled by a format string at write time is invisible to a
  check on its inputs. An argument placeholder is assembled the same way, so it is checked
  the same way.

- **The contract states the obligation.** `project-bootstrap` gains the requirement that a
  generated invocation surface passes a contributor's arguments to the agent, since a
  workflow that silently drops its input asks for what it was already given.

## Non-Goals

- **Changing any instruction body's guidance.** The prose is correct as of 0.3.0. This
  change adds an arguments section to each body and touches nothing else in them. A body
  that argued differently about needs or names would be re-opening a settled change.

- **Changing `htmlspec new change` or its arguments.** `<NAME>` stays required and
  `--subject` stays optional. This is the same non-goal `accept-a-described-change` stated,
  and it holds for the same reason: the defect is in what the agent receives, not in the
  command it then runs.

- **Making an argument required.** A bare `/htmlspec-new` with no description must keep
  working and must keep asking. The amended guidance already handles an empty description
  correctly — "Where a request carries no description of the work at all, asking is right" —
  so an empty placeholder must reach the agent as an empty one rather than as an error.

- **Supporting positional or named arguments.** `$1`, `$2`, and flag parsing are a surface
  this project does not need: a workflow takes prose, not fields. One placeholder for the
  whole remainder is the whole requirement.

- **Verifying 5.2 as part of this change.** That task belongs to an archived change and
  stays dropped. This change is verified by its own run against its own request, and what
  it establishes about the earlier change's prose is a remark rather than a reopening.

- **The `AGENTS.md` region.** A marked region in a shared file has no invocation and takes
  no arguments, so there is nothing to pass through. It is untouched, and the requirement
  is written so it does not reach a delivery kind that has no command surface.
