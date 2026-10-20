# The sync and the archival of `establish-a-runbook`

What the merge into `openspec/specs/` touched and what it did not, block by block, and what
archival did with the four deferred findings. Written at archival, on branch
`sync-and-archive-the-runbook-change`, against implementation commit `224acfa` released as
`0.13.0`.

## 1. The sync is a partial merge, and here is the proof

The change carries two delta specs. **The brief predicted one modified requirement and
misnamed it**, and that is worth stating first because it is the exact error the proof shape
exists to catch — the delta, not a prediction about it, is what was merged.

The brief said the sentence *"`htmlspec/` SHALL hold the project's accepted specifications, its
active changes, and its archive, each in its own place"* lives under the requirement **"A
project declares its name, version, and expected toolchain"**. It does not. It is the opening
sentence of **"A project has a declared root holding one asset store"**, which is what the
delta restates. And the delta restates **two** requirements, not one: the second is
**"Bootstrapping is repeatable without loss"**, whose protection list has the same three-place
shape and the same omission.

Had the merge been driven by the brief rather than by the delta, it would have edited a
requirement the delta never mentions and left one the delta does.

### `project-bootstrap` — before and after

| | count |
|---|---|
| requirement blocks at `HEAD` | 10 |
| requirement blocks after merge | 10 |
| byte-identical after merge | **8** |
| changed | **2** |
| requirements added or removed | **0** |
| `## Purpose` section | byte-identical, including its `TBD` placeholder |

The two that changed, and nothing else:

1. **A project has a declared root holding one asset store** — the `htmlspec/` sentence gains
   `and **its runbooks**`; two paragraphs are added (runbooks are a fourth place; the sentence
   is not declared closed); one paragraph is added under the one-store sentence; the
   bootstrapping scenario's `THEN` gains `and runbooks`; one scenario is added (*A runbook
   shares the same store*). Scenarios 3 → 4.
2. **Bootstrapping is repeatable without loss** — the protection list gains `**and
   runbooks**`; one paragraph is added; the re-run scenario's `THEN` gains `and runbooks`; one
   scenario is added (*A runbook survives a repeated bootstrap*). Scenarios 4 → 5.

**Every edit the delta makes to these two blocks is additive.** No sentence at `HEAD` was
deleted or narrowed — which is the difference from the precedent at `056df05`, where an
existing sentence about identifiers was bounded to "Within a unit". Here nothing needed
bounding: a three-item list becoming a four-item list does not weaken any claim about the
first three.

**Nothing the delta does not restate was touched.** Verified mechanically: the merge replaced
exactly the two named blocks by exact string match, each matching once, and a re-parse
afterwards found the same ten requirement names in the same order with eight blocks
byte-for-byte unchanged.

### `runbook-authority` — the eleventh capability

Created at `openspec/specs/runbook-authority/spec.md`. Delta `## Purpose` copied verbatim (no
`TBD` placeholder), `## ADDED Requirements` retitled to `## Requirements`, all six requirement
blocks byte-identical to the delta:

| | count |
|---|---|
| requirements | **6** |
| scenarios | **21** |

The six: a project keeps its procedures in a fourth place (4 scenarios); a runbook is two
documents of existing kinds (4); the canonical procedure carries no completion state (4); a
runbook carries no version and no changelog of its own (2); the project enumerates the
runbooks it holds (4); a runbook records what its history taught (3).

`openspec/specs/` went from ten capabilities to eleven.

### The other nine capabilities

Untouched, confirmed by `git status`, which shows exactly one modified file under
`openspec/specs/` (`project-bootstrap/spec.md`) and one new directory
(`runbook-authority/`). `artifact-kinds`, `contract-authority`,
`deterministic-formatting`, `document-schema`, `document-template`,
`document-validation`, `project-migration`, `toolchain-packaging`, and
`workflow-reporting` are byte-identical to `HEAD`.

### Ordering

The sync ran **before** archival, as `archive.md` asks and as `056df05` records. So archival
had nothing to report about requirements no accepted specification held — running it the other
way means answering a question you caused. `openspec archive` was therefore invoked with
`--skip-specs`, because the programmatic merge is a wholesale replacement and the intelligent
merge had already landed.

## 2. The archival

Archived as **`2026-09-18-establish-a-runbook`**.

**The date is 2026-09-18, not the 17th the implementation commit carries.** `archived_name`
composes `today()` in UTC with the change's name, and the archive's own history shows the same
lag repeatedly: `2026-09-06-distinguish-a-requirement-from-an-example` was added on the 7th,
`2026-09-11-migrate-an-openspec-project` on the 13th, `2026-09-15-catch-a-templates-placeholder-prose`
on the 16th. The directory name records when the change was archived; the `VERSION` file
records which release it shipped in. Only the second is a value any gate compares —
`project-bootstrap` requires that the last archived version be read from the recorded version
and never derived from a directory name.

All four documents and all eight evidence files moved intact. `VERSION` records `0.13.0`,
written into the change directory where `last_archived_version` reads it through
`recorded_version`.

Task 9.4 was the only one whose second half was open, deliberately, for the reason the previous
two changes each recorded and which this change's own release runbook now states as a
requirement (`req-the-recorded-version-waits-for-archival`): the `VERSION` file goes in a
directory archival creates. It is now closed, with what closing it measured.

**The version was not advanced.** `0.13.0` shipped with the implementation at `224acfa`.
Archival records a version; it does not advance one.

## 3. Verification

### `scripts/check` — all 12 stages green

```
format lint test doctest self-conformance self-format
self-runbooks self-runbooks-format self-assets prompt installed release
all 12 checks passed
```

`self-runbooks` and `self-runbooks-format` — the two stages this change added — report
`runbooks/release/specification.html` and `runbooks/release/tasks.html` each accepted with
**zero diagnostics** and each canonical. `release` is the stage that proves the archival is
well-formed, and it reports:

```
ok  19 archived change(s) record a version, and 13 version(s) are in the changelog
ok  declared version 0.13.0 has a changelog entry
```

**19, up from 18.** The count moving is the check confirming the file landed, rather than
merely that the eighteen that were already there still parse.

No red stage. The only cosmetic note is that `self-assets` reports `. is not a project, so its
5 stores were each checked on their own` — which is finding 4.1 below, visible in the check
output.

### `openspec validate --all`

Before the sync: 11 passed, 2 failed. After the sync: 12 passed, 2 failed — the new item is
`spec/runbook-authority`, passing. After archival: 11 passed, 2 failed, because
`change/establish-a-runbook` is no longer an active change and drops out of the enumeration.

The two failures are `change/log-a-runbook-run` and
`change/carry-a-runbook-to-another-machine`, both pre-existing, both failing for having no
delta specs. They are proposal-only successors and **were not touched**. `openspec list` now
shows exactly those two as the active changes.

**What the validator says about `establish-a-runbook` after archival is worth recording,
because it is indistinguishable from nonsense.** `openspec validate establish-a-runbook --type
change` reports `Change must have at least one delta. No deltas found` — and the identical
report comes back for `a-change-that-never-existed`. The validator's vocabulary has no word for
"archived", so an archived change and a name nobody ever used produce the same output. The
archived change's deltas are on disk at
`openspec/changes/archive/2026-09-18-establish-a-runbook/specs/`; the validator does not look
there. Do not read that report as a finding about the archived change.

### `htmlspec verify` and `htmlspec sync`

**Neither applies here, and the reason is finding 4.1.** `htmlspec sync --change
establish-a-runbook`, run from the freshly built binary, reports:

```
no htmlspec/ directory in . or any directory above it,
so there is no project here
```

There is no `htmlspec verify` subcommand at all — the twelve are `validate`, `format`,
`assets`, `archive`, `init`, `list`, `status`, `sync`, `store`, `instructions`, `new`,
`migrate`. This repository governs itself with `openspec/` markdown and deliberately is not an
html-spec project, so the html-spec sync and status operations have nothing to act on. The
`openspec` CLI is the right tool for this repository and was the one used.

### The staleness finding, measured again at archival

```
$ htmlspec --version                          # Nix profile binary, on PATH
htmlspec 0.12.0 (unknown)
$ cargo run -q --bin htmlspec -- --version    # freshly built
htmlspec 0.13.0 (224acfa5a017-dirty)
```

The PATH binary is a release behind. Every version-sensitive command in this archival used the
freshly built binary. This is deferred finding 4.2 below, reproduced rather than recalled.

## 4. The four deferred findings, and what the accepted contract says about them

The brief asks whether the accepted `runbook-authority` should state any of these. **The answer
is no for all four, and none was added.** Adding contract text a delta does not contain is the
failure `contract-authority` names: the project's obligations are what its accepted
specifications state, so a sentence introduced at archival is an obligation nobody proposed,
nobody reviewed, and no check was written against. These stay in evidence, which is the right
home for a finding that binds nothing.

The specific reasons differ, and they are worth separating because three of the four are not
merely "out of scope" — the contract as written already reaches them or already forbids
reaching them.

**4.1 The location is not dogfooded.** This repository's release runbook is at
`runbooks/release/`, not `htmlspec/runbooks/release/`. Creating `htmlspec/` here makes the
repository resolve as an html-spec project, which collapses five separately-checked fixture
stores into one and turns `self-assets` red with duplicate-asset errors. Measured with one
`mkdir` and no files, in `evidence/the-location-is-blocked-2026-09-17.md`.

Nothing to add: `project-bootstrap` states where a *project* keeps its runbooks, and this
repository is not one. A requirement excusing a non-project from a project's layout would be a
rule about a case the rule's subject excludes. The two new check stages point at
`runbooks/`, so the deviation is checked even though it is undocumented in the contract.

**4.2 `htmlspec init` staleness at `runbooks/release/tasks.html:57`.** The regenerate step
tells the reader to run `htmlspec init --tools claude,codex,pi` without saying that the
`htmlspec` on PATH may be the previous release. Reproduced above: 0.12.0 on PATH against
0.13.0 built.

Nothing to add, and this one is emphatic: a runbook's specification states what *that runbook*
requires of a run. `runbook-authority` says so in as many words — "A runbook SHALL NOT be
treated as stating an obligation of the project." A note about which binary to invoke belongs
in the runbook's own document, which a later change can edit without touching the contract at
all. Putting it in an accepted specification would state a fact about one machine's Nix profile
as an obligation of the project.

**4.3 `README.md:215` and the layout diagram at line 78 omit `runbooks/`.** Line 215 says
`init` "creates `htmlspec/` with `specs/`, `changes/`, and `changes/archive/`" and line 217
says it "leaves your specifications, changes, and archive alone" — both now three-item lists
where the contract states four. The diagram at line 78 shows `assets/`, `specs/`, and
`changes/` with no `runbooks/`.

Nothing to add, and here the contract is already correct and already binding. Both merged
requirements name runbooks in their lists; the README is a description of the program that has
fallen behind the program. It is a documentation fix, not a contract gap, and
`contract-authority` exists precisely because a README was once cited as a contract.

**4.4 The `no active runbooks` wording.** `label()` and `is_empty()` each gained one arm so the
empty case prints through the existing path, and the word "active" is carried over from
changes, where it distinguishes active from archived. A runbook is never archived, so there is
no inactive runbook for the word to exclude.

Nothing to add. `runbook-authority` requires that the enumeration succeed and report that
there are none; it does not prescribe the string, and a contract that specified output wording
would be the kind of rule that makes every copy-edit a spec change.

**4.5 A project-level rule reporting a `data-change` violation by document location.** A
runbook's units must declare `unchanged`
(`evidence/the-canonical-documents-change-state-2026-09-17.md`), and validation cannot report a
unit that declares otherwise, because no rule reads a document's location.

**This one is not merely unadded — the contract forbids adding it.** `runbook-authority`
states that "no rule SHALL exist whose subject is that a document belongs to a runbook". A
rule keyed on document location is exactly that rule. So the deferral is not an oversight
awaiting a later change; it is the contract's stated position, and a later change wanting the
check owes an argument for amending that sentence rather than an implementation. Task 14.1
already records this in the stage's own comment, and the stage comment is the right place
because it stops a reader over-reading what the two new stages close.

## 5. What was deliberately not done

- `log-a-runbook-run` and `carry-a-runbook-to-another-machine` were not started, touched, or
  authored. Their validation failures were not fixed. A separate agent takes them.
- The four findings above were not fixed.
- The version was not bumped.
- Nothing was merged. `main` is protected by a ruleset requiring a PR with `check` and `guard`
  green, and the merge is the user's call.
