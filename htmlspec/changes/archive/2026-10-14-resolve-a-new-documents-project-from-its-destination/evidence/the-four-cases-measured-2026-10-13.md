# The four cases, measured before the change

**Date:** 2026-10-13
**Binary:** `htmlspec 0.29.0 (139b206f26b6-dirty)`, built from `139b206` while the promoted
`htmlspec/` sat untracked in the working tree. No source file differed from `139b206`.

## How it was found

After `htmlspec migrate promote` placed `htmlspec/` and `spec.nix` in this repository (commit
`071d67a`), `scripts/check` failed one test:

```
---- new_document_creates_a_document_that_validates_immediately stdout ----
thread 'new_document_creates_a_document_that_validates_immediately' panicked at crates/htmlspec-cli/tests/cli.rs:1060:5:
assertion `left == right` failed: /tmp/nix-shell.lE62Sp/htmlspec-cli-init-33892/proposal.html: created a proposal

  left: 2
 right: 0
```

The test runs `new document` with the repository as its working directory and a temporary
directory as the destination. Before the promotion the repository held no `htmlspec/`, so no project
was found and the document got a store beside it. After the promotion the working directory resolves
to this repository's project, and the document is pointed at a store it cannot reach.

`new_document` in `crates/htmlspec-cli/src/commands.rs` resolves the project from the working
directory:

```rust
let assets_root = ProjectRoot::resolve(Path::new("."))
    .ok()
    .map(|project| project.htmlspec());
```

## The four arrangements

A scratch project made with `htmlspec init --tools none` inside a fresh `git init`, and a sibling
directory outside it. `<scratch>` stands for the scratch directory. Each line is the command's exit
status, then the first lines of what it printed.

| Case | Run from | Destination | `new document` | `validate` of the result |
| --- | --- | --- | --- | --- |
| A | inside the project | outside it | exit 2 | exit 1, 7 errors |
| B | inside the project | inside the project, outside `htmlspec/` | exit 2 | exit 1, 7 errors |
| C | outside the project | inside `htmlspec/` | exit 0 | the document validates; `htmlspec assets` exits 1 |
| D | outside the project | outside it | exit 0 | exit 0, accepted |

**A.** `new document proposal <scratch>/elsewhere/a/proposal.html`, run in the project:

```
[A new] exit 2
<scratch>/elsewhere/a/proposal.html: created a proposal
error: asset/unresolved at /link[href]: <link href="../../project/htmlspec/assets/sha256-03627137bcbb81c9/htmlspec.css"> reaches above this document for an asset store, and there is none in any directory above it. …
[A validate] exit 1
```

**B.** `new document proposal docs/proposal.html`, run in the project:

```
[B new] exit 2
docs/proposal.html: created a proposal
error: asset/unresolved at /link[href]: <link href="../htmlspec/assets/sha256-03627137bcbb81c9/htmlspec.css"> reaches above this document for an asset store, and there is none in any directory above it. …
[B validate] exit 1
```

Validation finds a document's store as the first `assets/` directory in the document's own directory
or above it (`store_for` in `crates/htmlspec-cli/src/commands.rs`). For `docs/proposal.html` that
search reaches `docs/` and the project root, and neither holds `assets/`. The project's store is
`htmlspec/assets/`, which is not above `docs/`.

**C.** `new document proposal <scratch>/project/htmlspec/changes/c/proposal.html`, run outside the
project:

```
[C new] exit 0
<scratch>/project/htmlspec/changes/c/proposal.html: created a proposal
[C] stores in the project: htmlspec/assets htmlspec/changes/c/assets
[C assets] exit 1
error: asset/duplicate at /assets/03627137…: these 2 locations hold identical bytes (03627137bcbb81c9), in 2 separate store(s): <scratch>/project/htmlspec/assets/sha256-03627137bcbb81c9/htmlspec.css, <scratch>/project/htmlspec/changes/c/assets/sha256-03627137bcbb81c9/htmlspec.css. …
```

**D.** `new document proposal <scratch>/elsewhere/d/proposal.html`, run outside the project:

```
[D new] exit 0
<scratch>/elsewhere/d/proposal.html: created a proposal
[D validate] exit 0
  0 errors, 1 warning: the document is accepted
```

The one warning is `review/example-content`, for the template's placeholder summary, which every new
document carries.

## What the measurement shows

Three of the four arrangements go wrong, and the one that works is the one where the working directory
and the destination agree that there is no project. The working directory decides which store the
document references, and the document's own location decides which store validation looks for. Where
the two differ, the document references a store that validation will not find (A, B), or the project
gains a second store (C).
