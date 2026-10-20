## Context

`htmlspec init` writes seven skill files and seven command files for each tool with
`Delivery::SkillsAndCommands` — Claude Code, Cursor, and Pi. Both writers put a `name:` key
in the frontmatter, and since `4972234` both take it from `workflow.directory`. The values
are therefore equal, per workflow, per tool: 21 command files declaring a name their sibling
skill file already declares.

A loader that reads both directories sees each workflow twice. The observed symptom is a
doubled listing — fourteen entries for seven workflows on a Pi session load — not a refusal.

The previous change reached this state deliberately. It needed a well-formed replacement for
`tool.invocation(workflow.name)`, considered `workflow.name` and `workflow.directory`, and
chose the directory partly *because* it matched the skill file, recording that as "one
identity per workflow" and writing it into the contract as a scenario. The reasoning was
sound about workflows and wrong about the field: two files agreeing on which workflow they
belong to is good, and two files claiming the same registry key is a collision. The contract
now asserts the collision, which is why the existing test suite is green.

## Goals

- A tool never receives two generated files declaring the same name.
- The contract states distinctness rather than sameness, so the mistake is not re-derivable
  from it.
- A test that fails on a repeat, since the character check cannot see one.

## Non-Goals

- Changing what a contributor types. Unchanged, and for the reason the previous change
  verified by loading: the file path determines the invocation.
- Reverting the previous change. Its test and its finding both stand.

## Decisions

### A command file declares no name

Three ways to break the tie:

**Give the command a distinct name** — `htmlspec-apply-command`, say. This keeps a field
whose only job would be to differ from another field. It also has to differ *per tool*, since
each tool's command directory is its own namespace, and inventing a spelling for a key
nothing reads is how the invocation ended up in there the first time.

**Drop `name:` from command files.** Ten `.pi/prompts/opsx-*.md` files in this repository
omit it and load. The previous change already established that the file path determines the
invocation — it verified this by regenerating and loading, and by the `opsx` comparison where
`name: "OPSX: Apply"` registers as `/opsx:apply`. A field that does not determine the
invocation, that a working installation omits, and whose presence causes a collision has no
remaining argument for existing.

**Stop writing command files for tools that also get skills.** Rejected outright: it changes
what a contributor types, which every version of this work has held fixed.

Taking the second. The command file keeps `description`, `allowed-tools`, and `generatedBy` —
`description` is what a tool displays, and `generatedBy` is required by an accepted
requirement about recording the generating version.

One consequence to state plainly: `.claude/commands/opsx/*.md` **do** carry a name, so
omitting it is not universal practice. It is permitted practice, which is the weaker claim
this decision needs. The strictly safer option is available if a loader turns out to want the
field — give it a per-tool distinct value — and the test added here would keep that honest.

### The scenario is withdrawn, not reworded

"One identity per workflow" is satisfiable only by the behavior being removed, so editing its
wording would leave a scenario that no longer describes anything the program does. It is
withdrawn and a distinctness scenario added.

This is the first withdrawal in this capability. The archive workflow names it as the one case
where deleting from an accepted spec is correct, and requires it be deliberate rather than a
side effect of rewriting a file — hence a task that names it and no other change to the
surrounding requirement.

The requirement's prose keeps its claim that a workflow has one generated identity, because
that remains true and is the useful idea. What changes is the sentence asserting the same
identity is written into every file kind.

## Risks

**A loader may require `name:` in a command file.** Mitigated by the `opsx` Pi prompts, which
omit it across ten files and load. Unverified for Cursor, whose command loader was not
tested here — this is the same inference-from-one-observation the previous change flagged in
its own non-goals, and it is recorded rather than resolved. If Cursor turns out to require
the field, the fallback is a distinct per-tool name, and the distinctness test added by this
change is what would keep that from re-colliding.

**The doubled listing may be cosmetic.** Nothing observed proves a loader *rejects* the
duplicate rather than listing it twice. The change is still worth making — a name field
nothing reads, duplicating a sibling's key, is not defensible on the grounds that the harness
tolerates it. That is the argument the previous change made about `/htmlspec:apply` working
under one loader, and it applies here unchanged.
