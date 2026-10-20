# The version gate, run rather than read

Tasks 1.1 and 1.2, recorded 2026-08-30 with `htmlspec 0.5.0 (bfc3835471f3-dirty)` built from
this working tree. Task 1.1 asks for the fallback to be shown by running the gate, because a
defect argued from source reading is a defect that might not be there.

## The setup

A scratch project, bootstrapped with `htmlspec init`, whose archive replicates this
repository's: the same 11 directory names, each carrying the `VERSION` file its original
carries.

```
2026-08-10-html-spec                                   0.1.0
2026-08-13-content-address-runtime-assets              0.1.0
2026-08-14-separate-artifact-kinds                     0.1.0
2026-08-17-project-bootstrap-cli                       0.1.0
2026-08-18-conform-generated-frontmatter-names         0.1.0
2026-08-18-install-the-full-workflow-set               0.1.0
2026-08-21-check-release-hygiene                       0.2.0
2026-08-21-distinguish-command-names-from-skill-names  0.1.0
2026-08-22-accept-a-described-change                   0.3.0
2026-08-24-pass-the-description-through                0.4.0
2026-08-24-report-in-the-contributors-terms            0.5.0
```

Highest recorded version: **0.5.0**. That is task 1.2, and it is measured from the files rather
than from the changelog: all 11 archived changes carry a `VERSION`, and none is missing.

The declaration was then set to `0.0.1` — a version far below everything in that archive, and
one no correct gate accepts.

## The run

```
$ htmlspec archive demo
version 0.0.1 follows 0.0.0
htmlspec/changes/demo/specification.html
  0 errors, 2 warnings: the document is accepted
the specification validates; archival may proceed to apply its contract changes to the
project's accepted specifications
exit: 0
```

**`version 0.0.1 follows 0.0.0`.** The gate compared against `0.0.0` while the archive in front
of it recorded `0.5.0`, and it permitted the archival. The program printed the number it
compared against, so the defect is visible in the ordinary output of a successful run rather
than only under a debugger.

## Why

`last_archived_version` (`crates/htmlspec-cli/src/commands.rs:1352`) reads archive *directory
names* and passes each to `archived_version` (`:1371`), which splits on `-` and returns the first
part `Version::parse` accepts. `Version::parse` (`crates/htmlspec-cli/src/version.rs:56`)
requires exactly three dot-separated numeric components.

A name like `2026-08-24-report-in-the-contributors-terms` splits into `2026`, `08`, `24`,
`report`, and so on. No part contains a dot, so no part parses, so `find_map` yields nothing for
every directory. `max()` over an empty iterator returns `None`, and the `unwrap_or(initial)`
supplies `0.0.0`.

The convention the gate reads is one this project has never written. The convention it records
is the `VERSION` file, which `toolchain-packaging`'s release-hygiene requirement asks for and
which every archived change carries.

## The scope of it

This is not a defect that fires on an edge case. `archived_version` returns `None` for every
directory name that does not embed a dotted version, and nothing in the toolchain writes such a
name — `archive` does not name archive directories at all, because it does not create them. So
the fallback is not a fallback in practice. It is the only path, in every project, since the
gate was written.

Every refusal message the version gate can produce is therefore unreachable except against a
declaration below `0.0.0`, which no declaration is. The gate has never refused anything.

## The second finding in the same run

The command reported success and the change did not move:

```
$ ls htmlspec/changes/
demo
$ ls htmlspec/changes/archive/ | grep demo
(no output)
```

`archive` said archival may proceed, exited zero, and left `demo` among the active changes. This
is the half-operation the change addresses, reproduced in three commands. A contributor who
reads the final line as a report of completion has been misled by it, and an agent that treats
exit zero as done will leave the change exactly here.
