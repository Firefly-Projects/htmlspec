# Three defects a review of the pull request found

**Date:** 2026-09-11
**Found by:** a code review of pull request #25, after the change was archived
**Fixed in:** `crates/htmlspec-cli/src/staging.rs`, `crates/htmlspec-cli/src/commands.rs`

Three real defects, all in code this change added, none caught by its own 32 test result lines.
Each is recorded here rather than quietly fixed, because the pattern across all three is worth
more than the fixes: **every one is a rule that was right in isolation and wrong against the
ordinary case.**

## 1. Promotion refused the ordinary case

`Staged::resolve` refused while the destination held any entry promotion would place, and
`PLACED` included `CHANGELOG.md`. A migration promotes into the repository the source project
already lives in, and such a repository normally has a root changelog already. This one does.

So a correctly staged migration hit this:

```
1 staging fault(s):
  the destination already holds CHANGELOG.md. Promotion places the whole migrated project or
  none of it, so it does not overwrite and does not merge
```

Nothing in the workflow said how to get past it, because there was no way past it.

**Two rules of this change deadlocked.** Task 5.9's requirement is that the release record be
*carried*, so that a migrated project's shared-version collision has an entry explaining it. The
whole-or-none rule then refused because the destination held one. Both rules are right. Their
conjunction was not.

**The repair:** `REFUSED_OVER` holds the project directory and the declaration only. A destination
already holding either is already an HTMLSpec project, and placing a second one over it is a merge.
A destination already holding a release record *satisfies* the carrying requirement, so promotion
proceeds and leaves that copy alone — overwriting it would replace a repository's own history with
the source's, which is a destructive edit nobody asked for by running a migration.

Pinned by `a_destination_holding_a_release_record_is_promoted_into`, which asserts the destination's
own bytes survive, and by `a_destination_holding_the_project_or_the_declaration_is_still_refused`,
which asserts the relaxation did not reach the other two.

## 2. The shared-version exemption was a substring test

```rust
if changes.len() < 2 || record.contains(&version) {
```

`record` is the whole changelog as text. Two ways that passes when it must not:

| Case | Why it passed |
| --- | --- |
| `0.1.1` shared, changelog names only `0.1.10` | `"0.1.10".contains("0.1.1")` is true |
| `0.1.1` shared, prose reads "supersedes the 0.1.1 line" | prose is not an entry, and the test read prose |

So a real unexplained collision passed the gate silently — the exact failure the fault exists to
report, and the check that was supposed to catch it was the thing letting it through.

**The repair:** `release_record_versions` reads `#`-prefixed lines only and parses each dotted
number through `Version::parse`. That is the rule `scripts/check-release` already applies to the
same file, which is what keeps the two from disagreeing about whether a version has an entry.

Pinned by `a_version_another_entry_merely_contains_does_not_explain_it`, using `0.1.1` against a
`0.1.10` heading whose prose also names the digits, and by
`release_record_versions_come_from_headings`, which asserts a bare year and a prose version are
both excluded.

## 3. Two format causes reported as one

```rust
if outcome.canonical != source {
    diagnostics.push(format!("{display}: not canonically formatted"));
}
```

A staged document differs from its canonical form for two different reasons: its serialization is
not canonical, or it names asset bytes the store does not hold and the filled reference differs
from what is on disk. Those need different repairs, and `commands.rs:458` — the established
`format` path — already distinguishes them.

The migration's check did not. It compared against `source` rather than against the rewritten
text, so a reference fault arrived as "not canonically formatted" and sent a contributor to the
serializer.

**This matters most here of all places.** A migration produces documents in bulk, so a wrong asset
prefix is wrong in every document of a kind. The misdirected diagnostic arrives dozens of times at
once, which is precisely the condition the check-last rule exists to make legible — and a
mislabelled class is worse than a halting check, because the shape it shows points the wrong way.

**The repair:** report each filled reference by name, with the address the staged document must
carry, and compare the canonical form against `rewritten.rewritten` so the formatting diagnostic
means only formatting.

Pinned by `a_stale_asset_reference_is_not_reported_as_a_formatting_fault`, which asserts both the
reference diagnostic appears and the formatting one does not.

## Each fix was checked by reverting it

A test written for a defect is not the same as a test that fails when the defect is present. So
each fix was reverted and the suite re-run:

| Reverted fix | What failed |
| --- | --- |
| `REFUSED_OVER` back to `PLACED` | `a_destination_holding_a_release_record_is_promoted_into` |
| the substring test restored | `a_version_another_entry_merely_contains_does_not_explain_it` |
| the comparison back to `source` | `a_stale_asset_reference_is_not_reported_as_a_formatting_fault` |

In every case the other tests in the same file still passed, so each test targets its own fix
rather than firing on any change nearby.

## One existing test had to change, and why that is a finding

`a_failed_move_leaves_the_destination_unchanged` provoked a mid-flight failure by occupying the
destination's `spec.nix` path with a directory. Once promotion learned to skip an occupied target,
that stopped failing — the test's provocation had become correct behavior.

Rewritten to make the destination read-only after `resolve` has passed, which is a cause promotion
cannot skip. The permissions are restored before any assertion runs, so a failing assertion does
not leave the scratch directory unremovable and take the next run's setup with it.

**The finding is that the test was coupled to a mechanism rather than to the property.** The
property is that a failure part-way through leaves the destination as it was, and that property was
never in doubt. A test that fails when the code becomes *more* correct is a test measuring the wrong
thing, and this one did until it was rewritten.

## What this says about the change's own verification

This change ran 32 test result lines, two deliberate mutations, and a partial migration of this
repository, and none of that found these three. The reason is visible in hindsight and worth
stating: **every test here constructed its own scratch destination, and a scratch destination is
empty.** The ordinary destination is a repository with a history — a changelog, a version, files
that predate the migration — and no test in this change ever pointed promotion at one.

`a_destination_holding_a_release_record_is_promoted_into` now does.
