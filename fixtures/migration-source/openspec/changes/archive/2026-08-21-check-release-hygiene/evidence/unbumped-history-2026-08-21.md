# Seven changes, one version, no changelog

Observed 2026-08-21 against `main` at `d7d0621`.

## The record

Seven changes are archived:

```
2026-08-10-html-spec
2026-08-13-content-address-runtime-assets
2026-08-14-separate-artifact-kinds
2026-08-17-project-bootstrap-cli
2026-08-18-conform-generated-frontmatter-names
2026-08-18-install-the-full-workflow-set
2026-08-21-distinguish-command-names-from-skill-names
```

`Cargo.toml:9` has read `version = "0.1.0"` across all seven. `ls CHANGELOG*` finds nothing.

Four of the seven changed output a contributor consumes. `install-the-full-workflow-set`
took the installed workflow count from one to seven. `conform-generated-frontmatter-names`
and `distinguish-command-names-from-skill-names` both changed the frontmatter of every
generated command file. `project-bootstrap-cli` changed what `init` does. A consumer holding
`0.1.0` cannot tell which of those they have.

## Why nothing objected

`scripts/check` has eight stages: `format`, `lint`, `test`, `doctest`, `self-conformance`,
`self-format`, `self-assets`, `prompt`. None reads the version and none looks for a
changelog. So no check in this repository has ever had an opinion about release hygiene.

Note that the existing `lint` stage is `cargo clippy`. A new stage must not be called `lint`
— the name is taken, and taking it again would make the suite's output ambiguous about which
lint failed.

## What cannot be used

The check must read files rather than git history. `flake.nix:39` filters the build source:

```nix
srcFilter = path: type:
  let
    keep = builtins.match ".*\\.(json|html|css|js|md|schema\\.json)$" path != null;
    isScript = builtins.match ".*/scripts(/.*)?" path != null;
  in
  keep || isScript || craneLib.filterCargoSources path type;
```

The derivation receives a filtered snapshot, not a repository. There is no `.git`, so
`git log`, `git diff`, and `git describe` are all unavailable inside the sandbox — which is
where CI runs the suite. A check written against git history would pass on a contributor's
machine and fail in CI, which is the exact divergence `scripts/check` exists to prevent.

Two things the filter does keep, both load-bearing for this change:

- `*.md` passes, so `openspec/changes/` and a root `CHANGELOG.md` are both readable.
- `scripts/` passes, so the check itself can be a script.

Confirmed by evaluating the filter's own pattern against a representative path:

```
$ nix eval --impure --expr '(builtins.match ".*\\.(json|html|css|js|md|schema\\.json)$" "openspec/changes/x/proposal.md") != null'
true
```

## What the app already has, and why it is not the answer here

`crates/htmlspec-cli/src/commands.rs:1281` holds `archive_version_gate`, which refuses to
archive unless a project's declared version is ordered after the last archived one. That is a
feature of the shipped program, aimed at a project a contributor bootstraps with
`htmlspec init`.

It does not apply to this repository. There is no `spec.nix` and no `htmlspec/` directory
here — the work lives in `openspec/` under the vendored OpenSpec tooling — so there is no
declaration for the gate to read, and `htmlspec archive` has never gated a change in this
repository. Six of the seven archives above were moved with `git mv`.

That gate is also weaker than it looks, for reasons worth recording even though they are out
of scope here: `last_archived_version` at `commands.rs:1341` recovers the previous version
from an archive directory name shaped `2026-08-14-0.2.0-add-dark-mode`, and none of the seven
names above carries a version, so the comparison falls back to `0.0.0`. Whether the shipped
program should enforce its own convention is a separate question about the product. This
change is about this repository's suite.
