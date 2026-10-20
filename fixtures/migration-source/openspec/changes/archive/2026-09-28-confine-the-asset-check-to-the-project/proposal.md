## Why

**A repository cannot hold an html-spec project and anything else that stores assets.** The
duplicate-asset check takes a project as its subject, and today "the project" means every asset
store beneath the repository root. document-validation's "A project must not store one asset's bytes
more than once" says the rule is evaluated where a project is in view. It never says where a project
ends, and the program answers "at the repository root".

That answer is wrong for the ordinary case, and this repository is the measured instance. It keeps
two kinds of store that are not a project's store:

- `assets/`, the toolchain's own source copy of the runtime, which the build embeds and `init`
  copies from;
- four stores inside conformance fixtures, which carry their own bytes **deliberately**, because
  each fixture is a project of its own.

With no `htmlspec/` directory, `htmlspec assets .` checks each store on its own and passes. **Create
one, even empty, and the repository resolves as a project.** The check then compares all five
stores against the new one. `2026-09-18-establish-a-runbook/evidence/the-location-is-blocked-2026-09-17.md`
measured it: one `mkdir` gives one duplicate, and `htmlspec init` gives nine. On 2026-09-26 the
paused migration of this repository measured nine again in a copy of HEAD. That is the
`self-assets` stage of `scripts/check`, so the suite turns red.

The comment above `one_project` in `crates/htmlspec-cli/src/commands.rs` already states that
comparing across the fixtures' stores "reported the fixture suite as duplicated, which is the check
misreading its own subject". The fix it made was to stop comparing when no project is in view. It
did not decide what to compare when one is.

**This blocks two things the project wants.** The release runbook cannot live at
`htmlspec/runbooks/release/`, which is where the program keeps runbooks, and the migration of this
repository cannot be promoted into its own root. Every adopter whose repository keeps fixtures,
vendored copies, or a second tool's assets meets the same wall the first time they run `htmlspec init`.

## What Changes

- **A project's stores are the ones inside its own `htmlspec/` directory.** The duplicate check
  compares bytes across those stores and no others. A store elsewhere in the repository belongs to
  something else: a fixture, a toolchain's source copy, or another project. It is not the project's
  second copy.
- **Stores outside the project are still checked, each on its own,** exactly as every store is
  checked today when no project is in view. A store that duplicates bytes *within itself* is reported
  wherever it lives.
- **The report says which stores it treated as the project's and which it did not.** A contributor
  whose duplicate is no longer reported can see why, instead of wondering whether the check ran.
- **Promotion's topology check and `htmlspec assets` make the same comparison from one source,** so
  a staged project and a promoted one are judged the same way.
- **Unchanged:** a project whose own tree holds two copies of the runtime still fails. That is the
  case "A copy per document is rejected" names, and every such copy is inside `htmlspec/`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-validation`: "A project must not store one asset's bytes more than once" states where a
  project's stores are, so that the comparison has a subject that stops at the project.

## Impact

- `crates/htmlspec-cli/src/commands.rs`: `assets` partitions the discovered stores into the
  project's and the others. It compares across the first set only, and checks each of the second
  set on its own. The promotion check calls the same partition.
- `crates/htmlspec-cli/src/discover.rs`: no change to discovery. The partition is a question about
  a store's path relative to the resolved project.
- CLI tests for a project beside a fixture store, a project beside a toolchain source store, and a
  project holding two stores inside itself.
- `scripts/check`: the `self-assets` stage keeps running `htmlspec assets .`, and its comment
  ("Run against the repository root, which is the project this suite judges") is corrected.
- **Not in this change**, and recorded so it is not assumed to follow:
  - The release runbook still tells each run to remove `htmlspec/` and `spec.nix`. That step
    exists for other reasons too: `init` declares a project at `0.0.0`.
  - `scripts/check-release` still reads `openspec/`.
  - `crates/htmlspec-cli/tests/migrate.rs` still uses this repository's `openspec/` as a fixture.

  Making this repository an html-spec project is its own change. This one removes the obstacle
  that belongs to the product.
