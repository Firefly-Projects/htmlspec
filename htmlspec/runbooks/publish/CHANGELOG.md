# Changelog

What this runbook's releases are. A release is the pair of canonical documents —
`specification.html` and `tasks.html` — as they stood when the project archived them, and a version
identifies one such pair. Nothing else moves it: a run's record is history rather than procedure, so
adding or editing one is not a release.

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
