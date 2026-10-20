# The two defects, reproduced before the change

**Date:** 2026-10-14
**Binary:** `htmlspec 0.29.1 (cbcfaa87fdb2)`, built from `main` at `cbcfaa8`, before any edit.

## How they were found

The code review of pull request #57 found that the first change this repository archived with
`htmlspec archive`, `2026-10-14-resolve-a-new-documents-project-from-its-destination`, came out of
archival with every document pointing at a store that does not exist. Each of its four documents
referenced `../../assets/`, which is right from `htmlspec/changes/<name>/` and one level short from
`htmlspec/changes/archive/<date>-<name>/`. None rendered its stylesheet or loaded its scripts, and
task 3.1's link to the accepted requirement, `../../specs/…`, led nowhere. `htmlspec validate` reported
nothing. The 29 references were rewritten by hand in #57, and its task 5.3 records that.

## The reproduction

A scratch project, made with `git init` and `htmlspec init --tools none`, its `spec.nix` set to
`0.1.0`, and one change, `sample`, made with `htmlspec new change` and given a specification with one
requirement. Then:

```
$ htmlspec archive sample --yes
archived to <scratch>/htmlspec/changes/archive/2026-10-14-sample at version 0.1.0

$ grep -o 'href="[^"]*htmlspec.css"' htmlspec/changes/archive/2026-10-14-sample/*.html
htmlspec/changes/archive/2026-10-14-sample/design.html: href="../../assets/sha256-03627137bcbb81c9/htmlspec.css"
htmlspec/changes/archive/2026-10-14-sample/proposal.html: href="../../assets/sha256-03627137bcbb81c9/htmlspec.css"
htmlspec/changes/archive/2026-10-14-sample/specification.html: href="../../assets/sha256-03627137bcbb81c9/htmlspec.css"
htmlspec/changes/archive/2026-10-14-sample/tasks.html: href="../../assets/sha256-03627137bcbb81c9/htmlspec.css"

$ ls -d htmlspec/changes/assets
"htmlspec/changes/assets": No such file or directory (os error 2)

$ htmlspec validate htmlspec/changes/archive/2026-10-14-sample
4 document(s): 0 error(s), 7 warning(s)
validate exit 0
```

The seven warnings are `review/*`. No `asset/*` diagnostic is among them.

## Why each hides the other

**Archival moves the directory and rewrites nothing.** `perform` in
`crates/htmlspec-cli/src/commands.rs` stages the recorded version and the runbook inventory into the
change, then renames the change directory into the archive. A change's documents reference the store
and the accepted specifications by relative paths written for `htmlspec/changes/<name>/`, and the
rename puts them one level deeper.

**Validation resolves an asset reference by the address it names, not by its path.** `store_for` finds
the first `assets/` directory in the document's directory or above it, and `Store::resolve` in
`crates/htmlspec-assets/src/store.rs` looks up the reference's last two components, its address and
its file name, in that store: "Only its last two components matter, because a stored asset is an
address and a file name however deep the document sits." So `../../assets/sha256-03627137bcbb81c9/htmlspec.css`
matches the bytes stored at `htmlspec/assets/sha256-03627137bcbb81c9/htmlspec.css`, although the path
reaches `htmlspec/changes/assets/`.

Archival's fault produces exactly the references validation's fault accepts, so no check this
repository runs could see either.
