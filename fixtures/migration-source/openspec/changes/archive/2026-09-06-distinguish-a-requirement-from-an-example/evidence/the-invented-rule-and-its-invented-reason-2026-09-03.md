# The invented rule, and its invented reason

Tasks 1.1 and 1.2, recorded 2026-09-03 against `main` at `80558b5`.

The complaint is the finding. An agent asserted a rule the project does not state, wrote a
justification beside it, and every check passed.

## The assertion

An earlier draft of the migration change's mapping table mapped `specs/<cap>/spec.md` to
`htmlspec/specs/<cap>.html`. The change's own design records the row and the reason attached to
it, at `openspec/changes/migrate-an-openspec-project/design.md:183`:

> An earlier draft of the table above mapped `specs/<cap>/spec.md` to `htmlspec/specs/<cap>.html`
> and justified it: a directory holding one file is a directory OpenSpec needs and HTMLSpec does
> not. Both halves were wrong, and the way they were wrong is the reason this decision exists.

The mapping table is the load-bearing detail. A destination column asserts where an artifact
lives, which is a claim about what the project requires, so every row owed either a requirement
or a statement of silence. The row supplied a rule instead.

## Where the rule came from

`README.md`, line 5:

```
# HTMLSpec

> Spec-driven development for AI coding agents, reviewed in a browser.

**Status:** early design proposal. The name, commands, and file layout are working ideas.
```

The status line is four lines from the top of the file and says, in its own words, that the file
layout is a working idea. The flat-file spelling appears below it. So the agent read a document
that disclaims its own authority on exactly the point it was consulted for.

**Nothing in the accepted contract states a layout beneath `htmlspec/specs/`.** Checked at
`80558b5`: `project-bootstrap` says the project directory holds its accepted specifications, its
active changes, and its archive, "each in its own place", and constrains nothing below that.
`ProjectRoot::specs()` resolves `htmlspec/specs` and fixes nothing under it. `archive.md`
directs an agent at `<accepted-specs-dir>` without naming a file.

## The second fault: the reason was false about the source

This is task 1.2, and it is the argument for stopping at the invention rather than at the error.

`<capability-path>` is a path, not a wrapper. The installed workflow guidance gives
`identity/user-auth` as an example and directs an agent to preserve an existing capability's
full path. So flattening `specs/<cap>/spec.md` to `htmlspec/specs/<cap>.html` either discards
hierarchy the source expressed, or encodes it as `identity-user-auth.html` — a naming convention
invented during a migration and enforced by nothing.

**One invention produced two errors.** The rule was unsupported *and* the reason for it was
wrong about the format it described. A reviewer who caught only the flattening would have fixed
the destination column and left the invention intact, because the invention is a habit rather
than a typo. That is why the repair is stated as an obligation about assertions rather than as a
correction to a table.

## Why nothing caught it

The report that carried this passed every check. A cited example and a cited requirement produce
identical artifacts:

- The document validates. Nothing in the rule set reads a justification.
- The claim reads as authoritative, because it names a destination and gives a reason.
- The reason makes it worse rather than better. A rule stated bare invites a question. A rule
  with reasoning attached reads as a decision the project already took, and nobody questions a
  settled decision.

That asymmetry is what the contract delta states as "an invented justification is worse than an
invented rule stated bare", and it is the specific shape of this case rather than a general
observation about confidence.

## What the migration change now does

Recorded because the case is repaired at its origin and this evidence should not read as an open
defect. `migrate-an-openspec-project`'s design now carries a decision titled *Where the
destination's contract is silent, the source's shape is preserved*, which takes the preserving
route and records the silence. The layout question itself stays open, and this change
deliberately does not settle it.
