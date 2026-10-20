# Migration mapping: OpenSpec source → HTMLSpec destination

Source root: `/tmp/transcript52/source` (paths below are relative to it).
Destination root: `/tmp/transcript52` (paths below are relative to it).

| Source artifact | Destination | Why this destination |
| --- | --- | --- |
| Accepted specification — `specs/<capability-path>/spec.md` | `htmlspec/specs/<capability-path>/spec.html` | `htmlspec/` is where the project lives and one of its places holds the accepted specifications (`project-bootstrap` → *A project has a declared root holding one asset store*); the destination already holds that place as `htmlspec/specs/`, so it is preserved rather than renamed. The contract fixes no filename and no arrangement *within* that place, so the source's shape is kept: one directory per capability, nesting intact (`identity/user-auth` stays two levels deep, not flattened to a composed name). The document declares kind `specification` (`artifact-kinds` → *A change is a directory of kinded documents*; `document-template` → *A document declares its identity and provenance*). The flat `htmlspec/specs/theme-selection.html` in the README is **not** a layout the project states (`contract-authority` → *The project's obligations are what its accepted specifications state*). |
| Change proposal — `changes/<name>/proposal.md` | `htmlspec/changes/<name>/proposal.html` | A change is a directory of kinded documents and the kinds are closed, so a proposal stays a proposal and does not merge into anything (`artifact-kinds` → *A change is a directory of kinded documents*). The change directory keeps its source name; the document declares kind `proposal`. |
| Change design — `changes/<name>/design.md` | `htmlspec/changes/<name>/design.html` | Same reason: the four kinds are separate documents, and a design is informed by its proposal rather than absorbed into it (`artifact-kinds` → *Every synthesis and every contract update is agent-driven*). Declares kind `design`. |
| Change tasks — `changes/<name>/tasks.md` | `htmlspec/changes/<name>/tasks.html` | Declares kind `tasks`; must carry at least one task unit, each individually addressable with a machine-readable completion state (`artifact-kinds` → *A tasks document carries the workable items*; `document-schema` → *A task declares completion state*). |
| Change delta specifications — `changes/<name>/specs/<capability>/spec.md`, one per capability touched | `htmlspec/changes/<name>/specification.html` — **all of them collapsed into this one file** | A change directory holds at most one specification artifact, and it is the same file as the change's as-built record (`artifact-kinds` → *Archival produces the specification and then updates the accepted contract*). So several capabilities' deltas cannot each become a file; they become one document, with each requirement's `added` / `modified` / `removed` state carried on the requirement itself rather than in a section heading (`document-schema` → *Semantic units declare a change state*). How that document arranges several capabilities' requirements is the agent's judgment, not a fixed rule. |
| Change evidence — `changes/<name>/evidence/<file>` (markdown, scripts, transcripts) | `htmlspec/changes/<name>/evidence/<file>`, byte-for-byte | The accepted contract says nothing about evidence *files* — `evidence` is a semantic unit inside a document (`document-schema` → *Semantic units are identified by class*), not a directory. Under that silence the existing shape is preserved and nothing is converted (`contract-authority` → *Silence is recorded and not filled*, preserving route). The migrated documents reference these files so a reader reaches them from the claim that cites them. |
| Change metadata — `changes/<name>/.openspec.yaml` (`schema:`, `created:`) | no destination file | `schema: spec-driven` names the source tool's workflow and has no reader in the destination; `created:` has none either. The contract fixes no place for it, and the destination's own declaration is forbidden from carrying this kind of material (`project-bootstrap` → *A project declares its name, version, and expected toolchain*: the declaration SHALL NOT carry authoring rules or project context, and a field nothing reads is excluded). Dropped, with the reason recorded per change rather than silently. For an archived change the date is not lost — it is already in the archive directory's name. |
| Archived change — `changes/archive/<YYYY-MM-DD-name>/…` | `htmlspec/changes/archive/<YYYY-MM-DD-name>/…`, with each document mapped by the rows above | The archive is one of the three places under `htmlspec/` (`project-bootstrap` → *A project has a declared root holding one asset store*); the destination already holds it as `htmlspec/changes/archive/`. The leading-date directory name is kept because that is the form the destination's archival writes and the form its predecessor resolution matches against with the date removed (`artifact-kinds` → *A stated predecessor is resolved across the archive*). All four documents survive archival, so all four are migrated (`artifact-kinds` → *Archival produces the specification and then updates the accepted contract*). |
| Archived change's recorded version — `changes/archive/<YYYY-MM-DD-name>/VERSION` | `htmlspec/changes/archive/<YYYY-MM-DD-name>/VERSION`, unchanged | Each archived change must record the version it shipped in, in a form the project's suite reads without version-control history (`toolchain-packaging` → *An archived change records the version it shipped in*), and the archival gate reads the last archived version from that record and explicitly **not** from the archive directory's name (`project-bootstrap` → *Archival requires the project version to advance*). Carried across as-is; no version is invented where the source recorded none. The destination's `spec.nix` `version` must then be at least the greatest migrated archived version, or the next archival is refused by that same requirement. |
| Project configuration — `config.yaml` (`schema:`, optional `context:`, `rules:`, `operations:`) | no destination file; the destination's identity stays in `spec.nix` at the root | The destination's declaration records exactly name, version, and the pinned toolchain, and is forbidden from carrying authoring rules or project context (`project-bootstrap` → *A project declares its name, version, and expected toolchain*). `htmlspec init` has already written `spec.nix`, and re-running it does not overwrite an existing declaration (`project-bootstrap` → *Bootstrapping is repeatable without loss*). Any `context:` or `rules:` the source carried is dropped with that requirement named as the reason. |
| Generated agent instructions — `AGENTS.md` region, `.claude/commands/openspec/*`, other tool files | not migrated; the destination's own `htmlspec init` writes `.claude/commands/htmlspec/*` and the marked region in `AGENTS.md` | These are generated from the destination program's own compiled workflow set, and a converted copy would name commands the destination does not provide — which its own checks reject (`project-bootstrap` → *Bootstrapping installs agent instructions for the tools a project uses*: an installed workflow's instructions SHALL NOT name a command the program does not provide). Instructions installed into a file a contributor also writes stay inside their marked region (`project-bootstrap` → *Instructions installed in a shared file are confined to a marked region*). |
| — (no source artifact) | `htmlspec/assets/sha256-*/…`, already present in the destination | The project holds exactly one asset store, in its root, and every migrated document references *that* store by a path relative to its own location rather than getting a copy (`project-bootstrap` → *A project has a declared root holding one asset store*; `document-template` → *Documents share one pinned runtime*). A store already present is verified against the pin rather than replaced (`project-bootstrap` → *Bootstrapping is repeatable without loss*). This is why the relative depth of a migrated document matters: `htmlspec/specs/identity/user-auth/spec.html` reaches the store by `../../../assets/…`, and `htmlspec/changes/<name>/proposal.html` by `../../assets/…`. |

## What this source actually holds

| Source file | Destination file |
| --- | --- |
| `source/specs/billing/spec.md` | `/tmp/transcript52/htmlspec/specs/billing/spec.html` |
| `source/specs/identity/user-auth/spec.md` | `/tmp/transcript52/htmlspec/specs/identity/user-auth/spec.html` |
| `source/changes/add-a-thing/proposal.md` | `/tmp/transcript52/htmlspec/changes/add-a-thing/proposal.html` |

The source carries no design, no tasks, no delta specifications, no evidence, no archived change, no
`VERSION`, no `.openspec.yaml`, and no `config.yaml`. Those rows have nothing to map here — an
absent source artifact is distinct from one that was dropped, and the plan should record it that
way. With no migrated archive, `spec.nix`'s `version = "0.0.0"` stands as bootstrapped.

`change add-a-thing` has a proposal and nothing else, which is a valid change in the destination: a
change need not have every kind present (`artifact-kinds` → *A change is a directory of kinded
documents*).

## Two things the table's destinations rest on, stated plainly

**What the contract fixes.** Only `htmlspec/` itself, and that it holds the accepted
specifications, the active changes, the archive, and exactly one asset store, "each in its own
place" (`project-bootstrap` → *A project has a declared root holding one asset store*). Also: one
specification artifact per change; the closed set of four kinds; per-requirement change state; the
archived version's readability by the gate.

**What the contract is silent on, and what was done about it.** The directory names `specs/`,
`changes/`, `changes/archive/`, `assets/`, the document filenames `proposal.html`, `design.html`,
`tasks.html`, `specification.html`, the `.html` extension, and the `spec.html` filename under a
capability are *not* stated by any accepted requirement. They are what the destination project
already holds on disk and what `htmlspec init`/`htmlspec new` write. Every one of them is therefore
taken by the preserving route rather than chosen here (`contract-authority` → *Silence is recorded
and not filled*), which is the right route because a migration applies one decision to every
artifact at once. The README's layout sketch was not used as authority for any of them
(`contract-authority` → *The project's obligations are what its accepted specifications state*).

Two further notes, neither of them a source artifact:

- The migration's own plan is a document of the `tasks` kind and lands in the destination as the
  migration's own change — `htmlspec/changes/<migration-change-name>/tasks.html` — so the migration
  is the one act in the project's history that has an artifact behind it.
- A change may record the change it follows as a whole-directory fact beside its documents
  (`artifact-kinds` → *A change states the change it follows*). OpenSpec has no source artifact that
  carries an ordering, and nothing may be inferred from a name or a date, so no migrated change
  states a predecessor; an absent declaration is complete as it stands and is not reported.
