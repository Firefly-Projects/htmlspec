# Handoff: groups 1–2 done, group 3 next

Written at the end of the session that implemented groups 1 and 2. Everything a
fresh context needs to start group 3 is here or in the files it names.

## State

- **Tasks: 12 of 40 complete.** Groups 1 and 2 done (including 2.6–2.8, added after
  the fact for work that was not in the original list). Groups 3–8 pending.
- **`scripts/check` passes all 6 steps** on the current working tree.
- **Nothing is committed.** The store under `assets/sha256-*/` is `git add`ed but not
  committed; everything else is unstaged or untracked. See "Working tree" below.
- `openspec validate content-address-runtime-assets --strict` passes.

## What groups 1–2 built

**`crates/htmlspec-assets/src/address.rs`** — the derivation. `stored_path(digest,
name)` yields `sha256-<first 16 hex>/<name>`. `ALGORITHM` and `ADDRESS_HEX_LEN` are
the constants; the truncation is safe only because the store is closed and the checks
compare full digests.

**`crates/htmlspec-assets/src/store.rs`** — `Store::discover(root)`, the type group 3
builds on. Read this file first. Its API:

- `Store::Absent` vs `Store::Present(Contents)` — **the distinction the whole scoping
  decision rests on.** No `assets/` directory means no store, which is not the same as
  a store holding nothing.
- `is_present()` — whether asset rules apply at all.
- `resolve(reference) -> Resolution` — `Match`, `Absent`, `Mismatch { addressed,
  found }`, or `NotAddressed`. `Absent` and `Mismatch` are deliberately distinct
  because their causes differ, and the spec requires distinguishable diagnostics.
- `duplicates() -> Vec<Duplicate>` — compares addressed **and unaddressed** files
  together. Two addressed copies cannot duplicate each other (identical bytes take one
  address and one path), so a comparison limited to the addressed layout would report
  nothing while the project held two full copies of the runtime.

**`crates/htmlspec-assets/build.rs`** — generates `OWN` and `VENDORED` from
`assets/vendor.lock.json` into `$OUT_DIR/assets.rs`. The hand-written `include_bytes!`
literals are gone. `Asset.path` is now a **content address**, and `Asset::name()` is
the file name. Any lookup must use `name()`; a lookup by `path` goes stale the moment
bytes change.

**`assets/vendor.lock.json`** — now documents the derivation rule in its comment block
and records an `own` array pinning `htmlspec.css`, `htmlspec-print.css`, `htmlspec.js`,
and `template.html`. The template carries `"stored": false` because it is the document
`init` produces rather than an asset copied beside it.

**`assets/template.html`** — asset paths are now `{{ASSET:<name>}}` placeholders,
resolved to addresses by `resolve_asset_placeholders` at render time. No hash is ever
written into the template.

**`flake.nix`** — `storedPath` derives the same address from the same lockfield field.
Verified by execution: `nix build .#checks.aarch64-darwin.assetProvenance` re-fetched
all four libraries from npm and reported `ok` for each.

## The decision group 3 must honor

The reference rule is **scoped to a project that has an asset store**, and a document
**cannot exempt itself**. Both are stated normatively in
`specs/document-validation/spec.md` (first requirement, third and fourth paragraphs)
and argued in `design.md` under "Scope the reference rule by the presence of a store".

Why it matters concretely: all 37 conformance fixtures reference
`assets/htmlspec.css` and friends, and **no `assets/` directory has ever existed beside
them** — a fixture directory holds only `input.html` and `expected.json`. Those
references are symbolic, present so the parser sees a realistic document. An
unconditional rule rejects the 5 `valid-*` and 2 `warning-*` fixtures, which assert
acceptance, and those 37 inputs are the pin for every rule the project already has.

A `class="fixture"` marker was considered and rejected: it is a suppression switch
inside the document for a non-downgradable rule, it would require amending the closed
vocabulary in `document-schema`, its own conformance fixture would be circular, and it
would edit all 37 inputs anyway. Do not reintroduce it.

## The blocker group 3 hits immediately

`crates/htmlspec-validate/src/engine.rs:131`:

```rust
pub type Rule = fn(&Projection, &mut Diagnostics);
```

**A rule receives no project root, so it cannot reach a store.** This is the first
real design decision of group 3 and it is not yet made. Facts bearing on it:

- `engine::run(parse, projection, rules, strictness)` has 5 callers:
  `crates/htmlspec-cli/src/commands.rs:160`,
  `crates/htmlspec-validate/tests/template_conformance.rs:19`,
  `crates/htmlspec-validate/examples/validate_file.rs:13`,
  `crates/htmlspec-validate/src/rules/mod.rs:88`,
  `conformance/tests/conformance.rs:117`.
- `htmlspec-validate` does **not** depend on `htmlspec-assets` today. Its dependencies
  are `htmlspec-schema`, `htmlspec-parse`, `serde`, `serde_json`, `jsonschema`,
  `kuchikiki`.
- The conformance harness calls `run` with no filesystem context at all, which is
  exactly the "no store" case that must stay passing.

Options, unranked, for whoever picks this up:

1. Add an optional context parameter to `engine::run` (a `Store` or an
   `Option<&Path>`), threaded through all 5 callers. The harness passes nothing, which
   scopes the rule out naturally.
2. Keep `Rule` as-is and evaluate the asset rules outside the per-document engine, as a
   project-level pass the CLI runs. This matches the third added requirement, which
   already puts topology in the check suite rather than in per-document validation.
3. A hybrid: reference resolution in the engine with context, duplication as a
   project-level pass.

Option 2 deserves weight: the specs describe both asset rules as properties of a
project, and it requires no change to `Rule` or to any existing caller.

## Also missing for group 3

- **`Namespace::Asset` does not exist.** `crates/htmlspec-validate/src/diagnostic.rs`
  has `Schema`, `Refs`, `Safety`, `Viz`, `A11y`, `Review`, `Parse`. The new rules need
  a namespace, and task 3.3 requires they be non-downgradable.
- **`is_downgradable` is hardcoded to one namespace**:
  `!matches!(self, Self::Safety)`. Making the asset rules non-downgradable means
  extending that, and `strict_mode_promotes_every_warning_except_a_review_marker` in
  `conformance/tests/conformance.rs` asserts the two existing exceptions from the
  outside.
- **Rule registration**: a rule is live only by appearing in `rules::ALL` in
  `crates/htmlspec-validate/src/rules/mod.rs:24`. `every_namespace_has_a_live_rule` in
  that file's tests asserts each namespace has an entry, and it will need the new one.
- **Fixture generation**: fixtures are generated from a stated single edit to a
  known-good base by `conformance/src/bin/generate_fixtures.rs`, not hand-written. New
  fixtures for tasks 3.7 and 3.8 go through that.

## Expected signals, so they are not mistaken for regressions

- **`Store::discover(".").duplicates()` currently reports 8 duplicate sets** — every
  asset exists both at its address and in the old `assets/vendor/` layout or at the old
  flat path. This is the additive-migration state the design predicts at step 3. It
  clears at task 7.4/7.5 when the old copies are removed. Do not "fix" it before then.
- Once task 4.3 wires the topology rule into `scripts/check`, **the suite will fail on
  that duplication**. Task 4.5 says so explicitly. The suite goes green again after
  group 7.

## Working tree

Staged: the 8 files under `assets/sha256-*/`.

Modified, unstaged: `assets/template.html`, `assets/vendor.lock.json`,
`conformance/tests/presentation.rs`, `crates/htmlspec-assets/Cargo.toml`,
`crates/htmlspec-assets/src/lib.rs`, `crates/htmlspec-assets/src/lockfile.rs`,
`crates/htmlspec-cli/src/commands.rs`, `flake.nix`.

Untracked: `crates/htmlspec-assets/build.rs`,
`crates/htmlspec-assets/src/address.rs`, `crates/htmlspec-assets/src/store.rs`,
`crates/htmlspec-assets/examples/store_probe.rs`,
`openspec/changes/content-address-runtime-assets/`.

`crates/htmlspec-assets/examples/store_probe.rs` is a small reporter that prints the
store's contents and duplicates. It was written for the migration and is useful for
task 4.5 and 8.2 evidence. Run it with
`cargo run -q -p htmlspec-assets --example store_probe`.

`#README.md#` is an Emacs autosave file the user has asked to leave alone. Do not
delete it and do not add it to `.gitignore`.

## One repair unrelated to this change

`conformance/tests/presentation.rs` read `openspec/changes/html-spec/documents`, which
moved when the previous change was archived. The failure pre-existed this change and
was pushed in commit `6956324`. It now discovers change document directories under both
`openspec/changes/` and `openspec/changes/archive/`, so archiving cannot silently drop
documents from those checks again. Recorded as task 2.8.
