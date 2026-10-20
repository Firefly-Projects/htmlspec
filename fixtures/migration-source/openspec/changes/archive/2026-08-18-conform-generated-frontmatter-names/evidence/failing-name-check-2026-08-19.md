# The check fails before the fix

Task 1.3 and 1.4. `every_generated_name_is_loadable` added in
`crates/htmlspec-cli/src/install.rs`, run against the tree **before** `write_commands` was
changed. This is the only naturally occurring failing case, so it is the one chance to see
the check work.

```
cargo test -p htmlspec-cli every_generated_name_is_loadable
```

```
test install::tests::every_generated_name_is_loadable ... FAILED

generated names a tool may reject:
…/.claude/commands/htmlspec/explore.md: name `/htmlspec:explore` carries `/`, which is outside lowercase a-z, 0-9, and hyphens
…/.claude/commands/htmlspec/new.md: name `/htmlspec:new` carries `/`, …
…/.claude/commands/htmlspec/continue.md: name `/htmlspec:continue` carries `/`, …
…/.claude/commands/htmlspec/ff.md: name `/htmlspec:ff` carries `/`, …
…/.claude/commands/htmlspec/apply.md: name `/htmlspec:apply` carries `/`, …
…/.claude/commands/htmlspec/verify.md: name `/htmlspec:verify` carries `/`, …
…/.claude/commands/htmlspec/archive.md: name `/htmlspec:archive` carries `/`, …
  -> Claude Code has 7 malformed file(s) of 14
…/.cursor/commands/htmlspec-{explore,new,continue,ff,apply,verify,archive}.md: name `/htmlspec-…` carries `/`, …
  -> Cursor has 7 malformed file(s) of 14
…/.pi/prompts/htmlspec-{explore,new,continue,ff,apply,verify,archive}.md: name `/htmlspec-…` carries `/`, …
  -> Pi has 7 malformed file(s) of 14

test result: FAILED. 0 passed; 1 failed
```

## Count per tool

| Tool | Files read | Malformed | Why |
| --- | --- | --- | --- |
| Claude Code | 14 | 7 | 7 command files, each `name: /htmlspec:<workflow>` |
| Codex | 7 | 0 | skills only — `$htmlspec-…` never reaches a file |
| Cursor | 14 | 7 | 7 command files, each `name: /htmlspec-<workflow>` |
| Pi | 14 | 7 | 7 command files, each `name: /htmlspec-<workflow>` |
| AGENTS.md | 1 | 0 | a marked region, no frontmatter |

21 failing files of 35 written, and **7 per tool rather than 1** — which is what shows the
check reads every file an install produces rather than stopping at the first.

The seven skill files per tool pass, from `workflow.directory`. They passed by coincidence
before this test existed; now the coincidence is asserted.

Codex's `$htmlspec-…` carries a `$` and would fail the same rule. It never does, because a
skills-only tool receives no command file — the malformed value was only ever written where
`Delivery::SkillsAndCommands` reaches. The rule now holds for both writers, so a tool
promoted to commands later cannot reintroduce it.
