# Where htmlspec diverges from OpenSpec's bootstrap, and why

Parity with `openspec init` is a stated goal of the `project-bootstrap-cli`
change, so every difference is a decision rather than an oversight. This file
records the four that exist, each with the measurement behind it. Two we closed
in `prompt-selection-guards`, one we keep deliberately, and one is deferred.

Measured 2026-08-17 against `openspec` 1.7.0 at
`/opt/homebrew/lib/node_modules/@fission-ai/openspec/dist`, and against
`htmlspec` built from `prompt-selection-guards`.

## Summary

| Situation | OpenSpec | htmlspec | Status |
| --- | --- | --- | --- |
| Tool directories present | Preselects them | Preselects them | Same |
| Nothing detected | Preselects nothing | **Preselects everything** | Diverged on purpose |
| Empty confirmation | Refuses, asks again | Refuses, asks again | Same, adopted |
| No terminal, nothing detected | Errors and exits | Advises and proceeds | Diverged on purpose |
| Re-run adopting a tool | Cannot, tracks configured | Can, tracks detection only | Deferred gap |

## 1. Preselection of detected tools: the same

An earlier note in this change claimed OpenSpec never preselects. That claim was
wrong, and it came from testing OpenSpec only in an empty repository, where
nothing exists to preselect. With tool directories present it preselects them:

```
$ mkdir .claude .codex && openspec init
Selected:  Claude Code   Codex        <- both [x]
```

We do the same, so there is no divergence here.

## 2. Nothing detected: we preselect everything, OpenSpec preselects nothing

`init.js:303`:

```js
const shouldPreselectDetected = !extendMode && configuredToolIds.size === 0;
```

A tool preselects only when detection found it. An empty detection therefore
opens a list of 34 unchecked rows.

Ours opens with every row checked. `preselected` in
`crates/htmlspec-cli/src/prompt.rs` returns every index when detection is empty.

**Why we diverge.** An empty detection means a project nobody has configured,
which is exactly the first run. That is where a default carries the most weight
and where detection has the least to say. A prompt offering nothing at that
moment teaches that the prompt is decorative, and unchecking two rows is less
work than checking three.

Scale is part of the argument, and it cuts differently for each program.
OpenSpec has 34 tools, so preselecting all of them would install 34 tool
directories for a contributor who pressed Enter. We have 5. Preselecting
everything is defensible at 5 and indefensible at 34, so this divergence is
contingent on our table staying small. **If `TOOLS` grows past roughly a dozen
entries, revisit this.**

## 3. The empty confirmation: adopted from OpenSpec

`init.js:361`, with a `throw` at `:365` behind it:

```js
validate: (selected) => selected.length > 0 || 'Select at least one tool',
```

Measured before we adopted it:

```
$ openspec init            # Enter on an empty list
Selected: (none selected)
Select at least one tool
$ ls -a                    # nothing written

$ htmlspec init            # Enter on an empty list, old behavior
the project is ready.
$ ls -a                    # htmlspec/ and spec.nix, zero instructions
```

Ours reported success for a bootstrap that installed nothing. This is the defect
that produced the Pi report: the project *was* established correctly, which is
what made the outcome hard to see.

We adopted the refusal in `prompt-selection-guards`, with one difference in the
message. Ours names the escape hatch:

```
select at least one tool with space, or cancel with esc and run
`htmlspec init --tools none` to establish the project without instructions
```

`--tools none` still succeeds. The asymmetry is the point: an argument is a
contributor answering the question, and an empty Enter is usually a contributor
who did not notice a question was asked.

## 4. No terminal with nothing detected: we proceed, OpenSpec errors

`init.js:310`. Measured:

```
$ openspec init < /dev/null
No tools detected and no --tools flag provided. Valid tools: ...
Use --tools all, --tools none, or --tools claude,cursor,...

$ htmlspec init < /dev/null
no AI coding tool directories were detected, so no instructions were installed.
Pass --tools to name them
...
the project is ready.
```

**We keep ours, and OpenSpec is wrong for our purposes.** Our accepted contract
requires that the bootstrap not block and not fail when no terminal is attached,
because a bootstrap that fails inside a continuous-integration job establishes
nothing. An error here would break `htmlspec init` in exactly the automated
context the requirement was written for.

Note that this is not inconsistent with section 3. A refusal at a prompt can ask
again, and a contributor is there to answer. A refusal with no terminal has
nobody to ask, so its only options are to fail or to proceed.

## 5. Configured against detected: a gap we have not closed

The line at `init.js:303` carries a distinction our types cannot express.
OpenSpec separates two facts:

- **configured** — OpenSpec installed into this project before.
- **detected** — the tool's directory exists.

Detected tools preselect on a first run only. On a re-run, only configured tools
preselect, so a re-run never adopts a tool the contributor declined the first
time. OpenSpec labels both states in its output, `pre-selected for first-time
setup` against `not pre-selected`.

Our `tools.rs` has one predicate, `Tool::detected`, which tests path existence.
It cannot tell the two apart. The consequence is narrow but real: a contributor
who declines Cursor on the first run, and who has a `.cursor/` directory for
unrelated reasons, is offered Cursor again on every re-run with no memory that
they said no.

**Deferred deliberately, and worth using the tool before deciding.** Closing it
means recording what we installed, which is a second kind of state in a project
directory, and that is a design decision rather than a fix. The interesting
question is whether the re-run case annoys anyone in practice. We will know that
from using `htmlspec init` on real projects, not from reasoning about it here.

## 6. One difference that is not a divergence

OpenSpec installs a skill and command per workflow across its full lifecycle.
We ship one workflow:

```
$ ls crates/htmlspec-assets/instructions/
archive.md
```

The proposal states this as a non-goal: `explore`, `propose`, `apply`, and
`verify` are deferred because archival is the one operation the program already
gates. This is a difference in scope rather than in bootstrap behavior, and the
accepted contract describes it correctly.
