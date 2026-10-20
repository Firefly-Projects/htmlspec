# The check fails on all three properties before anything satisfies it

Tasks 1.5 and 1.6. `scripts/check-release` run against the repository before a version
was recorded and before a changelog existed.

```
$ ./scripts/check-release
release hygiene:
!!! no CHANGELOG.md at the repository root. Every version the project ships records what changed in it
!!! 2026-08-10-html-spec records no version. Add VERSION naming the version it shipped in, so the release it belongs to is recoverable
!!! 2026-08-13-content-address-runtime-assets records no version. Add VERSION naming the version it shipped in, so the release it belongs to is recoverable
!!! 2026-08-14-separate-artifact-kinds records no version. Add VERSION naming the version it shipped in, so the release it belongs to is recoverable
!!! 2026-08-17-project-bootstrap-cli records no version. Add VERSION naming the version it shipped in, so the release it belongs to is recoverable
!!! 2026-08-18-conform-generated-frontmatter-names records no version. Add VERSION naming the version it shipped in, so the release it belongs to is recoverable
!!! 2026-08-18-install-the-full-workflow-set records no version. Add VERSION naming the version it shipped in, so the release it belongs to is recoverable
!!! 2026-08-21-distinguish-command-names-from-skill-names records no version. Add VERSION naming the version it shipped in, so the release it belongs to is recoverable
!!! no CHANGELOG.md heading names version 0.1.0. A shipped version records what changed in it

9 fault(s) across 7 archived change(s), declared version 0.1.0
$ echo $?
1
```

## What the nine faults cover

| Property | Faults | What failed |
| --- | --- | --- |
| Every archived change records a version | 7 | all seven archives |
| Every recorded version is distinct | 0 | unreachable — nothing records a version to compare |
| Every recorded version has a changelog entry | 2 | the file is absent, and `0.1.0` has no entry |

The distinctness rule reporting nothing is worth stating rather than glossing. It cannot
fire while property one fails, because there are no recorded versions to compare. That
ordering is why the rules are checked in this sequence and why task 6.2 exercises
distinctness separately, by recording a deliberate duplicate.

Each fault names the archive directory and what to do about it. Across seven archives,
"release hygiene failed" would not be actionable.

## The exit status

1, so the stage fails the suite. `scripts/check` collects a failing stage into its
`FAILED:` line and exits 1, and this check has no advisory mode to fall back to — which
is the constraint that made the archive rather than the working change the right subject.
See the design.
