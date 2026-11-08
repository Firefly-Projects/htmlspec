# The clocks: the prompt's spike, the ported cases, and the stages they left

Tasks 2.1 to 2.7, recorded on 2026-10-31 on the branch of the second pull request, which ports
`scripts/check-prompt`, `scripts/publish-crates`, and `scripts/check-publish-crates` to Rust and
deletes them. The machine is the one `the-suite-before-2026-10-30.md` names, an Apple M5 Max laptop,
18 cores and 64 GB, on macOS 27.2, with rustc 1.90.0. Nothing here is a goal or a gate: task 6.5
measures the suite after the last pull request, by task 1.2's method, against decision 18's goals.

## Task 2.1: a pseudo-terminal with no controlling terminal drives the prompt

**The question.** `scripts/check-prompt` used Python's `pty.fork`, which gives the child a
controlling terminal. Safe Rust cannot call `setsid` in a child, because `pre_exec` is `unsafe`, so
the port gives the binary a pseudo-terminal on its three standard streams and no controlling
terminal. Design decision 23 held an `unsafe` block in reserve, in its deny form, if that was not
enough.

**The method.** `crates/htmlspec-cli/tests/prompt.rs` opens a pseudo-terminal with `nix`'s
`openpty`, a safe function, at 24 rows by 80 columns, and runs `CARGO_BIN_EXE_htmlspec init` in a
fresh Git repository with its standard input, output, and error on the replica. It reads the
controller with `poll` and waits, for each key, for the text that key paints.

**What the spike found.**

- The prompt reads its keys from standard input when it is a terminal, as `prompt.rs` and
  crossterm 0.25's `tty_fd` both say, so no controlling terminal is needed for any of the four
  cases. **No `unsafe` was needed**, and the workspace's `unsafe_code = "forbid"` stands unchanged:
  decision 23's deny form was not taken.
- inquire redraws only the lines a key changes, so after the first frame nothing repaints the
  question or the help line. A test that waited for a whole frame to repaint waited forever. Each
  key now waits for the line it changes: a space on a checked row paints `> [ ]`, and a step down
  onto a checked row paints `> [x]`.
- inquire breaks a line longer than the window itself, in the middle of a word: at 80 columns the
  help line reads `…without an` and then `y instructions…`. So the cases read the text with every
  line break taken out, and look for nothing a break could split.
- The agent's shell that ran these has no controlling terminal (`tty` prints `not a tty`, and
  opening `/dev/tty` fails with `Device not configured`), so the plain runs are the case CI and the
  sandbox meet: the binary reads the pseudo-terminal's own 80 columns.

**Small windows.** A contributor's terminal is a controlling terminal, and crossterm reads the
window's size from `/dev/tty` when it can open it. `script -q /dev/null` gave the test binary a
controlling terminal, and `stty cols C rows R` set its size before the binary ran. In each, a case
that printed the raw bytes showed the binary wrapping at that width, so it read the small window and
not the pseudo-terminal's:

| Window | The question as painted | The four cases |
| --- | --- | --- |
| none (the plain run) | `Which tools should htmlspec set up?` on one line | passed, 0.72 s |
| 40 × 10 | on one line | passed, 0.36 s |
| 20 × 6 | `Which tools should` / ` htmlspec set up?` | passed, 0.72 s |
| 12 × 4 | `Which tool` / `s should htm` / `lspec set up` / `?` | passed, 0.36 s |

Only the width changes what the prompt writes; its frame's height comes from its own lines.

**The Linux sandbox.** `nix build .#checks.aarch64-linux.default` at `50aba63`, on this machine's
aarch64-linux builder, ran the whole suite in the Nix sandbox, where nothing has a controlling
terminal: all 17 stages passed, the prompt's four cases in 0.64 s and the tools crate's 124 tests in
0.10 s, and `no-python` read 1,654 tracked files from the source tree `HTMLSPEC_SOURCE_TREE` names.
`check-without-nix` passed them on Ubuntu outside Nix in pull request 100's first run, 41249978269.

## The ported cases, pair by pair

Each Python case has a named Rust counterpart. For each group of cases, one deliberate break, made
and then undone, shows its counterparts fail.

### `scripts/check-prompt`, 4 cases, to `crates/htmlspec-cli/tests/prompt.rs`

| Python | Rust |
| --- | --- |
| `a_fresh_repository_opens_with_every_row_checked` | `a_fresh_repository_opens_with_every_row_checked` |
| `an_empty_confirmation_is_refused_and_asks_again` | `an_empty_confirmation_is_refused_and_asks_again` |
| `cancelling_writes_nothing_and_exits_three` | `cancelling_writes_nothing_and_exits_three` |
| `the_flag_still_means_no_tools` | `the_flag_still_means_no_tools` |
| `the_default_binary_follows_cargo_target_dir` | not carried over, as task 2.2 says: `CARGO_BIN_EXE_htmlspec` names the binary |

The break: the prompt's validator made to accept an empty selection. `an_empty_confirmation_is_refused_and_asks_again`
failed with `["init"] exited before it showed "select at least one tool"`, in 0.72 s, not at the
deadline, because the child's exit ends the wait.

### `scripts/check-publish-crates`'s build matrix, 3 cases, to `xtask/src/facts.rs` and `xtask/src/main.rs`

| Python | Rust |
| --- | --- |
| `test_a_run_with_the_macos_jobs_builds_all_four_targets` | `facts::tests::a_run_with_the_macos_jobs_builds_all_four_targets` |
| `test_a_run_without_them_builds_the_two_linux_targets_and_nothing_on_macos` | `facts::tests::a_run_without_them_builds_the_two_linux_targets_and_nothing_on_macos` |
| `test_an_argument_it_does_not_know_is_a_usage_error` | `tests::an_argument_it_does_not_know_is_a_usage_error` |

The break: `ubuntu-24.04` changed to `ubuntu-22.04` in `xtask/release-facts.json`. The first two
failed. Before and after the facts moved into the file, `python3 scripts/release_facts.py matrix`,
`matrix --without-macos`, `targets`, and `user-agent` printed the same bytes, and so did each
`cargo xtask facts` subcommand.

### `scripts/check-publish-crates`'s two guards of the hand-kept list, to `xtask/src/workspace.rs`

| Python | Rust |
| --- | --- |
| `test_a_crate_the_list_does_not_name_stops_the_run_before_it_starts` | `a_member_marked_unpublished_is_left_out`, `a_new_publishable_member_joins_the_order_after_its_dependencies`, `a_cycle_is_refused` |
| `test_the_order_publishes_each_crate_after_its_dependencies` | `this_workspace_publishes_the_six_crates_in_the_order_the_list_kept` |

The break: development dependencies ignored when ordering. Three of the four failed, and seven of
the publisher's cases with them, since `htmlspec-assets` then sorted first.

### `scripts/check-publish-crates`'s publisher, 14 cases, to `xtask/src/publish_crates/tests.rs`

| Python | Rust |
| --- | --- |
| `test_a_rerun_that_finds_three_published_publishes_the_other_three` | `a_rerun_that_finds_three_published_publishes_the_other_three` |
| `test_a_429_answered_once_is_waited_out_and_the_set_finishes` | `a_429_answered_once_is_waited_out_and_the_set_finishes` |
| `test_a_crate_refused_with_429_is_not_built_again` | `a_crate_refused_with_429_is_not_built_again` |
| `test_the_budget_counts_the_time_publishing_takes` | `the_budget_counts_the_time_publishing_takes` |
| `test_a_time_to_try_again_that_has_passed_is_still_waited_for` | `a_time_to_try_again_that_has_passed_is_still_waited_for` |
| `test_words_in_the_build_log_are_not_read_as_answers` | `words_in_the_build_log_are_not_read_as_answers` |
| `test_a_publish_that_failed_after_its_upload_landed_counts_as_published` | `a_publish_that_failed_after_its_upload_landed_counts_as_published` |
| `test_a_wait_past_the_budget_stops_and_names_what_remains_and_a_rerun_finishes` | `a_wait_past_the_budget_stops_and_names_what_remains_and_a_rerun_finishes` |
| `test_the_budget_given_bounds_the_wait` | `the_budget_given_bounds_the_wait` |
| `test_an_index_answering_429_is_waited_out` | `an_index_answering_429_is_waited_out` |
| `test_a_version_cargo_finds_already_published_counts_as_published` | `a_version_cargo_finds_already_published_counts_as_published` |
| `test_any_other_failure_stops_and_names_what_remains` | `any_other_failure_stops_and_names_what_remains` |
| `test_every_request_carries_the_fixed_user_agent_and_nothing_else` | `every_request_carries_the_fixed_user_agent_and_nothing_else`, on the wire, and `cargo::tests::cargo_publishes_with_the_fixed_user_agent_and_the_locked_lockfile` |
| `test_a_workspace_naming_another_repository_publishes_nothing` | `a_workspace_naming_another_repository_publishes_nothing` |

Each runs on the virtual clock and asserts the waits it expects in virtual time: ten seconds for a
429 naming two, the floor of ten for a time already passed, thirty for a 429 naming thirty where the
budget has room, and none where the wait or the work passes the budget. Beside them,
`an_index_that_refuses_after_cargo_failed_stops_the_run_with_its_refusal`, two cases of the plan,
and the wire tests: 200 holding the version, 200 holding others, 404, 429 with seconds, 429 with a
date, 500, and a request closed unanswered, each held to `Host`, `User-Agent`, and `Accept` alone.

The breaks: the wait made without its ten-second floor failed four cases, among them
`a_429_answered_once_is_waited_out_and_the_set_finishes` and `an_index_answering_429_is_waited_out`;
`--fail` added to curl's command line failed six, the five wire tests of a status other than 200
and the command line's own.

## How long they take

At the branch's final code, in the development shell, under the suite lock:

**In one process,** as `cargo test` runs them: the prompt's four cases in 0.79 s in parallel and
1.22 s one at a time; the publisher's, the derivation's, the registry's, and the date's 47 tests in
0.23 s in parallel and 0.47 s one at a time; the publisher's nineteen cases in 0.24 s one at a time.
Earlier runs on the same build gave 0.72 s for the prompt and 0.06 s and 0.15 s for the 46 tests
then written, so these vary with what else the machine is doing.

**One process per test,** under `cargo nextest run`, which pays a process's start for each: 128
tests in 0.87 s. No publisher test over 0.27 s, the User-Agent case on the wire; the other wire
tests 0.08 to 0.16 s; the cases on the fakes 0.03 to 0.04 s each, but for the one that runs `cargo
metadata` on this workspace, 0.08 s. The prompt's cases took 0.53, 0.53, 0.87, and 0.87 s.

**The stages, on a warm laptop build,** in two passes of `scripts/check` by task 1.2's method, at
`67842d5`, beside the two warm passes of `the-suite-before-2026-10-30.md`. The first pass after the
branch's last edits rebuilt what the development shell's environment changed, which is why its
`lint` and `test` are slower.

| Stage | Before, warm | Before, warm | After, first warm | After, warm |
| --- | ---: | ---: | ---: | ---: |
| `format` | 0.4 | 0.4 | 0.4 | 0.5 |
| `readme-install` | 1.1 | 1.1 | 1.2 | 1.1 |
| `lint` | 1.4 | 1.5 | 4.7 | 1.5 |
| `test` | 130.9 | 128.1 | 140.7 | 129.2 |
| `doctest` | 0.4 | 0.5 | 1.2 | 1.3 |
| `self-conformance` | 1.5 | 1.5 | 1.9 | 1.6 |
| `self-format` | 1.4 | 1.4 | 1.5 | 1.5 |
| `self-runbooks` | 3.2 | 3.1 | 3.1 | 3.2 |
| `self-runbooks-format` | 3.1 | 3.1 | 3.1 | 3.1 |
| `self-assets` | 1.8 | 1.7 | 1.8 | 1.8 |
| `no-python` | – | – | 0.8 | 0.6 |
| `prompt` | 9.2 | 9.2 | – | – |
| `installed` | 1.8 | 1.7 | 1.9 | 1.8 |
| `publish-crates` | 57.8 | 57.7 | – | – |
| `notices-self-test` | 0.1 | 0.1 | 0.2 | 0.1 |
| `notices` | 0.4 | 0.5 | 0.4 | 0.4 |
| `formula-self-test` | 1.8 | 2.0 | 2.1 | 1.8 |
| `release-fixtures` | 0.0 | 0.0 | 0.1 | 0.0 |
| `release` | 0.0 | 0.0 | 0.0 | 0.0 |
| **The pass** | **216.6** | **213.5** | **165.1** | **149.6** |

The two stages that left took 67.0 and 66.9 s. Their cases run inside `test` now, which took 129.2
s against 130.9 and 128.1, because they run beside the rest of the workspace's tests and finish in
under a second. The seven Python stages that remain took 5.2 s on the warm pass. So the work the nine
former Python stages did took about 6 s here, against 72.2 and 72.3 at the base; task 6.5 measures
it formally after the last pull request. The before passes had no `no-python` stage, which the first
pull request added.

## The ratchet

Three Python files are deleted, `scripts/check-prompt` (316 lines), `scripts/publish-crates` (398),
and `scripts/check-publish-crates` (457), and their three program entries leave `xtask/python-debt`,
with the pinned comment line of `scripts/check` that said the prompt could not be a `cargo test`.
The lists held 40 entries and hold 36: in `xtask/python-debt`, 13 programs (twelve Python files and
the bash `scripts/check-formula`) and 16 lines; in `publish/no-python`, 4 programs, 2 lines, and the
permanent shell entry for `publish/export`. Sixteen Python files remain, twelve under `scripts/` and
four under `publish/`.

Three pins moved with their edits: `scripts/release_facts.py`, which reads the facts file and lost
`CRATES` and `PUBLIC_REPOSITORY`; `scripts/check-release`, whose docstring stopped naming
`check-prompt`; and the flake's two pinned lines, the suite's `nativeBuildInputs`, which gained
`pkgs.curl`, and the comment above it, which no longer names the prompt or the publisher.

Group 1's negative table, rerun at `67842d5` with the debt lowered, fails the stage in every case it
failed before and passes the untracked file, and `scripts/publish-crates` put back as it was is
refused by its name, its first line, its place under `scripts/`, and its mode.

## After the review

Pull request 100's review found no path that could publish when it should not, publish twice, or
loop, and named findings that were settled at `df468c7`, each recorded in design decisions 5, 7, 8,
and 12 and in the tasks' deviations:

- **How a run ends is tested.** `Closing` holds a run's exit code and every line it prints. Its
  cases hold 0 with `every crate is published at 9.8.7`; 1 with `publish-crates: stopped: …` and
  `publish-crates: 3 crate(s) remain: htmlspec-format, htmlspec-validate, htmlspec-cli. Run this
  again …`; 1 for a plan that `has no answer for 3 crate(s)`; and 2 for a refused repository, which
  a publish now gives before it prints a line, and for a release refused at its start. A stopped
  run made to exit 0, for the check, failed two of them.
- **A crate that depends on an unpublished member refuses the release.** With `htmlspec-format`
  marked `publish = false`, the release is refused naming `htmlspec-cli` and `htmlspec-validate`,
  and the run exits 2 before it asks the index anything. A development dependency named by path
  alone, which `cargo publish` strips, orders nothing.
- **The registry.** A redirect is refused, its status named; a 301 the listener sent was not
  followed. All three HTTP date forms are read, an RFC 850 year resolved by RFC 9110's rule. A 429
  with `X-Junk: caf\xe9` beside `Retry-After: 17` now reads as 17 seconds. Each answer goes to a new
  directory at mode 0700. The index's and cargo's failures are typed, and the repository a manifest
  names compares as a GitHub owner and name.
- **The budget** is at most 3600 seconds, the command line refusing more.
- **The plan** runs in its own job, `plan`, which nothing needs, with `--budget 120`.
- **The prompt's test** bounds every wait for a process, kills a child that passes the deadline,
  gives every other process nothing on standard input, and adds a case in a window 12 columns wide.
  With no controlling terminal it holds the question to have been painted broken; opened at 80
  columns for the check, that case failed. The five cases passed five runs in a row, each in 0.75
  to 0.77 s.

**The times at `df468c7`**, in the development shell under the suite lock: the prompt's five cases
in 0.79 s in parallel and 1.54 s one at a time; the publisher's, derivation's, registry's, and
date's 60 tests in 0.22 s in parallel and 0.50 s one at a time; the publisher's 24 cases on the fakes
and the wire in 0.23 s one at a time. Under `cargo nextest`, 143 tests in 0.85 s: no publisher test
over 0.26 s, and the prompt's cases 0.49, 0.50, 0.84, 0.85, and 0.85 s. Two warm passes of
`scripts/check` passed all 17 stages in 152.5 and 150.0 s, `test` taking 130.7 and 129.3 s.
