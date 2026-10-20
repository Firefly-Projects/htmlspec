# The installed instruction files drifted from 0.6.0, and nothing compared them

**Date:** 2026-09-06
**Subject:** the two findings in the review of PR #21, both confirmed
**Method:** every claim below read out of git history against the workspace version at each
commit, and the new check proven against each historical failure mode

## What I recorded, and what was true

I wrote that the installed files were **stale since 0.7.0**. They were stale since **0.6.0**.
The review was right and the correction matters here more than it usually would, because this
release's whole subject is whether an asserted claim is one the record supports.

Regeneration was complete through 0.5.0 — 35 installed files updated each time a body changed.
Then:

| commit | version | bodies changed | installed files updated |
| --- | --- | --- | --- |
| `bfc3835` | 0.5.0 | yes | 35 |
| `cf11132` | 0.6.0 | `archive.md`, +88 | **2 of 5** |
| `ee4301c` | 0.7.0 | yes, and added `sync` | **0** |
| `9af58cf` | 0.8.0 | yes | **0** |
| `80558b5` | 0.9.0 | yes | **0** |
| `06b3508` | 0.9.0 | yes | **0** |

So `sync` shipped in 0.7.0 and existed in the binary and in no file an agent could read, for
three releases. My 0.7.0 figure took the first release that regenerated *nothing* as the start
of the drift, and missed that 0.6.0 had already regenerated only part.

## The part that is worse than staleness

The two copies `cf11132` did update were **hand-edited, not regenerated**. After that commit
both still read:

```
generatedBy: htmlspec 0.5.0
```

while `Cargo.toml` said `0.6.0`. The stamp is written from `CARGO_PKG_VERSION` at run time
(`crates/htmlspec-cli/src/install.rs`), so a real `init` run on a 0.6.0 build could not have
printed 0.5.0. Somebody edited the generated file directly and left the stamp behind.

That is the failure mode a generated file is supposed to make impossible, and the stamp is
what caught it — three releases later, by a reviewer reading history rather than by any check.

## The second finding: no mechanical check, in a change about that exact shape

Task 5.1 asked me to *confirm by hand* that the installed bodies are what an agent reads. I did,
and it passed. But no stage of `scripts/check` compares an installed copy to the compiled body,
so the confirmation expires the moment the next body changes. `check-release` reads only the
archive and the changelog.

**This is the shape this change is about, applied to this project's own files.**
`project-bootstrap` already states it — a rule nothing checks is not a rule — and the 0.9.0
changelog names two prior instances: a gate that validated an invented convention across eleven
archives without refusing anything, and an instruction telling a contributor to apply a
change's contract with no operation that did it. This is the third, and it sat in the files this
project asks other projects to trust.

Five commits changed a body and regenerated nothing. A check would have caught every one.

## `scripts/check-installed`

It runs `init` into a scratch directory and diffs against the committed files.

**Not a body-by-body string comparison.** That would require the script to know how each tool's
file is assembled — frontmatter, field order, name prefixes — and that knowledge belongs to
`install.rs`. A second copy of it produces a check that passes while the real writer does
something else, which is the defect in miniature.

Proven against all three failure modes the history actually contains:

| scenario | result |
| --- | --- |
| a body edited in source, installed files not regenerated | FAIL, naming all 5 copies |
| an installed file absent (the `sync` case) | FAIL, naming the file |
| an installed file hand-edited (the `cf11132` case) | FAIL, naming the file |
| the tree as committed | ok, 40 files |

**A bug in my first version, caught by testing rather than by reading.** I filtered every
directory's entries by an `htmlspec` prefix, to skip the `openspec-*` skills that share
`.claude/skills`. But `.claude/commands/htmlspec/` is namespaced by its own path, so the files
inside are bare names — `apply.md`, `verify.md` — and the filter excluded all eight. The count
read 32 when it should have read 40, and drift scenario A reported four files instead of five.
`INSTALLED` is now a table of (path, shared) so each directory states whether it holds another
tool's entries.

## The flake

`scripts/check` runs the others as programs, so the sandbox must resolve each shebang.
`flake.nix` names them explicitly and its own comment says why: an unpatched one "took CI red
for five merges with `/usr/bin/env: bad interpreter`", and "a script added to the suite must be
added here in the same change."

I hit that comment's exact case. `nix flake check --no-build` evaluated clean — evaluation only
proves the flake parses — and `nix build` then failed:

```
=== installed ===
./scripts/check: line 126: scripts/check-installed: No such file or directory
FAILED: installed
```

Two causes, both mine: the script was absent from `patchShebangs`, and it was untracked, so
`srcFilter` left it out of the snapshot. Staging it and adding it to the list fixed both.
`nix build .#checks.aarch64-darwin.suite` now reports all 10 stages and 40 files, which is the
gate CI runs.

## After both findings

- `./scripts/check` — 10 stages, up from 9
- `nix build .#checks.aarch64-darwin.suite` — passes, `installed` included
- `openspec validate --all` — 11 of 11
