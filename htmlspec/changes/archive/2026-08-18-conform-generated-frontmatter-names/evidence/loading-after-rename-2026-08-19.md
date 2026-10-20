# Renaming the field does not change what a contributor types

Tasks 4.1 through 4.4. Run against this repository after the `write_commands` change, with
`htmlspec 0.1.0` built from this tree.

## The regeneration

```
./target/debug/htmlspec init --tools claude,codex,pi
  Claude Code: 14 file(s) in .claude
  Codex:        7 file(s) in .codex
  Pi:          14 file(s) in .pi
```

`git status` reports **14 modified files**, all of them command files:

```
 M .claude/commands/htmlspec/{apply,archive,continue,explore,ff,new,verify}.md
 M .pi/prompts/htmlspec-{apply,archive,continue,explore,ff,new,verify}.md
```

Each diff is one line:

```diff
-name: /htmlspec:apply
+name: htmlspec-apply-change
```

```diff
-name: /htmlspec-new
+name: htmlspec-new-change
```

**Correcting task 4.1's arithmetic:** it says 21 command files. 21 is the all-tools figure
(Claude Code 7 + Cursor 7 + Pi 7). This repository selects `claude,codex,pi`, and Codex is
skills-only, so **14** command files are regenerated here. The 21 figure is confirmed
separately by `every_generated_name_is_loadable`, which installs for every tool in `TOOLS`
and reads all 35 files.

Not modified: the 21 skill files across three tools. Their `name:` already came from
`workflow.directory` and is byte-identical.

## The seven `/htmlspec:*` commands still load, by their old spelling

This is the one risk that would make the change wrong rather than merely different, so it is
checked by loading rather than by reasoning.

**Claude Code 2.1.220**, asked to enumerate its own registered commands:

```
claude -p "List every slash command whose name begins with htmlspec, one per line, nothing else." --model haiku
  /htmlspec-apply-change      ← the skills, unchanged by this change
  /htmlspec-archive-change
  /htmlspec-continue-change
  /htmlspec-explore
  /htmlspec-ff-change
  /htmlspec-new-change
  /htmlspec-verify-change
  /htmlspec:apply             ← the commands, still their old spelling
  /htmlspec:archive
  /htmlspec:continue
  /htmlspec:explore
  /htmlspec:ff
  /htmlspec:new
  /htmlspec:verify
```

All seven `/htmlspec:*` commands register, from files now declaring
`name: htmlspec-*-change`. The spelling comes from `.claude/commands/htmlspec/<workflow>.md`.

**Pi**, through its own loader (`loadPromptTemplates` from
`@earendil-works/pi-coding-agent`, the same function the agent calls at startup):

```
/htmlspec-apply     <- .pi/prompts/htmlspec-apply.md
/htmlspec-archive   <- .pi/prompts/htmlspec-archive.md
/htmlspec-continue  <- .pi/prompts/htmlspec-continue.md
/htmlspec-explore   <- .pi/prompts/htmlspec-explore.md
/htmlspec-ff        <- .pi/prompts/htmlspec-ff.md
/htmlspec-new       <- .pi/prompts/htmlspec-new.md
/htmlspec-verify    <- .pi/prompts/htmlspec-verify.md
```

Pi's `PromptTemplate.name` is populated from the file stem; the `name:` key is not read at
all. Its documentation states it outright: "The filename becomes the command name."

## The `opsx` comparison

The reason to expect this, before the load proved it. The vendored `opsx` commands in this
repository declare:

```
.claude/commands/opsx/apply.md   name: "OPSX: Apply"     → registers as /opsx:apply
.pi/prompts/opsx-apply.md        (no name: key at all)   → registers as /opsx-apply
```

`/opsx:apply` is neither the declared name nor derivable from it — `"OPSX: Apply"` carries a
space and a capital. And pi's copy omits the key entirely and still registers. Both point the
same way: the file path determines the command. The load above is the proof; this is why it
was the expected result.

## The closing message is unchanged

```
Start a change with your agent:
  /htmlspec:new  (Claude Code)
  $htmlspec-new  (Codex)
  /htmlspec-new  (Pi)
```

Three distinct spellings, grouped per spelling, from `Tool::invocation` in
`report_next_command` — which after this change is its only non-test caller. With `--tools all`
in a scratch project the fourth line appears: `ask your assistant to run the new workflow
(AGENTS.md)`.

## AGENTS.md

`md5` before the regeneration and after: `591c6fc316bd89ffd645223aed6a481c`, unchanged.
The marked region carries no frontmatter, so this change does not reach it, and `git status`
does not list the file.

## Stray output removed

`init` also created `htmlspec/` and `spec.nix` in this repository, which uses `openspec/` as
its planning home. Both were untracked and were deleted; they are artifacts of running `init`
on an already-established project, not of this change.
