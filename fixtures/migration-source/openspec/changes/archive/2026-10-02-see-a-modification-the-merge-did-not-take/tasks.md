## 1. Re-measure before changing anything

- [x] 1.1 Repeat the measurement in `evidence/the-measurement-2026-09-30.md` against HEAD, with the scripts in its appendices and `./target/debug/htmlspec`. Done when every step is recorded with its exit statuses and payloads, and compared with the numbers there: `req-x` reached at every step; `req-y` reached in step 3a and absent in steps 3b, 4, and 5; step 6 reporting "applied in full" and "ready"; step 5 reporting "This change has not been synced" with `needsSync: true`; and `stated` holding `req-x`, `req-y`, and `req-z` in step 4. Any difference is recorded, not smoothed over
- [x] 1.2 Confirm that `state-a-modified-requirement-whole` is archived, and that `openspec/specs/artifact-kinds/spec.md` holds "A change's specification states a modified requirement whole". This change follows that one and relies on its rule. Then re-read `crates/htmlspec-assets/instructions/sync.md` and `archive.md` as it left them, and list every sentence that names a set `htmlspec sync` reports, or says what happens to a withdrawn requirement. Done when the list is recorded in `evidence/`, and the lines tasks 4.1 and 4.2 cite are corrected against it
- [x] 1.3 Check that no other active change modifies any of the four toolchain-packaging requirements this change modifies, with `grep -rl` over `openspec/changes/*/specs/toolchain-packaging/spec.md` for each heading. Done when the result is recorded. If one does, add a task "1.0 Re-base" here before any other work

## 2. Types and tests first

- [x] 2.1 In `crates/htmlspec-cli/src/commands.rs`, add the types design decisions 1, 5, and 6 name, with `merge_state` still leaving the new sets empty, so that the tests below compile and fail on their assertions rather than on the build: the closed three-case type for a requirement's declared state; `UntakenContent`; `MergeState::untaken_content`, `untaken_removals`, and `reached_in_full`; and `Findings` carrying the `MergeState` in place of `unmerged` and `untaken_renames`. Done when `nix develop -c cargo test -p htmlspec-cli` passes except for the tests 2.2 to 2.6 say fail on HEAD
- [x] 2.2 Add unit tests for `merge_state`, one per new scenario in "The command surface reports whether a change reached the accepted contract". Extend `specification_with` so a fixture can state statements, scenarios, notes, and change states. Done when each is recorded failing or passing, as stated, on HEAD with 2.1's types in place:
  - failing on HEAD: `a_word_the_accepted_specifications_have_not_taken_is_not_reached`, `a_scenario_the_accepted_specifications_have_not_taken_is_named_by_its_identifier`, `a_withdrawn_scenario_the_accepted_requirement_still_holds_is_not_kept`, `statements_in_another_order_are_not_taken`, `an_added_requirement_is_compared_as_a_modified_one_is`, `a_removal_the_merge_applied_is_reached`, `a_removal_the_merge_did_not_apply_is_an_untaken_removal`, and `a_rename_and_a_modification_neither_taken_are_both_reported`;
  - passing on HEAD, and kept as guards against an over-strict comparison: `scenario_order_and_notes_do_not_decide_reaching`, `a_requirement_carried_forward_is_compared_by_its_identifier`, and `a_modified_requirement_whose_identifier_is_not_held_is_absent_only`
- [x] 2.3 Extend `every_stated_requirement_is_in_exactly_one_set` to design decision 4's partition. Its fixture gains a modification taken, one not taken, a withdrawal applied, one not applied, a carried-forward requirement whose text differs, and one requirement both renamed and modified with neither taken. Extend `archival_and_sync_name_the_same_untaken_renames` into `archival_and_sync_name_the_same_untaken_work`, over all four sets. Add a withdrawal the merge applied to the fixture of `archival_and_sync_name_the_same_absent_requirements`, so that parity also shows it absent from neither. Done when all three fail on HEAD
- [x] 2.4 Add findings and payload tests beside `a_change_whose_only_unapplied_work_is_a_rename_is_unsynced` and `the_payload_carries_each_untaken_rename`: `a_change_whose_only_unapplied_work_is_a_word_is_unsynced`, `a_change_whose_only_unapplied_work_is_a_withdrawal_is_unsynced`, `the_payload_carries_where_content_differs_and_no_text`, which asserts that no statement or scenario text of either side appears in the payload, and `the_question_counts_every_untaken_set`. Done when each fails on HEAD
- [x] 2.5 Add end-to-end tests to `crates/htmlspec-cli/tests/archive.rs`, modeled on `a_change_whose_only_unapplied_work_is_a_rename_has_not_been_synced`. Done when all three fail on HEAD:
  - `a_change_whose_only_unapplied_work_is_a_word_has_not_been_synced` covers `sync` human and `--json`, and `archive --check` human and `--json`. It then merges the word and shows the finding clears.
  - `a_withdrawal_the_merge_applied_has_reached_the_contract` shows `sync` reporting the requirement as reached, and archival reporting nothing about syncing.
  - `a_withdrawal_the_merge_did_not_apply_has_not_been_synced`.
- [x] 2.6 Add `every_set_the_sync_payload_carries_is_named_in_its_instructions` to `crates/htmlspec-cli/tests/archive.rs`, as design decision 7 describes. Done when it passes on HEAD, fails once 3.3 adds the two keys, and passes again after 4.1

## 3. The comparison

- [x] 3.1 Make `merge_state` read each stated requirement's declared state, and keep each accepted requirement's heading name and content in place of the heading name alone. Judge the requirement by design decision 1's table. Compare content as decision 2 defines it, through one function returning `Option<UntakenContent>`. Leave `untaken_rename` unchanged. Done when the tests in 2.2 and 2.3 pass
- [x] 3.2 Make `MergeState::reached_in_full` the one statement of "the contract arrived in full". Have `sync`'s "applied in full" line (2475) and `Findings::needs_sync` both call it, and leave no other expression of that condition in the file. Done when `grep -n "absent.is_empty()" crates/htmlspec-cli/src/commands.rs` finds no such conjunction outside `reached_in_full` and the tests, and 2.4 passes
- [x] 3.3 In `sync`, add `untakenContent` and `untakenRemovals` to the payload (2439-2449), with decision 5's entry shape, and add the `differs` and `held` lines and one note per set to the human report. Change the absent note's "confirm it was withdrawn deliberately" to "abandoned". Done when 2.5's `sync` assertions pass
- [x] 3.4 In archival, extend `report_findings`, `question_for`, and `emit_archive_payload` to the two sets, with the notes decision 5 words, and make the same wording change to the absent note. Done when 2.4 and 2.5 pass
- [x] 3.5 Update the `sync` help in `crates/htmlspec-cli/src/main.rs` ("It compares requirement identifiers") and the `archive --yes` help ("Incomplete tasks and requirements that reached no accepted specification"), so that each says what is compared and reported. Done when `./target/debug/htmlspec sync --help` and `archive --help` read correctly
- [x] 3.6 Add the new comparison function to the list in `the_archive_check_reads_only_the_specification` in `crates/htmlspec-cli/tests/archive.rs`, so it is held to reading no proposal or design. Done when that test passes

## 4. The instructions

- [x] 4.1 In `crates/htmlspec-assets/instructions/sync.md`, make the edits design decision 7 lists: the table and the partition sentence; the modified-requirement section working through `untakenContent`; a new "Merging a withdrawal" section; "A requirement you should not merge"; "Finish by asking again"; and "What to report". Done when 2.6 passes and `./target/debug/htmlspec instructions sync` names all six payload sets
- [x] 4.2 In `archive.md`, add the two findings to step 4, correct step 4's sentence that each absent identifier left after the merge "should be a requirement the change deliberately withdrew", and say in step 5's withdrawal bullet that archival then counts the requirement as reached. Add the two sets to step 6's list of what archival still asks about. Done when `grep -n "deliberately withdrew" crates/htmlspec-assets/instructions/archive.md` prints nothing and `instructions archive` names both new sets
- [x] 4.3 Confirm that `new.md`, `continue.md`, `ff.md`, `apply.md`, `verify.md`, `explore.md`, and `migrate.md` need no edit, with `grep -n "absent\|untakenRenames\|sync --change\|withdrawn"` and a read of each hit. Done when each is recorded in `evidence/` as silent or edited, with the reason
- [x] 4.4 Regenerate the installed copies with `./target/debug/htmlspec init --tools claude,codex,pi`, remove the stray `htmlspec/` and `spec.nix` it creates, and run `scripts/check-installed`. Done when it passes and `git status` shows exactly the ten `sync` and `archive` files changed under `.claude/`, `.codex/`, and `.pi/`

## 5. Prove it

- [x] 5.1 Repeat 1.1 against the built change. Done when the results are recorded in `evidence/`, and they show:
  - step 1: `req-x` in `untakenContent` from paragraph 1, and "has not been synced";
  - step 2: the same entry, with `scenario-x-c` not held;
  - step 3a: `req-y` in `untakenRemovals`;
  - step 3b: `req-y` reached;
  - step 4: `req-x` in `untakenContent`, and `req-y` and `req-z` reached;
  - step 5: "applied in full" and "ready";
  - step 6: `req-x` in `untakenContent`, `req-y` in `untakenRemovals`, `req-z` reached, and "has not been synced".
- [x] 5.2 Mutation-test the change, restoring each file after each run, and record which test catches each mutation. Done when every mutation is caught, or a missing test is added:
  1. Compare `modified` requirements by identifier only.
  2. Drop `added` from the compared states.
  3. Compare `unchanged` requirements' content too.
  4. Count a `removed` requirement as reached when it is held, as HEAD does.
  5. Compare scenarios in order.
  6. Compare statements as a collection.
  7. Include notes in the content.
  8. Stop leaving scenarios marked `removed` out of the change's side.
  9. Leave `untaken_content` out of `reached_in_full`.
  10. Leave `untaken_removals` out of `reached_in_full`.
  11. Put statement text into a payload entry.
  12. Report a requirement both renamed and modified in the rename set only.
  13. Remove `untakenContent` from `sync.md`.
- [x] 5.3 Run `cargo test --workspace --all-targets`, then `scripts/check`. Done when every stage is green
- [x] 5.4 Record what the implementation revealed in `evidence/what-the-implementation-revealed-<date>.md`, including any case where the comparison and the merging agent's reading disagreed

## 6. Release

- [x] 6.1 Run the release runbook at `runbooks/release/tasks.html`, and report whether it worked as written
- [x] 6.2 Choose the increment and state the reasoning. `0.21.0`, a minor increment after `state-a-modified-requirement-whole`'s `0.20.0`, is expected, because `sync` and archival report findings they did not report before, and four accepted requirements change. The contract version stays `0.3.0`. Flag the choice for the contributor's confirmation
- [x] 6.3 Check that `scripts/check-release` passes after archival writes `VERSION` and the hand-written `runbooks.json` beside it
  - `openspec archive see-a-modification-the-merge-did-not-take --yes` (OpenSpec 1.13.1) archived the change as `2026-10-02-see-a-modification-the-merge-did-not-take`, continuing past this one open task. It modified four requirements in `openspec/specs/toolchain-packaging/spec.md`, and each merged block equals the delta's word for word. No other requirement changed
  - `VERSION` holds `0.21.0`. `runbooks.json` is a copy of the previous archive's, because the release runbook is unchanged: the sha256 of `runbooks/release/specification.html` and `tasks.html` still match the inventory's
  - `scripts/check-release`: `ok 27 archived change(s) record a version, and 21 version(s) are in the changelog` and `ok declared version 0.21.0 has a changelog entry`

## 7. Review

- [x] 7.1 Resolve what a review of the pull request found, and record what was and was not taken. Settle explicitly, with the contributor, the three decisions the design marks or leans on: scenarios compared by text rather than identifier, and so whether `state-a-modified-requirement-whole`'s identifier sentence still has a reason to stay (design decision 2); a requirement renamed and modified reported in both sets (decision 4); and an `added` requirement compared as a `modified` one is (decision 1)
  - **Settled with the contributor on 2026-10-02, all four accepted as built.** Seven scratch projects, run through the binaries built from this branch and from `main`, were the evidence:
    - **How `sync.md` reads `scenariosNotKept`.** A withdrawn scenario and an omitted one produce the same entry, `differs req-x (not kept: scenario-x-b)`, so only the change's specification tells them apart. A scenario it marks `removed` comes out. The accepted text of a scenario it rewords comes out, because the reworded scenario appears in both lists under one identifier. A scenario it never states is reported and holds the requirement, as artifact-kinds' "A change's specification states a modified requirement whole" requires. Design decision 7's "or does not state" is superseded by this reading
    - **Decision 2, scenarios compared by text.** A correct merge that wrote an added scenario under `scenario-x-c-2`, because another requirement in the accepted document already carried `scenario-x-c`, reads as reached. A comparison by identifier would report it as not taken. `state-a-modified-requirement-whole`'s identifier sentence stays, because the merging agent uses it to find which accepted scenario a stated one continues
    - **Decision 4, a requirement renamed and modified reported in both sets.** On `main` such a requirement showed only `renamed req-x`, and the word it added was invisible. On this branch it shows `renamed` and `differs` together, in one run
    - **Decision 1, an `added` requirement compared as a `modified` one is.** On `main`, an added requirement merged without one of its scenarios, and one whose identifier an unrelated accepted requirement already carried, both read "applied in full". On this branch both are reported
  - **Taken, from the code review of PR #42: a scenario changed inside a requirement marked `unchanged` was never compared.** The requirement was judged by its own mark alone, so it was carried forward and compared by identifier. A scenario marked `added` inside it, which the accepted requirement never took, left `sync` saying "applied in full" and archival saying "ready". Reproduced with the built binary before the fix. Nothing in validation or the contract requires such a requirement to be marked `modified`. `Declared::of_requirement` now judges it as one the change modifies when any scenario in it is one the change adds, modifies, or withdraws. The spec delta's `unchanged` bullet says so and gains the scenario "A scenario changed inside a requirement marked unchanged". Design decision 1 records the case and the validation rule it rejected, and `sync.md`'s table and the changelog say the same. `a_scenario_changed_inside_an_unchanged_requirement_is_compared` and the end-to-end `a_scenario_added_inside_an_unchanged_requirement_has_not_been_synced` fail without the fix
