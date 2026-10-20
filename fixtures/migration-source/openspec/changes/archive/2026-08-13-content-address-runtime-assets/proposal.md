## Why

The project stores its browser-side runtime as a copy per document, and one of those copies is already wrong. The archived `deterministic-formatting.html` ships `class="hs-skip-link"` markup against a stylesheet containing no rule for it, so the first control a keyboard user reaches does nothing. That copy was not left behind by later maintenance: `assets/htmlspec.css` has a single commit and arrived correct at `36rem` with the skip link, while the document's copy arrived two commits later at `778e451` carrying `34rem` and no skip link. It was written stale.

Two accepted requirements in `document-template` cannot both hold under the current layout. "Shared assets" requires each library to appear once in the project rather than once per document, which two 6.0 MB copies violate. "Runtime version is recorded" requires that a rendered document reproduce later, which a single mutable `assets/` directory breaks the moment a library is upgraded. Neither requirement is wrong. The directory layout is what cannot serve both.

Nothing detects this today. `openspec validate --all --strict` passes all six items with the broken document sitting in the archive, because validation reads documents and not the shape of the asset tree around them.

## What Changes

- Store each runtime asset once under a name derived from the hash of its bytes, so identical bytes occupy one location and different bytes coexist without overwriting each other.
- Have every document reference the exact asset hashes it was authored against, rather than a mutable path whose contents can change under it.
- Reject a document whose asset references do not resolve to stored bytes matching their recorded hash, closing the gap that let a stale copy ship undetected.
- Reject a project that holds more than one copy of the same asset bytes, making the existing "Shared assets" scenario checkable rather than aspirational.
- Re-render the archived `deterministic-formatting.html` against the current stylesheet, so the skip link it declares works.
- **BREAKING** Documents referencing assets by the mutable `assets/vendor/<file>` path no longer conform. Every existing document needs its references rewritten, which `format` performs.

Out of scope, recorded here because both were raised and deliberately excluded:

- **Git LFS for the vendored libraries.** `crates/htmlspec-assets/src/lib.rs` embeds these bytes with `include_bytes!`, and `flake.nix` reads `${./assets}` directly. LFS pointer stubs would compile cleanly into a binary shipping a broken runtime, and Nix has no smudge filter to prevent it. LFS also makes `git-lfs` a prerequisite, which contradicts the "Clean machine build" scenario in `toolchain-packaging`. The libraries are write-once anyway: 8 distinct blobs across all history in an 8.8 MB repository.
- **`README.html`.** At 4.0 MB it is the largest blob in the history and the only one that rewrites in full on every regeneration, but it raises a separate question about tracking generated output.

## Capabilities

### New Capabilities

None. This change alters how an existing capability stores and references its runtime, and adds rules to an existing validator.

### Modified Capabilities

- `document-template`: The "Documents share one pinned runtime" requirement gains content-addressed storage and per-document hash references as the mechanism by which its shared-assets and version-recording scenarios both hold. Initialization must establish the store and write hash references.
- `document-validation`: New requirements rejecting an asset reference whose bytes do not match its hash, and rejecting duplicate copies of identical asset bytes within a project. Asset topology becomes a checked property.

## Impact

- `assets/` — layout changes from `vendor/<file>` to hash-named storage. `assets/vendor.lock.json` already records a `sha256` per file and becomes the input to the naming scheme rather than only an audit record.
- `crates/htmlspec-assets` — `include_bytes!` paths and the `Asset.path` values move to hash-derived locations. The offline lockfile check continues to verify committed bytes.
- `flake.nix` — the `assetProvenance` derivation reads `${./assets}/${file.path}`, so its path construction follows the new layout. Its re-fetch and hash comparison logic is unchanged.
- `crates/htmlspec-validate` — two new rules, each needing a stable diagnostic identifier and conformance fixtures per the "Conformance is pinned by fixtures" requirement.
- Formatter — rewrites asset references to current hashes, which must remain idempotent under `deterministic-formatting`.
- `openspec/changes/archive/2026-08-10-html-spec/documents/deterministic-formatting.html` — re-rendered, and its duplicated 6.0 MB asset tree collapses into the shared store.
- `scripts/check` — gains the topology check so continuous integration and a contributor verify the same property.
