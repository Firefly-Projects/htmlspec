# What a path check would change, measured before the change

**Date:** 2026-10-14
**Tree:** this repository at `cbcfaa8`.

A rule that checks an asset reference's path, as well as its address, rejects a reference the
address-only rule accepts exactly when the address is held by the store validation finds and the
path reaches some other directory. This counts those references, so the change knows what it would
newly reject before it rejects anything.

## Method

A script walks every `.html` file in the repository, leaving out `.git/`, `target/`, `.claude/`,
every `evidence/` directory, and every `assets/` directory, whose files are stored bytes rather than
documents. For each document it finds the store as `store_for` does, the first `assets/` directory in
the document's directory or above it. It then reads each value of an attribute through which an element
loads an asset (`link href`, `script src`, `img src`, and the rest of `REFERENCING_ATTRIBUTES`), keeps
those whose last two components are an address and a file name the store holds (the references
validation accepts today), and resolves each one's directory part against the document's directory,
lexically. A reference counts when that directory is not the store's.

## Result

```
(root)                  1 documents,     0 addressed references accepted today,    0 reaching another directory, in 0 document(s)
fixtures              124 documents,     2 addressed references accepted today,    0 reaching another directory, in 0 document(s)
htmlspec               13 documents,    91 addressed references accepted today,    0 reaching another directory, in 0 document(s)
htmlspec (archive)    148 documents,  1036 addressed references accepted today,    0 reaching another directory, in 0 document(s)
openspec                4 documents,    26 addressed references accepted today,   21 reaching another directory, in 3 document(s)
runbooks                2 documents,    14 addressed references accepted today,    0 reaching another directory, in 0 document(s)
```

**In everything this repository validates, a path check changes no verdict.** The 161 documents under
`htmlspec/`, after #57's hand repair, the 124 fixtures, the release runbook's two documents, and
`README.html`: 1143 references accepted, and every one reaches its store.

The 21 references are in the three hand-converted documents under
`openspec/changes/archive/2026-08-14-separate-artifact-kinds/documents/`. Each references
`../../../../assets/`, which from that directory is `openspec/assets/`, a directory that does not exist.
The store validation finds for them is the repository's root `assets/`. So they are broken on disk today,
the address-only rule accepts them, and no check this repository runs validates `openspec/`. The promoted
project carries the same three files under that change's `evidence/documents/`, where validation does not
read them.

## After the fix, measured by the program itself

**Binaries:** `htmlspec 0.29.1`, built from `cbcfaa8`, and this change's build after tasks 2.1 to 2.3.
Each validated the same trees with `--json`, and a script compared each document's verdict.

```
htmlspec: 164 documents, accepted 164 before and 164 after, 0 verdict(s) changed
fixtures: 124 documents, accepted 46 before and 46 after, 0 verdict(s) changed
runbooks: 2 documents, accepted 2 before and 2 after, 0 verdict(s) changed
README.html: 1 documents, accepted 0 before and 0 after, 0 verdict(s) changed
openspec: 7 documents, accepted 6 before and 0 after, 6 verdict(s) changed
  openspec/changes/archive/2026-08-14-separate-artifact-kinds/documents/design.html: accepted True -> False
  openspec/changes/archive/2026-08-14-separate-artifact-kinds/documents/proposal.html: accepted True -> False
  openspec/changes/archive/2026-08-14-separate-artifact-kinds/documents/tasks.html: accepted True -> False
  openspec/changes/archive/2026-09-11-migrate-an-openspec-project/evidence/a-migrated-design-2026-09-09.html: accepted True -> False
  openspec/changes/archive/2026-09-11-migrate-an-openspec-project/evidence/the-collapse-produced-2026-09-09.html: accepted True -> False
  openspec/changes/archive/2026-09-11-migrate-an-openspec-project/evidence/the-plan-for-this-repository-2026-09-09.html: accepted True -> False
```

The 164 under `htmlspec/` are the 161 measured above and this change's three documents. `README.html`
and 78 of the fixtures fail for reasons this change does not touch, and fail the same way before and
after.

**The measurement above undercounted `openspec/`.** It left out every `evidence/` directory, so it
missed three records under `2026-09-11-migrate-an-openspec-project/evidence/`. Each references
`assets/sha256-…/`, a store beside it that does not exist; the addresses are held by the repository's
root `assets/`, which is the store validation finds for them, so 0.29.1 accepted them. They are now
rejected, with the path each one leads to. No check this repository runs validates `openspec/`, and
the promoted project keeps the same records in its own evidence, where validation does not read them.
So the conclusion stands: in everything this repository validates, the path check changes no verdict.
