## Context

`htmlspec init` writes 35 instruction files for a five-tool project: seven skills and seven
commands each for the three tools that take both, seven skills for the one that takes only
those, and one marked region. Every skill and command file carries YAML frontmatter this
program generates, and `name:` is the first key in it.

Two functions write that key from two different sources. `write_skills` uses
`workflow.directory`. `write_commands` uses `tool.invocation(workflow.name)`. The first
yields `htmlspec-apply-change`; the second yields `/htmlspec:apply`, `$htmlspec-apply`, or
`/htmlspec-apply` depending on the tool.

Nothing in the accepted contract says what belongs in that field, and no test reads one. The
divergence is not a decision recorded anywhere — it is two functions written at different
times.

## Goals

- Every `name:` this program writes is loadable under the strictest rule observed.
- The invocation has one producer.
- A workflow added later cannot introduce a malformed name without failing the build.

## Non-Goals

- Changing what a contributor types, or renaming any directory or file. See the proposal.

## Decisions

### The command's name becomes the workflow's identity, not its invocation

`write_commands` stops calling `tool.invocation` and uses the same identity the skill file
uses.

The question is which identity. Three candidates:

- **`workflow.name`** — `apply`. Short, and collides: a flat command directory already
  requires the `htmlspec-` prefix on the *file* name to avoid clashing with another tool's
  unrelated `apply.md`, and a bare `apply` in the name field reintroduces exactly that
  ambiguity one level in.
- **`workflow.directory`** — `htmlspec-apply-change`. What the skill uses, so the two files
  for one workflow agree.
- **A third spelling** — a new field on the table. More surface for no gain.

`workflow.directory` wins on the strength of the collision argument the file-naming code
already makes at `install.rs:132`. It also means a workflow has *one* generated identity
rather than two, which is the property that makes the new test a single rule rather than two.

The cost is honest: `htmlspec-apply-change` is longer than `/htmlspec:apply` and reads less
like something you type. That is correct, because it is not something you type. The thing you
type is printed by the closing message, from `Tool::invocation`, which is where it belongs.

### `Tool::invocation` keeps exactly one caller

After this change, `tool.invocation` is called from `report_next_command` and from its own
test. That is the point rather than a side effect.

The invocation is a fact about a tool's command surface — `/htmlspec:{}` for one,
`$htmlspec-{}` for another — and its consumer is a human reading the closing message. Two
consumers meant two things could disagree about the spelling; more importantly, one of them
was writing it into a field where it is not a spelling of anything. This is the same
reasoning the `entry_point` flag rests on: one fact, one producer, and a test on the shape.

`Tool::invocation` is not removed and its `SingleFile` branch is untouched. A shared-file tool
gets no command file, so it was never reached by the frontmatter writer anyway.

### The check walks generated output, not the table

The test installs for every tool into a scratch directory and reads every file that lands,
asserting each `name:` matches `^[a-z0-9-]+$`.

Asserting over the table instead — "every `workflow.directory` is kebab-case" — is cheaper and
weaker. It would not have caught the actual defect, because the defect is not in the table:
`workflow.name` and `workflow.directory` are both fine, and the malformed value is produced by
a format string at the moment of writing. A check on the inputs cannot see a fault introduced
by the writer.

So the check reads what a contributor's project would receive. It is the same choice
`every_tool_receives_at_least_one_file` already makes — asserted against real files rather
than against the table's shape — and for the same reason.

**The character set is `^[a-z0-9-]+$`**, taken from the one warning text available ("must be
lowercase a-z, 0-9, hyphens only"). Five tools load these files and this program has one
tool's rule in evidence. Conforming to the strictest observed rule is safe in the direction
that matters: a name legal under a stricter rule is legal under a looser one, and the reverse
is what produces a warning on somebody else's machine.

### The frontmatter contract is stated once, for both file kinds

The delta puts the obligation on generated instructions generally rather than on commands
specifically. The command writer is where the fault is today; the skill writer is one
underscore in a table entry away from the same fault, and it currently conforms by accident.
A rule that named only commands would leave that accident undocumented and unprotected.

## Risks

- **A tool might key off the command's `name:` field for its invocation.** If some loader
  reads `name:` to decide what `/`-command to register, changing it to
  `htmlspec-apply-change` would change or break the spelling. Evidence says otherwise: the
  vendored `opsx` commands declare `name: "OPSX: Apply"` and register as `/opsx:apply`, which
  is neither the name nor derivable from it — the file path is what determines the command.
  Task 4 verifies this by loading the regenerated files rather than by reasoning about it,
  because it is the one risk that would make the change wrong rather than merely different.

- **`^[a-z0-9-]+$` may be stricter than four of the five tools require.** Accepted. The
  failure direction is a name that is legal everywhere instead of legal in four places.

- **The test is slower than a table assertion**, since it performs a real install per tool
  into a scratch directory. The install tests already do this, so the cost is known and
  small.

## Migration

Re-running `htmlspec init` refreshes every command file, which the contract already documents
as safe and idempotent. The `generatedBy` line makes a project still carrying the old
frontmatter identifiable.

Nothing a contributor types changes. `toy_project/` and this repository both carry the old
form and are regenerated by the same command.
