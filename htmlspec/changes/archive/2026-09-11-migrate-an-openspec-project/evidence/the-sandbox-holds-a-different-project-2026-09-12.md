# The sandbox holds a different project

**Date:** 2026-09-12
**Found by:** the `check` job on pull request #25, which failed while every stage passed locally
**Fixed in:** `flake.nix`, `crates/htmlspec-cli/tests/migrate.rs`

## What failed

```
test every_file_in_this_repository_classifies ... FAILED
no entity classified as configuration: {"accepted-specification": 10, "change-document": 51,
  "converted-document": 4, "delta-specification": 29, "evidence": 86, "recorded-version": 17}
FAILED: test
```

Six kinds where the survey finds eight. `configuration` and `change-configuration` were both
absent, and both are the kinds whose files end in `.yaml`.

## Why

`flake.nix` gives the build sandbox a filtered snapshot rather than the working tree. Its filter
kept `.json`, `.html`, `.css`, `.js`, `.md`, `scripts/`, and `VERSION`. It did not keep `.yaml`, so
`openspec/config.yaml` and the nine per-change `.openspec.yaml` files never reached the sandbox at
all. The survey enumerated a project that genuinely did not hold them and classified nothing as
either kind.

**The filter's own comment already stated the rule this broke**, one release before the break:

> A file the suite reads must be in the source or the suite passes locally and fails here.

That sentence was written for `VERSION`, which needed its own rule because it carries no extension.
The rule generalizes and nobody applied it when the migration tests started reading `.yaml`.

## The fix found two more, and those are the dangerous ones

Adding `.yaml` was the loud repair. Testing the filter against every extension the project holds
found `.txt` and `.py` were dropped too — `mutation-testing.txt`, `prove-listing.py`, and
`prove-sequence.py`, three evidence files.

**Those three would never have failed a test.** Evidence also arrives as `.md`, so the `evidence`
kind stays present whatever happens to three files. `every_file_in_this_repository_classifies`
checks that each kind appears at least once, so it cannot see a kind losing three of its members.
The survey would have reported a smaller project, and every test would have passed.

| Extension | What is lost | What notices |
| --- | --- | --- |
| `yaml` | two whole entity kinds | the kind check, loudly |
| `txt` | one evidence file | nothing |
| `py` | two evidence files | nothing |

## The new check, and why the old one was not enough

`every_extension_the_project_holds_reaches_the_survey` asserts that the survey finds at least one
file of every extension this repository's project holds: `md`, `yaml`, `html`, `txt`, `py`, and the
no-extension case that `VERSION` occupies. It catches a filter dropping a file **without needing a
whole kind to disappear first**, which is the gap the three silent files sat in.

Both messages now name `flake.nix`. The old one said "no entity classified as configuration", which
sends a reader to the classifier — and the classifier was correct the whole time. A diagnostic that
names the wrong file costs more than one that says less.

## Checked by reverting, in the sandbox rather than locally

Local `cargo test` cannot see this defect, because the working tree holds every file. So the
verification ran where the defect lives:

```
$ nix build .#checks.aarch64-darwin.suite     # with the fix
all 10 checks passed

$ # filter reverted to what continuous integration ran
$ nix build .#checks.aarch64-darwin.suite
test every_file_in_this_repository_classifies ... FAILED
test every_extension_the_project_holds_reaches_the_survey ... FAILED
the survey found no yaml file, out of {"<none>": 17, "html": 7, "md": 177}
FAILED: test
```

The reverted build reproduces the continuous-integration failure exactly, on my own machine, and
the new test fails alongside the old one. `177` markdown files against `0` yaml is the measurement
that makes the cause unambiguous.

## The fourth instance of one shape

This change and its review have now produced four of these, and the pattern is worth naming rather
than fixing four times:

| The rule | What the check reached |
| --- | --- |
| The destination holds the whole project or none of it | Every entry, including one the destination legitimately already held |
| A shared version needs a changelog entry explaining it | Any digits anywhere in the file |
| A template's example content carries no authority | Identifiers only, never the prose |
| **Every file the suite reads is in the source** | **Five extensions and one filename, not the set the tests actually read** |

Every one is a rule that is right, paired with a check whose reach falls short of it. Three of the
four were found by a person or a review rather than by the suite.

**The distinctive thing about this fourth one is that the rule was written down and still not
applied.** The other three had no statement to check against. This one had a comment stating the
obligation exactly, sitting six lines above the pattern that violated it — so the repair for that
class is not a better comment.
