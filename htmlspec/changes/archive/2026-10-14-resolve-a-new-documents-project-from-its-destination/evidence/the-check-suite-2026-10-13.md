# The check suite, with the promoted project in place

**Date:** 2026-10-13
**Tasks:** 4.2 and 4.3.
**Binary:** `htmlspec 0.29.1`, built from this branch: the promotion commit `071d67a` and this change.

## `scripts/check` inside `nix develop`

```
$ nix develop -c scripts/check
=== format ===
=== lint ===
=== test ===
=== doctest ===
=== self-conformance ===
=== self-format ===
=== self-runbooks ===
=== self-runbooks-format ===
=== self-assets ===
=== prompt ===
!!! prompt failed
=== installed ===
!!! installed failed
=== release-fixtures ===
!!! release-fixtures failed
=== release ===
!!! release failed
FAILED: prompt installed release-fixtures release
```

Every stage that runs Rust passes: `format`, `lint`, `test` (48 test binaries, every one `ok`, none
failed), `doctest`, `self-conformance`, `self-format`, `self-runbooks`, `self-runbooks-format`, and
`self-assets`. The four that fail fail before they start, each with the same line:

```
error: tool 'python3' not found
```

That is macOS's `/usr/bin/python3`, a shim that asks the developer tools for the interpreter, failing
under the SDK settings `nix develop` exports. It is not a fault in any stage, and the same four failed
the same way before this change, at `139b206` with the promoted project in place.

## The four Python stages, run outside the shell

With `cargo` from the shell on the path, because `check-installed` builds the binary:

```
=== scripts/check-prompt
  ok  a_fresh_repository_opens_with_every_row_checked
  ok  an_empty_confirmation_is_refused_and_asks_again
  ok  cancelling_writes_nothing_and_exits_three
  ok  the_flag_still_means_no_tools
all 4 prompt cases passed
exit 0
=== scripts/check-installed
  ok  45 installed instruction file(s) match what `init` writes
exit 0
=== scripts/check-release --self-test
  ok  23 shared fixture case(s) read the same way both readers read them
exit 0
=== scripts/check-release
  ok  36 archived change(s) record a version, and 31 version(s) are in the changelog
  ok  declared version 0.29.1 has a changelog entry
exit 0
```

## Before 4.3

The first outside run of `check-installed` failed: all 45 installed instruction files differed from
what `init` writes, because each records the version that generated it, and the version had moved to
0.29.1. `htmlspec init --tools claude,codex,pi` regenerated them. Each changed one line,
`generatedBy: htmlspec 0.29.0` to `0.29.1` (27 indented under `metadata:`, 18 at the top level), and
nothing else. `spec.nix` and `AGENTS.md` were not changed.
