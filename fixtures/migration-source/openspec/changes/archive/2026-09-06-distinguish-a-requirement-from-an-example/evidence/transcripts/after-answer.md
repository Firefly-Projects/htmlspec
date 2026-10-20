# Migration mapping: `/tmp/transcript52/source` (OpenSpec) → `/tmp/transcript52` (HTMLSpec)

Source root: `/tmp/transcript52/source`
Destination root: `/tmp/transcript52`, whose project directory is `/tmp/transcript52/htmlspec`
Contract read: `openspec/specs/` (nine accepted specifications)

`<name>` is a change's directory name, `<capability-path>` a capability's path under `specs/`
(possibly nested — the source holds both `billing` and `identity/user-auth`).

## The table

| Source artifact | Destination path | What the destination is |
| --- | --- | --- |
| `specs/<capability-path>/spec.md` (accepted specifications) | `htmlspec/specs/<capability-path>/spec.html` | The project's accepted specifications — the place `project-bootstrap`'s *A project has a declared root holding one asset store* requires `htmlspec/` to hold for them. The nesting is carried over unchanged, and `spec` as the leaf name is the source's; the contract fixes neither, so both are preserved rather than reshaped (see "Where the contract is silent"). `.html` because `document-schema`'s *Semantic units are identified by class* requires each semantic unit to be an HTML element carrying a reserved class, so the Markdown `### Requirement:` / `#### Scenario:` headings become `<section class="requirement">` and `<article class="scenario">` units with `req-` and `scenario-` identifiers, as *Every semantic unit has a stable unique identifier* requires. |
| `changes/<name>/proposal.md` | `htmlspec/changes/<name>/proposal.html` | The change's `proposal` document. `artifact-kinds`' *A change is a directory of kinded documents* closes the set of kinds to `proposal`, `design`, `tasks`, `specification` and requires each document to declare which it is; `document-template`'s *A document declares its identity and provenance* requires that declaration be machine-readable. Prose alone conforms — `artifact-kinds`' *A proposal argues value and the tool does not shape it* requires no unit in it. |
| `changes/<name>/design.md` | `htmlspec/changes/<name>/design.html` | The change's `design` document. Same kind vocabulary and same freedom of form: `artifact-kinds`' *A design reasons in whatever form the reasoning takes* requires no unit and reports none. |
| `changes/<name>/tasks.md` | `htmlspec/changes/<name>/tasks.html` | The change's `tasks` document. `document-validation`'s *A tasks document must carry at least one task* rejects one carrying no task unit, so each source checklist line becomes a `task` unit; `document-schema`'s *A task declares completion state* requires each to declare `todo`, `in-progress`, `done`, or `dropped` in a machine-readable attribute, and a `dropped` task to state why. |
| `changes/<name>/specs/<capability-path>/spec.md` (delta specifications, one per capability the change touches) | `htmlspec/changes/<name>/specification.html` — **all of them, collapsed into that one file** | The change's single specification artifact. The collapse is required, not chosen: `artifact-kinds`' *Archival produces the specification and then updates the accepted contract* states "A change directory SHALL hold at most one specification artifact", so five source deltas cannot become five destination documents. `document-schema`'s *Semantic units declare a change state* is where the source's `## ADDED Requirements` / `## MODIFIED Requirements` headings go — as `data-change="added"` / `"modified"` / `"removed"` on each requirement, since the destination has no section heading that carries it. |
| `changes/<name>/evidence/*.md` and any script beside them | `htmlspec/changes/<name>/evidence/` — same file names, content unchanged | Carried across, not converted. Nothing in the contract addresses evidence *files* on disk; the `evidence` unit that `document-schema`'s *Semantic units are identified by class* reserves is a unit inside a document, which is a different thing. So the source's shape is preserved. Verified in a scratch project: `htmlspec validate` walks the change directory and reports nothing about a non-HTML file sitting in `evidence/`. |
| `changes/archive/<date>-<name>/` (whole directory, with its documents, deltas, and evidence) | `htmlspec/changes/archive/<date>-<name>/`, each file mapped by its own row above | The archive — the third place `project-bootstrap`'s *A project has a declared root holding one asset store* requires `htmlspec/` to hold. The `<date>-<name>` directory name is contract-stated rather than preserved by accident: `toolchain-packaging`'s *An archived change is named for its date and refuses to collide* has archival name the directory for the date of archival followed by the change's name, and not prepend a second date to a name that already carries one. Keeping the source's date matters downstream — `artifact-kinds`' *A stated predecessor is resolved across the archive* matches a predecessor against an archived directory *with its leading date removed*. |
| `changes/archive/<date>-<name>/VERSION` | `htmlspec/changes/archive/<date>-<name>/VERSION` — same file, same content | The version that archived change shipped in. `toolchain-packaging`'s *An archived change records the version it shipped in* requires each archived change to record it in a form the project's own suite reads without version-control history, and requires recorded versions to be distinct. `project-bootstrap`'s *Archival requires the project version to advance* is what reads it: the last archived version comes from this record and explicitly **not** from the archive directory's name, so this file cannot be dropped as redundant with the directory it sits in. `VERSION` as the file name is the program's — observed in this toolchain (`htmlspec 0.9.0`), not stated by any requirement. |
| The highest version recorded across the migrated archive | `spec.nix`, the `version` field, set to a value ordered **after** it | The project declaration. `project-bootstrap`'s *A project declares its name, version, and expected toolchain* requires a declaration at the project root carrying name, version, and a pinned program reference, expressed in the language the build already uses. *Archival requires the project version to advance* is why the value matters: the first archival after migration is refused unless the declared version is ordered after the last archived one, and that requirement also forbids the program from advancing it — so migration must leave a value a contributor can archive from. The destination's `spec.nix` currently declares `name = "transcript52"; version = "0.0.0"`. |
| `changes/<name>/.openspec.yaml` (`schema`, `created`) | **No destination.** Dropped. | Dropped because both fields have consumers only in the source. The schema version moves into each document instead — `document-template`'s *A document declares its identity and provenance* requires every document to declare the schema version it conforms to, so the fact survives per document rather than per directory. The creation date has no reader: `artifact-kinds`' *A change states the change it follows* forbids the program inferring an ordering from a change's creation date, and *A document declares its identity and provenance* forbids a document declaring a lifecycle status at all. |
| `openspec/config.yaml` — the `schema` field | **No destination.** Dropped, for the reason in the row above. | |
| `openspec/config.yaml` — the `context` and `rules` fields | **No destination.** Dropped, and this one is forbidden rather than merely unused. | `project-bootstrap`'s *A project declares its name, version, and expected toolchain* states "The declaration SHALL NOT carry authoring rules or project context. Those have no consumer." So this content cannot be folded into `spec.nix`. In the source project being migrated here, both fields are commented-out examples and carry nothing — which `artifact-kinds`' *A proposal argues value and the tool does not shape it* is not about, but which the plan should record as *empty in the source*, distinct from *content dropped*. |
| Generated agent instructions — `AGENTS.md`, `CLAUDE.md`, `.claude/commands/openspec/*.md`, and any other tool directory the source carries | **Not migrated.** The destination's own are installed by `htmlspec init`. | `project-bootstrap`'s *Bootstrapping installs agent instructions for the tools a project uses* makes these the bootstrap operation's output, and *Bootstrapping is repeatable without loss* has a re-run refresh them while leaving specifications, changes, and archive untouched. Migrating the source's copies would install instructions naming OpenSpec commands, which the same capability's requirement forbids: an installed workflow's instructions must not name a command the program does not provide. The destination already holds `/tmp/transcript52/.claude/commands/htmlspec/`. |

## Two things the table does not show, which the migration still has to get right

**Every migrated document references the one asset store.** `project-bootstrap`'s *A project has a
declared root holding one asset store* states that a project holds exactly one, in its root, and that
every document references that store rather than one of its own. The destination's store is already
at `/tmp/transcript52/htmlspec/assets/` with eight assets in it, so nothing new is established.
The relative depth differs per row, because `document-validation`'s *An asset reference must match
the bytes it names* rejects a reference that does not resolve to stored bytes whose hash equals the
name — and, separately, reports an ascending reference with no store above it. Measured in a scratch
project: `htmlspec/specs/billing.html` gets `../assets/sha256-…/htmlspec.css`, and
`htmlspec/specs/identity/user-auth/spec.html` gets `../../../assets/sha256-…/htmlspec.css`. The
nesting a capability path carries is therefore also a fact about that document's asset references.

**Identifiers are the load-bearing part of the conversion.** The accepted-specification rows and the
delta-collapse row both turn Markdown headings into units carrying `id` attributes.
`document-validation`'s *Cross-references must resolve* fails a document whose reference resolves to
nothing, and `toolchain-packaging`'s *The command surface reports whether a change reached the
accepted contract* compares a change's requirement identifiers against the accepted specifications'
by identifier alone, reading neither for meaning. So an identifier derived one way in a change's
`specification.html` and another way in `htmlspec/specs/<capability-path>/spec.html` produces a
change that reports as unsynced forever, with nothing wrong in either document.

## Where the contract is silent, and what I did about it

Recording these because `contract-authority`'s *An assertion about the contract names the requirement
it rests on* requires naming a requirement or naming the silence, and its *Silence is recorded and not
filled* requires one of three routes after the recording. This is a decision taken at scale — one
choice applied to every artifact at once — where that requirement prefers the preserving route.

1. **`htmlspec/specs/`, `htmlspec/changes/`, and `htmlspec/changes/archive/` as the three directory
   names.** The contract is silent on these spellings. *A project has a declared root holding one
   asset store* requires `htmlspec/` to hold accepted specifications, active changes, and the archive
   "each in its own place" — it names the three places without naming them. **Route taken: preserve
   what exists.** `htmlspec init` already created these three directories in the destination, so
   there is a shape and I kept it. Not cited from the README, which shows the same names:
   `contract-authority`'s *The project's obligations are what its accepted specifications state* makes
   a README non-normative, and its scenario "A layout stated only in a README" says so directly.

2. **`proposal.html`, `design.html`, `tasks.html`, `specification.html` as the four document file
   names.** The contract requires each document to *declare* its kind — *A change is a directory of
   kinded documents* — and never fixes a file name. **Route taken: preserve what exists**, where what
   exists is the program's behavior rather than a requirement: `htmlspec new change` writes exactly
   those first three names, and `htmlspec archive` refuses with "has no specification.html" when the
   file is named anything else. I verified that by renaming it to `spec.html` in a scratch project and
   watching archival refuse. That refusal is a fact about this toolchain, not an obligation the
   project states.

3. **`spec.html` as the leaf name for an accepted specification, and keeping `identity/user-auth`
   nested rather than flattening it to `identity-user-auth.html`.** The contract is silent. **Route
   taken: preserve what exists** — the source's `<capability-path>/spec.md`, minus the extension. I
   confirmed the destination toolchain finds it: `htmlspec list --specs --json` in a scratch project
   enumerated `identity/user-auth/spec.html`, `billing.html`, and `identity-user-auth.html` alike, so
   nesting costs nothing in discoverability. The README's `htmlspec/specs/theme-selection.html` shows
   the flat form and I am not treating it as a layout, for the reason in point 1. Flattening would
   also be the expensive kind of mistake here: it composes a name out of a hierarchy, and there is no
   requirement to appeal to when someone later asks why.

4. **`.html` as the extension.** The contract requires the *content* to be HTML — *Semantic units are
   identified by class* — and states nothing about the file's name. **Route taken: I chose it, and the
   choice is mine rather than the project's.** Measured: `htmlspec list --specs --json` did not
   enumerate a `.md` file I dropped into `htmlspec/specs/`, and `htmlspec validate` did not validate
   it. So `.md` produces specifications the destination's own tooling cannot see. Renaming a file is
   the cheapest correction available if a contributor disagrees.

5. **Whether an *active* source change may hold a `specification.html` at all.** *Archival produces
   the specification and then updates the accepted contract* has archival "create the change's
   specification artifact, **or update it if the change already has one**", so an artifact existing
   before archival is contemplated rather than forbidden. The delta-collapse row relies on that
   reading. **Route taken: preserve what exists** — the source's active changes carry delta content,
   and it has to land somewhere.

**One thing I am explicitly not citing.** `openspec/changes/migrate-an-openspec-project/`
is an active, unarchived change whose `specs/project-migration/spec.md` addresses this exact
migration — including a requirement that a nested capability path survive rather than be flattened,
and a requirement that delta specifications collapse into one specification. It agrees with rows 3
and 5 above. It is **not** part of the contract:
`contract-authority`'s *The project's obligations are what its accepted specifications state* has a
scenario, "A change's specification before archival", making a citation of it as a current obligation
incorrect. **This is the change that would settle points 1 through 4 above**, and archiving it is what
would turn them from my choices into the project's rules.
