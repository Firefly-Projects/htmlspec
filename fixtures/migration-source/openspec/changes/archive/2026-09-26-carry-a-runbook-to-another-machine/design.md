# Design

## Context

See `proposal.md` for motivation and the four deltas under `specs/` for observable behavior.
The predecessor `log-a-runbook-run` is implemented but still active. Its logging requirements
must be retained when these deltas are eventually synced; this change does not replace them.

`runs::start` currently resolves a `ProjectRoot`, copies the procedure and every specification
requirement, validates the composed record, and then writes it. `runs::check` and
`htmlspec_validate::rules::runs::conformance` report missing outcomes without gating. There is
no structured machine-prerequisite field: `Requirement` exposes statements, scenarios, and notes.
Examples in HTML are inert. The proposal forbids changes to the schema and artifact vocabulary.

`ProjectRoot` requires an `htmlspec/` directory, not an archive or `spec.nix`, to locate runbooks.
The repository itself still uses OpenSpec planning and root-level `runbooks/`; production layout
and this repository's release script therefore need distinct filesystem adapters. Existing
`Version` parses three numeric components; archive versions are in `VERSION`. `scripts/check-release`
reads files without Git because the Nix check source has no repository history.

## Goals / Non-Goals

**Goals:** Put parsing, observation, readiness, and record publication behind separate typed
boundaries. Preserve existing document validation and advisory run review. Make the authoring
notation, package-manager boundary, receiving layout, and release comparison precise enough to
implement without another product decision.

**Non-Goals:** A general command runner, dependency solver, installer, transport command, history
sync service, cryptographic authentication of releases, or enforcement while an operator manually
edits an existing run. No machine-dependent rule enters the per-document validator.

## Decisions

### 1. Use one versioned check declaration in an existing requirement note

The selected default is built-in probes. Free-form commands would grant a carried document the
power to install software or run procedure steps during what the operator requested as inspection.
Parsing English would make synonymous prose change behavior unpredictably.

A machine requirement contains one ordinary paragraph with `data-field="note"`. Its projected
text starts with the exact marker `preflight/v1 ` followed by a JSON object. For example:

```html
<p data-field="statement">The run MUST start outside an existing HTMLSpec project.</p>
<p data-field="note">preflight/v1 {"kind":"state","check":"outside-project"}</p>
```

This is an explicit, narrow use of an existing string as a wire format, required by the proposal's
no-schema-change boundary. It is not an unstructured core representation. Deserialize immediately
into a closed tagged enum, reject unknown fields, and retain requirement identity in every error.
Do not give downstream code raw JSON maps or ask it to reparse notes. Ordinary notes are prose;
any note beginning `preflight/` attempts a declaration and an unknown version is an error.
Only one declaration is allowed per requirement. All declared checks are conjunctive.

The public v1 forms are:

| Kind | Required fields beyond `kind` | Meaning |
| --- | --- | --- |
| `tool` | `program`, `version` | A bare executable name on the invocation's PATH, with an exact numeric `major.minor.patch` version |
| `toolchain` | `manifest`, `shell`, `tools` | A local `flake.nix`, its named development shell, and a nonempty list of required executable names supplied by that environment |
| `skill` | `scope`, `path` | A readable, nonempty `SKILL.md` in `home` or `workdir` scope, at the stated relative path |
| `state` | `check` | Either `outside-project` or `clean-git-worktree` |

Examples: `{"kind":"tool","program":"python3","version":"3.13.2"}`;
`{"kind":"toolchain","manifest":"flake.nix","shell":"default","tools":["cargo","rustc"]}`;
`{"kind":"skill","scope":"home","path":".agents/skills/example/SKILL.md"}`.
Examples illustrate the format and do not prescribe a Python pin for the release runbook.

Relative paths reject absolute roots and `..`; program names reject separators and option-like
spellings. Skill paths are resolved within their selected scope, including symlink containment;
a global skill is not satisfied by a same-named file bundled beside the runbook. Manifest paths
are relative to the invocation's working directory. No implicit search among agent installations
or fallback from one scope to another occurs. Unsupported states need a later adapter, not an
escape hatch for shell code. A normal requirement can still describe a manual obligation.

### 2. Distinguish facts from evaluation and readiness from publication

Introduce CLI-local `preflight` and `runbook_release` modules. Reuse the parser and validator as
boundaries; the schema model deliberately remains permissive so it can report malformed documents.
Use private constructors for domain values after those checks. The important planned interfaces
are these contracts, with support types defined before implementations:

```rust
fn parse_contract(model: &DocumentModel) -> Result<MachineContract, ContractErrors>;
fn observe(check: &MachineCheck, host: &mut impl HostProbe) -> Observation;
fn assess(contract: &MachineContract, facts: Observations) -> PreflightAssessment;
fn prepare_run(source: ValidatedRunbook, host: &mut impl HostProbe)
    -> Result<ReadyRun, StartFailure>;
fn publish_run(run: ReadyRun, destination: VacantRecord) -> Result<StartedRun, PublishError>;
fn compare_release(previous: PreviousRelease, current: RunbookRelease)
    -> Result<PublishableRelease, ReleaseFaults>;
```

`MachineCheck` has `Tool`, `Toolchain`, `Skill`, and `State` variants. `WorkingState` has the two
supported states. `RequirementId`, `ProgramName`, `ScopedRelativePath`, `RunbookVersion`,
`ProjectVersion`, and `ContentDigest` have distinct validated representations. Wrap the existing
`Version` for the two meanings rather than comparing raw strings or changing the shared parser.

`Observation` distinguishes a measured fact from `Unavailable`, `TimedOut`, and `ProbeFailure`.
`PreflightAssessment` is `Ready(PassedChecks)` or `NotReady(NonEmptyFindings)`; invalid contracts
are a separate error. Every observation is bound to a check identity. Missing or duplicate facts
cannot produce a ready result. `ReadyRun` owns the validated source snapshot, version, and passed
evidence; its constructor is private and it is consumed on publication. A standalone readiness
report exposes no constructor or persisted token that can create a `ReadyRun`.

`HostProbe` supplies only scoped filesystem inspection, executable resolution, and the fixed
probe operations. Pure parsing, comparison, and report assembly have no process, clock, or
filesystem capability. JSON reports serialize typed structures and include requirement IDs,
expected conditions, observed facts, outcomes, and remedies; they do not reuse arbitrary probe JSON.

Add `PrerequisiteFailure` to the CLI `Outcome` enum for a completed negative readiness assessment,
mapping it to exit 1. Do not stuff machine failures into `DocumentViolation`: these share the
unsatisfied-contract exit class but have different domain meanings. Preserve all existing outcome
codes; malformed declarations and load failures remain operational failures at exit 2.

### 3. Keep probes finite and the package manager authoritative

For direct tool checks, resolve the executable once and invoke that path with the fixed argument
`--version`, with no shell and closed stdin. Accept exactly one standalone three-component numeric
version token from bounded output only when the process exits successfully; reject ambiguous or
unparseable output and nonzero exits. Compare numerically.
Tools with another version interface need a package-manager adapter or a future explicit adapter;
the runbook cannot supply custom arguments, regular expressions, or shell fragments.

The Nix adapter inspects the selected local locked flake for the current system. It evaluates
the selected dev shell's `buildInputs` and `nativeBuildInputs` with network access, lock updates,
and import-from-derivation disabled, then queries already-realized input closures. It resolves
each requested tool on the actual PATH and compares its canonical executable location with those
inputs' closures. A tool appearing only through a shell hook is unavailable unless its provenance
can be established from the declared inputs. Neither a manifest's existence nor `IN_NIX_SHELL`
proves readiness. No versions are copied out of the flake into a second dependency catalog.

Pin the adapter invocation and parsed response shape in tests against the Nix version supported
by this project. The adapter must never call `nix develop`, `build`, `shell`, a fetch command, or
anything that realizes missing outputs. Missing locked sources, missing realized inputs, evaluation
requiring a build, or unsupported Nix behavior produce `Unavailable` with the prerequisite to fix.
Ordinary evaluator caches are not a readiness guarantee; target files, locks, and installations
remain unchanged. No generic package-manager plugin framework is needed for this one adapter.

Skill checks establish readable installed content, not its quality or behavioral correctness.
`outside-project` uses the existing nearest-project boundary from the invocation directory.
`clean-git-worktree` uses a fixed read-only Git status operation with optional locks disabled;
modified, staged, conflicted, and untracked files all make it false. An absent repository is
unavailable, not clean. Runbook storage remains independent of the work directory: `--store`
selects documents, not where a state probe runs.

Use sequential probes for deterministic ordering. Each has a 10-second deadline, a 64-KiB combined
output limit, and closed stdin; exceeding either limit is unavailable readiness. Cancellation
stops and reaps outstanding processes, including children started by the Nix adapter, before
returning. Reuse suitable process management if available; any narrow safe dependency needed for
process cleanup must preserve the workspace's `unsafe_code = "forbid"`. Tests inject the clock
and probe responses rather than depending on the developer's installed tools.

### 4. Gate the existing start operation and preserve the logging distinction

Validate and snapshot both canonical documents, the version, and its changelog before collecting
machine observations. Treat unreadable files distinctly from absent ones; do not retain the
current `read_to_string(...).ok()` fallback where it would bypass a machine contract. A missing
specification or version is now a migration refusal for new starts, while old record review stays
unchanged. Zero declared checks is legal and is reported as zero.

Use the existing DOM copying and reference-rewriting path. Match successful machine checks to
the copied requirements and replace their generated `not-run` verifications with `passed`
verifications recording the observation and method. Other verifications remain `not-run`. Add
the runbook version to readable record header prose and the start report, never to
`htmlspec-version`, which identifies the document schema.

Validate the complete record before publishing. Use exclusive, non-overwriting publication of a
fully written temporary file in the destination filesystem; a successful rename that overwrites
another record is not acceptable. Handle same-name concurrent starts as a collision. Own temporary
files with cleanup guards, remove a newly created empty `runs/` on recoverable write failure, and
never remove an existing directory or another process's record. Crash leftovers must be hidden
temporary files ignored by record enumeration, with an explicit safe retry/cleanup diagnostic.

The formal-methods gate applies because checking, snapshotting, and publishing are ordered effects.
There is no existing model-tool setup; use this transition specification and deterministic
state-transition tests rather than introducing a model checker:

| State | Trigger and next state | Required invariant |
| --- | --- | --- |
| Unresolved | Resolve storage/work roots and parse -> Validated or Refused | No destination created |
| Validated | Observe every declared check -> Ready or Refused | No run record; all facts bound to this snapshot |
| Ready | Compose and validate -> Prepared or Refused | Every machine check passed; no procedure step executed |
| Prepared | Exclusive publication -> Started or Refused | A record is complete and valid, or absent; prior records preserved |
| Any pre-publication state | Cancellation -> Cancelled | Owned probes and temporary files cleaned up; no published record |
| Started | Operator advances record | Canonical procedure remains unstarted; later conformance remains advisory |

State variables are source snapshot, working context, collected observations, temporary-file
ownership, and destination occupancy. Forbidden transitions include Refused -> Started, a
standalone report -> Started, and replacement of an occupied record. A retry begins at Unresolved;
there is no cached readiness. Concurrent changes to external tools after observation remain an
explicit observation-time limitation. Snapshot ownership prevents document edits during preflight
from changing which procedure receives the passed evidence. Process crash after publication may
leave a valid record without a success message; retry reports the existing record, never overwrites.

### 5. Carry a minimal receiving layout and keep the work target explicit

The supported manual copy shape preserves existing relative asset references:

```text
portable/
  htmlspec/
    assets/                         one existing shared asset store
    runbooks/release/
      specification.html
      tasks.html
      VERSION
      CHANGELOG.md
      runs/                         created on the first local start
```

There is no transport operation to implement. Register `portable` with the existing store command
when the work target is elsewhere, then invoke preflight and run using that store from the target
directory. A name remains a name; no additional path selector is added. Original project
`spec.nix`, archives, prior records, and accepted specs need not travel. Required manifests and
installed skills belong to the target environment, not this copy. Old run records are local
history; leaving them behind does not invalidate the carried contract or assert zero prior runs.

### 6. Publish runbook release snapshots through the existing archive transaction

Canonical release content is exactly the bytes of `specification.html` and `tasks.html` after
canonical formatting. Hash both with the existing SHA-256 facility. New records, record edits,
and changelog prose do not alter these fingerprints; updated asset references inside either
document do. This intentionally makes an authored canonical-document edit require a release even
if its effect is editorial. A version-only release is permitted with its changelog entry.

Each new archive carries `runbooks.json`, a versioned typed inventory with one entry per present
runbook: directory name, runbook version, two document digests, and the runbook changelog as a
snapshot. The enclosing archive's `VERSION` supplies the source project version. Changelog
snapshots are necessary: a deleted or renamed runbook must not make past releases unverifiable.
An empty inventory is explicit. The format rejects duplicate names, malformed digests, unknown
versions/fields, and duplicate changelog version headings.

Read history ordered by parsed project version and compare against the last snapshot for that
runbook name, including history before a temporary removal. Identical content at the same version
is a repeated snapshot of one release, not a duplicate release. Reject a downgrade even with
identical bytes. New names start above `0.0.0`; renaming intentionally starts a new lineage because
the existing contract forbids a second stable runbook identity.

Persist an adoption marker `runbook-releases.json` at the planning root containing the first
project version subject to inventories. Publish it atomically with the first inventory; it is
never advanced on later releases. Any archive at or above that version must have an inventory.
The marker prevents deleting the first inventory from turning missing history into a legacy
exemption. An inventory without its marker, or a marker naming an absent adoption archive, is an
error. The release checker independently verifies this policy from files without Git.

Extend the existing archive planning/commit path, including `--check`, rather than doing a second
post-archive write. Parse and compare all releases before mutation; stage the inventory with the
change; publish the adoption marker with rollback on failure. Preserve unrelated active-change
files on refusal and retry. Do not use `staging.rs`'s project-migration API as if it were an archive
transaction; inspect and extend the actual archive path in `commands.rs`.

For this publication transition, state consists of the immutable prior history, proposed inventory,
active-change location, adoption-marker ownership, and archive destination occupancy. The sequence
is Read -> Compared -> Staged -> Committed. Comparison failure or cancellation before commit returns
to the original filesystem state; a staging/commit failure rolls back only owned writes. The
archive destination must be vacant, and publication must not replace an existing inventory or
adoption marker. Retry rereads history. If process termination interrupts the multi-file commit,
retain a small transaction journal naming owned paths and intended digests; recovery completes or
rolls back that transaction before accepting another publication, and refuses conflicting edits.
This is local filesystem coordination, with no remote retries or exactly-once delivery claim.

`scripts/check-release` gains the fourth question about released runbooks. Its OpenSpec adapter
uses root `runbooks/`, `openspec/changes/archive/`, and `openspec/runbook-releases.json`; product
code uses the corresponding `ProjectRoot` paths. Share format fixtures across the Python checker
and Rust implementation to prevent divergent version ordering and legacy handling. Check current
version/heading presence, but compare document changes only at archival: unarchived work can
legitimately still be choosing its release number. This resolves the proposal's comparison gap
without making a carried run depend on an unavailable archive.

### 7. Decisions the implementation reached that this design did not state

Recorded here rather than left in the code, because each one is a place where an implementer had to
choose and a later reader would otherwise have to reverse-engineer the choice.

**An archive whose recorded project version will not parse is skipped, not refused.** History is
ordered by recorded versions, so such an archive cannot be placed in it. Refusing every later archival
over it was the alternative and is worse: the fault is in a file belonging to some *other* change, and
a gate blocking the project on that has no route back through this operation.
`last_archived_version` already draws this line for the project version gate, for the stated reason
that reporting a malformed `VERSION` belongs to `scripts/check-release` — which names the change
carrying it, and asks about every archive's version independently of this reader.

**A withdrawn requirement's machine-check declaration is not a check.** It follows the line `runs.rs`
already draws twice: a record does not copy a withdrawn requirement and `--check` does not demand an
outcome for one. A check declared by a withdrawn requirement would be copied into no verification, so
it could never be attested to — and refusing a run over an obligation that no longer applies is the
defect `is_withdrawn` exists to prevent. That helper is now crate-visible, so there is one statement of
it rather than three.

**A declaration on a requirement with no identifier is refused rather than checked anonymously.** The
design says to retain requirement identity in every error but not what happens where there is none. A
passed check is copied into the record as a verification naming its requirement, so a requirement
nothing can name is one a record cannot attest to.

**`ReadyRun` has a private constructor and is consumed by publication, with no conversion from a
report.** The design requires that a standalone readiness report cannot become permission; this makes
it unrepresentable rather than unchecked. `PreflightReport` holds no token and `prepare_run` is the
only path to a `ReadyRun`, so reuse has no spelling.

**Publication is a hard link rather than a rename.** The design requires exclusive, non-overwriting
publication of a fully written temporary file and forbids a rename that overwrites. `std::fs::rename`
replaces its destination, and the atomic exclusive create wanted here is `renameat2` with
`RENAME_EXCL` — unavailable without the `unsafe` FFI this workspace forbids. `link(2)` fails with
`EEXIST` where the name is taken, including where it is taken by a broken symlink, which is the case
`nothing_is_at` was written to refuse. The same primitive publishes the adoption marker.

**The journal holds paths and digests, and recovery is decided by which side of the move the
interruption fell on.** The move is the commit point, so "the target exists or the change does"
answers where the transaction got to without a log of steps. Recovery completes past the commit point
and rolls back before it, and refuses where a file it owns holds bytes it did not intend — completing
would publish content nobody compared, and rolling back would destroy somebody's work.

**A staged inventory or recorded version found in an active change refuses archival.** Those files are
written into the change directory so the rename carries them, which means an interrupted publication
leaves them somewhere this program never otherwise writes. Overwriting would publish a release nobody
compared; removing them silently would destroy the only evidence of what the interrupted run intended.

**Cancellation is a signal handler, and the process group is why it has to be.** A probe runs in its
own group so that stopping it can stop everything it started — and the same property means a terminal's
interrupt no longer reaches it. Without a handler, `Ctrl-C` during a Nix evaluation would end this
process and leave the evaluator running, which is the opposite of what the design requires.
`signal-hook` installs one, and it is already in the dependency tree through `inquire`, so this adds no
vendored subtree.

**Two dependencies, under the allowance the design already grants.** `command-group` for the group kill
`std` cannot express under `unsafe_code = "forbid"`, and `signal-hook` for the handler above. Both are
pinned exactly, confined to `preflight/host.rs`, and four transitive crates between them.

**The release runbook declares one machine check, and it is the toolchain one.** `python3` and `git`
are needed by the procedure and are not declared: a `tool` check states an exact version, and this
procedure needs any `python3` whose standard library reads a file. Pinning one would grow a second
dependency language beside `flake.nix` for a constraint the procedure does not have — the proposal's
counter-argument, applied. They stay obligations of the run, stated in the procedure's prose. Neither
`outside-project` nor `clean-git-worktree` is declared, for the reasons the migration plan gives.

**That check is stricter than the procedure strictly needs, and the runbook says so.** It requires
provenance from the pinned environment, so a contributor whose `cargo` comes from rustup reading the
same `rust-toolchain.toml` is refused. The alternative was a check that accepts any `cargo` and
therefore establishes nothing about the pin. The broken `~/.cargo/bin/cargo` symlink this procedure has
already met is the case that makes the distinction worth its cost.

**A tool inside an input's *closure* counts as supplied by it.** Measured in this repository: `jq`
resolves to the `-bin` output of a derivation whose declared input is the `-dev` output, so a prefix
comparison against the declared inputs alone would report a tool the pinned environment does supply as
a shadowed one. The closure is queried once, and only where the direct comparison left something
unexplained.

**The pinned Nix response is a recorded answer, parsed without a store.** The bytes in the test came
from a real `nix eval` against this repository's flake, but whether the store paths in them are
realized *here* is a different question — so parsing is pinned against the recording, and realization
is tested against directories a test creates. A test reaching into `/nix/store` would pass on a
contributor's machine and fail on a clean one.

**The devShell gains no packages.** The procedure needs `python3` and `git` and the flake's devShell
supplies neither, so `scripts/check` cannot run inside `nix develop`. That is a real gap, and closing it
changes what `nix develop` provides, which this change does not claim.

**Two run-tests moved off an absent specification.** `a_runbook_stating_nothing_has_runs_nothing_checks`
and `the_records_summary_does_not_describe_an_absent_section` deleted `specification.html` to reach a
runbook stating nothing, and an absent specification is now a migration refusal for a new start. Both
reach the same subject through a specification that states nothing instead, and the third —
`a_record_that_does_not_validate_is_not_written_and_does_not_wedge_the_run` — now meets its refusal at
load, naming the runbook's own document rather than a record that was never composed. Its subject,
nothing written and no wedge, is unchanged.

## Risks / Trade-offs

- Note declarations add an authoring protocol inside an existing field -> version and document it,
  reject unknown fields, preserve IDs, and test projection/formatting round trips. This is the
  explicit compromise required by keeping the document schema unchanged.
- Built-in checks cover fewer cases than shell commands -> report unsupported conditions honestly;
  retain manual requirements without falsely claiming preflight checked them.
- Nix inspection has platform and version differences -> isolate the adapter, refuse unavailable
  provenance, and test against the pinned environment with fake observations for ordinary tests.
- A tool's version process can itself misbehave -> bound execution and output and document that
  locally installed tools and Nix are trusted; inspection is not a process sandbox.
- Readiness can become stale immediately -> rerun at start, record exact observations, and make no
  guarantee that environmental state remains fixed during manually executed steps.
- New starts require metadata older runbooks lack -> provide migration guidance and keep old record
  review usable. Canonical documents and release metadata should travel together.
- Snapshots and an adoption marker add release bookkeeping -> one archive transaction writes them,
  and the ordinary release stage checks them without introducing a second publication command.
- Carried version metadata can be tampered with -> it identifies the claimed release and is not a
  signature. Signing and cross-machine history reconciliation remain outside this change.

## Migration Plan

1. Land the typed contracts and regression tests before implementing the probe and publication
   bodies. Keep the predecessor's logging behavior and current no-overwrite guarantees covered.
2. Implement parsing and adapters, the CLI report, and integrated run gating. Teach existing run
   fixtures to carry explicit version metadata; retain legacy fixtures for historical review.
3. Implement version comparison and archive inventories together with the release checker. During
   implementation this repository has no adoption marker yet; the first published inventory and
   marker are recorded together at release, not fabricated during planning.
4. Migrate `runbooks/release` with its first independent version and a candid adoption changelog.
   Declare only entry preconditions. The historical `init` hazard occurs in a scratch directory
   later in this procedure, so do not incorrectly require the whole release run to start outside
   a project. Demonstrate `outside-project` in a separate bootstrap fixture. Similarly, do not
   require a clean tree for a procedure whose intended starting state contains release edits.
5. Update runbook authoring and CLI documentation, plus affected archive instruction sources for
   the new release gate. Regenerate installed copies through the existing installer if those
   sources change. Include new metadata in the Nix source filter and migration classification.
6. Verify a copied minimal project with the source unavailable, then run the repository suite.
   Before publication the change can be reverted normally. After publication preserve inventories,
   marker, and run records; a rollback must retain their reader or be an explicit compatibility
   change, rather than delete recorded release history.
