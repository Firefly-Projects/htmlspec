# The measurement, repeated before anything changed

Tasks 1.1 and 1.2, run in a `git clone` of `51c5aea`, the merge of #35 and 0.16.0, with the
`htmlspec 0.16.0` built from that tree. A clone rather than an exported copy, because
`ProjectRoot::resolve` stops at a `.git` boundary, and a copy without one would resolve against
whatever lies above it.

## 1.1 Nine duplicates, and one of them is not the project's

Before `init`:

```
note: . is not a project, so its 5 stores were each checked on their own. …
checking 14 stored asset(s) in 5 store(s) under .
each stored asset appears once
```

After `htmlspec init --tools none`, `htmlspec assets .` exits 1:

```
checking 22 stored asset(s) in 6 store(s) under .
9 duplicated asset(s): the project stores bytes it could share
```

**The count is nine, as the proposal says, and the nine are two different cases.**

- **Eight** are the eight runtime files at their current addresses, in both `./assets/` and
  `./htmlspec/assets/`: `htmlspec.css` (`aac8967609f80d9f`), `htmlspec-print.css`
  (`04ac9a24fc436a87`), `htmlspec.js`, `mermaid.min.js`, `vega.min.js`, `vega-lite.min.js`,
  `vega-embed.min.js`, and `vega-lite-schema.json`. This is the toolchain's source copy
  against the project's store. The two revisions `retained` since 0.16.0 are not among them,
  because `init` stores the current revision and those bytes differ.
- **One** is between two conformance fixtures, not between a fixture and the project:
  `fixtures/conformance/invalid-asset-unresolved/assets/sha256-d8779069bc923a3c/htmlspec.css`
  and `fixtures/conformance/valid-asset-reference-matches-stored-bytes/assets/…/htmlspec.css`.
  Two fixtures carry the same fixture stylesheet. Each fixture is a project of its own, so this
  is two projects sharing bytes. The note the no-project path already prints calls that "not
  duplication".

The proposal describes the nine as "all five stores against the new one". The ninth shows the
comparison is wider than that: once a project resolves, every store is compared with every other,
including two stores neither of which is the project's. The partition the design describes removes
both cases. Neither fixture store lies under `htmlspec/`, so each is checked on its own and compared
with nothing.

## 1.2 Only the topology check changes

Each document stage of `scripts/check`, plus a validation of every fixture document, was run in the
copy with and without the `htmlspec/` and `spec.nix` that `init` created
(`validate fixtures/dogfood`, `format --check fixtures/dogfood`, `validate runbooks`,
`format --check runbooks`, `validate fixtures`). **Every result is identical in the two runs**, per
document, down to the count of each error and warning line. Across the whole tree,
`validate fixtures` exits 1 in both, because the invalid conformance fixtures are invalid on purpose.

The one difference is `validate htmlspec`, which exits 2 without the directory, because the path
does not exist, and 0 with it. That is a fact about the path, not about the check.

**The proposal's claim holds: the project's existence changes the topology check and nothing
else.** Asset references resolve against the nearest store above each document, and every document
outside `htmlspec/` has a nearer store than the project's, or none, in the fixtures' deliberate case.

## 4.2 The same copy, against the built change

The same clone and the same `init`, with the binary built from this change:

```
$ htmlspec assets .
note: 1 store(s) within the project's htmlspec/ were compared with one another. 5 store(s)
outside it belong to something else and were each checked on its own: ./assets,
./fixtures/conformance/invalid-asset-mismatch/assets,
./fixtures/conformance/invalid-asset-stale-stylesheet-reference/assets,
./fixtures/conformance/invalid-asset-unresolved/assets,
./fixtures/conformance/valid-asset-reference-matches-stored-bytes/assets
checking 22 stored asset(s) in 6 store(s) under .
each stored asset appears once
```

**Zero duplicates, exit 0**, as the design predicts. The structured payload reports
`"projectStores": ["./htmlspec/assets"]` and the five others under `"otherStores"`, with
`"stores": 6` and `"storedAssets": 22` unchanged in meaning.

The check still reports a fault that is the project's. A copy of `htmlspec/assets/` beside a
change's documents, `htmlspec/changes/a-change/assets/`, in the same tree reports
`8 duplicated asset(s)` and exits 1. It was removed after the measurement.

**A readability fix made on the way.** The first run of the built change named each store by the
directory holding it, as discovery returns it, so the repository root's store read as a bare `.`.
Stores are now named by their `assets/` directory in the note and in both payload lists.
