# What the implementation revealed

**Date:** 2026-10-03
**Tasks:** 5.5 and 5.6

## 5.5: the checks

- `scripts/check` inside `nix develop`: the first nine stages pass, format and lint among them. The four
  Python stages fail there with `error: tool 'python3' not found`, as they have for the last three
  changes. Run outside the shell with the same Rust toolchain first on `PATH`, all four pass:
  - prompt: `all 4 prompt cases passed`
  - installed: `45 installed instruction file(s) match what init writes`
  - release-fixtures: `23 shared fixture case(s) read the same way both readers read them`
  - release: `declared version 0.21.0 has a changelog entry`
- The test stage ran 1,194 tests, and none failed.
- The first `scripts/check` failed its lint stage. Clippy's `large_enum_variant` refused `Reading`, whose
  `Read` case held a whole projection beside an `Unread` of two strings. The projection is now boxed.

## 5.6: what the work found

- **The planning text named the wrong contract version again.** The design's Migration Plan and task
  6.2 said the contract stays `0.2.0`. It has been `0.3.0` since `reject-change-markup-inside-a-unit`.
  Both were corrected before the measurement.

- **Two types are named `Declared`.** This change's `htmlspec_validate::references::Declared` is what
  one place declares, as design decision 4 names it. The previous change, `see-a-modification-the-merge-did-not-take`,
  added a private `Declared` enum in `crates/htmlspec-cli/src/commands.rs`, for what a change says it
  did to a requirement. The validator's type is not re-exported at the crate root, and `commands.rs`
  names it by its module path, so the two never meet in one scope. Renaming the CLI's enum was not this
  change's work. **The review can decide whether one of them should move.** It did: at review the
  command line's enum became `ContractEffect`, and the validator's type kept its name (task 7.1).

- **The change specification's file name moved to the layout module.** `ProjectRoot::placement` needs
  `CHANGE_SPECIFICATION` to tell a change's specification from its other documents, and the constant
  lived in `commands.rs`. It now lives in `project.rs`, which owns every other position in the layout,
  and `commands.rs` re-exports it, so no caller changed.

- **Task 5.3's expectation for two links was wrong, and the comparison was right.** The two units
  linking to another change's specification resolve, because the identifiers they name are also
  declared by accepted specifications, where those changes' requirements were merged. A reference
  names a unit, not a file. `the-rule-measured-2026-10-03.md` records both.

- **One test assertion was too broad, and it was narrowed, not weakened.** `a_change_with_no_specification_reaches_the_accepted_contract`
  first asserted that the output never contains the word "specification". The tasks template's own
  `review/example-content` warning says "a change's specification" in general, so the assertion failed
  against correct behavior. It now asserts that no `refs/*` diagnostic is reported and that the output
  neither names `specification.html` nor says "could not be read", which is what the scenario "A change
  with no specification yet" means by "nothing is reported about the missing specification".

- **Every call to `engine::run` passes its scope explicitly.** 47 calls in the validator's tests, the
  conformance suites, the template test, the `validate_file` example, and the command line now pass a
  scope, and every one except `validate_source` passes `Scope::loose()`. There is no default, so a caller
  that wanted a document's real surroundings cannot get loose resolution by omission.

- **The design's sketch named no constructor for `Declared`.** It is `Declared::gather`, from projections
  and the documents that could not be read, with `Default` as the place that declares nothing. What a
  scope holds is read only by the reference rule: `Scope::find` and the `Found` it returns are private
  to the validator.

- **The evidence check in `ProjectRoot::placement` is also implied by position.** A file directly in a
  change's `evidence/` has the evidence directory as its parent, which is not a change, so the positional
  test alone would place it loose, and so would anything nested beneath it. The explicit check is kept,
  as task 3.5 asks, because it states the rule artifact-kinds gives directly, rather than leaving
  evidence loose only as a consequence of the layout. Mutation 8 showed it is pinned only by the
  placement test.

- **One doc comment that states the one-document rule was left as it is.** `crates/htmlspec-cli/tests/run.rs`
  at the fixture helper `strip_requirements` says "a reference resolves within one document" about a
  runbook's specification, where it is still true. Task 3.7 named only the assertion message at line 243.

- **The machine paused the session's file work during the proof.** A `cargo test` run stopped for 16
  minutes between starting and running its first test binary, a shell's `sleep 25` took more than 300
  seconds, and the staged tree's validation took 97 to 553 seconds of wall time while its CPU time stayed
  near 20. A CPU-bound control ran normally. Six further runs of the command line's unit tests all
  finished: five in parallel, each reporting its tests done in 10.13 seconds, and one at a time, in
  16.79. So the timing
  in task 5.2 is reported as CPU time, and no test in this change hangs.
