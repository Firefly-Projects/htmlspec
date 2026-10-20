# Tasks

## 1. Establish typed boundaries

- [x] 1.1 Define the closed machine-check, observation, assessment, and structured-error types from design decisions 1–2, including a distinct `PrerequisiteFailure` CLI outcome, before their implementations; verify `cargo check -p htmlspec-cli --locked` and review that raw note JSON cannot reach evaluation.
- [x] 1.2 Define validated requirement IDs, program names, scoped paths, separate runbook/project versions, source snapshots, and the private `ReadyRun` construction path; verify signatures bind passed observations to the source copied into a record.
- [x] 1.3 Define typed release inventories, content digests, adoption metadata, and release-comparison outcomes; add shared wire-format fixtures and verify both Rust and the Python checker can consume their valid shape.

## 2. Specify failures before implementing behavior

- [x] 2.1 Add parser tests for all four check forms, ordinary notes, zero checks, malformed/unknown fields, duplicate declarations, unknown format versions, and invalid paths; verify failing cases retain the requirement ID and never become ready.
- [x] 2.2 Add deterministic probe/evaluation tests for missing tools, ambiguous versions, mismatches, missing skills, wrong scope, dirty trees, nested projects, missing facts, timeouts, and output limits; run the focused tests and confirm they initially expose the missing behavior.
- [x] 2.3 Add run-start transition tests using injected observations and failures; verify refusal/cancellation creates no record, a standalone pass cannot bypass fresh checks, record collisions preserve bytes, and ordinary log conformance stays advisory.
- [x] 2.4 Add release-comparison fixtures for first release, unchanged content, version-only release, changed content without a bump, downgrade, malformed predecessor, rename/removal, missing inventory after adoption, and pre-adoption archives; verify expected decisions are identical in both readers.

## 3. Parse and observe prerequisites

- [x] 3.1 Implement `preflight/v1` parsing from projected requirement notes through the validated constructors; verify parser tests and a projection/formatting round trip preserve the declaration and obligation.
- [x] 3.2 Implement the bounded probe process boundary with closed stdin, captured output, cancellation, and cleanup; verify fake-clock tests plus a real child-process timeout test prove termination without mixing output into CLI stdout.
- [x] 3.3 Implement fixed tool-version, installed-skill, and working-state probes against the invocation context; verify PATH shadowing, skill symlink containment, Git untracked/conflicted files, and the bootstrap-in-an-existing-project regression.
- [x] 3.4 Implement read-only Nix manifest inspection with locked offline evaluation and no import-from-derivation; verify selected-shell inputs are observed without builds, downloads, lock changes, or shell hooks using adapter tests against the supported Nix environment.
- [x] 3.5 Implement Nix executable-provenance comparison against already-realized input closures; verify matching tools pass, PATH shadowing fails, and missing cached inputs or an unsupported manager report unavailable readiness.
- [x] 3.6 Implement pure assessment and typed human/JSON reports in document order; verify mixed pass/fail/unavailable results, zero checks, bounded observations, remedies, and all specified exit-code mappings.

## 4. Expose preflight and integrate run creation

- [x] 4.1 Add `htmlspec preflight <runbook>` in `main.rs` and route existing `--store`/`--json` options; verify CLI help, invalid names, separate store/work directories, parseable JSON on failure, and no prompts in JSON mode.
- [x] 4.2 Load and validate canonical documents, version, and changelog into one source snapshot; verify unreadable or absent specifications cannot silently bypass checks, unversioned starts give migration guidance, and valid zero-check contracts pass.
- [x] 4.3 Require a fresh `ReadyRun` before composition and copy machine-check evidence into matching passed verifications; verify generated records validate and record the version while other log requirements remain `not-run` and canonical tasks remain unchanged.
- [x] 4.4 Publish records completely and without replacement, with cleanup of owned temporary files; verify injected write failures, concurrent same-name starts, crash leftovers, and retry behavior preserve existing records and never expose a partial final document.
- [x] 4.5 Preserve `run --check` and legacy record review without current-machine checks; verify existing conformance/strict-mode tests plus a legacy record lacking version metadata, updating only new-start fixtures for the intentional metadata requirement.

## 5. Enforce runbook releases at publication

- [x] 5.1 Implement canonical-document fingerprinting and typed version/changelog checks using the shared release fixtures; verify records do not affect fingerprints and malformed versions, duplicate headings, and changed content at one version fail.
- [x] 5.2 Implement prior-snapshot discovery ordered by recorded project versions with adoption-marker handling; verify archive renames do not change the predecessor, deleted/renamed runbooks keep readable history, and malformed history fails closed.
- [x] 5.3 Extend archival readiness and commit planning with all runbook comparisons and the staged inventory; verify `archive --check` reports the same refusals as publication and writes nothing.
- [x] 5.4 Publish inventories and the first adoption marker within archival's transaction and define interrupted-publication recovery; verify failure injection and cancellation preserve the active change and prior history, and retry neither overwrites an archive nor fabricates a release.
- [x] 5.5 Extend `scripts/check-release` with the fourth question using the repository's OpenSpec/root-runbooks adapter; verify `--root` fixture tests cover published history and allow unarchived edits whose version increment is still pending.
- [x] 5.6 Add the release-checker regression tests to the normal suite and classify new inventory/adoption files in migration/source filtering where needed; verify the shared fixtures are present in the Nix build source and migration surveys omit no new metadata.

## 6. Migrate examples and document authoring

- [x] 6.1 Give `runbooks/release` its first independent `VERSION` and adoption changelog, with only true entry prerequisites; verify normal runbook validation/formatting and `scripts/check-release`, and ensure later scratch-directory assumptions were not promoted to entry checks.
- [x] 6.2 Add satisfiable and unsatisfiable runbook fixtures under `fixtures/conformance/` with explicit fake host observations in their harness; verify both HTML documents conform while only the satisfied fixture passes preflight, without relying on the test machine's installations.
- [x] 6.3 Document the exact declaration format, probes and limitations, exit codes, metadata migration, and manual receiving layout in the project's CLI/runbook documentation; verify every example parses and references an implemented command.
- [x] 6.4 Update affected archive instruction sources with runbook publication obligations and regenerate their installed copies through the existing path; verify `scripts/check-installed` and keep the workflow count unchanged.

## 7. Verify the complete workflow

- [x] 7.1 Add an integration test that copies a minimal receiving layout, makes the source unavailable, registers/selects the receiving store, and starts from a separate working directory; verify successful and refused starts, version/evidence recording, and one offline asset store without source history.
- [x] 7.2 Run `cargo test -p htmlspec-cli --all-targets --locked`, the release-checker fixtures, and the Nix adapter integration checks; verify all transition and failure cases pass, including no provisioning on unavailable dependencies.
- [x] 7.3 Run `scripts/check` and `nix flake check`, then `openspec validate carry-a-runbook-to-another-machine --strict`; record actual outcomes and any exact unrun command with its reason, and inspect the final diff for unintended schema changes, stale generated files, or lost predecessor logging requirements.
