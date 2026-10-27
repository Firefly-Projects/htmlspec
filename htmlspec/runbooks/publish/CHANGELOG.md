# Changelog

What this runbook's releases are. A release is the pair of canonical documents —
`specification.html` and `tasks.html` — as they stood when the project archived them, and a version
identifies one such pair. Nothing else moves it: a run's record is history rather than procedure, so
adding or editing one is not a release.

## 0.3.0

Published with the project's 1.1.1 release.

**A run records the README's install jobs.** The release workflow gains two jobs that run the
README's own install commands against the published release: `install`, on both Linux targets, and
`install-homebrew`, once the formula is in the tap. Step 6 carries a failure of either to step 8
rather than stopping, because publication has already happened. Step 8 records both, records a tag
older than the latest release as untested by them, and, when the README's commands were broken at
the tag, stays open until the fixing patch release's run of the job that failed passes. A job that
failed for another reason, a timeout waiting for `latest` or the network, is run again and its rerun
recorded. On an older line the tap is not updated, so Homebrew is recorded as untested. "Each channel's version
is recorded" says so, with a scenario for the README broken at the tag.

## 0.2.1

Published with the project's 1.1.0 release.

**The export names the account it pushes as, and a run records it.** For a GitHub target,
`publish/export --bootstrap` and `--push` now print `pushing as LOGIN`, the account `gh` is signed
in as, before they reach the target, and refuse when `gh` names none, when it names another account
just before the push, or when this machine's url rewrites would send git over SSH rather than
`https://github.com`. Their git commands reach GitHub with `gh`'s credentials, so the credential
helper the v1.0.0 run had to set by hand, after its first bootstrap push found none, is no longer
needed. Steps 3 and 4 say the export prints the account, and that the run records it. What the
procedure consumes says the export pushes as the account `gh` is signed in as.

**What this release does not do.** The nine steps, their order, and the six requirements are
unchanged, so this is a patch.

## 0.2.0

Published with the project's 1.0.1 release.

**A run's preflight checks the interpreter its steps run.** `req-the-pinned-toolchain-is-what-runs`
names `python3` beside `cargo` and `rustc`, and the development shell supplies it. The first run of
0.1.0, the v1.0.0 release, passed its preflight and then stopped at its second step: inside
`nix develop` on macOS, `python3` resolved to `/usr/bin/python3`, a stub that asks the developer
tools at `DEVELOPER_DIR` for the interpreter, and the shell points that at the Nix SDK, which has
none. `publish/export` exited with "error: tool 'python3' not found". Now that machine is refused
before the run starts, with where `python3` resolved, and inside the shell it runs the shell's
interpreter. The specification gains the scenario of an interpreter that resolves outside the shell,
and the list of what the procedure consumes says the shell supplies `python3`.

**What this release does not do.** The nine steps and their order are unchanged, and so are the
other five requirements. `gh`, `git`, `nix`, and `brew` stay undeclared, because each runs inside
the shell from where the machine installed it.

## 0.1.0

The first release of this runbook, published with the project's 0.34.0 release.

**A release is published by one run of this runbook, after its change is archived and merged.**
The release runbook prepares a release inside its change. This one takes it from this repository's
`main` to the public repository, `Firefly-Projects/htmlspec`, as a reviewed export, tags the public
merge commit through GitHub's API, watches the release workflow publish the binaries, the crates,
and the formula, and installs the version from each of the four channels. The steps were first
planned as the end of the release runbook, and the code review of that plan, #63, found that a run
after archival would copy the steps already done inside the change.

**The hazards that review found are requirements here.** No step pushes a ref from this repository
to the public one, because a pushed tag carries every private commit behind it. The tag names the
commit that merged the export in the public repository. The first run bootstraps the empty public
repository with its ruleset off for that one push, and stops unless the ruleset is active again.
Two more steps were in no plan: the first release adds `firefly-developers` as each crate's
co-owner, and every release installs from each channel and records what it prints.

**What this release does not do.** No release has been published by it yet. Steps 1, 2, and 4 were
rehearsed against a local target, and the commands of steps 3, 5, and 7 were read against GitHub's
and cargo's documentation. The steps against the public services first run at the first release
after 1.0.0's archival.
