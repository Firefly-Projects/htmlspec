## 1. Confirm the premises before building on them

- [x] 1.1 Confirm that a renamed requirement already keeps its identifier: build a two-change fixture that renames `req-x` with the same `id`, and check that `htmlspec validate` accepts both documents and that `htmlspec sync --change` reports `req-x` as reached. That report is the defect this change fixes, so record it as the "before" state
- [x] 1.2 Confirm how the projection derives a requirement's `title` when the heading carries `<span class="hs-id">`, and whether whitespace normalization makes a reflowed heading compare equal. Check with a heading passed through `htmlspec format`, and record the answer, because decisions 3 and 4 depend on it
- [x] 1.3 Re-read `openspec` 1.13.1's `dist/core/specs-apply.js` rename handling and list each rule against its counterpart in `specs/document-schema/spec.md` and design decision 1. Any OpenSpec rule with no counterpart is either added here or named as a deliberate difference in the design

## 2. The contract in the schema and the projection

- [x] 2.1 Add `renamedFrom` (optional, `nonEmptyText`) to the requirement definition in `crates/htmlspec-schema/schema/document.schema.json`, and amend the `changeState` description of `unchanged` as design decision 2 states. Check with `cargo test -p htmlspec-schema`
- [x] 2.2 Add `renamed-from` to the requirement row of the projection specification's section 4 field table, with the single-valued rule and the exclusion from `statements`, in `crates/htmlspec-schema/doc/projection.md`
- [x] 2.3 Record the minor bump and the date in `crates/htmlspec-schema/CHANGELOG.md` under its bump table, and check that the contract tests in `crates/htmlspec-schema/tests/contract.rs` agree with the new version

## 3. Projection

- [x] 3.1 Project a requirement's `data-field="renamed-from"` paragraph into `renamedFrom` in `crates/htmlspec-parse`, and keep it out of `statements`. Check with a projection test for each document-schema scenario named "The previous heading is a field, not a statement" and "A requirement with no rename"

## 4. Validation

- [x] 4.1 Add the six placement and consistency rules from the document-schema delta to `crates/htmlspec-validate` as errors with distinct codes: not a requirement, on `added`, on `removed`, repeated, empty, and equal to the heading. Check with one unit test per rule that fails before the rule exists
- [x] 4.2 Add conformance fixtures, one valid and one invalid per rule, through `conformance/src/bin/generate_fixtures.rs`. Check that the two hand-written placeholder fixtures the generator is known to delete survive regeneration, or restore them and say so
- [x] 4.3 Check that `htmlspec format` leaves a `renamed-from` paragraph canonical and idempotent: format twice, and diff

## 5. The merge comparison

- [x] 5.1 Extend `merge_state` in `crates/htmlspec-cli/src/commands.rs` with the untaken-rename set as design decision 4 states. An untaken rename leaves `reached`, and an identifier the accepted specifications do not hold is never an untaken rename. Check with unit tests for each toolchain-packaging scenario that names a rename
- [x] 5.2 Report untaken renames from `sync` and from archival, human and structured, each naming the identifier and both headings. Make "not synced" true when either set is non-empty. Check that the existing test for "the two operations disagree about what is unmerged" still holds with a rename in the fixture
- [x] 5.3 Add a CLI test for the change whose only unapplied work is a rename: archival reports it as not synced, and names both the untaken rename and `sync`

## 6. Instructions

- [x] 6.1 State in `crates/htmlspec-assets/instructions/sync.md` how a rename is merged: new heading, same identifier, no `renamed-from` carried into the accepted specification, and the modification applied to the same requirement when both are present
- [x] 6.2 State the same in `archive.md` wherever it describes applying a contract, and state how to produce a renaming change's specification
- [x] 6.3 In `migrate.md`, replace the `## RENAMED Requirements` → "no state exists" row. The new row maps to `renamed-from` on the `TO:` requirement, with the state taken from the rest of the delta and a chain resolving to the first heading's identifier. Remove the instruction to record renames as open
- [x] 6.4 Regenerate the installed copies under `.claude/`, `.codex/`, and `.pi/` with the command the release runbook names, never by hand. Check with `scripts/check-installed`

## 7. Migration

- [x] 7.1 Rewrite `the_delta_section_the_destination_cannot_express_is_surfaced` in `crates/htmlspec-cli/tests/migrate.rs`. Its tripwire looks for `"renamed"` in the schema and does **not** fire on a `renamedFrom` field, so it would keep calling the rename open after this change lands. It becomes a test that the survey still surfaces RENAMED sections and that the schema can now express them
- [x] 7.2 Check the chained identifier rule against this repository's two real renames. Derive the identifier for the `toolchain-packaging` requirement now titled "The command surface names its operations as OpenSpec names them", and check that it is `req-a-single-command-exposes-validate-format-and-init` in both renaming changes and in the accepted specification

## 8. Presentation

- [x] 8.1 Add the "Renamed from" generated-content rule to the screen and print stylesheets as new content-addressed revisions, through the `assets/vendor.lock.json` procedure. Move the templates to the new addresses, and leave every existing revision in place. Check with `cargo test -p htmlspec-assets` and `conformance/tests/presentation.rs`
- [x] 8.2 Render a document that carries a renamed-and-modified requirement, with styles on and with styles off. Check that the previous heading is readable in both and labelled in the first

## 9. Prove it

- [x] 9.1 Mutation-test each new rule and the untaken-rename set: neuter each one alone, and record which test catches it. A mutation that nothing catches needs a new test before this task is done
- [x] 9.2 Run `cargo test --workspace --all-targets`, then `scripts/check`, with every stage green
- [x] 9.3 Record what the implementation revealed, including anything from task 1.3 that the design did not anticipate

## 10. Release

- [x] 10.1 Run the release runbook at `runbooks/release/tasks.html`, and report whether it worked as written
- [x] 10.2 Choose the increment and state the reasoning. The archive instructions reserve this for a human, so flag it for confirmation
- [x] 10.3 Check that `scripts/check-release` passes after archival writes `VERSION`
