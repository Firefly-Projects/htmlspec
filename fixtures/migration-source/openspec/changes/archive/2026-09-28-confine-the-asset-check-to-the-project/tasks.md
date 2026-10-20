## 1. Re-measure before changing anything

- [x] 1.1 In a copy of HEAD, run `htmlspec init --tools none`, then `htmlspec assets .`, and record the exact count and every location reported. The expected count is nine. Record the actual count, because the proposal and the design both rest on it
- [x] 1.2 In the same copy, run `htmlspec validate` over `fixtures/` and over the new `htmlspec/`. Check whether any stage other than `self-assets` changes result when `htmlspec/` exists, and record the answer. The proposal claims only the topology check is affected

## 2. Tests first

- [x] 2.1 Add CLI tests for the four scenarios in `specs/document-validation/spec.md`: a project beside a fixture store, a project beside a root toolchain store, two stores inside the project, and an outside store that duplicates within itself. Check that the first two fail on HEAD and the last two pass on HEAD
- [x] 2.2 Add a test that `assets` and the promotion check give the same verdict on the same tree, so that one of them cannot drift from the other

## 3. The partition

- [x] 3.1 Write one function that partitions discovered stores into the project's and the others by canonical path prefix against `ProjectRoot::htmlspec()`, and unit-test it with a symlinked path and with a store at `htmlspec/assets` itself
- [x] 3.2 Use it in `assets`: compare across the project's stores only, and check each outside store on its own. Check that the tests from 2.1 pass
- [x] 3.3 Use it in the promotion check in place of its separate topology loop. Check with `cargo test -p htmlspec-cli --test promote`
- [x] 3.4 Report the partition in the human report and as `projectStores` and `otherStores` in the structured payload. Check that the existing payload keys keep their values in the existing tests

## 4. The suite

- [x] 4.1 Correct the `self-assets` comment in `scripts/check`
- [x] 4.2 Repeat 1.1 against the built change and record the result. The expected result is zero duplicates. If any are reported, record what they are before changing anything else

## 5. Prove it

- [x] 5.1 Mutation-test the change. Make the partition return every store as the project's, then make it return none, and check that a test catches each case
- [x] 5.2 Run `cargo test --workspace --all-targets`, then `scripts/check`, with every stage green
- [x] 5.3 Record what the implementation revealed

## 6. Release

- [x] 6.1 Run the release runbook at `runbooks/release/tasks.html`, and report whether it worked as written
- [x] 6.2 Choose the increment and state the reasoning, and flag it for confirmation
- [x] 6.3 Check that `scripts/check-release` passes after archival writes `VERSION`
