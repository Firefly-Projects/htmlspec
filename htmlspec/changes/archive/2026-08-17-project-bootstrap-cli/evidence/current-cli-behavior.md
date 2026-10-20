# Current CLI behavior in a fresh repository

Measured 2026-08-14 against `target/debug/htmlspec` built from `9c3a3c7`
(`Separate a change into four artifact kinds`). Every transcript below is verbatim.

## 1. There is no command that establishes a project

```
$ git init -q . && htmlspec
Validate, format, and create HTMLSpec documents

Usage: htmlspec [OPTIONS] <COMMAND>

Commands:
  validate  Validate documents against the contract
  format    Rewrite documents into their canonical form
  assets    Check a project's asset topology
  archive   Check a change's specification before it is archived
  init      Create a new document that validates immediately
  help      Print this message or the help of the given subcommand(s)
...
$ echo $?
2
```

`init` is the only candidate, and it describes itself as creating a document.

```
$ htmlspec init
error: the following required arguments were not provided:
  <PATH>

Usage: htmlspec init --change <PATH>

For more information, try '--help'.
$ echo $?
2
```

There is no argument meaning "set this repository up," because the operation does
not exist. Note also that the usage line renders as `init --change <PATH>` even
though `--change` was not given.

## 2. Pointing init at the repository root is refused

```
$ htmlspec init . --change
htmlspec: . already exists; init will not write into an existing change directory
$ echo $?
2
```

The obvious guess fails, and the message reports a fact the contributor already
knew. `commands::init` returns `OperationalFailure` on `path.exists()`, and the
repository root always exists.

## 3. Each change carries its own copy of the runtime

```
$ htmlspec init htmlspec/changes/a --change
established 8 shared asset(s) in htmlspec/changes/a/assets
htmlspec/changes/a/proposal.html: created a proposal
htmlspec/changes/a/design.html: created a design
htmlspec/changes/a/tasks.html: created a tasks
htmlspec/changes/a: created a change with 3 document(s). ...

$ htmlspec init htmlspec/changes/b --change    # same, into b/

$ du -sh htmlspec/changes/a/assets htmlspec/changes/b/assets
6.0M	htmlspec/changes/a/assets
6.0M	htmlspec/changes/b/assets
```

Both stores hold the same eight addresses. `commands::init` derives the store from
the *document's* directory when `--assets` is absent:

```rust
let assets_directory = match assets_root {
    Some(root) => root.join("assets"),
    None => document_directory.join("assets"),
};
```

So one store per change is the default, and one store per project requires the
contributor to pass `--assets` on every invocation.

## 4. The command that exists to catch this cannot see it

```
$ htmlspec assets
no asset store under ., so there is nothing to check
$ echo $?
0
```

`commands::assets` calls `Store::discover(root)` on the directory named and does
not descend. With 12 MB of duplicated assets two directories down, it reports
nothing and exits zero.

The duplicate-detection requirement in `toolchain-packaging` is therefore
satisfied by a command that cannot observe the duplication the tool's own default
behavior produces.

## 5. What does work

Not everything is broken, and the parts that work are why this change is
additive rather than a rewrite:

```
$ htmlspec validate
validating ./htmlspec/changes/a/design.html
  no diagnostics: the document is accepted
... (3 documents)

3 document(s): 0 error(s), 0 warning(s)
$ echo $?
0
```

`validate`, `format`, the four kind templates, and the exit-code contract are all
sound. What is absent is the layer above them whose subject is a project.

## Exit-code survey

| Invocation | Code | Reading |
| --- | --- | --- |
| `htmlspec` (bare) | 2 | clap usage error |
| `htmlspec init` | 2 | missing required argument |
| `htmlspec init . --change` | 2 | operational failure |
| `htmlspec validate` (no documents) | 0 | correct; nothing to validate |
| `htmlspec assets` (no store) | 0 | correct per spec, misleading here |
| `htmlspec archive foo` | 2 | operational failure |

No code currently means "the contributor declined," which is the gap
`Outcome::Cancelled` fills once a prompt exists.

## Upstream comparison

`openspec@1.7.0` at `/opt/homebrew/lib/node_modules/@fission-ai/openspec`.
`dist/core/init.js` is 799 lines. Its `execute` sequence: validate write
permissions → pointer guard → legacy cleanup → tool detection → animated welcome
screen → searchable multi-select → directory structure → per-tool skill and
command generation → config → success message naming the next command.

Its tool table (`dist/core/config.js`) holds 35 entries. This change ports five.
Its delivery decision is spread across four predicates in
`dist/core/command-surface.js` and can represent a tool that receives nothing —
handled by printing a correction paragraph at the end of `init`. The `Delivery`
sum type in this change's design exists to make that state unconstructible.
