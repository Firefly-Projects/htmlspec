# The guard failed open: three bypasses, one root cause

The runbook guard shipped in `d8523bb` could be walked past. Three ways, all measured, all
destroying the runbook. **Two were found by review; the third I found by probing my own fix, and
it was live in the first build of this repair.**

## The root cause, stated once

`refuse_a_runbook` resolved its own project **from the argument directory** and returned
`Ok(())` — "not a runbook, proceed" — on every condition it could not interpret.

`ProjectRoot::resolve` ascends and returns the first ancestor holding a child `htmlspec/`,
*including the directory itself*, and stops at a `.git`. So the argument got to choose which
project the guard judged it against. Every bypass below is that one sentence.

**This is the same root cause as the `..` defect in the previous round**, which the earlier
evidence recorded as a separate mistake: the guard trusted a path it derived rather than the
project it was given. Twice is a pattern, and it is why the project is now a parameter.

## The three bypasses

| # | Condition | Mechanism | Result on `d8523bb` |
| --- | --- | --- | --- |
| A | `htmlspec/` inside the runbook | runbook resolves as its own project, so `runbooks()` is `<runbook>/htmlspec/runbooks`, which does not exist → `canonicalize` fails → `Ok(())` | `archived to …/archive/2026-09-18-release`, `runbooks/` empty |
| B | `.git` inside the runbook | `resolve` hits its repository-boundary stop, finds no project → `else` arm → `Ok(())` | same |
| C | broken symlink at `htmlspec/runbooks` | `Path::exists` **follows symlinks** and reports `false` for a broken one → read as "no runbooks directory" → `Ok(())` | archived a change past a check it could not make |

Neither A nor B is contrived. A runbook documenting how to bootstrap a project has a nested
`htmlspec/`; a runbook shipping an example repository has a `.git`.

**C is mine.** It was live in the first build of this fix, because I replaced the failing
`canonicalize` with `exists()` and `exists()` follows symlinks. Found by asking what else could
make the guard answer "no runbooks here" when something is there. Fixed with
`symlink_metadata`, which does not follow, and which distinguishes `NotFound` — the one honest
"proceed" — from every other I/O error.

## The two fixes, together

**1. The project is a parameter.** Every caller has already resolved the project it is operating
on; rediscovering one from the argument asks a question with a wrong answer available. The
`--store` arm passes the store's project, which is what composed the candidate path.

This forced a reordering in `resolve_change`: the project is now resolved before either route,
where the path route previously reached a decision without one. That ordering *was* the bug.

**2. The unknowable cases refuse.** This function exists to prevent an irreversible `rename`, so
an I/O error it cannot interpret must not read as permission. Three former `Ok(())` returns are
now errors whose message says it is failing closed and why.

The one case that still proceeds is a project with no `runbooks/` at all — bootstrapped before
runbooks existed — distinguished by `ErrorKind::NotFound` rather than by a failed canonicalize.

## The wedge my first fix created

`new_change_in` used `project.change(name)` with no check. Measured:

```
$ htmlspec new change ../runbooks/sneaky
.../htmlspec/changes/../runbooks/sneaky: created a change with 3 document(s).
$ htmlspec list --runbooks   ->  sneaky  tasks
$ htmlspec list              ->  no active changes
$ htmlspec archive ../runbooks/sneaky --yes  ->  refused: is a runbook, not a change
```

A directory that lists as a runbook, is invisible to `list`, and that every change-taking
operation refuses.

**My guard is what made it unrecoverable, and I verified that rather than accepting the review's
word.** Built `88759b6^` in a worktree, ran the same typo, and archived the result:

```
$ htmlspec archive ../runbooks/sneaky --yes     # pre-PR binary
archived to /private/tmp/wedgepre/htmlspec/changes/archive/2026-09-18-sneaky at version 0.1.0
```

Pre-PR the typo produced a directory that was odd and still archivable. **So the fix for a data
loss converted a recoverable mistake into a wedge.**

### Fixed at the constructor, not the call site

`ProjectRoot::named_change` and `ProjectRoot::is_a_name`, refusing a name containing `/`, `\`,
or equal to `.` or `..`. `AGENTS.md` §2: this invariant had been re-litigated at three separate
call sites, which is the signal it belongs in a constructor.

**A correction to what I reported last round.** I justified the runtime guard over a newtype by
saying `project.change` is called in "23 places". That counted test call sites. **There are 7
production call sites**, and the number came from a grep I read as the answer to a question it
did not answer — the same carelessness this project's 0.10.0 release recorded three times.

The 7 do not all want the check: five compose from a name the program produced (a directory
entry it just read), where a `Result` would be an error case that cannot happen. So
`change()` stays infallible and `named_change()` is the checked counterpart for the operations
that take a name from a contributor. That is a smaller change than a newtype and it puts the
invariant in one place.

**Rejected: sanitizing.** Rewriting `../runbooks/sneaky` to `sneaky` would create a change under
a name the contributor did not type, and `artifact-kinds` requires a derived name to be reported
so a contributor who disagrees can say so.

Both separators are refused on every platform. A backslash is a separator on Windows and a legal
filename character elsewhere; accepting it here would make one name mean two things depending on
where the command ran.

## Hunting for a fourth bypass

Two reviews each found a way past this guard, so I went looking rather than waiting. **What I
checked, including where I found nothing** — the negative list is the point.

Started from the question "what performs an irreversible filesystem operation?" and grepped for
`fs::rename` and `fs::remove_dir_all` in non-test code. Three sites: archival's, and two in
`staging.rs`.

| Probe | Result |
| --- | --- |
| `migrate promote` over a project holding a runbook | **Safe.** `Staged::resolve` refuses over an existing `htmlspec/` or `spec.nix` (`REFUSED_OVER`), and a runbook only exists inside a project. Already pinned by `a_destination_holding_the_project_or_the_declaration_is_still_refused`. |
| Broken symlink at `htmlspec/runbooks` | **Bypass C.** Found here. Fixed. |
| Symlink in `changes/` pointing at a runbook | Safe — `canonicalize` resolves it into `runbooks/`. |
| Subdirectory inside a runbook (a future `runs/`) | Safe — `starts_with` covers descendants. |
| Runbook named `archive` | Safe. |
| Trailing slash, `./`, embedded `/./` | All safe. |
| `--store` arm with bypass A applied | Safe — the store's project is passed. |
| Runbook in a *different* project than the caller's cwd | Safe. |
| `select_change` with no argument | Safe — reads `changes/` only, and `active_changes` never sees `runbooks/`. |
| `new document` into an existing runbook document | Safe — refuses to overwrite. |
| `format` (which rewrites files) on a runbook | Safe by design: formatting a runbook's documents is correct, and canonical output is what the new check stage asserts. |
| `new change .`, `..`, `""` | All refused by `is_a_name`. |

**The general form worth keeping: a guard against an irreversible operation should be tested by
asking what makes it answer "no" wrongly, not by asking whether it answers "yes" correctly.** All
three bypasses are the guard saying "not a runbook" about a runbook. None would have been found by
adding more cases that *are* runbooks.

## `runbooks/` is now inside the check suite

**Two new stages rather than two edited arguments, because `validate` and `format` each take one
path** — `validate fixtures/dogfood runbooks` is a usage error. So `self-runbooks` and
`self-runbooks-format` join the suite, which is now 12 stages. Both pass.

What they do **not** close: validation still cannot report the `data-change` violation from the
previous round, because no rule reads a document's location and `runbook-authority` forbids one
that does. A project-level rule is the successor.

## Verification

- All four new tests **verified failing on `d8523bb`**, each for its own assertion: "archival
  accepted a runbook with a nested htmlspec/ directory", "…with a nested .git directory", "a path
  was accepted as a change name", "the refusal does not say it is failing closed".
- Two existing unit tests were retargeted from `resolve_change` to `refuse_a_runbook`, **because
  the fix made `resolve_change` the wrong subject for a unit test**: it now resolves the project
  from the process's current directory, which is the point. Confirmed they still fail against a
  neutered guard, so retargeting did not make them vacuous.
- `cargo test --workspace`: 33 targets, no failures. `scripts/check`: all 12 stages.
- `openspec validate establish-a-runbook --type change`: valid.
