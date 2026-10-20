## Context

See proposal.md — Why for the motivation and the defect that prompted this change. Requirements are in `specs/document-template/spec.md` and `specs/document-validation/spec.md`.

The constraints that shape the approach are all existing project facts:

- `assets/vendor.lock.json` already records a lowercase-hex SHA-256 per vendored file, plus the archive URL and archive hash it came from. The hash this change needs already exists and is already audited.
- `crates/htmlspec-assets/src/lib.rs` embeds every asset with `include_bytes!` at a literal relative path. The compiler resolves these at build time, so any layout is fixed at compile time and cannot be computed from a runtime lookup without a build script.
- `flake.nix` builds `${./assets}/${file.path}` from the lockfile's `path` field and compares the committed bytes against the recorded hash. The `path` field is therefore load-bearing in both Nix and Rust.
- Documents reference assets by relative path (`href="assets/htmlspec.css"`), which is what makes them render offline.
- `scripts/check` is the single definition of a healthy revision, and both `nix flake check` and continuous integration call it.

## Goals / Non-Goals

**Goals:**

- One stored copy per distinct byte sequence, and coexistence of differing revisions, from one mechanism.
- An asset reference that names the bytes it expects, so a mismatch is detectable without a second source of truth.
- Preserve the offline guarantee: a document opens from disk with no network and no build step.
- Keep the existing provenance chain (lockfile hash, Nix re-fetch) working rather than replacing it.

**Non-Goals:**

- Garbage collection of unreferenced asset bytes. See Decisions.
- Deduplicating anything other than runtime assets. Document bodies are not content-addressed.
- Changing the set of approved libraries or their versions.
- Git LFS and `README.html`, both excluded in proposal.md — What Changes.

## Decisions

### Store assets at `assets/<algo>-<hash>/<filename>` rather than renaming the file

The stored path keeps the original filename inside a hash-named directory, so `assets/sha256-18327bef…/mermaid.min.js`. The alternative was renaming the file itself to the hash, as `assets/sha256-18327bef….js`.

Keeping the filename preserves the extension that a browser and a text editor use to choose a content type and a syntax mode, and it keeps a directory listing legible to a human, who can still tell Mermaid from Vega. A hash-named directory delivers the same uniqueness guarantee, because the directory name is what carries the hash.

The `<algo>-` prefix is deliberate. It makes the digest algorithm explicit at every reference site, so a future migration to another algorithm is a visible change rather than an ambiguous one.

### Truncate the hash in the path to 16 hex characters, and record the full digest in the lockfile

A full SHA-256 is 64 hex characters, which makes every asset reference in every document unreadable and every diff wide. 16 hex characters is 64 bits of digest.

The truncation is safe here because the store is closed. Entries come only from `vendor.lock.json` and the project's own stylesheets, all of which carry their full digest in the lockfile. Validation compares the full digest, not the truncated one. The truncated form is an addressing convenience, and a collision between two entries is detectable at store-build time by comparing full digests, which the check suite does.

Alternative considered: full 64-character paths. Rejected because the readability cost is paid in every document, in every diff, forever, and the collision risk the truncation introduces is eliminated by the closed store and the full-digest comparison.

### Derive stored paths from the lockfile at build time, not at runtime

`include_bytes!` needs a literal path. Rather than hand-maintaining hash paths in Rust source, a build script reads `assets/vendor.lock.json`, computes each stored path, and generates the `Asset` table. The lockfile stays the single source of truth for what the runtime is, which is what it already claims to be.

This keeps `flake.nix` working with a one-line change to its path construction: it already reads the same lockfile and the same `sha256` field, so it derives the same path by the same rule.

Alternative considered: a runtime directory scan. Rejected because it breaks the standalone-artifact requirement in `toolchain-packaging` — a packaged binary must run with no source checkout, so its assets must be embedded at compile time.

### Scope the reference rule by the presence of a store, not by a marker in the document

The rule resolves a document's asset references against stored bytes. A conformance fixture references `assets/htmlspec.css` and no such file has ever existed beside it: the reference is symbolic, present so the parser and the projection see a realistic document. Applying the rule unconditionally rejects the 5 valid and 2 warning fixtures, which assert acceptance, and those 37 inputs are the pin for every rule the project already has.

So the rule fires only where a project has an asset store. A document with no store around it has no bytes to disagree with.

Alternative considered: a reserved `fixture` class marking a document as exempt. Rejected on three counts. It adds a suppression switch inside the document for a rule the specs classify as non-downgradable, which is the same downgrade spelled as markup. It requires amending the closed vocabulary in `document-schema`, whose seven reserved classes each name a semantic unit of a specification rather than a file's role in a test suite. And the conformance requirement would then need a fixture proving the exemption works — a document declaring itself exempt to prove that documents declaring themselves exempt are skipped — which is circular. It also edits all 37 inputs, so it saves no work over rewriting them.

Alternative considered: a field in each fixture's `expected.json`. Rejected as unnecessary once the store's presence already answers the question, and it would add a harness concept and edit 37 files to record something derivable.

The specs state the scope explicitly, including that a document cannot exempt itself, so the marker cannot be reintroduced later as an implementation convenience.

### Never collect unreferenced assets

An archived document must keep rendering, and the only way to be sure its bytes survive is to never delete asset bytes. The store therefore grows monotonically.

The cost is bounded and small. Each vendored file has changed once in the project's history, giving 8 distinct blobs in an 8.8 MB repository. A rule that can delete bytes an archived document needs is more expensive than the disk it saves.

### Rewrite references in the formatter, not by hand

`format` rewrites a document's asset references to the current stored hashes. This makes the migration mechanical and keeps a document canonical.

This interacts with `deterministic-formatting`. Formatting must stay idempotent, so a second pass must be byte-identical, which holds because the rewrite is a pure function of the lockfile and the document. It must also not widen a diff beyond the units an edit touched, so reference rewriting must not reflow surrounding markup.

CAUTION: a formatter that rewrites references to *current* hashes will silently re-point an archived document at newer bytes. The formatter therefore rewrites references only when the referenced bytes are absent from the store, and reports rather than rewrites when the document names bytes that exist. Re-pointing a document is an explicit operation, not a side effect of formatting.

## Risks / Trade-offs

- **The formatter silently upgrades an archived document's runtime** → The formatter only fills in absent references, and reports a document naming bytes that exist rather than rewriting it. Re-pointing is explicit. A fixture pins this behavior.
- **Truncated-hash collision between two store entries** → The store is closed and every entry carries a full digest in the lockfile. The check suite compares full digests and fails on any truncated-form collision.
- **Nix and Rust derive stored paths by different rules and drift apart** → Both read `assets/vendor.lock.json` and apply one documented derivation. A check asserts the path Nix builds and the path the Rust build script generates agree for every lockfile entry.
- **Duplicate detection is a project-level property, so per-document validation cannot see it** → The rule lives in the check suite as well as the validator, per the third added requirement. Without that, the suite passes while the project violates the contract, which is the failure mode this change exists to fix.
- **Re-rendering the archived document alters a reviewed artifact** → The change is limited to asset references and the stylesheet the document renders with. Its semantic content is unchanged, which validation confirms by accepting the re-rendered document against the same contract.
- **The truncated path is not self-verifying to a reader** → A human cannot confirm 64 bits by eye. This is why the rule is machine-checked and classified as a non-downgradable error.

## Migration Plan

1. Build the content-addressed store from `assets/vendor.lock.json` and the project stylesheets, leaving the existing `assets/vendor/` tree in place.
2. Generate the Rust asset table from the lockfile, and update `flake.nix` path construction to the same derivation. The provenance check must still pass at this point, proving the bytes did not change.
3. Add the two validation rules and their conformance fixtures, plus the check-suite wiring. The suite now fails on the existing duplication, which is the expected signal.
4. Rewrite references in the project's documents, including the archived `deterministic-formatting.html`, which is re-rendered against the current stylesheet so its declared skip link works.
5. Remove the duplicated asset trees, including the 6.0 MB copy under the archived change. The suite now passes.
6. Delete the old `assets/vendor/` layout once nothing references it.

Rollback: steps 1 and 2 are additive and leave the old layout intact, so a revert before step 5 restores the prior state without data loss. After step 5 the duplicated copies are gone, and recovery is by Git revert, which is safe because the bytes remain in history.
