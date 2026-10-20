# Each rule observed failing on its own

Tasks 6.1 through 6.4. The check passes on this repository, so passing proves only that
nothing is wrong today. Each rule was made to fail in isolation and then restored. A check
whose failures were never observed is a check that has only been seen passing.

## A changelog with no entry for the declared version

Note what this tests. Deleting `CHANGELOG.md` outright exercises the *absent file* branch,
which is a different fault. The rule under test is a file that exists and does not name the
version, so the file was replaced with one carrying no version heading.

```
$ printf '# Changelog\n\nNo entries yet.\n' > CHANGELOG.md
$ ./scripts/check-release; echo $?
release hygiene:
!!! no CHANGELOG.md heading names version 0.1.0. A shipped version records what changed in it

1 fault(s) across 7 archived change(s), declared version 0.1.0
1
```

One fault, not two: the seven archives still record `0.1.0`, and their versions are still
distinct under the baseline. Restored, and the check passes again.

## Two archives claiming one version

The distinctness rule only applies above the baseline, so a duplicate at `0.1.0` is
correct and cannot be used to exercise it. Two archives were set to `0.2.0`, and `0.2.0`
was given a changelog entry so that only distinctness could fail.

```
$ ./scripts/check-release; echo $?
release hygiene:
!!! 2026-08-21-distinguish-command-names-from-skill-names records version 0.2.0, which 2026-08-18-install-the-full-workflow-set already records. A version names the change that produced it

1 fault(s) across 7 archived change(s), declared version 0.1.0
1
```

The fault names both archives, which is what makes it actionable — the second one alone
would leave a reader grepping for the first. Restored, and the check passes again.

## An archive with no recorded version

```
$ mv openspec/changes/archive/2026-08-14-separate-artifact-kinds/VERSION /tmp/V.bak
$ ./scripts/check-release; echo $?
release hygiene:
!!! 2026-08-14-separate-artifact-kinds records no version. Add VERSION naming the version it shipped in, so the release it belongs to is recoverable

1 fault(s) across 7 archived change(s), declared version 0.1.0
1
```

One archive named, and the other six pass. Restored, and the check passes again.

## Why each test isolates one rule

The pre-fix run reported nine faults at once, which showed the check working but could not
show which rule produced which fault. These three runs each produce exactly one fault, so
each rule is attributable. The distinctness rule in particular could not be seen at all in
the pre-fix run — no archive recorded a version, so there was nothing to compare — which is
why it needed a test that creates a duplicate rather than removes one.
