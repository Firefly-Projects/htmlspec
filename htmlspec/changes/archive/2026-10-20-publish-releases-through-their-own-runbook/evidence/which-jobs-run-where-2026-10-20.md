# Which release jobs run where

Task 1.4, recorded on 2026-10-20, for `.github/workflows/release.yml` as design decision 9 left
it, and amended the same day for the code review's finding on labels, task 5.1. It is a reading
of the conditions, checked by actionlint. The runs of this change's own pull request, #69, match
its private, unlabelled row: `verify`, the two Linux builds, and `sums` passed, and `suite`,
`formula`, `release`, `crates`, and `tap` were skipped.

## The conditions

| Job | Needs | Runs when |
| --- | --- | --- |
| `verify` | nothing | always |
| `build` | `verify` | always after `verify`, over `verify`'s matrix: all four targets when `macos` is true, else the two musl targets |
| `suite` | `verify` | the ref is a tag |
| `sums` | `verify`, `build` | every build entry passed. It expects as many tarballs as the matrix built. |
| `formula` | `verify`, `sums` | `macos` is true |
| `release` | `sums`, `formula`, `suite` | the repository is `Firefly-Projects/htmlspec` and the ref is a `v` tag, and all three needs succeeded |
| `crates`, `tap` | `release` | the same repository and tag condition, and `release` succeeded |

`macos` is decided once, in `verify`, as `github.repository == 'Firefly-Projects/htmlspec' ||
contains(github.event.pull_request.labels.*.name, 'release-check')`. A job whose condition holds
no status function runs only when every job it needs succeeded. So a job skipped or failed
upstream skips every job below it, and `build` is never skipped while `verify` runs. Its matrix
simply has no macOS entry.

## Repository, event, and label, to the jobs that run

"Private" is this working repository. The workflow starts at all only on a version tag, or on a
pull request that touches one of the paths it lists.

| Repository | Event | Label | verify | build | suite | sums | formula | release, crates, tap |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| public | push of tag `vX.Y.Z` | none | runs | 4 targets | runs | 4 tarballs | runs | run, when everything above passed |
| public | pull request opened, pushed, or reopened | any | runs | 4 targets | skipped | 4 tarballs | runs | skipped: not a tag |
| public | `labeled`, `release-check` | `release-check` | runs | 4 targets | skipped | 4 tarballs | runs | skipped: not a tag |
| public | `labeled`, any other label | any | runs | 4 targets | skipped | 4 tarballs | runs | skipped: not a tag |
| private | pull request opened, pushed, or reopened | without `release-check` | runs | 2 Linux targets | skipped | 2 tarballs | skipped | skipped |
| private | pull request opened, pushed, or reopened | with `release-check` | runs | 4 targets | skipped | 4 tarballs | runs | skipped: not public, not a tag |
| private | `labeled`, `release-check` | `release-check` | runs | 4 targets | skipped | 4 tarballs | runs | skipped |
| private | `labeled`, any other label | without `release-check` | runs | 2 Linux targets | skipped | 2 tarballs | skipped | skipped |
| private | `labeled`, any other label | with `release-check` | runs | 4 targets | skipped | 4 tarballs | runs | skipped |
| private | push of tag `vX.Y.Z` | none | runs | 2 Linux targets | runs | 2 tarballs | skipped | skipped: not public |

Every event of one pull request shares one concurrency group, and a newer run cancels the run in
progress. No run skips `verify`, so every run of a pull request runs the jobs its labels call for,
and a run for `release-check` cancels the Linux-only run it supersedes.

As first built, a run started by another label skipped every job, in a concurrency group named by
its own run id, so that it would not cancel the run in progress. The code review found the cost.
`gh pr checks` reports the newest run of each check by name, and a skipped job's run is newer than
the one it hides, so a `verify`, `sums`, or `formula` that had failed then read as skipped. The checks
other workflows run, the required `check` and `guard` among them, were out of its reach.

## On a public tag, nothing publishes unless all four builds and the formula passed

- `build` runs four entries, because `macos` is true in the public repository whatever the
  labels. A failed entry fails `build`, which skips `sums`, `formula`, and then `release`.
- `sums` refuses unless `SHA256SUMS` lists four tarballs, the count `verify` wrote.
- `formula` renders the formula from `SHA256SUMS`, and `render-formula` refuses a release lacking
  any of the four targets. Then it audits, installs, and tests the formula. A failed or skipped
  `formula` skips `release`, because `release` needs it and its condition holds no status
  function.
- `release` itself first refuses a `SHA256SUMS` that names no tarball for one of the four targets
  `python3 scripts/release_facts.py targets` prints. So the guarantee holds in the publishing job,
  whichever job above it was skipped.
- `crates` and `tap` need `release`.

## Checked

`actionlint` 1.7.12, with ShellCheck 0.11.0 on its path, reported no error in any of the four
workflows. ShellCheck was confirmed active by a scratch workflow with an unquoted `$FOO`, which
actionlint reported as SC2086. `scripts/check-publish-crates` checks the two matrices
`release_facts.py` prints against the four entries the workflow used to write out.
