## Context

Every command that collects documents by walking a directory calls one function,
`crate::discover::documents(root)`. It returns `root` itself when `root` is a file. Otherwise it
returns every `.html` file beneath `root` in sorted order, skipping `.git`, `.jj`, `assets`,
`node_modules`, `result`, `target`, and dot-directories. Its callers in
`crates/htmlspec-cli/src/commands.rs`:

| Caller | Walks |
| --- | --- |
| `validate` | the named path, or `htmlspec/` when none is named |
| `format` (writing and `--check`) | the same |
| `list --specs` | `htmlspec/specs/` |
| the accepted-heading lookup for renames | `htmlspec/specs/` |
| promotion's `conformance` | the whole staged project |

The layout itself (`specs/`, `changes/`, `changes/archive/`, `runbooks/`) is derived in one place,
`ProjectRoot` in `crates/htmlspec-cli/src/project.rs`, whose header argues that a path a caller
composes is a path that can drift. `ProjectRoot::resolve` walks upward from a path to the nearest
`htmlspec/`, stopping at `.git`, and returns an absolute path. It does not canonicalize.

`confine-the-asset-check-to-the-project` solved the same shape of problem for stores: a
positional question ("is this store the project's?") answered on canonical paths against a
location `ProjectRoot` derives.

## Goals / Non-Goals

**Goals:**

- One definition of "a change's evidence directory", in the module that owns the layout.
- Every walking caller reaches it without being able to forget to. The spec requires that the
  exclusion is "defined once and reached by every such operation".
- A named path is never excluded. The walk's root is exempt, and only directories the walk comes
  across can be skipped.

**Non-Goals:**

- Stores beneath evidence. `discover::stores` is untouched, and so is the topology partition.
  No evidence in this repository holds a store, and the duplicate check reads stores rather than
  documents. A later change can decide this against a real case.
- Reporting how many evidence files a walk skipped. `validate` already prints one
  `validating <path>` progress line per document, so a contributor can see which files were
  checked. A new count in the human report and the JSON payload would be a new output contract
  for a fact the spec does not ask to be reported.
- Anything about what an evidence file contains. Its format is its author's choice, and nothing
  reads it.

## Decisions

### The definition lives on `ProjectRoot`

`ProjectRoot` gains one method answering whether a directory is a change's evidence directory:

- its name is `evidence`; and
- its parent is a change directory. A change directory is either a direct child of `changes()`
  other than `archive()`, or a direct child of `archive()`.

The spelling `evidence` becomes a private constant beside `SPECS`, `CHANGES`, `ARCHIVE`, and
`RUNBOOKS`, for the reason that module's header gives: a path a caller composes is a path that can
drift.

**Considered: a name test in `discover.rs`**, skipping any directory called `evidence` the way
`target` is skipped. It is one line, and it is wrong. A capability whose path is `evidence`, at
`htmlspec/specs/evidence/spec.html`, would stop being validated, and nothing would report it. The
spec's "A capability that happens to be named evidence" scenario exists to catch this.

**Considered: excluding `changes/archive/evidence`.** This is not a change's evidence. `archive/`
is not a change, so a directory named `evidence` directly inside it is an unexpected entry, and it
stays walked. A directory called `archive` directly within `changes/` is the archive, never a change
named "archive", so its `evidence/` child is not a change's either.

### The comparison is on canonical paths

The walk composes paths from whatever root it was given, which may be `.`, `htmlspec`, or a path
through a symbolic link. `ProjectRoot::resolve` returns an absolute path that is not canonical.
Comparing the two lexically would miss the exclusion whenever the spellings differ. Both sides are
therefore canonicalized, and the fallback to `std::path::absolute` is the one `topology.rs` uses for
a path that cannot be resolved. Canonicalization runs only for directories named `evidence`, so the
cost is one system call per such directory rather than one per directory walked.

### Discovery resolves the project itself

`documents(root)` calls `ProjectRoot::resolve(root)` once, and excludes nothing when that fails.
The signature does not change.

**Considered: passing `Option<&ProjectRoot>` explicitly**, as `topology::check` takes one. That is
the narrower grant, and it is what this project's instructions prefer by default. It is rejected
here because the requirement is that no walking caller can omit the exclusion. With a parameter,
`None` is a valid argument, and a caller that passes it gets the old behavior with nothing to report
the difference. `topology::check` differs because its callers already hold a resolved project and
the partition is reported. Discovery's callers mostly do not hold one: `validate` and `format`
receive a bare path by design, because `subject` must keep working on loose documents and on
fixtures. Resolving inside discovery puts the invariant where nobody has to remember it, and
discovery already performs filesystem reads.

Resolution is from the walk's root, not from the current directory. A contributor who runs
`validate some/other/project/htmlspec` gets that project's evidence excluded, not their own.

### The root is exempt, and exemption is structural

The exclusion test is applied inside the walk to each child directory before descending, never to
the root. So `validate htmlspec/changes/x/evidence` walks everything beneath it, which is the
spec's "Naming the evidence directory checks what it holds" scenario. A path named explicitly
beneath evidence, such as `evidence/transcripts`, is walked in full for the same reason: its own
children are not directly within a change directory.

### Writing is refused where checking is not

Artifact-kinds states that no operation rewrites a file beneath a change's evidence. The first version of
this design exempted a named root from the exclusion without asking what `format` would then do with
it, and `format` rewrote a named evidence directory in full. A review of the pull request found this,
and it was reproduced against all seven of this repository's records. So `format`'s writing mode
asks `ProjectRoot::is_within_change_evidence` of every document before writing any. If one is a
record, it names each record, writes nothing, and exits as an operation that could not run.
`validate` and `format --check` are unaffected, because a check writes nothing.

**Considered: skip the records and write the rest.** Rejected. It only differs from refusing when a
named directory holds both records and documents, and a walk never produces that mix, because the
exclusion removes the records first. Where it could arise, a partial write reports success for a
request it half ignored.

**Considered: allow the write when named.** Rejected by the contributor. It would make a record
safe from every operation except the one most likely to be run on it by mistake.

## Risks / Trade-offs

- **[A contributor expects `validate` to check a record they meant as a document.]** → Evidence is
  defined by position alone. A document is one only when it sits outside `evidence/`, and a record
  can always be checked by naming it.
- **[Resolution inside discovery surprises a caller that walks a directory outside any project but
  inside a repository that holds one.]** → The exclusion is positional against that project's own
  `changes/`. A directory outside `htmlspec/` never matches, so a fixture walked from inside this
  repository after its migration is unaffected. A test covers this case.
- **[Silent exclusion hides a misplaced document.]** → A document at `changes/x/evidence/design.html`
  would go unchecked. That is the definition working: a file placed there is a record by position.
  The workflow instructions never place a document there, and `status` and `archive` find a
  change's documents by name rather than by walking.

## Migration Plan

None is needed. The change only narrows what a walk collects inside a project, and no project in
use holds HTML evidence except the migration this change unblocks. The document schema is
unchanged, so the contract version stays `0.2.0`. The release is an ordinary minor increment.
