# What the implementation revealed

Task 5.3.

- **The comparison was wider than the proposal said.** One of the nine duplicates in task 1.1 was
  between two conformance fixtures, with no project store involved. Once any project resolved,
  every store was compared with every other, including pairs neither of which was the project's.
  The partition removes that case too, because both fixture stores lie outside `htmlspec/`.
  `evidence/the-measurement-2026-09-27.md`.
- **Only the topology check depends on the project's existence** (task 1.2). Every other document
  stage, and a validation of every fixture document, gave identical per-document results with and
  without `htmlspec/`. The proposal's claim holds, measured.
- **Resolving symbolic links is necessary on the machine this project is developed on.** macOS
  reaches the temporary directory through `/var` → `/private/var`, so a partition comparing raw
  paths would misclassify a project reached through one spelling and discovered through the other.
  The partition compares canonical paths, falls back to the absolute path where a path cannot be
  resolved, and one test fails without it. `evidence/mutation-testing-2026-09-27.md`.
- **A store is named by its `assets/` directory in the report**, not by the directory holding it,
  which is what discovery returns and what read as a bare `.` for the repository root.
- **The partition is stated whenever a project is in view**, including "No store outside it was
  found". The first version stated it only when outside stores existed, and the requirement says
  the report states which stores were whose. The structured payload always carries both lists.
- **Promotion now resolves the staged tree as a project before checking it.** It passes the
  staged root to the same `topology::check` as `assets`. In practice the staged tree holds one
  store, because the staging check refuses a second, so promotion's result is unchanged for every
  correct staging. `the_asset_check_and_promotion_give_one_verdict` keeps the two from drifting.
- **Unchanged, and still recorded as not this change's:** the release runbook still removes the
  stray `htmlspec/` and `spec.nix` that `init` creates, because `init` also declares a project at
  `0.0.0`. `scripts/check-release` and `tests/migrate.rs` still read `openspec/`. What this change
  removes is the one obstacle that belonged to the product: with it, `htmlspec init` in this
  repository no longer turns `self-assets` red.
