## 1. Re-measure before changing anything

- [x] 1.1 In a scratch project built with `htmlspec init --tools none`, copy the three HTML evidence files from `2026-09-11-migrate-an-openspec-project/evidence/` into one change's `evidence/`. Run `validate`, `format --check`, and `format`. Record each exit status, every diagnostic naming an evidence file, and whether `format` changed any evidence bytes, compared with `cmp`. The proposal rests on the 2026-09-09 measurement, so this reproduces it against HEAD
- [x] 1.2 In the same scratch project, record the exit status of `htmlspec migrate promote --check` over a staged tree that holds the same evidence, to confirm that promotion refuses on HEAD for this reason and no other

## 2. Tests first

- [x] 2.1 Add CLI tests for the toolchain-packaging scenarios: validating a project that holds HTML evidence, `format` leaving evidence byte-identical and `format --check` passing, naming an evidence file, naming the evidence directory, and outside a project every file being a document. Check that the first two fail on HEAD and the last three pass on HEAD
- [x] 2.2 Add CLI tests for the artifact-kinds scenarios not covered by 2.1: evidence nested below `evidence/`, a capability at `htmlspec/specs/evidence/spec.html` that is still validated (and is reported when invalid), and an archived change's evidence. Check which of these fail on HEAD, and record which do
- [x] 2.3 Add a promotion test for "Carried evidence in the destination format does not gate promotion": a staged project with non-conforming HTML evidence promotes, and the file arrives byte-identical. Check that it fails on HEAD

## 3. The definition and the walk

- [x] 3.1 Add the evidence location to `ProjectRoot` as a private constant and a method answering whether a directory is a change's evidence directory. Compare on canonical paths. Unit-test it for an active change's and an archived change's evidence, `changes/archive/evidence`, `specs/evidence`, a nested `evidence/evidence`, and a project reached through a symbolic link
- [x] 3.2 Make `discover::documents` resolve the project from its root and skip, during the walk, every child directory the method identifies. The root must never be skipped. Unit-test that a root inside evidence is walked in full, and check that the tests from 2.1 through 2.3 pass
- [x] 3.3 Check that no caller of `discover::documents` filters evidence on its own, and that the doc comments on `documents`, `subject`, and `conformance` state the new behavior

## 4. The instructions

- [x] 4.1 In `crates/htmlspec-assets/instructions/migrate.md`, make the `evidence/**` row cite artifact-kinds' "A change's evidence is kept in the change and is not one of its documents" and say that carried HTML evidence does not gate promotion. Regenerate the installed copies with `./target/debug/htmlspec init --tools claude,codex,pi`, remove the stray `htmlspec/` and `spec.nix`, and check that `scripts/check-installed` passes

## 5. Prove it

- [x] 5.1 Repeat 1.1 and 1.2 against the built change and record the results. The expected result is that every command succeeds and the evidence bytes are unchanged
- [x] 5.2 Mutation-test the change. Replace the positional test with a name-only test, exempt nothing (root included), and skip nothing, and check that a test catches each mutation
- [x] 5.3 Run `cargo test --workspace --all-targets`, then `scripts/check`, with every stage green
- [x] 5.4 Record what the implementation revealed

## 6. Release

- [x] 6.1 Run the release runbook at `runbooks/release/tasks.html`, and report whether it worked as written
  - **It worked as written, in its order.** Step 1 moved the workspace to `0.18.0` and relocked seven workspace members offline, and nothing else. Step 2 ran from `./target/debug/htmlspec` rebuilt at 0.18.0. A bare `htmlspec` still resolves to an older profile binary, as the last two releases recorded. It regenerated the installed files and created the stray `htmlspec/` and `spec.nix` the runbook warns about, which were removed. Step 3 wrote the entry. Step 4 passes for the declared version, and `scripts/check` is green at 13 of 13
  - **The gap recorded twice before is still open**: the runbook does not say that archival here also needs a hand-written `runbooks.json` beside `VERSION`. Task 6.3 writes both
- [x] 6.2 Choose the increment and state the reasoning, and flag it for confirmation
  - **0.18.0, confirmed by the contributor.** Two requirements are added and the observable behavior of `validate`, `format`, and promotion changes inside a project, which is what 0.17.0 was a minor increment for. The document schema is untouched, so the contract stays at `0.2.0`
- [x] 6.3 Check that `scripts/check-release` passes after archival writes `VERSION` and the archive's `runbooks.json`
  - **Passes**: 24 archived changes record a version, 18 versions are in the changelog, and the declared `0.18.0` has an entry. `VERSION` reads `0.18.0`. `runbooks.json` is byte-identical to the previous archive's, because the release runbook is unchanged at `0.1.0`. Its two recorded digests equal `shasum -a 256` of `specification.html` and `tasks.html`, and its changelog equals `runbooks/release/CHANGELOG.md` byte for byte

## 7. Review

- [x] 7.1 Resolve what a review of the pull request found, and record what was and was not taken
  - **Taken: `format` rewrote a named record.** The two new requirements contradicted each other. artifact-kinds said no operation rewrites evidence, while toolchain-packaging said a named path is operated on as named, and the code followed the second. Reproduced: `format htmlspec/changes/measure-evidence/evidence` rewrote all seven records. The contributor chose "never write evidence". toolchain-packaging now says a named path is *checked* as named, and that `format`'s writing mode refuses a record: it names it, writes nothing, and exits as an operation that could not run. artifact-kinds' sentence now covers a named record explicitly. It is covered by `formatting_a_named_record_is_refused_and_writes_nothing` and by mutations 7 and 8
  - **Not taken: an asset store beneath evidence counts as the project's.** This is true, and it is the case the design lists as a non-goal. No evidence holds a store, and nothing in the `migrate` instructions tells anyone to carry one there. Left for a change whose subject is a store inside a record
