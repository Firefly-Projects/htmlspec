# The first-run experience, after the change

Recorded 2026-08-14 from `target/debug/htmlspec`, in an empty repository. Verbatim.

Compare against `evidence/current-cli-behavior.md`, which recorded the same
sequence before this change: bare `init` failed with *"the following required
arguments were not provided: <PATH>"*, and there was no invocation that meant
"set this repository up".

## The whole sequence

```
$ git init && htmlspec init --tools claude
/private/tmp/finalrun/htmlspec: created the project structure
established 8 shared asset(s) in /private/tmp/finalrun/htmlspec/assets
/private/tmp/finalrun/spec.nix: declared the project at version 0.0.0
Claude Code: 2 file(s) in .claude

the project is ready.
  Create your first change:  htmlspec new change <name>
  Then open its proposal in a browser and tell your agent what to change.

When the work is done, archive it:
  /htmlspec:archive

$ htmlspec new change add-dark-mode
.../htmlspec/changes/add-dark-mode/proposal.html: created a proposal
.../htmlspec/changes/add-dark-mode/design.html: created a design
.../htmlspec/changes/add-dark-mode/tasks.html: created a tasks
.../htmlspec/changes/add-dark-mode: created a change with 3 document(s). The
specification is produced at archival, from the completed change.

$ htmlspec validate

3 document(s): 0 error(s), 0 warning(s)
```

Every command the output names is a command that exists and works. That is the
whole of what this change was for.

## The interactive selection

Driven through a pseudo-terminal in a directory containing `.claude/` and `.pi/`:

```
? Which tools should htmlspec set up?
> [x] Claude Code (detected)
  [x] Pi (detected)
  [ ] Codex
  [ ] Cursor
  [ ] AGENTS.md
[space toggles, enter confirms, esc cancels. An empty selection is fine — the
project is still established.]
```

Detected tools sort first and arrive pre-checked. Escaping:

```
? Which tools should htmlspec set up? <canceled>
cancelled; nothing was written. Run `htmlspec init --tools none` to establish
the project without installing instructions
EXIT=3
```

Nothing was written — `ls htmlspec` reported no such directory. Exit 3 is the
new `Outcome::Cancelled`, reached through the ordinary return path. OpenSpec's
equivalent calls `process.exit(0)` inside the prompt, which reports success for
an install that did nothing.

## Per-tool command spellings

With Claude Code and Codex both selected, the closing message prints one line
per distinct spelling, labelled:

```
When the work is done, archive it:
  /htmlspec:archive  (Claude Code)
  $htmlspec-archive  (Codex)
```

Codex correctly received skills and no command file — it has no command surface,
which is what the `SkillsOnly` delivery variant exists for.

## Non-interactive behavior

```
$ htmlspec init            # no terminal attached (Command::output)
detected: Claude Code
... completes, does not block
```

Asserted against a wall clock in `init_does_not_block_when_no_terminal_is_attached`
rather than trusting the interactivity check, because a prompt appearing here
would hang a continuous-integration run forever.

## The withdrawn spelling

```
$ htmlspec init doc.html
htmlspec: init no longer takes a path: its subject is the project, not a
document. To create one document, run `htmlspec new document proposal
doc.html`. To create a change, run `htmlspec new change <name>`. To set up this
project, run `htmlspec init` with no arguments
EXIT=2
```

The argument still parses so the program can name its replacement. Rejecting it
at the parser would produce a bare usage line, and a reader who used the old
spelling yesterday needs to know where document creation went.

## Re-running is safe

```
$ sed -i '' 's/0.0.0/0.2.0/' spec.nix   # a hand-edited version
$ htmlspec init --tools claude
... the project is ready.
$ grep version spec.nix
  version = "0.2.0";
```

The edit survived. Skills are refreshed, and `specs/`, `changes/`, and
`archive/` are untouched.

## The packaged artifact

Built with `nix build`, then run with no source checkout:

```
$ ./result/bin/htmlspec --version
htmlspec 0.1.0 (unknown)

$ cd /tmp/pkgtest && git init -q . && .../result/bin/htmlspec init --tools none
the project is ready. ...

$ .../result/bin/htmlspec archive x
htmlspec: the declared version is still 0.0.0, which is the last archived
version. ...
```

The version gate reads `spec.nix` from the packaged binary **with no Nix present
at runtime**. This is what makes the deviation recorded in the design necessary:
task 8.2 called for evaluating the declaration through Nix, and doing so would
have broken `archive` on exactly the machines the standalone-artifact
requirement exists to serve.

## The check suite

```
$ ./scripts/check
=== format === lint === test === doctest === self-conformance ===
=== self-format === self-assets ===
all 7 checks passed
EXIT=0
```
