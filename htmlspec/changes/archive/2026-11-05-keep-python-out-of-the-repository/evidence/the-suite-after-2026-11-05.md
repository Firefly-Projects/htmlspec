# The suite measured after the port

Task 6.5, recorded on 2026-11-05, at `200e8d6`, the head of the sixth pull request's branch once
tasks 6.1 to 6.4 had landed on it, by the method `the-suite-before-2026-10-30.md` gives, on the same
machine. Each number below stands beside the one it replaces. Design decision 18 sets the two goals
they are held against, and neither gates anything.

## The machine

The Apple M5 Max laptop, 18 cores and 64 GB, on macOS 27.2, that measured the suite before. Every
local pass ran in the flake's development shell, so the compiler was rustc 1.90.0 and the per-test
runner cargo-nextest 0.9.136. The shell supplies no interpreter now: `python3` resolved on its search
path only to macOS's own `/usr/bin/python3`, after the shell's entries, and no stage or test runs it.
No wrapper was on `PATH`, and nothing else ran the suite while it was measured.

## The method

As before, with what the port changed:

1. `scripts/check 2>&1` three times, under the shared suite lock, in a worktree of the commit with its
   target directory removed first, so the first pass built from an empty target directory and the
   second and third ran warm, straight after it. Each line was stamped with `$EPOCHREALTIME` as it
   arrived, and a stage's seconds run from its marker to the next, the last one's to the `all 14
   checks passed` line.
2. Then `cargo nextest run --workspace --all-targets --locked --no-fail-fast --status-level all
   --final-status-level none`, and the same for the private workspace with `--manifest-path
   publish/Cargo.toml`.
3. The four Python suites are gone. The cases they held, and those of `scripts/check-prompt` and of
   the release-fixtures stage, are tests now, so they were timed as `cargo test` runs them, in one
   process each, twice: the tools crate's, `cargo test -p htmlspec-xtask`; the prompt's, `cargo test
   -p htmlspec-cli --test prompt`; and the release fixtures' one reader,
   `the_release_fixtures_say_what_this_reader_decides`.

**Continuous integration.** Each stage's seconds from GitHub's timestamps on the stage markers in
`gh run view --log`, for the three latest completed runs of `check.yml` and of
`check-without-nix.yml`: the push of `6a26702` to `main` and the last runs of the fifth pull request,
whose suite this branch leaves as it found it but for task 6.1's flake and the probe runner's clock.
This pull request's own runs are recorded in the pull request. The other jobs' durations are
`completedAt` less `startedAt`.

The nine stages that ran Python were `readme-install`, `prompt`, `installed`, `publish-crates`,
`notices-self-test`, `notices`, `formula-self-test`, `release-fixtures`, and `release`. Four keep
their names and their questions and run the tools crate, and the other five are tests in `test`.

## Every stage, on the laptop

Seconds per stage. Before, the three passes of `the-suite-before-2026-10-30.md` at `09e3a5a`, in
the order its 18 stages ran; after, the three passes at `200e8d6`, whose 14 stages run in the order
`format`, `lint`, `test`, `doctest`, the four `self-` stages, `self-assets`, `no-python`,
`readme-install`, `installed`, `notices`, and `release`.

| Stage | Before, empty | Before, warm | Before, warm | After, empty | After, warm | After, warm |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `format` | 0.6 | 0.4 | 0.4 | 0.6 | 0.5 | 0.5 |
| `lint` | 9.9 | 1.4 | 1.5 | 9.7 | 0.2 | 0.2 |
| `test` | 144.0 | 130.9 | 128.1 | 151.9 | 124.5 | 124.7 |
| `doctest` | 3.6 | 0.4 | 0.5 | 0.7 | 0.7 | 0.7 |
| `self-conformance` | 6.3 | 1.5 | 1.5 | 5.5 | 0.8 | 0.7 |
| `self-format` | 1.4 | 1.4 | 1.4 | 0.5 | 0.5 | 0.5 |
| `self-runbooks` | 3.2 | 3.2 | 3.1 | 2.2 | 2.2 | 2.2 |
| `self-runbooks-format` | 3.1 | 3.1 | 3.1 | 2.1 | 2.2 | 2.2 |
| `self-assets` | 1.8 | 1.8 | 1.7 | 0.8 | 0.9 | 0.8 |
| `no-python` | – | – | – | 2.5 | 0.8 | 0.7 |
| `readme-install` | 1.2 | 1.1 | 1.1 | 0.4 | 0.4 | 0.4 |
| `prompt` | 9.3 | 9.2 | 9.2 | – | – | – |
| `installed` | 2.5 | 1.8 | 1.7 | 1.2 | 1.2 | 1.2 |
| `publish-crates` | 58.7 | 57.8 | 57.7 | – | – | – |
| `notices-self-test` | 0.3 | 0.1 | 0.1 | – | – | – |
| `notices` | 0.5 | 0.4 | 0.5 | 0.7 | 0.7 | 0.7 |
| `formula-self-test` | 2.8 | 1.8 | 2.0 | – | – | – |
| `release-fixtures` | 0.1 | 0.0 | 0.0 | – | – | – |
| `release` | 0.0 | 0.0 | 0.0 | 0.3 | 0.3 | 2.9 |
| **The nine Python stages, and the four that remain of them** | **75.4** | **72.2** | **72.3** | **2.6** | **2.6** | **5.2** |
| The other stages | 173.9 | 144.1 | 141.3 | 176.7 | 133.2 | 133.3 |
| **The pass** | **249.5** | **216.6** | **213.5** | **179.3** | **135.8** | **138.5** |

The third pass's `release` took 2.9 seconds, against 0.3 in the other two. Its four lines were
printed together at the end, so the time went before the tool printed, in `cargo xtask`'s start or
its read of the workspace; it did not recur, and nothing in the stage changed between the passes.
The `self-` stages are faster than before because the program's build script no longer reruns in a
worktree, as `one-reader-2026-11-02.md` records.

## The two goals

**First goal, no test waits on a fixed sleep: met.** Read from the per-test times below, slowest
first, and from the tests' sources. No test sleeps, and none waits on a real-time wait it imposes.
The two that did, both noted before this pull request, are fixed in it:

| Test | Before | After | How |
| --- | ---: | ---: | --- |
| `preflight::host::tests::a_real_child_process_is_stopped_at_the_deadline` | 10.040 | 0.049 | The probe runner reads its deadline from a clock passed in, which says the deadline has passed on its third asking, so a real `sleep 30` is stopped at its deadline without the test waiting the ten seconds out. |
| `preflight::host::tests::a_real_cancellation_stops_a_running_child` | 0.2 of sleep, then the run | 0.047 | The cancellation is raised by the clock the runner asks once it has found the child still running, not by a thread that slept 200 milliseconds first. |

Undoing the first, by making the runner read the real clock again, failed both tests: the deadline's
after 10.00 seconds, its clock asked no times rather than three, and the cancellation's as a timeout,
its flag never raised. The tests' sources hold no `sleep`. What a test still waits for is a bound on a
failure, never a pause: the prompt's 30-second deadline per read, which a passing case meets in under
a second, and the 30 seconds `init_does_not_block_when_no_terminal_is_attached` allows a run that
takes a fraction of one. The slowest tests are slow because they run the binary or `git` many times,
and their files hold no sleep, duration, timeout, or clock. In the program itself, the probe runner
polls a running child every 5 milliseconds and the crate publisher's real clock waits out a 429; both
are the program's behaviour, and no test reaches either wait.

**Second goal, the nine former Python stages' work under 10 seconds on a warm laptop build: met,
at 4.2 and 6.8 seconds**, from 72.2 and 72.3. Each warm pass's number is the four stages that remain,
2.6 and 5.2 seconds, and the cases the other five became, timed in one process as `cargo test` runs
them, 1.58 seconds in the slower of two rounds:

| Cases | Tests | Round 1 | Round 2 |
| --- | ---: | ---: | ---: |
| the tools crate's, `htmlspec-xtask` | 265 | 0.76 | 0.74 |
| the prompt's, `tests/prompt.rs` | 5 | 0.82 | 0.80 |
| the release fixtures' one reader | 1 | 0.00 | 0.00 |

That is an upper bound. The tools crate's 265 tests include the `no-python` stage's own 66, which no
former stage ran, and inside `test` all of them run beside the rest of the workspace's tests, where
they add less than their own time. Under `cargo nextest`, one process per test, the prompt's five
took 1.76 seconds between them and none more than 0.55, and no test of the tools crate took more than
0.77, the slowest being the `no-python` stage's cases that make a scratch repository; the publisher's
slowest, the User-Agent case on the wire, took 0.34.

## Every stage, in continuous integration

Seconds per stage. The first three runs are `check.yml`, in the Nix sandbox: 41929211565, the push
of `6a26702` to `main`, and 41925644249 and 41912998049, the fifth pull request's last two heads. The
last three are `check-without-nix.yml`: 41925644240, 41912997947, and 41678569102, its last three
heads. Before, the six runs of `the-suite-before-2026-10-30.md`, whose nine Python stages took 86.1,
81.5, and 80.1 seconds in the sandbox and 71.6, 69.9, and 69.2 without it.

| Stage | Nix 41929211565 | Nix 41925644249 | Nix 41912998049 | No Nix 41925644240 | No Nix 41912997947 | No Nix 41678569102 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `format` | 0.9 | 1.1 | 0.9 | 1.4 | 0.9 | 1.3 |
| `lint` | 59.7 | 86.5 | 64.3 | 78.7 | 58.1 | 86.7 |
| `test` | 624.0 | 778.4 | 628.9 | 810.2 | 448.3 | 777.8 |
| `doctest` | 1.2 | 1.4 | 1.1 | 1.3 | 0.9 | 1.3 |
| `self-conformance` | 18.3 | 24.4 | 18.3 | 23.1 | 17.9 | 24.4 |
| `self-format` | 0.2 | 0.2 | 0.2 | 0.2 | 0.1 | 0.2 |
| `self-runbooks` | 2.1 | 2.4 | 2.5 | 3.1 | 1.4 | 2.5 |
| `self-runbooks-format` | 2.0 | 2.3 | 2.9 | 3.9 | 1.3 | 2.3 |
| `self-assets` | 0.7 | 0.5 | 0.5 | 0.6 | 0.3 | 0.6 |
| `no-python` | 6.4 | 6.9 | 5.3 | 7.4 | 5.5 | 7.5 |
| `readme-install` | 0.1 | 0.1 | 0.1 | 0.2 | 0.1 | 0.2 |
| `installed` | 0.6 | 1.9 | 0.6 | 0.7 | 0.4 | 0.7 |
| `notices` | 0.6 | 0.7 | 0.5 | 0.8 | 0.5 | 0.8 |
| `release` | 0.1 | 0.1 | 0.1 | 0.2 | 0.1 | 0.2 |
| **The four that remain of the nine** | **1.4** | **2.8** | **1.3** | **1.9** | **1.1** | **1.9** |
| The other stages | 715.4 | 904.3 | 725.0 | 929.7 | 534.7 | 904.5 |
| **The pass** | **716.8** | **907.1** | **726.3** | **931.6** | **535.8** | **906.4** |

Before, the passes took 966.9, 801.8, and 789.0 seconds in the sandbox and 965.2, 950.9, and 976.2
without it. A shared runner's load varies by more than the work the port removed: `test` alone ranges
from 448 to 810 seconds across these six runs.

## The jobs that gain a build

| Job | Before | Run after | After |
| --- | ---: | --- | ---: |
| `pr-guard`, `guard` | 7 | 41925761233 and 41925644151, the fifth pull request's last head | 18 and 20 |
| `publish.yml`, `test` | 31 | 41925644283, that head, its cache restored; 41929211765, the push to `main` | 61; 167 |
| `release.yml`, `verify` | 173 | 41925644060, labelled `release-check` | 375 |
| `release.yml`, `formula` | 33 | 41925644060 | 33 |

The guard installs the toolchain and builds the pull-request rules' own member, where the Python
needed neither; verify builds the tools once for every job after it, as design decision 12 records;
and `publish.yml` builds and tests the private workspace, where the Python ran 65 cases in 40.8
seconds on the laptop.

## Every test, on the laptop

**The workspace.** `cargo nextest` ran 1,967 tests in 51.1 seconds, and all passed, with one skipped,
where it ran 1,670 in 55.1 before. The slowest twenty, in seconds:

| Seconds | Test |
| ---: | --- |
| 18.266 | `htmlspec-cli::order the_listing_shows_each_prerequisites_state` |
| 16.442 | `htmlspec-cli show::tests::the_steps_pair_as_many_as_a_longest_common_subsequence_and_never_cross` |
| 15.212 | `htmlspec-cli::verbs every_get_operation_leaves_the_project_as_it_was` |
| 13.004 | `htmlspec-cli::payloads every_field_a_payload_schema_declares_is_produced` |
| 12.821 | `htmlspec-cli::order the_graph_is_the_same_bytes_every_run_and_every_creation_order` |
| 12.179 | `htmlspec-cli::verbs format_and_put_format_agree` |
| 11.849 | `htmlspec-cli::verbs validate_and_get_validity_agree` |
| 11.112 | `htmlspec-cli::verbs a_withholding_option_writes_nothing_under_the_verb` |
| 10.148 | `htmlspec-cli::runbooks adding_the_fallback_to_a_released_runbook_names_it` |
| 9.933 | `htmlspec-cli::progress the_report_and_preflight_agree_about_every_record` |
| 8.583 | `htmlspec-cli::promote a_systematic_mapping_fault_is_reported_as_a_whole_set` |
| 8.563 | `htmlspec-cli::prerequisites a_chain_of_three_starts_in_order` |
| 8.155 | `htmlspec-validate rules::assets::tests::the_formatter_finds_the_references_these_rules_judge_in_every_repository_document` |
| 7.941 | `htmlspec-cli::order graph_is_refused_alone_and_changes_nothing_under_json` |
| 7.869 | `htmlspec-cli::prerequisites migrate_starts_only_after_a_finished_passed_backup` |
| 7.376 | `htmlspec-cli::order the_graph_draws_each_edge_case` |
| 6.269 | `htmlspec-cli::progress each_run_gets_one_line_in_name_order` |
| 6.259 | `htmlspec-cli::runbooks upgrading_a_released_runbook_names_it_and_says_its_version_must_advance` |
| 6.009 | `htmlspec-cli::prerequisites a_self_reference_or_a_cycle_through_the_runbook_is_an_invalid_contract` |
| 5.798 | `htmlspec-cli::runbooks the_heading_advice_says_which_requests_rewrote_the_runbook` |

They are the before's slowest, in nearly the same order, less
`a_real_child_process_is_stopped_at_the_deadline`, which was tenth at 10.040 seconds and takes 0.049
now. Each runs the program many times over a scratch project.

**The private workspace.** `cargo nextest` ran its 121 tests in 18.7 seconds, where `publish/test`'s 65
cases took 40.8 seconds. Its slowest are the export's cases, which make a source and a target
repository and run the export against them, each running `git` again and again while the others do
the same: `a_second_export_of_one_source_replaces_its_branch` at 13.0 seconds,
`a_delivery_exports_only_a_commit_main_holds` at 13.0, and `a_push_needs_the_tree_a_person_reviewed`
at 11.4. None sleeps. `cargo nextest` marked one of the 121 leaky, as it did one of the 91 in the fifth
pull request, whose review traced it to nextest's own concurrent spawning on macOS.

## What the review was asked to look at, checked without one

The review of pull request 104 was stopped before it reported, and the contributor directed the work
to proceed, so no finding was raised. The three points it had been asked to look at were checked
instead, at `c06c269` and then at the archival's head, by the agent that built the pull request. That
is a self-check and not a review.

**No Python is reachable through the flake's inputs.** On the laptop, on 2026-11-05:

| What | Store paths | Named for Python or PyPy | Holding `bin/python`, `bin/python3`, or `bin/python3.N` |
| --- | ---: | ---: | ---: |
| the development shell's search path, entered with `nix develop --ignore-environment` | 34 | 0 | 0 |
| their runtime closure, `nix path-info -r` | 103 | 0 | 0 |
| the suite derivation's inputs, `nix-store --query --references`, each output present | 26 of 34 | 0 | 0 |
| their runtime closure, `nix-store --query --requisites` | 370 | 0 | 0 |

The eight inputs not present are documentation and headers nothing realised: `bash`'s info, man, doc,
and dev outputs, `bash-interactive`'s info and doc, `zstd`'s man, and `curl`'s devdoc. Inside the
shell so entered, `command -v python3` finds nothing. The derivations that built those inputs do
include Python, 123 of the 1,777 in `nix path-info -r --derivation` of the suite's derivation, as a
compiler's build does, and none of them is an input of the sandbox. Six executables in the suite's
closure begin `#!/usr/bin/env python3`: clang's `analyze-build`, `hmaptool`, `intercept-build`,
`run-clang-tidy`, and `scan-build-py`, and gettext's `spit`. Clang's five are on the development shell's
search path too. With no interpreter in either closure, none of them can start in the sandbox, and in
the shell one started by hand would reach a machine's own `python3`; no file the repository holds runs
any of them. The specification records it as an accepted risk.

**`ALLOWED_LINES` excuses its one line, once, and nothing else.** The judge takes the allowed lines as
a value, so a test can allow a line on which the matcher finds more than a name, and
`another_finding_on_an_allowed_line_is_not_excused` was added: on an allowed text that names
`python3` and hands `tools/run` to bash, the name is excused and the handoff is still refused, and an
allowed text that only hands a file off excuses nothing and is reported unmet. Beside it, the
existing tests hold the line excused once, so a second copy is a finding; the line changed by one
byte a finding, with the allowance unmet; and the same text in `scripts/check` a finding there, with
the flake's allowance unmet. The allowance made to excuse every rule failed
`another_finding_on_an_allowed_line_is_not_excused` and
`the_allowed_line_excuses_a_name_and_nothing_else`. The line's place in `flake.nix` is not pinned,
only its text: the same line moved within the file, at the same indentation, is still excused once.

**The proposal's "What working looks like", item by item:**

| Item | At the head |
| --- | --- |
| The flake names no Python package; the shell adds no `python3` to the path; the sandbox finds none | Met. The flake's one line naming the interpreter is the allowed question, and no Nix identifier holds `python`; the closures above; `nix flake check` passed with the question asked. |
| `scripts/check --list` prints fourteen stages, none running Python; `scripts/` holds `check` and `install-hooks`, both bash | Met. `format`, `lint`, `test`, `doctest`, the four `self-` stages, `self-assets`, `no-python`, `readme-install`, `installed`, `notices`, `release`; both files begin `#!/usr/bin/env bash`. |
| `scripts/tidy.py`, a `python3` first line, or a step running `python3` fails the stage, naming each | Met, with one word differing: the stage says `.github/workflows/check.yml:72: names python3` where the proposal wrote `runs python3`. |
| Deleting an allow-listed `prove-*.py`, a new bash script, or a new hook fails it | Met. |
| The publisher's tests on a fake clock, the prompt's waiting for text, no test sleeping | Met, as the goals above record. |
| `cargo xtask check-release` refuses an inventory entry at `0.1` exactly as archival does | Met. On a scratch copy of the archival's head with one archived inventory entry written `0.1`, both refused it in the same words, "runbook inventory entry 0 is not readable: publish's version is not one: a version is three dot-separated numbers like 0.1.0, and this has 2 component(s)": the release check exiting 1, and `htmlspec archive --check` exiting 2 because "the published runbook history cannot be read". With the entry as it was, archival's check was ready. |
| The release runbook runs `cargo xtask set-version X.Y.Z` and `cargo xtask check-release`; the publish runbook runs `publish/export` and takes the User-Agent from `cargo xtask facts user-agent`; preflight on either checks `cargo` and `rustc` | Met. Both declarations read `"tools":["cargo","rustc"]`, and `htmlspec preflight` reported each runbook ready. |
| `release.yml` holds no `python3`; its Linux jobs run one tools binary, built once in its first job | Met. No line names Python; verify runs `cargo build --locked -p htmlspec-xtask` and uploads `xtask-linux`, which plan, sums, release, and crates take through `.github/actions/xtask`. |
| The suite measured before and after against the two goals | Met, above. |
