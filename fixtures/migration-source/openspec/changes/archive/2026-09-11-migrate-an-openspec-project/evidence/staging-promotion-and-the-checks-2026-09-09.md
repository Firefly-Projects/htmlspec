# Staging, promotion, and the checks that gate it

**Date:** 2026-09-09
**Tasks:** 5.1–5.9, 6.1–6.4
**Method:** `crates/htmlspec-cli/src/staging.rs` (the structural checks and the move),
`commands.rs::conformance` and `migrate_promote` (the stages),
`crates/htmlspec-cli/tests/promote.rs` (end to end, through the compiled program). One deliberate
mutation on the whole-set reporting behavior.

## The four stages, and why the order is the requirement

`migrate promote <plan> --survey <payload> [--staged <path>] [--into <path>] [--check]`

| Stage | What it does | Why here |
| --- | --- | --- |
| Coverage | the plan against the survey | first, so a plan missing an entity kind is reported before anything is read from the staging tree |
| Structure | outside the destination, resolves as a project, one store, versions where the gate reads them, destination unoccupied | before any document is read, because a tree in the wrong place is a mistake about the operation rather than about content |
| Conformance | `validate`, `format --check`, `assets` over the **complete** tree | last, and whole |
| Move | one step, whole or none | only where all three above are clear |

`--staged` is optional, and that is the stage order made reachable rather than merely documented.
An agent runs the coverage check at stage 4 with nothing staged, and the whole of promotion at
stage 7, with one command. `the_coverage_check_runs_before_anything_is_staged` pins it, and asserts
the report names `--staged` so a contributor knows what it did not do.

## The staging checks, and what each one is for

| Fault | The failure it prevents |
| --- | --- |
| `InsideTheDestination` | a second asset store inside the destination, so `htmlspec assets` fires on the migration's own working directory and a contributor cannot tell that from a real defect |
| `NotAProject` | a staging area no check can resolve, which pushes every check past promotion and makes promotion the step that discovers faults |
| `Incomplete` | the same, one absence at a time — reported together |
| `SeveralStores` | promoting the defect `htmlspec assets` exists to report |
| `DestinationOccupied` | an overwrite or a merge, neither of which is whole-or-none |
| `VersionInAnArchiveName` | the `<date>-<version>-<name>` fault: a naming violation **and** a version the gate never reads |
| `DeclarationDisagrees` | a migrated project whose next archival passes without advancing |
| `UnexplainedSharedVersion` | a migrated project failing its own release check on arrival |

Every fault is reported together. A staged tree with three faults reported one at a time costs
three runs, which is the waste the check-last rule exists to prevent one stage later.

## Promotion is a move, and that is asserted on the references

The claim "the staged tree mirrors the destination's paths, so promotion rewrites nothing" is
checked on what a document *says*, not on where a file sits. Four documents at four depths:

```
htmlspec/specs/one.html                                        ../assets
htmlspec/specs/project-bootstrap/spec.html                     ../../assets
htmlspec/changes/m/proposal.html                               ../../assets
htmlspec/changes/archive/2026-08-10-html-spec/specification.html   ../../../assets
```

Each reference is resolved from the document's own directory and compared against the store — in
staging, and again after the move, with identical reference text. **The test resolves `..` by
popping rather than by `Path::components()`**, which keeps `..` rather than resolving it: a
comparison leaving `..` in both sides holds for any depth at all and proves nothing. That was a
real defect in the first version of this test, caught because the assertion failed.

`a_complete_migration_promotes_and_the_result_is_a_project` closes it from the other end. The
promoted project passes `validate`, `assets`, and `format --check` **from its new location**, which
is only true if no reference needed rewriting.

## The move is whole-or-none, in two parts

Refusing while the destination holds anything promotion would place is the first part — nothing is
overwritten and nothing is merged. The second is the rollback, and it needed a test that provokes a
genuine mid-flight failure rather than a precondition:

`a_failed_move_leaves_the_destination_unchanged` occupies the declaration's destination path with a
non-empty directory **after** `resolve` has passed. So the project directory has already arrived
when the second move fails. Without the rollback the destination would hold a migrated `htmlspec/`
and no declaration — a project the toolchain resolves and cannot read the version of. The test
asserts the destination holds none of it and the staged tree is whole again, so the migration is
retryable.

`rename` is one operation and cannot half-move a directory. The `EXDEV` fallback copies then
removes, because a staging location on another filesystem is the common case rather than the rare
one.

## The destination is unchanged at every stage boundary, not just the last

`an_abandoned_migration_leaves_the_destination_unchanged_at_every_stage` walks all four boundaries
in one test and compares a **content** snapshot of the destination after each:

1. a coverage failure (an entity added to the payload)
2. a staging fault (the declaration lowered below the greatest archived version)
3. a conformance failure (an identifier given a capital)
4. a clean `--check` run

Names alone would miss an edit and lengths would miss a swap, so the snapshot holds bytes. The same
test asserts the staged tree is byte-for-byte unchanged across all four, so nothing above consumed
its own input either.

## The whole-set behavior, checked rather than described

Task 6.3 asked for the case the check-last requirement was written for. One mapping rule broken —
an identifier that keeps its heading's capitals — applied to **12 staged documents**, because that
is how a wrong rule behaves: it is wrong everywhere it applies.

The assertion is not that a diagnostic appears. It is that every one of the 12 appears in one
report, with the document count stated before the list, so "12 documents report the same
diagnostic" is legible as a different finding from "capability-3 is broken".

**Mutation.** Returning from `conformance` at the first document with a diagnostic:

```
test a_systematic_mapping_fault_is_reported_as_a_whole_set ... FAILED
capability-1 was not reported, so the run stopped early
```

Twelve documents rather than this repository's 401 accepted scenarios. The requirement is about
reporting a class rather than an instance, and 12 identical diagnostics establish that as well as
401 while keeping the test to seconds — and a test staging 401 real scenarios would need 401
hand-written source scenarios, which is the migration itself rather than a test of it. Recorded
here because task 6.3 named the larger figure.

## Task 5.9 could not be satisfied as written, and why that is right

The task said to confirm `scripts/check-release` passes against the migrated project. It cannot:

```
$ scripts/check-release --root /tmp/stagetest/repository
no parseable version in /private/tmp/stagetest/repository/Cargo.toml
```

The script reads `Cargo.toml` and `openspec/changes/archive`. A migrated project has `spec.nix` and
`htmlspec/changes/archive`. This is not an oversight in the script — its own docstring states the
reason:

> The facts it reads are facts about *this repository* — its workspace version, its archive
> directory, its changelog. A test inside a shipped crate asserting them would make the product's
> test suite depend on its own repository's release history, so a consumer running `cargo test`
> would be checking html-spec's changelog rather than their own.

So the check that belongs to this repository stays here, and the **obligation** it enforces moved
into the promotion gate, where a migrated project can be held to it. `Fault::UnexplainedSharedVersion`
refuses a shared version the release record does not name, `CHANGELOG.md` is one of the three
entries promotion places, and two tests cover both sides:

- `a_shared_version_with_no_release_record_entry_is_refused` — this repository's real shape (three
  changes at `0.1.0`, one at `0.2.0`, a record naming only `0.2.0`), asserting **every** change is
  named, because the repair is one entry covering all of them.
- `a_shared_version_the_release_record_explains_is_carried` — the half that matters more. Inventing
  distinct versions to satisfy distinctness would record releases nobody installed, which
  `toolchain-packaging` calls worse than recording the shared version honestly.

Presence of the version in the record is what is checked, never the prose. That is
`toolchain-packaging`'s own scenario "An entry's contents": the release check "reads the entry's
presence and applies no rule to the entry's prose".

## A promotion, run by hand

```
$ htmlspec migrate promote plan.html --survey survey.json --staged staging --into repository --check
checked 1 document(s) and 8 stored asset(s) across the complete staged project
1 surveyed entities, 1 plan items
every surveyed entity has a disposition
every check passed and nothing was moved. Run this without --check to promote
exit: 0
$ ls -A repository | wc -l
0

$ htmlspec migrate promote plan.html --survey survey.json --staged staging --into repository
...
promoted: htmlspec, spec.nix, CHANGELOG.md
the source project is untouched. Removing it is a separate act you take after reading the result
exit: 0

$ cd repository && htmlspec new change try && htmlspec archive try --check
htmlspec: the declared version is still 0.1.0, which is the last archived version. Raise
`version` in spec.nix before archiving...
exit: 2
```

The last three lines are task 5.5's claim proven where it matters: the gate read the version the
migration carried, so the migrated project's next archival is forced to advance past its real
history.

## State of the suite

- `cargo test`: **all green**, zero failures. 17 unit tests in `staging.rs`, 6 integration tests in
  `tests/promote.rs`, plus the existing 24 in `plan.rs` and 5 command tests.
- `cargo clippy --all-targets`: **no warnings**.
- `./scripts/check`: **all 10 checks passed**.
