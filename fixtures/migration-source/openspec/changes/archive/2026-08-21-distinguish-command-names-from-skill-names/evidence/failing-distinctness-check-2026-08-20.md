# The distinctness check fails before the fix

Tasks 1.3 through 1.5. `no_tool_receives_a_name_twice` added in
`crates/htmlspec-cli/src/install.rs`, run against the tree **before** `write_commands` was
changed. This is the naturally occurring failing case.

```
cargo test -p htmlspec-cli no_tool_receives_a_name_twice
```

```
test install::tests::no_tool_receives_a_name_twice ... FAILED

one tool would load the same name from two files:
…/.claude/commands/htmlspec/explore.md: name `htmlspec-explore` is already declared by
  …/.claude/skills/htmlspec-explore/SKILL.md
…/.claude/commands/htmlspec/new.md: name `htmlspec-new-change` is already declared by
  …/.claude/skills/htmlspec-new-change/SKILL.md
… five more …
  -> Claude Code declares 7 name(s) twice across 14 file(s)
… seven for Cursor …
  -> Cursor declares 7 name(s) twice across 14 file(s)
… seven for Pi …
  -> Pi declares 7 name(s) twice across 14 file(s)
```

## Per-tool count

| Tool | Files received | Names declared twice |
| --- | --- | --- |
| Claude Code | 14 | 7 |
| Cursor | 14 | 7 |
| Pi | 14 | 7 |
| Codex | 7 | 0 — absent from the output |
| AGENTS.md | 1 region | n/a |

21 repeats total. **Codex does not appear**, which is the check reading delivery rather than
assuming: it is `Delivery::SkillsOnly`, so it receives no command file to collide with. That
absence is what shows the fault is the second file rather than the name itself.

## The character check is blind to this

`every_generated_name_is_loadable` reports **no** malformed name on the same tree. Both
halves of every duplicate pair are `htmlspec-apply-change` and friends — well-formed under
the lowercase-digits-hyphens rule. The character check cannot see a repeat, which is why this
is a second check rather than an extension of the first.

One qualification, recorded rather than smoothed over: the same commit adds a non-vacuity
assertion to `every_generated_name_is_loadable`, so that check does *not* pass unmodified on
the pre-fix tree. Its character assertion passes — no file is reported as rejectable — and
the failure is only the new count:

```
assertion `left == right` failed: expected one declared name per workflow per tool that receives skills
  left: 49
 right: 28
```

49 is the pre-fix total: 4 tools × 7 skills, plus 3 tools × 7 commands. 28 is what remains
once command files stop declaring a name. The assertion exists because after the fix an empty
sweep would look identical to a clean one, and it is calibrated to the post-fix tree by
design.
