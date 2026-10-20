# Relocation, and the orphaned-reference gap

Measured 2026-08-14 against `target/debug/htmlspec` built from `9c3a3c7`. Every
transcript is verbatim. The first section answers a question that was asked; the
second records a defect found while answering it.

## 1. A whole-project copy relocates cleanly

Asset references are relative by construction. `relative_reference` in
`crates/htmlspec-cli/src/commands.rs:760` computes the ascent from the document to
the store and returns `None` where no relative path exists, so the writer cannot
emit an absolute path.

For a change three levels below the project root:

```
$ grep -o 'href="[^"]*"\|src="[^"]*"' htmlspec/changes/add-dark-mode/proposal.html | sort -u
href="../../../assets/sha256-0350261bb5666167/htmlspec-print.css"
href="../../../assets/sha256-e4f5de7cecc4f691/htmlspec.css"
href="#hs-content"
src="../../../assets/sha256-18327bef70d96fb5/mermaid.min.js"
src="../../../assets/sha256-35a9821df838825b/vega-lite.min.js"
src="../../../assets/sha256-70bfdc84b15f11f3/vega.min.js"
src="../../../assets/sha256-84427613110957a7/htmlspec.js"
src="../../../assets/sha256-c36254270219eee5/vega-embed.min.js"

$ grep -Eo '(href|src)="(/|[a-z]+://)[^"]*"' .../proposal.html
none
```

Seven asset references, one fragment link, no absolute path and no remote URL.

Copied and renamed — the harder case, since a rename defeats anything that
recorded the original path:

```
$ cp -R proj moved-elsewhere && mv moved-elsewhere totally-different-name
$ cd totally-different-name && htmlspec validate
  no diagnostics: the document is accepted
3 document(s): 0 error(s), 0 warning(s)

  OK   ../../../assets/sha256-0350261bb5666167/htmlspec-print.css
  OK   ../../../assets/sha256-18327bef70d96fb5/mermaid.min.js
  OK   ../../../assets/sha256-35a9821df838825b/vega-lite.min.js
  OK   ../../../assets/sha256-70bfdc84b15f11f3/vega.min.js
  OK   ../../../assets/sha256-84427613110957a7/htmlspec.js
  OK   ../../../assets/sha256-c36254270219eee5/vega-embed.min.js
  OK   ../../../assets/sha256-e4f5de7cecc4f691/htmlspec.css
```

7 of 7 resolve. Offline rendering and relocatability are the same property: a
document that opens from a file path cannot rely on a server, and therefore
cannot use an absolute path either.

## 2. The boundary: one change moved out of its project

```
$ cp -R proj/htmlspec/changes/add-dark-mode orphan/
$ cd orphan/add-dark-mode
  MISS ../../../assets/sha256-0350261bb5666167/htmlspec-print.css
  MISS ../../../assets/sha256-18327bef70d96fb5/mermaid.min.js
  MISS ../../../assets/sha256-35a9821df838825b/vega-lite.min.js
```

Expected, and stated as a trade in the proposal. The references still ascend three
levels; above the orphan there is no store to ascend to.

## 3. The defect: strict validation accepts the orphan

```
$ ls -d assets
ls: assets: No such file or directory

$ htmlspec validate . --strict
  no diagnostics: the document is accepted
3 document(s): 0 error(s), 0 warning(s)

$ htmlspec validate proposal.html --json
accepted: True
diags: 0
```

Three documents, twenty-one dead asset references between them, zero diagnostics,
accepted under `--strict`.

### Why

Two mechanisms combine, and both are individually correct.

`store_for` (`commands.rs:212`) walks upward for a store and returns
`Store::Absent` when it finds none. Then every asset rule opens with a guard.
From `crates/htmlspec-validate/src/rules/assets.rs:17-21`:

> references `assets/htmlspec.css` and no such file has ever sat beside it: the
> reference is symbolic, present so the parser sees a realistic document. Resolving
> against a store that is absent would reject every fixture in the suite, and those
> inputs are the pin for every rule this project has. So the rules read
> `[Store::is_present]` and return early when there is no store.

The reasoning holds. The consequence is that "this document never had assets" and
"this document lost its assets" produce the same verdict.

### The signal that separates them

A conformance fixture's asset references do not ascend. A generated document's do.
Measured across the whole fixture suite:

```
$ grep -rho '\(href\|src\)="[^"]*assets[^"]*"' fixtures/conformance/ | wc -l
     341
$ grep -rho '\(href\|src\)="[^"]*assets[^"]*"' fixtures/conformance/ | grep -c '\.\./'
0
```

**0 of 341** fixture asset references contain `../`. Sample fixture references:

```
href="assets/htmlspec.css"
href="assets/sha256-d8779069bc923a3c/htmlspec.css"
```

Against a generated document's:

```
href="../../../assets/sha256-e4f5de7cecc4f691/htmlspec.css"
```

So ascent distinguishes the two cases exactly. An ascending reference asserts a
store above the document, and no store found above it contradicts the reference
rather than placing the document out of scope. A non-ascending reference with no
store stays silent, which keeps all 41 fixtures passing.

Reference shape is not an escape hatch a document can use. It is written by
whatever generated the document, and editing a reference to stop ascending breaks
the reference itself.

### Rejected alternatives

- **Require a store unconditionally.** Fails 41 fixtures, which are the pin for
  every rule the project has.
- **Key the rule on `spec.nix` being present.** Reads better than it works: a
  change directory carries no `spec.nix`, so an orphan looks storeless and
  unmarked exactly as it does now.

## 4. What is pinned for the config file

```
$ htmlspec --version
htmlspec 0.1.0 (9c3a3c79b1fe-dirty)
```

The binary knows its crate version and revision. No project records which binary
it expects, so the reproducibility requirement — two machines produce identical
results from the same revision — is currently kept by convention: both enter the
same development shell. `spec.nix` makes that expectation a checkable fact.

The flake already pins `nixpkgs`, `rust-overlay`, `crane`, and `flake-utils`, and
exposes `packages.default`. A pinned binary reference is a flake input with a
hash, which is why the declaration is Nix rather than YAML.
