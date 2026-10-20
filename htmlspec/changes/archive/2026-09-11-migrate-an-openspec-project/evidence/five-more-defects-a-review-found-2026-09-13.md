# Five more defects a review found

**Date:** 2026-09-13
**Found by:** a second code review of pull request #25, at medium depth
**Fixed in:** `crates/htmlspec-cli/src/migrate.rs`, `staging.rs`, `plan.rs`, `commands.rs`

Five findings, all real. I checked each against the code before touching anything and reproduced
three of them end to end. This brings the review total on this change to **nine defects**, none of
which its own 793-test suite found.

## 1. The survey skipped three whole levels of the project

`Survey::of` enumerated exactly three things: `openspec/config.yaml` by name, everything under
`specs/`, and everything under each change *directory*. A file sitting loose in `openspec/`,
`openspec/changes/`, or `openspec/changes/archive/` was invisible.

Reproduced on a seven-file project:

```
=== files on disk ===          === what the survey enumerated ===
openspec/AGENTS.md                (nothing)
openspec/changes/NOTES.md         (nothing)
openspec/changes/archive/README.md (nothing)
openspec/project.md               (nothing)
openspec/config.yaml              configuration
openspec/specs/a/spec.md          accepted-specification
openspec/changes/archive/x/proposal.md  change-document

3 of 7
```

**Four of seven files lost.** And not contrived ones: OpenSpec itself writes `openspec/project.md`
and `openspec/AGENTS.md`, and this change's own proposal names the second as an entity it
deliberately does not migrate — which still has to reach the plan as a disposition.

Two failures followed. The coverage gate could not report the files, so they went missing with every
check passing — the exact failure the module comment says it exists to prevent. And an agent who
*did* write a plan line for `openspec/project.md` had it reported under `unsurveyed` as "the plan was
written against a different survey", which points at deleting the correct line.

**The repair is the requirement read literally.** The enumeration "SHALL be derived from what the
source holds rather than from a list of the entity kinds migration expects", so the survey now walks
the whole project directory and classifies what it finds. All seven files enumerate, and the four
previously invisible ones arrive as `unrecognized` — a line a contributor decides about.

`classify_outside_a_change` gives `project.md` and `AGENTS.md` no kind of their own on purpose.
Giving them one would be this module deciding what they become, which belongs to the plan.

## 2. The cross-device fallback left the destination partly populated

`rename` fails with `EXDEV` when staging and destination are on different filesystems, which a
staging directory under the system temporary directory usually is. The fallback copies and then
removes — so it writes into the destination *before* anything leaves staging.

A failure part-way could not be rolled back. `promote` unwinds the entries it recorded as moved, and
the entry being copied is not recorded until the copy returns. So a full disk on the copy left the
destination holding a partial project while `migrate_promote` reported:

> The destination is as it was, and the staged project is whole, so this is safe to run again

Both false, and the retry then tripped `DestinationOccupied` with no way past.

The fallback now cleans up after itself on either failure. It is also extracted as
`copy_then_remove`, and that extraction is the finding underneath the finding — see below.

## 3. The shared-version check read a changelog that promotion discards

`version_faults` read the *staged* `CHANGELOG.md`. `promote` deliberately leaves a destination's
existing copy alone. So a migration whose agent added the baseline entry to the staged record — which
is what the workflow instructs — passed the check and then promoted a project whose actual changelog
lacked it.

Reproduced end to end:

```
$ htmlspec migrate promote ... --staged staged --into repo
promoted: htmlspec, spec.nix

$ cd repo && grep "^#" CHANGELOG.md
# Changelog
## 2.0.0
$ for f in htmlspec/changes/archive/*/VERSION; do cat $f; done
0.1.0
0.1.0
```

Two changes at `0.1.0`, and no heading explaining it. The migrated project fails its own release
check on arrival "reporting a collision it cannot explain" — verbatim the outcome the fault exists to
prevent.

The check now reads the record that survives: the destination's where it has one, the staged copy
otherwise. Both sides are pinned, because reading the surviving record must not mean refusing a
destination that already explains the collision — that is the deadlock this change fixed once
already.

## 4. A malformed `derives:` note vanished

`value.rsplit_once(" -> ")` requires the exact spelling. With `->` unspaced, an em dash, or a colon,
the `if let` fell through: nothing recorded, nothing reported. A plan misspelling the separator on
all 120 of its requirement lines produced zero derivations, zero collisions, and a clean report — the
systematic-fault shape the rest of this change is built to surface.

Unreadable lines are now kept and reported, with the separator named. A contributor told only
"unreadable" has to guess which of three plausible spellings is wanted.

## 5. A plan item with no destination passed, and skipped collision checking

`Coverage::of` inspected `source`, `disposition`, and `reason` — never `destination`, though the
requirement says the plan records "its destination and its disposition" for each entity.

The second half is worse than the missing report. `collisions()` opened with
`let Some(destination) = item.destination.as_deref() else { continue; }`, so an item missing that one
note had **every** one of its `derives:` lines excluded from collision detection. An agent collapsing
sixteen changes' deltas omits the note on a few items, coverage reads complete, and two distinct
headings deriving one identifier inside one collapsed specification reach the staged tree — surfacing
at stage 6 as a duplicate-identifier error in a document that must then be re-collapsed by hand.

## Two tests I wrote that tested nothing

**The mutation check earned its place three times over here.** Reverting each fix and re-running is
how I found that my own first two attempts at finding 2's test were worthless:

| Attempt | Why it passed against the unfixed code |
| --- | --- |
| Lock the destination's parent directory | `create_dir_all` fails before `rename` runs, so the fallback is never entered |
| Occupy the destination so `rename` fails | `rename` returns `ENOTEMPTY` (66), not `EXDEV` (18), so `crosses_devices` is false and the fallback is skipped |
| Drive `copy_then_remove` directly | Fails against the unfixed code, reporting `the destination kept a partial entry: Ok(["nested"])` |

`EXDEV` needs two real filesystems to provoke, so no test driving `rename_or_copy` can reach that
branch at all. **Naming the fallback as its own function is what makes its recovery testable** — the
extraction is not tidying, it is the difference between a tested repair and an asserted one.

The third attempt also needed the obstruction *nested* inside the destination rather than at its top
level, because `copy_tree` creates the top level first and fails before writing anything removable.
A partial write is the state the cleanup exists for, and only a nested obstruction produces one.

## Every fix checked by reverting it

| Reverted | What failed |
| --- | --- |
| Survey walks `specs/` only | 5 tests, including `a_file_loose_at_any_level_of_the_project_is_enumerated` |
| The copy's cleanup | `a_failed_copy_leaves_no_partial_entry_in_the_destination` |
| Reading the surviving record | `a_shared_version_explained_only_by_the_discarded_record_is_refused` and its counterpart |

Findings 4 and 5 are new reports rather than behavior changes, so their tests assert the new fields
directly.

## What nine defects in one change say

Every one of the nine is a rule that was right, paired with a check whose reach fell short of it. The
count is the finding now, not any individual repair.

Two of the nine were found by a person reading a file. Six were found by review. One was found by
continuous integration. **None was found by the test suite**, and this round adds the reason to the
one already recorded: the suite's constructed projects were as tidy as its constructed destinations
were empty. A seven-file source project with files loose at three levels is four lines of setup, and
no test in this change had one until now.
