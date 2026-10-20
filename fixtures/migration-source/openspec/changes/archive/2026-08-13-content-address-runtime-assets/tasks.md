## 1. Derive stored paths from the lockfile

- [x] 1.1 Document the path derivation rule (`assets/<algo>-<truncated-hash>/<filename>`, 16 hex characters) in `assets/vendor.lock.json`'s comment block, so the lockfile states the rule it is now the input to
- [x] 1.2 Add a function in `crates/htmlspec-assets` that maps a lockfile entry's full `sha256` and file name to its stored path, with unit tests covering the truncation length and the algorithm prefix
- [x] 1.3 Add a test asserting no two lockfile entries share a truncated hash, so a collision in the closed store fails the suite rather than silently aliasing
- [x] 1.4 Extend the derivation to the project's own stylesheets and script (`htmlspec.css`, `htmlspec-print.css`, `htmlspec.js`, `template.html`), which have no lockfile entry today, by recording their hashes alongside the vendored entries

## 2. Build the content-addressed store

- [x] 2.1 Populate the store from the lockfile and the project stylesheets, leaving the existing `assets/vendor/` tree in place so this step is additive
- [x] 2.2 Generate the Rust `Asset` table from the lockfile in a build script, replacing the hand-written `include_bytes!` paths in `crates/htmlspec-assets/src/lib.rs`
- [x] 2.3 Update `flake.nix` path construction in `assetProvenance` to use the same derivation, and confirm the re-fetch and hash comparison still pass against the new layout
- [x] 2.4 Add a check asserting the path Nix builds and the path the Rust build script generates agree for every lockfile entry, so the two cannot drift apart
- [x] 2.5 Verify `cargo test -p htmlspec-assets` still verifies committed bytes against recorded hashes offline
- [x] 2.6 Add `Store::discover`, which distinguishes an absent store from an empty one, resolves a reference to Match/Absent/Mismatch/NotAddressed, and reports duplicates across addressed and unaddressed files together
- [x] 2.7 Replace the template's hardcoded asset paths with `{{ASSET:<name>}}` placeholders resolved at render time, so no hash is ever written into the template
- [x] 2.8 Repair `conformance/tests/presentation.rs`, which read a document directory that moved when the previous change was archived. Discover change document directories under both `openspec/changes/` and `openspec/changes/archive/` so archiving cannot silently drop documents from these checks. This failure pre-existed this change and was pushed in `6956324`

## 3. Validate asset references

- [x] 3.1 Add the rule rejecting a reference whose named bytes are absent from the store, with a stable diagnostic identifier
- [x] 3.2 Add the rule rejecting a reference whose stored bytes hash differently from the name, with its own distinct diagnostic identifier
- [x] 3.3 Classify both rules as errors that cannot be downgraded to warnings, alongside the existing safety rules
- [x] 3.4 Scope both rules to a document validated within a project that has an asset store, so a document with no store around it has its references left unresolved
- [x] 3.5 Verify the existing 37 conformance fixtures still pass unchanged, since their references are symbolic and no store sits beside them
- [x] 3.6 Add a test that a marker inside a document cannot exempt it, so the scope stays a property of the store rather than a switch a document carries
- [x] 3.7 Add conformance fixtures pinning each rule: one valid and one invalid per rule, each invalid fixture declaring the rule identifier it violates
- [x] 3.8 Add a fixture for the stale-reference case, where a document names one stylesheet revision and a different revision is stored

## 4. Validate project asset topology

- [x] 4.1 Add the rule rejecting a project that stores identical asset bytes at more than one location, reporting every location holding them
- [x] 4.2 Confirm the rule does not report differing revisions of one file as duplicates, and reports nothing for a directory with no store
- [x] 4.3 Wire the topology rule into `scripts/check`, since it is a project-level property that per-document validation cannot observe
- [x] 4.4 Add conformance fixtures for the duplicate case, the copy-per-document case, and the distinct-revisions case
- [x] 4.5 Verify the suite now fails on the project's existing duplication, which is the expected signal before the migration completes

## 5. Reference rewriting in the formatter

- [x] 5.1 Rewrite an asset reference only when the referenced bytes are absent from the store, and report rather than rewrite when the named bytes exist
- [x] 5.2 Add a fixture pinning that behavior, so a formatter run cannot silently re-point an archived document at newer bytes
- [x] 5.3 Confirm formatting stays idempotent under the new rewrite: a second pass is byte-identical
- [x] 5.4 Confirm reference rewriting does not widen a diff beyond the reference itself, per the diff-churn requirement

## 6. Initialization

- [x] 6.1 Have `init` write asset references matching the bytes the project stores
- [x] 6.2 Have `init` reuse an already-stored asset rather than writing a second copy
- [x] 6.3 Verify a document initialized into a project with no assets still renders offline without a further step
- [x] 6.4 Add a test that initialization cannot emit a reference whose hash disagrees with the stored bytes, which is the defect class this change exists to eliminate

## 7. Migrate existing documents

- [x] 7.1 Rewrite asset references in the project's own documents, including `README.html`, to the stored hashes. **`README.html` needed no rewrite:** it inlines its assets as `data-inline` script elements and carries no `assets/` reference at all, and proposal.md excludes it from this change. The archived `deterministic-formatting.html` was the only project document with references to rewrite
- [x] 7.2 Re-render the archived `openspec/changes/archive/2026-08-10-html-spec/documents/deterministic-formatting.html` against the current stylesheet, so its declared `hs-skip-link` works
- [x] 7.3 Confirm the re-rendered document still validates against the same contract and its semantic content is unchanged
- [x] 7.4 Remove the duplicated 6.0 MB asset tree under the archived change
- [x] 7.5 Remove the old `assets/vendor/` layout once nothing references it

## 8. Verify the whole property

- [x] 8.1 Confirm `scripts/check` passes, including the new asset rules
- [x] 8.2 Confirm each library appears exactly once in the project, satisfying the shared-assets scenario that is currently violated
- [x] 8.3 Confirm the skip-link defect is fixed by opening the re-rendered document and reaching the skip link by keyboard. **Verified interactively by Scott in Firefox on 2026-08-13:** Tab reveals "Skip to content" and Enter reaches the content region. The static conditions (link first focusable, target present, stylesheet carries both the hidden and `:focus` rules) are recorded in `evidence/group-8-verification.md`; the keyboard interaction itself needed a person, because this machine has no Chrome and no WebDriver
- [x] 8.4 Reintroduce each defect the new rules exist for and confirm the suite fails, since a check that has never failed is a check nobody has tested. All three new rules fail on their defect. **Found a coverage boundary:** a document honestly naming an older revision that lacks a class it declares is accepted by `asset/*`; the archive's actual defect (an unaddressed reference) is caught by `format --check` instead. Recorded in `evidence/group-8-verification.md`
- [x] 8.5 Record evidence for the measured claims: store size before and after, the duplication removed, and the browser verification of the skip link
