# Both checks observed failing before the repair

Task 5.3. A test that has never failed is a test whose subject is unproven, so each was run
against the unrepaired tree first.

## The table-level check

`every_workflow_declares_an_optional_argument` in
`crates/htmlspec-assets/src/instructions.rs`:

```
test instructions::tests::every_workflow_declares_an_optional_argument ... FAILED

explore's body never substitutes $ARGUMENTS, so an argument reaches no agent
```

It fails on the first workflow in lifecycle order, which is what a per-workflow assertion
should do — the point is that no body carried the token, and naming one is enough to stop
the run.

## The file-level check

`every_invocable_file_declares_and_consumes_an_argument` in
`crates/htmlspec-cli/src/install.rs`:

```
test install::tests::every_invocable_file_declares_and_consumes_an_argument ... FAILED
```

It reports **49 missing declarations and 49 missing substitutions** — one pair per invocable
file, which is 7 workflows across the 4 tools with a command surface, counted as skills plus
commands where a tool receives both:

```
Claude Code   7 skills + 7 commands = 14
Codex         7 skills              =  7
Cursor        7 skills + 7 commands = 14
Pi            7 skills + 7 commands = 14
                                      49
```

A sample of what it names:

```
.../.pi/prompts/htmlspec-new.md: declares no `argument-hint`, so a tool has nothing to
  substitute and what a contributor typed is dropped
.../.pi/prompts/htmlspec-new.md: body never substitutes $ARGUMENTS, so an argument a tool
  passes reaches no agent
```

Both halves are reported for every file, which is the arrangement the design asked for: the
frontmatter that declares an argument and the body that consumes it are separate omissions,
and a file with one and not the other is still broken.

`AGENTS.md` appears nowhere in the output. That is by construction rather than by an
exclusion in the test — `Installed.regions` is a separate field from `skills` and
`commands`, so iterating the latter two cannot reach it (task 5.4).

## Why this failure is worth recording rather than assumed

The fault was live in a released version. 0.3.0 shipped with all 49 files missing both
halves and a fully green suite, because nothing read the invocation surface. The count is
the useful part: it shows the defect was uniform across every workflow and every tool, not
a gap in the one workflow that happened to be under discussion when it was found.
