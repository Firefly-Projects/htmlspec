## Context

See proposal.md for why. The current mechanism is in `assets` in `crates/htmlspec-cli/src/commands.rs`:

1. `ProjectRoot::resolve` walks upward and succeeds at the first directory with an `htmlspec/`
   child. It stops at a `.git` boundary.
2. `root` becomes the resolved project's **path** — the directory *holding* `htmlspec/`, which in a
   repository is the repository root.
3. `crate::discover::stores(root)` finds every directory under that root with an `assets/` child,
   skipping dot-directories, `node_modules`, `result`, and `target`.
4. Each store is checked for duplicates within itself. Then, if a project resolved (`one_project`),
   `rules::topology::duplicates_across` compares every store with every other.

Step 2 is where the subject escapes the project. A project's documents live in `htmlspec/specs/`,
`htmlspec/changes/`, and `htmlspec/runbooks/`, and its one store is `htmlspec/assets/`
(`ProjectRoot::establish`). Nothing the project owns lives outside `htmlspec/` except the
declaration, `spec.nix`, which is not an asset.

Promotion (`migrate promote --staged`) runs a second copy of the same topology loop over the
staged tree, near the end of its conformance function. It is written separately from `assets`.

## Goals / Non-Goals

**Goals:**

- A project's duplicate check compares only the project's own stores.
- Every store in view is still checked for duplication within itself.
- `assets` and promotion share one partition, so they cannot disagree.

**Non-Goals:**

- **Making this repository an html-spec project.** The release runbook, `scripts/check-release`,
  and `tests/migrate.rs` still assume `openspec/`. See the proposal's Impact.
- **Changing asset-reference resolution.** "An asset reference must match the bytes it names"
  resolves a document's references against the nearest store above it, and that is unaffected. A
  project document's nearest store is `htmlspec/assets/`, found before any store at the repository
  root.
- **A declaration that excludes paths.** No `exclude:` list in `spec.nix`. The project's extent is
  a fact the layout already states, and a list would be a second statement of it that can drift.

## Decisions

### 1. The extent is `htmlspec/`, read from the resolved project

The partition is: a store is the project's if its directory is `project.htmlspec()` or lies beneath
it. Every other discovered store is "outside". This is a path-prefix test on canonicalized paths,
which is exact, and it needs no new configuration.

**Alternatives considered.**

- **Compare only stores that project documents reference.** This finds the stores a project
  actually depends on. It was rejected because it makes the topology check depend on parsing every
  document. A store that nothing references yet — `init`'s fresh store — would then escape the check.
- **Skip known fixture directories by name.** Rejected: it is this repository's layout encoded into
  the product, and it does nothing for an adopter's `vendor/` or `docs/assets/`.
- **Make the repository's root `assets/` a symlink to `htmlspec/assets/`.** Rejected: it fixes
  this repository and nobody else's, and a build input that points into a project the toolchain
  itself establishes is circular.

### 2. Outside stores are checked on their own, and named

Outside stores go through `rules::topology::duplicates` one at a time, exactly as the no-project
path treats every store today. The human report gains one line naming how many stores were treated
as the project's and how many were checked on their own. The structured payload gains
`projectStores` and `otherStores` path lists beside the existing `stores` count, which is kept and
means the total. The existing no-project note ("… is not a project, so its N stores were each
checked on their own") stays for that case.

### 3. One partition function, used by `assets` and by promotion

The partition lives in one function that takes a project root and a list of stores and returns the
two sets. `assets` uses it on the resolved project. The promotion check uses it on the staged root,
where the staged tree's only store is its `htmlspec/assets/`, so the result is unchanged for every
correct staging. A staged tree with a second store outside `htmlspec/` is already refused by the
staging structure check, "holds more than one asset store", which this change does not touch.

### 4. The `self-assets` comment is corrected, not the stage

The stage still runs `htmlspec assets .`. Today the repository is not a project, so it keeps
checking every store on its own. The comment claiming the repository root "is the project this suite
judges" is false today, and the change corrects it.

## Risks / Trade-offs

- **A project that deliberately keeps a second store outside `htmlspec/` for its own documents is no
  longer caught.** → A project's documents live under `htmlspec/` by the layout
  `ProjectRoot::establish` creates, and a document outside it is not a project document. A document
  under `htmlspec/` that references a store outside it still resolves through "An asset reference
  must match the bytes it names". So a dependency like that remains visible, as a reference, even
  though it is no longer a duplicate.
- **The payload grows two keys.** → Additive. `accepted`, `storedAssets`, `stores`, and
  `diagnostics` keep their meanings.
- **The measured nine duplicates stop being reported, and a reader of the archive can think that the
  defect was fixed by hiding it.** → The CLI tests pin both halves: the fixture and toolchain cases
  pass, and a second store inside the project still fails.

## Migration Plan

No document changes. A project whose check passed still passes. A repository that failed only
because of stores outside its project now passes, and that is the intended change. Rollback is a
revert.
