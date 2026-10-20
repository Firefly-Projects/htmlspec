# The amended guidance, run against a described change

Task 5.2. The recorded request was put through `/htmlspec-new` in
`~/projects/toy_project`, a fresh project with empty `htmlspec/specs/` and
`htmlspec/changes/`, installed from this tree.

The request:

```
/htmlspec-new I want to start a new app that is a tally mark creator.
It should have 2 tabs the first is one where you can add tally boxes that you can name
and click on, each is tied to a counter.
The second is a tab with stats.
```

No change was created. `htmlspec/changes/` holds `archive/` and nothing else.

## The amended guidance was installed and was read

The toy project carries the new text, not the old:

```
grep -c "a need takes one of two forms" .claude/commands/htmlspec/new.md          # 1
grep -c "a need takes one of two forms" .claude/skills/htmlspec-new-change/SKILL.md  # 1
```

Every amendment is visible in the agent's response, which is why this run is evidence
that the prose repair landed rather than evidence that it failed:

- The absence form, named as such: "the need's second form, since a new project has no
  measurement or error message to quote". Section 2's amendment, applied.
- The refusal to invent: "I won't invent one: a proposal whose need I made up is exactly
  the document archival later can't check anything against". Task 4.5 and the second
  Non-Goal, preserved.
- The round trip disclaimed: "without asking you to approve the name or restate scope
  you'd already given". Tasks 3.4 and 4.1, applied.
- `explore` offered as a fallback the contributor may choose, not as the routing an
  absence of capability gets sent down. Task 4.2, applied.

The agent also volunteered the three questions the absence form asks — who wants it and
what they cannot do now, what working looks like, what is excluded — and marked them
optional rather than required. That is section 2 working as written.

## Why it refused anyway: the description never arrived

The agent's own account of its input:

> Your message is the new-change workflow instructions themselves — the stance, the
> document order, the rules for each kind — but it carries no idea to write about.

It received the instruction body and nothing else. The generated command file has no
argument placeholder:

```
grep -c 'ARGUMENTS' .claude/commands/htmlspec/new.md                    # 0
grep -c 'ARGUMENTS' .claude/skills/htmlspec-new-change/SKILL.md         # 0
```

`write_commands` in `crates/htmlspec-cli/src/install.rs:124` emits `description`,
`allowed-tools`, and `generatedBy`. It emits no `argument-hint`, and no instruction body
contains `$ARGUMENTS`. So a description typed after `/htmlspec-new` is discarded before
an agent sees it.

Given only instructions and no description, asking is the behavior tasks 4.5 and the
second Non-Goal deliberately kept. The agent was correct on the inputs it actually had.
It did the right thing with the wrong input.

## What this does and does not establish

**Established:** the guidance no longer refuses a described change on the grounds that an
absence of capability is not a need. The reasoning that produced the original refusal is
gone, and the response reaches the opposite conclusion on every point section 2 and
section 3 amended.

**Not established:** that an agent derives a name from a real description. The
`add-tally-counters` name in the response is quoted from the worked example the amended
`new.md` itself ships, not derived from the tally-mark text. Derivation from a
contributor's own description remains untested, because that description has not once
reached an agent.

This is why task 5.2 is dropped rather than checked. Its subject — what an agent does on
the first exchange with a described change — is unobservable through the command surface
until the surface passes the description through.

## The defect this surfaced

`/htmlspec-new <description>` discards the description. That is the same outcome this
change was written to repair — a contributor states the work and is asked for it again —
reached by a different mechanism. The original fault was prose that overrode a correct
inference. This one is codegen that withholds the input the inference needs.

No amendment to the instruction bodies can fix it, which is the reason the guidance
reading correctly was weaker evidence than it looked. It belongs to
`project-bootstrap`, which owns the structure of generated files
(`openspec/specs/project-bootstrap/spec.md:137`), and unlike the prose repair it has a
testable surface in `install.rs`. Filed as its own change.
