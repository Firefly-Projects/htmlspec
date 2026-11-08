# The suite measured before anything changed

Task 1.2, recorded on 2026-10-30, at `09e3a5a`, this change's base on `main`, before any of its
work. Task 6.5 repeats every measurement below by the same method, on the same machine, and sets
each number beside the one it replaces. Nothing here is a goal or a gate: design decision 18 sets
the two goals, and neither gates anything.

## The machine

An Apple M5 Max laptop, 18 cores and 64 GB, on macOS 27.2, the machine the explainer behind this
change measured. Every local pass ran in the flake's development shell, so the compiler was rustc
1.90.0 and the interpreter Python 3.13.14, the flake's own. No wrapper was on `PATH`, and nothing
else ran the suite while it was measured.

## The method

**Local passes.** In a clean Git worktree of the commit, holding no `target/` directory, under the
shared suite lock (a directory made with `mkdir`, holding the measuring shell's process id, and
waited on while another holds it), one harness ran in `nix develop -c`:

1. `scripts/check 2>&1` three times. The first pass ran after `rm -rf target`, so it built from an
   empty target directory. The second and third ran warm, straight after it.
2. Each line of a pass's output was stamped with the wall clock as it arrived, by bash's
   `$EPOCHREALTIME` in a `while IFS= read -r line` loop on the pipe. A stage's seconds run from its
   `=== name ===` marker to the next marker, and the last stage's to the `all 18 checks passed`
   line. A pass's total runs from its first marker to that line.
3. Then `cargo nextest run --workspace --all-targets --locked --no-fail-fast --status-level all
   --final-status-level none`, read for each test's time from its `PASS [ seconds ]` line.
4. Then three of the four Python suites, each run directly with unittest's `--durations 0`:
   `python3 publish/test`, `python3 scripts/check-crate-notices`, and
   `python3 scripts/check-publish-crates`. `scripts/check-check-formula` was not run on its own,
   because this session runs nothing that could reach the machine's Homebrew outside the suite. Its
   cases' times are the differences between the stamps on its result lines in the second pass.
   `scripts/check-prompt` buffers its output until it ends, so its cases cannot be told apart, and
   only its stage time is given.

**Continuous integration.** `gh run view --log` of the three latest completed runs of `check.yml`,
which runs the suite in the Nix sandbox, and of `check-without-nix.yml`, with each stage's seconds
taken from GitHub's timestamps on the same markers, the same way. The other jobs' durations are
`completedAt` less `startedAt` from `gh run view --json jobs`, for the latest run of `pr-guard.yml`
and `publish.yml`, and the latest run of `release.yml` that ran its macOS jobs.

The nine stages that run Python are `readme-install`, `prompt`, `installed`, `publish-crates`,
`notices-self-test`, `notices`, `formula-self-test`, `release-fixtures`, and `release`.

## Every stage, on the laptop

Seconds per stage, in the suite's order.

| Stage | Empty target | Warm | Warm |
| --- | ---: | ---: | ---: |
| `format` | 0.6 | 0.4 | 0.4 |
| `readme-install` | 1.2 | 1.1 | 1.1 |
| `lint` | 9.9 | 1.4 | 1.5 |
| `test` | 144.0 | 130.9 | 128.1 |
| `doctest` | 3.6 | 0.4 | 0.5 |
| `self-conformance` | 6.3 | 1.5 | 1.5 |
| `self-format` | 1.4 | 1.4 | 1.4 |
| `self-runbooks` | 3.2 | 3.2 | 3.1 |
| `self-runbooks-format` | 3.1 | 3.1 | 3.1 |
| `self-assets` | 1.8 | 1.8 | 1.7 |
| `prompt` | 9.3 | 9.2 | 9.2 |
| `installed` | 2.5 | 1.8 | 1.7 |
| `publish-crates` | 58.7 | 57.8 | 57.7 |
| `notices-self-test` | 0.3 | 0.1 | 0.1 |
| `notices` | 0.5 | 0.4 | 0.5 |
| `formula-self-test` | 2.8 | 1.8 | 2.0 |
| `release-fixtures` | 0.1 | 0.0 | 0.0 |
| `release` | 0.0 | 0.0 | 0.0 |
| **The nine Python stages** | **75.4** | **72.2** | **72.3** |
| The other nine | 173.9 | 144.1 | 141.3 |
| **The pass** | **249.5** | **216.6** | **213.5** |

So on a warm build the nine Python stages took 72.2 and 72.3 seconds, a third of the pass, inside
the explainer's 69 to 83. Design decision 18's second goal holds this number, the sum for the nine,
against 10 seconds.

## Every stage, in continuous integration

Seconds per stage. The first three runs are `check.yml`, in the Nix sandbox: 41145897950, the push
of `09e3a5a` to `main`, and 41143870939 and 40990131617, two pull request runs of this change's
plan. The last three are `check-without-nix.yml`: 40907725211 and 40899635859, pull request runs of
the branch that merged as `a11ff12`, and 40815691779, a pull request run from before the `formula-self-test` stage
existed, so its pass has 17 stages. A dash is a stage the pass did not have.

| Stage | Nix 41145897950 | Nix 41143870939 | Nix 40990131617 | No Nix 40907725211 | No Nix 40899635859 | No Nix 40815691779 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `format` | 0.9 | 0.7 | 0.7 | 1.5 | 1.1 | 1.1 |
| `readme-install` | 1.2 | 1.0 | 1.0 | 0.9 | 0.9 | 1.0 |
| `lint` | 74.2 | 57.6 | 53.5 | 79.2 | 76.4 | 81.7 |
| `test` | 739.5 | 610.3 | 604.6 | 770.4 | 765.4 | 782.8 |
| `doctest` | 8.8 | 6.7 | 6.4 | 10.1 | 9.3 | 10.1 |
| `self-conformance` | 22.7 | 17.6 | 17.0 | 25.8 | 23.8 | 26.1 |
| `self-format` | 7.6 | 5.9 | 5.6 | 0.2 | 0.2 | 0.2 |
| `self-runbooks` | 9.8 | 7.7 | 7.6 | 2.5 | 2.2 | 2.2 |
| `self-runbooks-format` | 9.6 | 7.7 | 7.5 | 3.1 | 2.0 | 2.1 |
| `self-assets` | 7.8 | 6.1 | 6.0 | 0.8 | 0.5 | 0.6 |
| `prompt` | 15.5 | 13.8 | 13.5 | 8.3 | 8.2 | 8.4 |
| `installed` | 7.9 | 6.4 | 6.0 | 0.6 | 0.6 | 0.6 |
| `publish-crates` | 57.8 | 57.4 | 56.9 | 58.1 | 57.8 | 58.3 |
| `notices-self-test` | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 | 0.1 |
| `notices` | 0.7 | 0.5 | 0.5 | 0.8 | 0.7 | 0.7 |
| `formula-self-test` | 2.8 | 2.1 | 2.0 | 2.6 | 1.5 | – |
| `release-fixtures` | 0.0 | 0.0 | 0.0 | 0.1 | 0.0 | 0.0 |
| `release` | 0.1 | 0.2 | 0.1 | 0.1 | 0.1 | 0.1 |
| **The nine Python stages** | **86.1** | **81.5** | **80.1** | **71.6** | **69.9** | **69.2** |
| The other stages | 880.9 | 720.3 | 708.9 | 893.6 | 880.9 | 906.9 |
| **The pass** | **966.9** | **801.8** | **789.0** | **965.2** | **950.9** | **976.2** |

## The jobs that gain a build

| Job | Run | Seconds |
| --- | --- | ---: |
| `pr-guard`, `guard` | 41143922872 | 7 |
| `publish.yml`, `test` | 41145898023 | 31 |
| `release.yml`, `verify` | 40907725143, labelled `release-check` | 173 |
| `release.yml`, `formula` | 40907725143 | 33 |

## Every test, on the laptop

**The Rust tests.** `cargo nextest` ran 1,670 tests in 55.1 seconds, and all passed, with one skipped.
The slowest twenty, in seconds:

| Seconds | Test |
| ---: | --- |
| 18.069 | `htmlspec-cli::order the_listing_shows_each_prerequisites_state` |
| 15.460 | `htmlspec-cli::verbs every_get_operation_leaves_the_project_as_it_was` |
| 15.394 | `htmlspec-cli::bin/htmlspec show::tests::the_steps_pair_as_many_as_a_longest_common_subsequence_and_never_cross` |
| 12.687 | `htmlspec-cli::payloads every_field_a_payload_schema_declares_is_produced` |
| 12.531 | `htmlspec-cli::order the_graph_is_the_same_bytes_every_run_and_every_creation_order` |
| 12.272 | `htmlspec-cli::verbs format_and_put_format_agree` |
| 12.145 | `htmlspec-cli::verbs validate_and_get_validity_agree` |
| 11.164 | `htmlspec-cli::verbs a_withholding_option_writes_nothing_under_the_verb` |
| 10.074 | `htmlspec-cli::runbooks adding_the_fallback_to_a_released_runbook_names_it` |
| 10.040 | `htmlspec-cli::bin/htmlspec preflight::host::tests::a_real_child_process_is_stopped_at_the_deadline` |
| 9.752 | `htmlspec-cli::progress the_report_and_preflight_agree_about_every_record` |
| 8.461 | `htmlspec-cli::promote a_systematic_mapping_fault_is_reported_as_a_whole_set` |
| 8.423 | `htmlspec-cli::prerequisites a_chain_of_three_starts_in_order` |
| 8.019 | `htmlspec-validate rules::assets::tests::the_formatter_finds_the_references_these_rules_judge_in_every_repository_document` |
| 7.833 | `htmlspec-cli::order graph_is_refused_alone_and_changes_nothing_under_json` |
| 7.636 | `htmlspec-cli::prerequisites migrate_starts_only_after_a_finished_passed_backup` |
| 7.348 | `htmlspec-cli::order the_graph_draws_each_edge_case` |
| 6.161 | `htmlspec-cli::runbooks upgrading_a_released_runbook_names_it_and_says_its_version_must_advance` |
| 6.051 | `htmlspec-cli::progress each_run_gets_one_line_in_name_order` |
| 5.956 | `htmlspec-cli::prerequisites a_self_reference_or_a_cycle_through_the_runbook_is_an_invalid_contract` |

One of them is a Rust test that waits on a real clock: `a_real_child_process_is_stopped_at_the_deadline`
takes 10.0 seconds, which is the deadline it exercises. Decision 18's first goal reads the slowest
tests of the after-measurement, so task 6.5 has this one to answer for as well as the Python's.

**`scripts/check-publish-crates`**, 19 cases in 58.2 seconds:

| Seconds | Case |
| ---: | --- |
| 10.863 | `test_a_429_answered_once_is_waited_out_and_the_set_finishes` |
| 10.815 | `test_an_index_answering_429_is_waited_out` |
| 10.798 | `test_a_crate_refused_with_429_is_not_built_again` |
| 10.785 | `test_every_request_carries_the_fixed_user_agent_and_nothing_else` |
| 6.779 | `test_the_budget_counts_the_time_publishing_takes` |
| 1.110 | `test_a_wait_past_the_budget_stops_and_names_what_remains_and_a_rerun_finishes` |
| 0.842 | `test_the_budget_given_bounds_the_wait` |
| 0.805 | `test_words_in_the_build_log_are_not_read_as_answers` |
| 0.775 | `test_a_version_cargo_finds_already_published_counts_as_published` |
| 0.772 | `test_a_publish_that_failed_after_its_upload_landed_counts_as_published` |
| 0.763 | `test_a_time_to_try_again_that_has_passed_is_still_waited_for` |
| 0.763 | `test_any_other_failure_stops_and_names_what_remains` |
| 0.720 | `test_a_rerun_that_finds_three_published_publishes_the_other_three` |
| 0.511 | `test_the_order_publishes_each_crate_after_its_dependencies` |
| 0.511 | `test_a_crate_the_list_does_not_name_stops_the_run_before_it_starts` |
| 0.510 | `test_a_workspace_naming_another_repository_publishes_nothing` |
| 0.039 | `test_an_argument_it_does_not_know_is_a_usage_error` |
| 0.027 | `test_a_run_with_the_macos_jobs_builds_all_four_targets` |
| 0.026 | `test_a_run_without_them_builds_the_two_linux_targets_and_nothing_on_macos` |

The five slowest spend 50.0 seconds between them, the waits after an HTTP 429 the explainer found.

**`scripts/check-crate-notices`**, 28 cases in 0.031 seconds. The slowest took 0.003 seconds.

**`scripts/check-check-formula`**, 5 cases in 1.752 seconds, by the stamps in the second pass:
0.43 seconds to the first result line, the interpreter's start included, then 0.29, 0.30, 0.39,
and 0.37.

**`scripts/check-prompt`**, 4 cases, 9.2 seconds as a stage on a warm build. Its cases are not
separable here, as the method says.

**`publish/test`**, 65 cases in 40.8 seconds. It is not a stage of the suite: `publish.yml` runs it.
The slowest ten:

| Seconds | Case |
| ---: | --- |
| 3.229 | `test_a_github_delivery_names_the_account_before_it_pushes` |
| 2.356 | `test_a_second_export_of_one_source_replaces_its_branch` |
| 2.332 | `test_a_delivery_exports_only_a_commit_main_holds` |
| 2.252 | `test_a_target_off_github_is_pushed_as_before_without_asking_gh` |
| 1.913 | `test_a_push_needs_the_tree_a_person_reviewed` |
| 1.566 | `test_no_export_commit_reaches_the_source_history` |
| 1.548 | `test_a_push_needs_the_contributors_sign_off` |
| 1.507 | `test_push_lands_the_export_as_a_branch_on_top_of_main` |
| 1.488 | `test_an_export_of_a_full_commit_id_is_titled_within_the_guards_rules` |
| 1.477 | `test_an_account_switched_before_the_push_is_refused` |
