# Changelog

What changed in each version this project has shipped, newest first.

One entry per version, and each heading names its version so `scripts/check-release` can
find it. That check also requires every archived change to record the version it shipped
in, in a `VERSION` file beside its documents. Prose inside an entry is not parsed — a
changelog exists to be read by a person.

## 1.0.0 — commit the contract and the command line

Version 1.0 is a promise, and two requirements now state it. Nothing a document may contain
changes, and no command, flag, exit code, or payload field is added, renamed, or removed. What
changes is that the suite fails when a later release breaks either promise.

**The document contract follows semantic versioning.** Contract `1.0.0` is the `0.7.0`
vocabulary, unchanged. Within a major version, no later minor or patch reports an error on a
document an earlier one accepted, except for a defect its changelog entry names. A minor can
add vocabulary and warnings. The schema keeps accepting `0.1.0` to `0.7.0` beside `1.0.0`, so
every document written so far validates with exactly the diagnostics it had. A new document, the
four templates, and a run's record declare `1.0.0`. A record declared `0.1.0` in its head and its
masthead until now. A conformance test redeclares every fixture at every accepted version and
finds no error under `1.0.0` that the earlier declaration did not report.

**The command surface is stable within 1.x.** Every operation keeps its name in both spellings,
the OpenSpec one and the request verb. Every flag keeps its name and meaning, every exit code its
value, and every `--json` field its name and type. A minor can add any of them, and only 2.0
removes or renames one. Human-readable output is not covered, so its wording can keep improving.

- `crates/htmlspec-cli/tests/surface/1.0.json` records the surface, read from the program's own
  parser rather than from `--help`: each operation with every spelling clap accepts, aliases
  included, its arguments, the exit codes, `--help`, `--version` and the `help` subcommand, and
  each payload's schema. The suite fails when the program lacks anything a released surface of its
  major holds, and when the crate declares a minor that has no file. The test that rewrites the
  current minor's file refuses to record a loss.
- Each operation that writes a payload publishes a JSON Schema in
  `crates/htmlspec-cli/schema/payload/`, thirteen in all, with every field it can emit. A
  published schema admits a field it does not declare, because a later 1.x release can add one and
  a consumer ignores what it does not know. The suite reads the same schemas strictly, and every
  integration test holds the payloads it produces to both readings. A new test produces every
  field the schemas declare at least once, including the ones ordinary fixtures never write: an
  unreadable document's `error`, a curtailed report, each variant of `show`, the promotion gate's
  findings, and the preflight report `run --as` writes when the machine falls short.

**What this release does not do.** It publishes nothing. The first public release is the first
run of the `publish` runbook, `htmlspec run publish --as v1.0.0`, after this change is archived.

## 0.34.0 — publish releases through their own runbook

A release now has a procedure for reaching the public: a second runbook, `publish`, beside
`release`. `htmlspec list --runbooks` lists both, and after a release's change is archived and
merged, `htmlspec run publish --as vX.Y.Z` starts a record holding only the publishing steps. The
program does not change.

**Nine steps, from `main` to four channels.** The run takes the archived release from `main`,
reviews the exact export locally with `publish/export --ref SOURCE`, and pushes that reviewed tree
to `Firefly-Projects/htmlspec` as a pull request. It merges it, tags the public merge commit through
GitHub's API, watches the release workflow, and installs the version from the flake, Homebrew,
crates.io, and the release page. No step pushes a ref from this repository, because a pushed tag
carries every private commit behind it. On the first run only, the run bootstraps the empty public
repository with its ruleset off for that one push and stops unless the ruleset is active again, and
it adds `firefly-developers` as each crate's co-owner. The runbook's specification states six
requirements of a run, among them the hazards the review of the 1.0 plan found.

**A read survives the squash merge that lands it.** `publish/READINESS.md` records each read by
the content it read, the id `git rev-parse COMMIT:PATH` prints, rather than by the commit it was
made at. A squash merge left every read's commit out of `main`'s history twice, at #64 and at #67,
and each time the export refused every path. The export now compares content, refuses a row that
still names a commit with the command that converts it, and names the content to record in every
refusal.

**The road to the public repository closes three gaps.** The code review of this release found
each one. The export's refusal now names every file changed since its read, however many, where it
named twenty and then the content to record covered the rest unread. `--push` and `--bootstrap`
deliver only a commit `refs/remotes/origin/main` holds, so the sign-off they read is one a reviewed
pull request landed. The pre-push hook compares the repository an address names rather than a list
of spellings, so it refuses the public repository with a user, a port, or a slash after the scp
colon, three spellings it let through.

**The release workflow builds on macOS only where it is needed.** In the public repository a pull
request runs all four builds and the formula check, a version tag runs every job, and nothing
publishes unless all four builds, the formula, and the suite passed. In this working repository a
pull request builds the two Linux targets, and the two macOS builds and the formula check run only
once it carries the label `release-check`. Adding the label starts them without another push, and
adding any other label runs what a push would. In a private repository a macOS minute costs as much
as ten Linux minutes. The build matrix now comes from `python3 scripts/release_facts.py matrix`,
which states each target's runner beside it.

**What this release does not do.** It publishes nothing, and no run of the new runbook exists yet.
Steps 1, 2, and 4 were rehearsed against a local target, and the steps against GitHub, crates.io,
and Homebrew first run at the first release after 1.0.0's archival. The label `release-check` is
for the contributor to create, and until then no pull request here runs the macOS jobs.

## 0.33.0 — ship through four channels

One version tag in the public repository, `Firefly-Projects/htmlspec`, now reaches four
channels: the Nix flake, `nix run github:Firefly-Projects/htmlspec`; a GitHub release with a
binary for each of four platforms and their `SHA256SUMS`; crates.io, `cargo install --locked
htmlspec-cli`; and Homebrew, `brew install Firefly-Projects/tap/htmlspec`. The program changes
only in the revision it reports: a Nix build and a build from crates.io now name the commit they
came from, where both printed `(unknown)`.

**One workflow builds everywhere and publishes from one place.** `.github/workflows/release.yml`
runs on every pull request, in either repository, and on a version tag. It refuses a tag that
names another version than `Cargo.toml` declares or sits on a commit `main` does not hold,
packages every crate as a dry run, builds `aarch64-apple-darwin`, `x86_64-apple-darwin`, and the
two Linux targets statically linked with musl, packs each binary with its license texts, writes
`SHA256SUMS`, and checks the Homebrew formula rendered from those digests in a tap made for the
purpose. On a tag it also runs the whole suite. Only on a version tag in the public repository,
and only once the suite has passed, does it publish, in the order a mistake costs least: the
GitHub release, then the crates, then the formula. A rerun finishes a release a run before began.

**The crates are publishable.** Each internal dependency pins the workspace's exact version, and
`scripts/set-version` moves the version and every pin together and relocks; `scripts/check-release`
fails on a pin left behind. The asset store moved from `assets/` into
`crates/htmlspec-assets/assets/`, so the crate that embeds it holds it. Every crate carries the
`LICENSE` and a README of its own, and `htmlspec-assets` carries `THIRD-PARTY-LICENSES`, the
texts of the vendored Mermaid and Vega libraries and of every library bundled inside them, and
declares `MIT AND BSD-3-Clause AND ISC AND Apache-2.0 AND Unlicense`.

**A release stopped partway finishes when run again.** `scripts/publish-crates` publishes the six
crates one at a time, skips each version crates.io already holds, and waits out its limit on new
crates, so the first release publishes in two waves. It runs for at most 30 minutes on the clock,
publishing and waiting together, and stops itself naming what remains. It decides that a crate is
published from the registry's index or cargo's own error, never from words in the build's log,
and a crate refused with 429 is not built again. Every request it makes carries the fixed
User-Agent `htmlspec-release (https://github.com/Firefly-Projects/htmlspec)` and nothing personal.
A new suite stage, `publish-crates`, runs it against a stub index and a stub cargo, so the suite
has 14 stages. `scripts/release_facts.py` states the targets, the crates, the User-Agent, and the
reading of `[workspace.package]` once for every script of the release.

**The export refuses a changed file nobody read.** `publish/export` refused a file added under an
allowlisted path since its read, and let one changed since then cross. It now compares each file's
content at the exported commit with its content at the commit the read names, and refuses any that
differs.

**The release runbook is 0.3.0.** Its version step runs `scripts/set-version`, its specification
requires that, and both its documents carry the fallback stylesheet.

**What this release does not do.** It publishes nothing: no crate, binary, or formula exists until
the first version tag in the public repository, and the runbook a person follows for that tag
comes with `publish-releases-through-their-own-runbook`. Three of the four builds and the
Homebrew check run only on GitHub, so this change's pull request is the first place they run.

## 0.32.0 — publish a public tree

Nothing the program does changes, apart from the version it reports. This release builds the
one road from this repository, which stays private as the working record, to the fresh public
repository `Firefly-Projects/htmlspec`, and it adds the license. It publishes nothing: the first
export runs after the 1.0.0 release.

**An MIT license.** `LICENSE` holds the MIT text, `Copyright (c) 2026 Firefly Systems LLC`.
`Cargo.toml` already declared MIT, and now a file says so.

**The road out refuses before it publishes.** `publish/export` reads a committed ref, never the
working tree, and copies the paths `publish/allowlist` admits. It rewrites the private material
`publish/rewrites.tsv` names, and refuses when a rewrite's text is not there as counted. It scans
the result against `publish/patterns.tsv` and refuses any match, a PNG's text chunks included,
any binary file `publish/READINESS.md` does not record as read by a person at its exact blob id,
and any file added under an allowlisted path since its read. It runs `scripts/check` inside the
exported tree. Only then does it commit, on top of the public repository's own `main` and on
nothing of this repository's history, as the GitHub noreply identity in `publish/identity`. A
local run is the review: it prints the exported tree, and `--push` delivers only the tree a
person reviewed, opening a pull request or updating the one already open for that source.
`publish/scan` audits a tree or a ref on its own. None of `publish/` crosses, because its tables
name the very strings they guard against, and `.github/workflows/publish.yml`, which runs the
tooling's tests on every pull request here, stays behind with it.

**`main-guard.yml` retires.** A branch ruleset already protects `main`, so the workflow that
reported a push after the fact is gone, and the README and the pre-push hook describe the
ruleset. The hook also refuses every push from the private working repository to the public
one, tags included, because such a push would carry this repository's history.

**What this release does not do.** It makes no public repository commit: a push and a bootstrap
both need the contributor's sign-off in `publish/READINESS.md`, which is not there yet. The
public repository's ruleset refuses a direct push, so its first commit is a recorded step: the
contributor disables the ruleset, runs `publish/export --bootstrap`, and enables it again, and
the export refuses to push while the ruleset is in any other state. A scan in the public
repository's own continuous integration is left for a later change.

## 0.31.0 — read a document whose stylesheet cannot load

**A document stays readable where its stylesheet cannot load.** Every document reaches its
stylesheet in the project's one asset store, in a folder above it. Safari, opening a document from
disk, refuses a stylesheet in a parent folder, so every document there rendered as bare HTML,
edge to edge in the browser's default font. Chrome and Firefox load it. Every document the program
now writes, from `htmlspec new document`, `htmlspec new change`, and a run's record, carries a
small fallback stylesheet in its head: a centred column in the house serif and ink, monospace code,
and a skip link hidden until a keyboard user reaches it.

**The fallback changes nothing where the stylesheet loads.** It sits in a cascade layer,
`htmlspec-fallback`, and sets only properties the store's stylesheet also sets on the same
selectors, outside any layer. An unlayered declaration always beats a layered one, so each of its
properties is overridden whenever the stylesheet loads. A test holds that rule against the
stylesheet the toolchain embeds. Rendered by Chrome, a specification with the fallback and without
it gave byte-identical screenshots.

**Existing documents gain it only when you ask.** `htmlspec format --fallback [path]` adds it where
it is absent and replaces one that differs from the current text. It leaves archived changes and run
records as written, because they are history. `htmlspec format --check --fallback` reports each
document that lacks it. Plain `htmlspec format` never adds, changes, or removes it. Adding it to a
runbook changes the runbook's canonical documents, so its next archival asks for a new runbook
version. This repository's accepted specifications carry it now. Its release runbook gains it at
the runbook's next revision.

**What this release does not do.** The fallback gives a readable page, not the design: no contents
column and no styling of requirements, scenarios, or task states. Diagrams and charts still need the
store's scripts, so a figure falls back to the source in its `<details>` element wherever those
scripts cannot load. Validation says nothing about a missing or stale fallback. The contract's
version stays 0.7.0, because what a document may contain does not change.

## 0.30.2 — release a repository that is its own project

Nothing the program does changes, apart from the version it reports. This release is the
repository's own, and it makes its release runbook safe to follow again.

**The release runbook no longer tells a run to delete the project.** At 0.1.0 its step for
regenerating the instruction files said to remove the `htmlspec/` and `spec.nix` that `htmlspec init`
creates, because this repository was not then an html-spec project. Since #55 they are its project and
its declaration. Measured, the removal deletes 504 files, and the asset stage that step named as its
safety net passes afterwards. The runbook is now at 0.2.0: the step runs `init` from the repository
root, removes nothing, and stops if `init` reports that it created a project or if `git status` lists
a path outside `.claude/`, `.codex/`, and `.pi/` that it did not list before `init` ran, other than
an asset `init` reports establishing in the store. The step before leaves the advanced version
uncommitted, so the comparison is with that listing rather than with a clean tree. 0.30.1's warning
not to follow the step is withdrawn.

**The runbook describes the version `htmlspec archive` records.** Archival writes an archived change's
`VERSION` from `spec.nix`, so a run no longer writes it by hand, and the runbook requires `spec.nix` to
be raised at archival to the version `Cargo.toml` declares. Nothing compares the two, and the release
check notices a mismatch only while no changelog heading names the stray version.

**What this release does not do.** The release check's Python stages still fail inside `nix develop`
on macOS, where the system `python3` cannot run; run them outside the shell with its `cargo` on the
path. `htmlspec init` run beneath an existing project still establishes a second one where it runs.

## 0.30.1 — govern this repository from htmlspec

Nothing the program does changes, apart from the version it reports. This release is the
repository's own. Since #55 promoted its project into
`htmlspec/`, the repository held that project and the OpenSpec project it came from, and its release
check, its runbook stages, and three of its tests still read the OpenSpec copy, which nothing writes
any more. The two releases archived since, 0.29.1 and 0.30.0, were checked by nothing in the suite.

**The repository's checks read its project.** `scripts/check-release` reads the archive, the runbook
adoption marker, and the runbooks under `htmlspec/`, and checks all 38 archived changes where it
checked 36. The `self-runbooks` stages check `htmlspec/runbooks`.

**The release runbook is in the project.** It moved to `htmlspec/runbooks/release/` unchanged, still
at 0.1.0, and `htmlspec list --runbooks` lists it. The two stylesheet revisions its documents link were
copied into `htmlspec/assets/`, so they open from the project's own store.

**`openspec/` and the OpenSpec workflows are gone.** The OpenSpec project moved, unchanged, to
`fixtures/migration-source/openspec/`, where the migration survey's tests read it, because
`htmlspec/` is not an OpenSpec project and the survey refuses one. The 52 OpenSpec skills, commands,
and prompts under `.claude/`, `.codex/`, and `.pi/` were removed, and OpenSpec's own installer restores
them. The presentation checks and the asset rules' agreement test read `htmlspec/`, and fail rather
than pass when a place they read is missing.

**What this release does not do.** The release runbook's text is unchanged, and it is out of date. Its
step for regenerating the instruction files says to remove the `htmlspec/` and `spec.nix` that
`htmlspec init` creates. In this repository those are the project and its declaration, and `init`
leaves an existing project as it is. Do not follow that step; a later change revises the runbook.
Comments in the program's sources that cite evidence under `openspec/changes/` are not repointed
either.

## 0.30.0 — keep archived documents pointing at their store

`htmlspec archive` moves a change from `htmlspec/changes/<name>/` to
`htmlspec/changes/archive/<date>-<name>/`, one level deeper, and rewrote nothing in it. A change's
documents reference the project's store and its accepted specifications by relative paths written
for where the change was, so every archived document referenced `../../assets/`, a store that does
not exist from the archive. It opened with no stylesheet and none of its scripts, and its links to
accepted requirements led nowhere. `htmlspec validate` accepted it, because validation looked up a
reference's address and file name in the store above the document and never asked where the path in
front of them led. This repository's first archival with `htmlspec archive` came out that way, and
its 29 references were repaired by hand.

**Archival now keeps every reference reaching what it reached.** Each relative reference in a
change's documents that would reach a different file from the archive is rewritten to reach the same
one: the store's `../../assets/` becomes `../../../assets/`, and a link to `../../specs/` becomes
`../../../specs/`. A reference within the change, such as `evidence/…` or `tasks.html#…`, moves with
it and is left alone, and so are URLs, absolute paths, fragments, and an author's quotation of a
reference in text. The change's evidence is never rewritten. The rewrite is part of archival's
journaled transaction: each rewritten document is staged in the change and takes its document's
place only after the move, so a refusal or an interruption before the move leaves every document as
it was, and the next run completes one that came after it. The journal's format is now
`htmlspec-archive-journal/v2`; a v1 journal left by an earlier release is still settled, and an
earlier release refuses a v2 one rather than completing it without the rewritten documents.

**Validation now checks where an asset reference's path leads.** A reference whose path, from its
document, reaches a directory other than the project's store is reported as `asset/unresolved`, with
the directory it leads to and the path to the store. `htmlspec format` is unchanged: it still leaves a
reference that names stored bytes as written.

**What this release can reject.** A document whose asset references reach a directory other than its
store, which 0.29.1 accepted whenever the address was stored, now fails validation. Nothing this
repository validates changes its verdict. The document contract stays at `0.7.0`.

## 0.29.1 — find a new document's project from where it will live

`htmlspec new document` chose the asset store a new document references by the directory the command
ran in, not by where the document would live. Validation looks for a document's store where the
document lives, in its own directory and above it, so the two agreed only when the command ran
beside the destination. Run from inside a project with a destination outside it, `new document`
wrote a document that validation rejected and exited 2. Run from outside a project into its
`htmlspec/`, it wrote a second copy of the runtime into the project, which `htmlspec assets` then
reported as a duplicate. This repository met the first case the day it became an html-spec project.

**A new document's store now follows its destination.** A document beneath a project's `htmlspec/`
references the project's store, establishing it there if the project has none yet. Any other document
gets a store beside it, as a document outside every project always has. That includes a document in a
project's root, beside `htmlspec/`: it used to reference the project's store by a path validation
never checked, so a reference naming bytes no store held was accepted, and it now gets a store that
validation checks. A destination spelled through `..` is placed where it lands, and one that names
`htmlspec/` another way, in another case where the filesystem ignores case or through a link, is
placed where the filesystem has it. `new change` is
unchanged, because its subject is a project. Tests pin all four ways the working directory and the
destination can differ, and document-template's "A project can be initialized with a conforming
document" now states the rule.

**What this release does not do.** It rewrites no existing document, and the document contract stays
at `0.7.0`. It is the first change made in this repository's own html-spec form.

## 0.29.0 — name each command by its request verb

A command's name says what it is about, and nothing said whether it changes the tree: `status` and
`format` read alike, and only the help text says that one of them rewrites every document it is given.
Two names carry an expectation from OpenSpec that is wrong here: OpenSpec's `sync` and `archive`
write the accepted specifications, and this program's never do.

**Every operation now also answers to a request-verb spelling**, borrowed from htmx's request
keywords and carrying the meaning HTTP gives them:

- `get` never writes: `get validity`, `get list`, `get status`, `get sync`, `get show`, `get assets`,
  `get preflight`, `get instructions`, `get registrations`, and `get survey`;
- `post` creates something that was not there: `post change`, `post document`, `post registration`,
  `post run`, and `post promotion`;
- `put` creates or replaces a whole thing: `put project` and `put format`;
- `patch` changes something that exists: `patch archive`;
- `delete` removes something: `delete registration`.

The verb names the most an operation writes in any invocation, so `put format --check` writes nothing,
as `format --check` does. Both spellings run one operation with the same arguments, options, output,
and exit status. A test runs every operation under both spellings and compares exit status and the
files each one changes. `htmlspec --help` lists the verbs under their own heading, and
`htmlspec get --help` names the operation each object runs.

**What this release does not do.** Every OpenSpec spelling stays, behaves as before, and is still the
one the program's messages and installed instructions name. No command sends a request: the verbs are
vocabulary. The document contract stays at `0.7.0`.

## 0.28.1 — fill only the references a document loads

`htmlspec format` fills an asset reference whose bytes the store does not hold. It found those
references by searching the source for `href="` and `src="`, so it also found them in text. An author
who quoted `href="assets/htmlspec.css"` in a `<code>` element, or escaped a `<link>` in an example,
had the words rewritten to a hashed path, and `format --check` failed until the words were wrong. It
also filled hyperlinks, which the asset rules deliberately never judge, and missed `<object data>`,
which they do.

**The formatter now fills exactly the references the asset rules judge.** Both read one list of the
attributes through which an element loads an asset. The formatter reads a document's tags as the HTML
parser does, so text, comments, and the contents of `<script>`, `<style>`, `<textarea>` and
`<template>` are never references. It follows the parser into SVG and MathML, where those elements are
ordinary, and through a script's escaped text. A value is filled in any quoting and in any letter case. A test
holds the formatter to the parsed tree over every document in this repository and over generated
ones. deterministic-formatting's "Formatting preserves meaning" now states the fill and its bounds.

**What this release does not do.** It changes no committed document, and the document contract stays
at `0.7.0`. A fill still happens only when the bytes are absent, and a reference naming stored bytes
is still kept. The self-migration's staged documents keep the drafters' workaround until the re-stage
removes it.

## 0.28.0 — migrate into the extended vocabulary

Since `0.23.0`, the contract has had ways to say what a migration out of OpenSpec recorded as open
questions: the words a modification changed (`0.23.0`), a task done differently (`0.25.0`), a partial
check (`0.26.0`), and an accepted risk (`0.27.0`). A migration still carried each source distinction as an open question,
because the rule it follows records as open what the destination cannot express.

**A distinction the destination can express is now mapped, not recorded as open.** The rule is
generalized from the rename precedent, and the `migrate` instructions map each one, citing the
requirement that decides it: a modified requirement is stated whole, with the words it changed marked
against the text it replaced; a task keeps the state its record supports, and one done another way is
`done` with a deviation; a unit the author's words name carries `data-refs` beside its link; a partial
check is `partial`, with what it did not check; and a risk's stated settlement becomes its response.

**The survey reads each change's own configuration.** `migrate survey` reports, for each change,
whether it has a `.openspec.yaml`, every key it sets, and its `follows:` value, which becomes
`FOLLOWS`. The creation date crosses only inside the carried file, because the project records none.

**Proved on this repository's own migration.** Its working folder's scripts, brief, and draft checker
follow the mapping, and six packets re-drafted under the revised brief pass the checker with no
finding. The document contract stays at `0.7.0`.

**What this release does not do.** It promotes nothing: re-drafting the whole migration in the new
vocabulary is the self-migration's next step, and the checker's run over the older drafts is its
worklist. Promotion's checks, the survey's entity kinds and counts, and the coverage gate are
unchanged.

## 0.27.0 — record how a risk was settled

A risk recorded what could go wrong and, in a required paragraph, what holds it down. A risk a change
chose to bear had nowhere to say so. Its author wrote a mitigation paragraph that denied being one,
as the release runbook's two risks do, or left the risk out.

**A risk declares how it was settled**, in a `data-response` attribute: `mitigated` or `accepted`.
It projects into a `response` field that every risk in the model carries, and a risk that declares
none is `mitigated`, which every risk written before this release is.

**A mitigated risk states its mitigation, as every risk did.** One with no
`data-field="mitigation"` paragraph is an error, `schema/missing-mitigation`. **An accepted risk
needs none.** It may keep one to state a partial measure, and its statement and notes say why it is
borne. A risk partly held down is `mitigated`, with a mitigation that says how far the measure
reaches.

**The template and the instructions teach it.** The specification template's example risk declares
`data-response="mitigated"` and names the alternative, and the `new` instructions say how a design
records a risk's settlement. The document contract moves to `0.7.0`, and `0.1.0` to `0.6.0` stay
accepted.

**What this release does not do.** Every existing risk keeps its meaning and its validity: each
carries a mitigation. The stylesheet does not render the response. The release runbook's two risks
still project `mitigated` until a later release of the runbook declares them `accepted`. Migration
maps the archive's accepted risks in `migrate-into-the-extended-vocabulary`.

## 0.26.0 — record a partial verification

A verification recorded `passed`, `failed`, `blocked`, or `not-run`, and a check that reached only
part of what it names had no honest outcome. Recorded as `passed`, it claimed the whole scenario;
recorded otherwise, it hid that the part it reached held. Two staged checks had been removed in
review for exactly this, and others were kept as `passed` while covering part.

**A verification outcome, `partial`.** The check ran against what the verification names, reached
part of it, and found that part to hold. A check whose covered part did not hold is still `failed`,
and a check of something nearby is not a partial check of the scenario at all.

**A partial verification says what it did not check**, in one or more paragraphs marked
`data-field="unchecked"`, projected into a new `unchecked` field. A `partial` verification with
none is an error, `schema/missing-unchecked`. A check that failed or was blocked may say what it
never reached too.

**It counts wherever a verification counts.** It clears `review/unverified-requirement`, draws
`review/stale-verification` when its target moves, because the check ran, and in a run's record it
is a recorded outcome. The screen stylesheet labels it, in a new revision, and the `archive`
instructions say when to use it. The document contract moves to `0.6.0`, and `0.1.0` to `0.5.0`
stay accepted.

**What this release does not do.** No document changes: none carried `partial` or `unchecked`
before. Migration's partial checks are mapped by `migrate-into-the-extended-vocabulary`, and the
staged migration's other `passed` checks are not re-audited. An `unchecked` paragraph on a `passed`
verification is accepted, and preflight still writes `passed` or `not-run` for a machine check,
which has no part to miss.

## 0.25.0 — record a task done differently

A task could be marked `done` or `dropped`, and nothing recorded that it was done another way than
its words say. An author had two ways to write the difference, and both misreported it: a note on a
`done` task, which no field and no count reads, so the task claimed its words were carried out, or
a `dropped` task with the difference as its reason, which counted finished work as abandoned. The
paragraph that would say it was refused as naming no field.

**A done task may state a deviation.** One paragraph marked `data-field="deviation"` states what
was done, where that departs from the task's words. It projects into a new `deviation` field. The
task stays `done`, because the work happened, and its words stay as written.

**A deviation belongs only to a done task, and only once.** A deviation on a `todo`,
`in-progress`, or `dropped` task is an error, `schema/deviation-not-done`, and a second deviation
paragraph in one task is a projection fault. A task whose work partly happened is `done`, with a
deviation that says which part.

**`list` and `status` count it apart.** A change with such a task shows it after its state, as
`2/4  in-progress  1 done differently`, and a change with none prints what it printed before. The
task still counts as done. `list --json` carries `doneDifferently` beside `total`, and
`status --json` carries `doneDifferently` and `doneDifferentlyTasks`, naming each one.

**The template and the instructions teach it.** The tasks template's example gains a done task with
a deviation. The `apply` instructions say how to settle such a task, and `verify` and `archive` say
to read its deviation, not its words, as what was done. The document contract moves to `0.5.0`, and
`0.1.0` to `0.4.0` stay accepted.

**What this release does not do.** It migrates none of the archived tasks that carry a note instead;
`migrate-into-the-extended-vocabulary` maps them. A repeated `reason` or `mitigation` paragraph is
still kept silently, as before. Archival does not report tasks done differently, and the stylesheet
gives the deviation paragraph no label.

## 0.24.0 — see what a modification changed

A change's specification states a modified requirement whole, and a requirement can run to fifteen
paragraphs and eighteen scenarios. `sync` says whether the accepted text holds it and, where it does
not, the first paragraph and the scenarios that differ, but never what either side says. So a
reviewer who saw `modified` on a requirement had to find the edit alone, and an edit of one word
looked like no edit at all. OpenSpec has a view for this, `openspec show <change> --diff`, and
html-spec had none.

**`htmlspec show <change> --diff` shows each requirement beside the accepted one.** For each
requirement `sync` compares by its content — one the change marks `added` or `modified`, and one
marked `unchanged` that holds a changed scenario — it shows the paragraphs and scenarios both sides
hold as a count, and quotes the rest from the document that holds each. A paragraph the change edits
is shown beside the accepted paragraph it pairs with, and the words only one side holds are marked on
that side, `{+like this+}` on the change's and `[-like this-]` on the accepted one. A paragraph that
shares fewer than half its words with any partner is shown whole on each side. Scenarios are paired by
identifier, and one the change adds, withdraws, or leaves out is shown as such. With `--json`, the same
facts as a payload.

**It agrees with `sync`, because it asks the same comparison.** It shows a requirement as differing
exactly where `sync` reports its content not taken, against the same accepted requirement, and its
first paragraph not held by both is the one `sync` names. On the change that added `sync` to the
command list, it shows one paragraph of ten with `{+sync,+}` marked, and all eighteen scenarios held.

**An author's `<ins>` and `<del>` are shown as written, where the accepted text confirms them.** Where
the marks account for the whole difference, the view marks the words the author marked. Where they
leave part of it unmarked, the view marks words by its own comparison and says so.

**Nothing it prints is merge text.** Every text is labelled with the document that holds it, no text
combines the two sides, and there is no hunk header and no line marked whole as added or removed, so
neither `patch` nor `git apply` can take it. It reads and never writes. The `verify` and `archive`
instructions send a reader to it, and `sync`'s do not: a merging agent works from `sync`'s report.

**What this release does not do.** `show <change>` without `--diff` refuses and names it: there is no
other view of a change yet. An archived change is refused, because the accepted text now holds its
edit together with every later change's, so what the view would show is not what it modified. A
requirement the change withdraws, or carries forward with every scenario unchanged, is listed and not
compared. Notes, `renamed-from` paragraphs, and presentational markup are not compared, as `sync` does
not compare them. The document contract stays at `0.4.0`, and `sync`, archival, and their payloads are
unchanged.

## 0.23.0 — a modified requirement can mark the words it changed

A change's specification restates a modified requirement whole, and nothing in it said which words
moved. A one-word edit to a requirement of ten paragraphs and eighteen scenarios looked like the
other nine paragraphs. HTML has elements for this, `ins` and `del`, and the projection read both as
presentational: an inserted word came out right, and a deleted one stayed in the statement, so a real
edit projected as "its active changes, and its archive, and its runbooks", which is neither the text
before it nor after it. `0.19.0` made both an error inside every unit.

**A modified requirement or scenario in a change's specification may now mark its words.** `<ins>`
around words the change added, `<del>` around words it deleted, kept where they stood, inside a
requirement or a scenario marked `modified`, in a change's specification, active or archived.
Marking is permitted, not required.

**The model reads the current text, and records the deleted words apart.** The content of a `del`
supplies no field: a statement, a scenario entry, a title, a note, the summary, and a caption hold
the text as it now reads, and a link inside a `del` is no reference, though it is still a link. A
requirement and a scenario record the text of each `del` in a new `deletions` list. The document
contract moves to `0.4.0` for the field and for projection rule 8.6, and `0.1.0` to `0.3.0` stay
accepted.

**Everywhere else the markup is still `schema/change-markup`, and the message says why.** In an
accepted specification, a change's proposal, design, or tasks, a runbook's document, a change's
evidence, or a document outside any project; in a unit marked otherwise, or a decision, evidence,
risk, verification, or task; in a heading or a `renamed-from` paragraph; and around or as a unit.
So a merge that leaves a mark in the accepted contract is refused by `htmlspec validate`.

**The merge writes the current text.** The `sync` and `archive` instructions say so: keep an `ins`'s
words without the element, and drop a `del` with its words. The `archive` instructions say how to mark
a modified requirement's words. A new revision of the screen stylesheet draws an inserted word with a
double underline, so it does not look like a link, keeps a deleted word struck through, and gives each
a start and an end marker that a screen reader can announce and the page does not draw.

**What this release does not do.** No command marks words for you, and nothing checks that a unit's
marks are complete. Archived changes and the accepted specifications carry no marks today, and none
are added. A task, a decision, and the other units that no accepted specification holds still take no
marks. A figure whose caption or diagram source is wholly inside a `del` now reports that it has none,
the one construct that can newly fail, and no document carries one. Documents that name the earlier
stylesheet keep it.

## 0.22.0 — a task can reference the requirement it serves

The tasks template told an author that "a task may reference the requirement it serves with
`data-refs`, once that requirement exists in this change's specification". Following that advice
exactly produced `refs/unresolved` and exit status 1, because a reference resolved within the one
document that made it and nowhere else. A requirement only the accepted specifications held failed
the same way. So authors linked to the requirement instead, and nothing checked a link to another
document: one to `specification.html#req-does-not-exist` was accepted.

**A `data-refs` entry now resolves in three places, in order.** It is tried first in its own document.
In one of a change's documents, it is tried next in that change's specification. Then it is tried in the
project's accepted specifications. The first place that declares the identifier is the one the reference
means. Where a document sits decides what it reaches, not its kind, so a change's proposal, design, and
tasks reach the same places. An archived change reaches what an active one does. An accepted
specification reaches the other accepted specifications. A document outside any project, a file in a
change's evidence, and a runbook's documents, a run's record included, still reach only themselves.

**What a reference finds there is held to the same rules.** A requirement the change's specification
marks `removed` is `refs/removed-target` for the change's own documents, even while the accepted
specification still holds it, because the change's specification is tried first. An identifier two
accepted specifications declare resolves. A reference that resolves nowhere is reported with the places
that were searched, and a reachable document that cannot be read, or needs the parser to recover, is
named as unread instead of read as empty. Validating one document reads the documents it reaches for
their identifiers and reports only on the one named. A fragment link, `href="#…"`, still resolves only in
its own page.

**For library callers, `engine::run` takes a scope** beside the asset store, and `Subject` carries it.
`Scope::loose()` is a document that reaches only itself, which is what every caller got before. The
tasks template's note, the `new` instructions, the `run` help, and the sentences `run` writes into a new
record now say where a reference reaches.

**What this release does not do.** No document that validated before fails, and nothing about it changes
except a reference's message, which now names the places searched. The `review/*` warnings still read one
document. A reference cannot reach another change's documents, or its own change's proposal, design, or
tasks. A plain link to another document is still not checked. The document contract stays `0.3.0`.

## 0.21.0 — sync sees a modification nobody merged

`htmlspec sync` and archival compared requirement identifiers and nothing else, so they gave two wrong
answers. A requirement a change modifies already carries an identifier the accepted specifications
hold, so a word, a paragraph, or a scenario nobody merged was reported as reached: `sync` said "the
change's contract has been applied in full", and archival said the change was ready. A requirement the
change withdraws read the other way round. While the accepted specification still held it, it was
reached. Once the merge correctly deleted it, it was absent, and archival said "This change has not been
synced" and asked. Measured on one change, the untouched accepted specification was called fully
applied, and the correctly merged one was called unsynced.

**Each requirement is now judged by the change state the change declares on it.** One the change adds
or modifies has reached the contract only where an accepted requirement holds its identifier with the
statements and scenarios the change states. Statements are compared in order, scenarios as a
collection, and both as whitespace-normalized projected text. Scenarios the change marks `removed` are
left out, and the accepted requirement must no longer hold them. Notes, the `renamed-from` paragraph,
and change states are not compared. One the change withdraws has reached the contract when no accepted
specification holds it, so a correct withdrawal is never reported as absent. One the change carries
forward `unchanged` is still compared by identifier, because another change may have edited it since,
unless a scenario in it is one the change adds, modifies, or withdraws: then the change has changed it,
and it is compared as a modified requirement is.

**Two new sets, and the report says where, never what.** `untakenContent` names each requirement whose
content did not arrive, with the first statement paragraph that differs and the scenarios each side
holds that the other does not, by identifier. `untakenRemovals` names each withdrawal still held. Both
are in `sync --json` and archival's `--json`, and both count as not synced, so `needsSync` is true and
archival asks. No payload or note carries a statement or a scenario's text from either side, because a
text in a worklist reads as the text to write. `sync` prints a `differs` line and a `held` line for
them. One condition, `reached_in_full`, now decides "applied in full" for both operations.

**What changed in existing payloads.** `reached` narrows to requirements that actually arrived.
`absent`, and archival's `unmergedRequirements`, no longer hold a requirement the change marks
`removed`. The `stated` and `untakenRenames` keys are unchanged.

**The `sync` and `archive` instructions work every set.** `sync.md` merges a modified requirement
through `untakenContent`, gains "Merging a withdrawal", and says what `absent` holds now, a requirement
whose work was abandoned. A test reads the keys `sync --json` emits and fails if the instructions do
not name one. `archive.md` lists the two new findings. Its stop before archiving a held requirement
stays, because archival now reports one and asks, and `--yes` archives over it.

**What this release does not do.** It adds no validation rule: whether a restatement matches the
accepted text is asked when the change is merged, not every time a document is validated. It shows no
diff of what a modification changed. That is planned as `show-what-a-modification-changed`. It does not
compare a heading where no rename is stated. Text the projection does not carry, such as a table inside
a requirement, is not compared, and whitespace inside inline code is normalized away. The document
contract stays `0.3.0`, and every document that validated before still validates.

## 0.20.0 — a modified requirement is stated whole

A change that modified a requirement could leave a scenario out, and nothing said whether that meant
"withdrawn" or "kept". The contract told the merge to apply partial updates, so a change could state
only the part it altered. This project's own archive shows both readings in one change on one day:
`separate-artifact-kinds` left "Status visible to a reviewer" out of one requirement to withdraw it,
and left "Unknown subcommand" and "Discoverable help" out of another, which its merge kept. The text
was the same in both cases.

**A change's specification now states a modified requirement whole.** That is artifact-kinds' new
requirement "A change's specification states a modified requirement whole". The change states every
statement and every scenario the requirement will hold once merged. Each scenario carries its own change
state, `unchanged`, `modified`, or `added`, and the identifier of the accepted scenario it continues. A
scenario the change withdraws is stated and marked `removed`, never left out. An accepted scenario a
modified requirement leaves out is a defect the merging agent reports to the contributor, and it is
neither kept nor deleted on the agent's reading. The partial form is recorded as rejected, with its
reason.

**The merge applies a modified requirement as stated.** "Every synthesis and every contract update is
agent-driven" loses its partial-updates paragraph. The merge still leaves every requirement a change
does not state untouched. The `sync` instructions gain "Merging a modified requirement", because
`htmlspec sync` lists a modified requirement as `reached` whether or not its text was merged. The
section says how to apply one. It also says to hold the requirement and report two cases: a scenario
marked `unchanged` whose text differs from the accepted one, and an accepted scenario the change neither
states nor marks `removed`. The `archive` instructions show how to state a modified requirement, with an
example holding a scenario in each state. Both workflows stop before archiving while a requirement is
held, because `htmlspec archive` compares identifiers and would count a held requirement as reached.

**The `sync` instructions no longer fold an added requirement into an accepted one.** They told an agent
to decide whether a requirement in `absent` was "an addition to one already accepted". A requirement
folded in that way kept its identifier out of the accepted specifications for good, and the only report
left for it called it withdrawn. An `added` requirement that restates an accepted one is now reported to
the contributor with both identifiers, because the change stated a modification as an addition.

**What this release does not do.** It adds no check. A specification stating part of a requirement still
validates, and `sync` still reports it reached. Comparing a modified requirement with the accepted text
is planned as `see-a-modification-the-merge-did-not-take`. No command, rule, unit, or field changes, so
the document contract stays `0.3.0`, and every document that validated before still validates. A new
conformance fixture, `valid-modified-requirement-stated-whole`, pins that a `modified` requirement
holding a scenario in each of the four states is accepted, as it already was. Marking which words of a
statement changed is planned as `mark-the-words-a-modification-changed`. This repository's own OpenSpec
skills, generated by OpenSpec 1.7.0, still carry the partial rule. They are OpenSpec's files and are not
regenerated here.

## 0.19.0 — change markup in a unit is an error

An author who marked an edit with HTML's own change markup wrote a wrong contract, and nothing said
so. `del` marks text an edit removed and `ins` text it added, but the projection gives neither a
meaning, so their text joins the field around them. A requirement reading `… and <del>nothing
else</del><ins><code>store</code></ins> SHALL name them.` projected the statement "… and nothing
elsestore SHALL name them." A paragraph wrapped in `del` became a second statement, a deleted link
still verified the requirement it named, and a requirement wrapped whole in `del` projected as one
the document still states. `validate` accepted all four, under `--strict` too.

**`ins` and `del` in or around a semantic unit are now an error, `schema/change-markup`.** Each
element is reported once: at the unit it is, when its own class names one, else at the unit it sits
in, else at the first unit it encloses. The message
quotes the element with its text and says what to write instead: the text as it now reads, without
the markup. A unit a change edits is marked `data-change="modified"`, and one wrapped whole is marked
`removed` or `added` instead. Only HTML's own elements count: an SVG or MathML element of the same
name is not change markup. The rule reaches every unit in every kind, whatever contract version a
document declares. `ins` is rejected as well as `del`, because a unit that marks one side of an edit
states half of a diff.

**The document contract is `0.3.0`, and it still accepts `0.1.0` and `0.2.0`.** A document can stop
validating at this minor version, which the `0.x` clause of the contract's versioning policy allows.
Before this release, no HTML file in this repository, and none in its staged migration, carried the
markup. The only ones now are the two conformance fixtures that pin the rule. New documents
declare `0.3.0`. The projection is unchanged: every document that still validates projects the model
it did before.

**What this release does not do.** It gives change markup no meaning, so there is still no way to
mark a changed word inside a unit. That is planned as `mark-the-words-a-modification-changed`. It
does not report the markup outside every unit. Prose there projects into no unit's field, and a
proposal may show a wording change with it. The document summary, a figure's caption, and a link's
text still run deleted and inserted words together, and none of them states a contract. `s` and
`strike` are not change markup. For contributors, the conformance fixture generator no longer
deletes the four fixtures written by hand.

## 0.18.0 — a change's evidence is a record, not a document

A project could not keep evidence written as HTML. Given a directory or no path, `validate` and
`format` treated every `.html` file beneath it as a document. That included the files in a change's
`evidence/`. A record declaring an earlier contract version, or naming a stylesheet the project no
longer stores, failed `validate`, and `format` rewrote it. Measured on this repository's seven HTML
records, `format` rewrote all seven. Promotion refused a migration on the same grounds. A staged
project carrying three of those records drew 13 diagnostics, all about the records, with nothing
else wrong in the tree.

**A change's evidence now has a stated place, and it is not a document of the project.** The
`evidence/` directory directly within a change directory, active or archived, holds that change's
evidence at any depth. `validate`, `format` in both modes, and promotion's checks pass over it. So a
record in this format sits there unchanged at whatever contract version it was written against, and
it does not hold back the project's checks or a migration. The rule is positional. A capability
whose path is `evidence` is still a specification. A directory called `evidence` that is not
directly within a change, including one directly inside the archive, is walked like any other.
Outside a project nothing is excluded, so the conformance fixtures behave as before.

**Naming a record still checks it, and nothing writes one.** `htmlspec validate` and
`htmlspec format --check`, given an evidence file, a change's evidence directory, or a directory
inside it, report what they always did. The exclusion applies to what a walk comes across, never to
what a contributor asked for. `htmlspec format` given such a path refuses: it names each record,
writes nothing, and exits as an operation that could not run. The exclusion is defined once, in the
function every walking command calls, so `validate` and `format` cannot disagree about which files
are documents.

**What this release does not do.** It does not migrate this repository. It removes the last
obstacle the product put in that migration's way. It says nothing about asset stores beneath
evidence: none exist, and the duplicate check reads stores rather than documents. It does not report
how many evidence files a walk passed over. `validate` already names each document it checks, so
the records' absence from that list is visible.

## 0.17.0 — a project's stores are the ones inside it

A repository could not hold an html-spec project and anything else that stores assets. The
duplicate check compares a project's stores, and "the project" meant every store beneath the
repository root. So a conformance fixture carrying its own bytes, or a toolchain's source copy of
the runtime, counted as the project storing its bytes twice. In this repository, `htmlspec init`
turned `htmlspec assets .` from clean to nine duplicates, none of them the project's.

**A project's stores are now the ones within its own `htmlspec/` directory, and only those are
compared with one another.** Every other store in view is still checked on its own, so a store that
holds the same bytes twice is reported wherever it lives. It is just no longer compared with the
project's. A second copy of the runtime inside the project, beside a change's documents for
example, still fails as it did.

**The report says which stores were whose.** Whenever a project is in view, `htmlspec assets` states
how many stores within `htmlspec/` it compared and names each store outside it that it checked on
its own. The structured payload gains `projectStores` and `otherStores` path lists. `stores` keeps
its meaning as the total. Promotion's staged-project check now makes the same comparison through the
same function, so a staged project and a promoted one cannot be judged differently.

**What this release does not do.** It does not make this repository an html-spec project. The
release runbook still removes the `htmlspec/` and `spec.nix` that `init` creates, and
`scripts/check-release` and the migration tests still read `openspec/`. It does not change how a
document's asset references resolve, and it adds no way to exclude paths: the project's extent is
the directory the layout already names.

## 0.16.0 — a requirement can say what it used to be called

A renamed requirement kept its identifier and lost its history. A change that renamed one wrote the
new heading on the old identifier, which is correct, and nothing in the document said which heading
it replaced. So a reader of the change met a requirement under a name the accepted contract did not
hold, and had to look up the identifier to learn which one it was. OpenSpec records this with a
`## RENAMED Requirements` section. This project's own archive renames one requirement twice, and
migrating that archive had nowhere to put either rename.

**A requirement can now state the heading it replaced**, in one paragraph marked
`data-field="renamed-from"`. Its own heading is the new name. The rename is separate from the change
state, so both of OpenSpec's cases survive. A requirement renamed and rewritten is `modified` with
the paragraph, and one renamed and otherwise untouched is `unchanged` with it. The paragraph projects
into the requirement's `renamedFrom` field, and never into its statements. With styles on, it reads
as `Renamed from “…”`, and with styles off it is a plain line under the heading.

**Six validation errors bound a rename**, each with its own code under `schema/rename-*`: a rename
outside a requirement, on an `added` or `removed` requirement, stated twice, empty, or naming the
requirement's own heading. Headings are compared by name, without the identifier the template
shows inside them, so `<h3>The name <span class="hs-id">req-x</span></h3>` is named `The name`.

**`sync` and archival now notice a rename nobody applied.** A rename keeps its identifier, so a
comparison of identifiers alone called an unapplied rename reached. `sync` said the change had been
applied in full, and archival said nothing. Where a change's requirement states a rename, both now
compare its heading with the accepted one, and report a difference in a new `untakenRenames` set
that names the identifier and both headings. A change whose only unapplied work is a rename is
reported as not synced, and `needsSync` is true for it. **One reported fact moved:** such a
requirement is no longer in `reached`. That is the defect being fixed, and the `stated`, `reached`,
and `absent` keys are otherwise unchanged.

**The contract is `0.2.0`, and it still accepts `0.1.0`.** The schema pinned one version exactly, so
a minor bump alone would have invalidated every document already written, which a minor bump
promises never to do. `schemaVersion` is now an enumeration of the versions this toolchain accepts,
and new documents declare the newest. The stylesheets have new revisions. The previous ones stay
committed at their addresses for the documents that name them, and the lockfile records them under
a new `retained` list, so no committed byte is left unpinned.

The `sync`, `archive`, and `migrate` instructions say how a rename is merged and migrated. Migration
carries a source rename as a `renamed-from` paragraph rather than an open question. A renamed
requirement keeps the identifier derived from the heading it was first established under, through
every rename and into the accepted specification.

**What this release does not do.** It does not detect a rename. A heading that changed without a
`renamed-from` paragraph is not reported, because that is an edit this change did not state. It does
not merge anything: `sync` still reports, and an agent applies the rename. It does not check that an
accepted specification left out the `renamed-from` paragraph when the rename was applied. The
instructions say to leave it out, and a stale one still states a true fact about the last rename.

## 0.15.0 — a procedure that says what it needs

A runbook that runs only on the machine it was written on is a note to self. Moved elsewhere, every
assumption it makes goes unstated — the tools it calls, their versions, the skills it needs
installed, the state of the tree it starts from — so the failure arrives partway through. This
project met that failure on itself: a release step assumed it was not standing inside a project,
nothing recorded the assumption, and it was found by breaking it.

**A runbook now declares what it needs of a machine, and `htmlspec preflight <runbook>` reports what
is missing before a run starts.** A machine requirement is an ordinary requirement unit whose note
begins `preflight/v1` followed by one JSON object, in one of four closed forms: `tool` (an
executable at an exact version), `toolchain` (a locked `flake.nix`, a development shell, and the
executables it supplies), `skill` (an installed `SKILL.md` in `home` or `workdir` scope), and `state`
(`outside-project` or `clean-git-worktree`). **A runbook cannot supply a command, an argument, or a
pattern**: a carried document that could run code during an inspection is not an inspection.

**Starting a run performs the same checks afresh, and there is no way to skip them.** An earlier
preflight is not permission — the type that permits a start has no conversion from a report — and a
refused start writes no record. A check that passes is copied into the new record as a passed
verification naming what was observed, and the record now states the runbook version that ran. A
completed preflight with unmet requirements exits `1`, sharing the class with a violated contract; a
malformed declaration exits `2`.

**This is the one gate in the runbook design, and the reason it gates is the opposite of why run
conformance does not.** A run in progress legitimately has requirements it has not reached, so
`run --check` stays advisory and is unchanged. A run started on a machine missing its tools is
legitimately nothing, and a warning about a missing dependency is one an operator reads after the
failure it predicted.

**The package manager stays authoritative.** A toolchain check evaluates the development shell's
inputs offline and compares where each named tool resolves against the store paths those inputs
already occupy. It never runs `nix develop`, a build, or a fetch; an environment that is not realized
is reported as *unavailable* rather than failed, and no version is copied out of the manifest into a
second dependency catalog. Every probe is bounded — no shell, closed stdin, ten seconds, 64 KiB — and
interrupting one stops everything it started.

**A runbook now carries a version of its own, and 0.13.0's argument against one is answered rather
than reversed.** That release said a runbook's history is derived from the archive, as a
specification's is. On another machine the archive is absent, so a carried runbook could not say
which release it came from. So a runbook has `VERSION` and `CHANGELOG.md` beside its documents,
archival compares each runbook's canonical documents against the last release it published and
refuses one version for different content, and `scripts/check-release` asks a fourth question about
it. **The cost is stated rather than discovered later: every edit to a runbook's canonical documents
now owes a greater version and a changelog heading**, including an editorial one. A run's record and
changelog prose do not count. The release runbook adopts `0.1.0` with this release and declares one
machine check — the pinned toolchain — which is stricter than the procedure strictly needs: a `cargo`
from rustup reading the same `rust-toolchain.toml` is refused, because a check that accepts any
`cargo` establishes nothing about the pin.

**Older runbooks have no `VERSION`, and starting one is now refused with the migration** rather
than recorded anonymously. Reviewing the runs they already have is unaffected.

**What this release does not do.** It does not install anything: preflight reports, names a remedy,
and repairs nothing, because choosing how to install a tool on an unknown machine is judgment. It
does not check that a tool *works*, only that it is present at a stated version, and a skill check
establishes that content is installed, not that it is good. It adds no transport command — a runbook
is carried by `git`, an archive, or `cp`, into the small layout the README describes, and registered
with `htmlspec store`. It adds no schema change, unit, or artifact kind, no workflow, and the count
of installed instruction files stays at 45. A carried version identifies a claimed release and is not
a signature. And `python3` and `git`, which the release procedure needs, are not declared checks: a
`tool` check pins an exact version and the procedure needs any `python3` at all, so pinning one would
grow the second dependency language this release refuses to add.

## 0.14.0 — a procedure that leaves a record

The previous release gave a procedure a place to live. It gave a run nowhere to go — which
reproduced, inside the new thing, the exact failure the new thing existed to fix: the release
procedure's own history was two archived `tasks.md` files that bind nothing, and a runbook whose runs
leave no record has the same problem under a better name.

**`htmlspec run <runbook> --as <name>` starts a run and writes its record** to
`htmlspec/runbooks/<name>/runs/`. The record is a `tasks` document carrying the runbook's procedure
copied as it stood when the run started, so it says *which version of the procedure ran* — a
procedure improves, and a record that pointed at the current one would describe a run that did not
happen that way. Beside the steps it carries the requirements the runbook states its runs must
record, and one entry per requirement for the outcome and how that outcome was reached.

**The duplication is forced, not chosen, and it was measured rather than reasoned about.** A
reference resolves within one document only: a record naming a requirement that lives in the
runbook's specification fails `refs/unresolved` — an error — so a record that pointed outward would
have been invalid on the day it was written. The document kind is forced the same way: declared as a
specification, the copied procedure draws one error per step.

**`htmlspec run <runbook> --check` reports which requirements each run has recorded nothing for, and
it never fails.** A run interrupted at step six and picked up an hour later is *one* record, edited —
two records would each be wrong about the batch, and a record written once could never honestly say a
step is in progress. So a run under way legitimately has requirements it has not reached, and a check
that failed the batch halfway through the bake is one an operator learns to skip. The severity is the
one the contract already reserves for a report that marks work for attention rather than gating it,
and strict mode cannot promote it.

**Whether a run's outcome was *good* is nobody's business but the reader's.** A recorded failure
satisfies the obligation: a batch that reached 200°F when the recipe says 375°F is a bad batch and a
correct record of one, and a rule that treated it as a document fault would teach an operator to
record a pass. What is reported is a requirement with *no* outcome at all.

**What this release does not do.** It adds no artifact kind, no unit, and no schema change — the
verification unit already carried an outcome, a method, and its references, and this points existing
vocabulary at a new subject. It adds no workflow and no `/htmlspec:runbook` command, so the count of
installed instruction files stays at 45. It does not count a runbook's runs, which is derived from the
records and belongs with a runbook's history. It does not decide when a run should happen. And
**`htmlspec status` is unchanged**: pointed at a runbook by name it still refuses, and run progress is
the run operation's own report rather than a second shape bolted onto a command whose flag says
`--change`.

**A run's record cannot be archived, and this was verified with a record on disk rather than
reasoned about.** The previous release shipped a data loss twice — archival destroying a runbook, and
then a guard that failed open three ways — so a new directory inside that same tree was checked
through every resolution route, with the two known bypasses re-applied one level down. It holds. The
one thing worth knowing: a project whose `htmlspec/runbooks` is a broken symlink can no longer
archive anything until that is fixed, which is the guard failing closed as it is meant to.

**Two findings this release did not close, stated so a reader meets them rather than inherits them.**
A recorded *value* still has nowhere to go but prose: a release run that advanced to a version,
regenerated 45 files, and passed a check puts three values in one paragraph, and nothing can read one
of them out. And a runbook with forty records lists identically to one with none, because the
enumeration reads which documents a runbook holds and a record is not one of them.

## 0.13.0 — a place to keep a procedure

Five of this project's eighteen archived changes carry a `## N. Release` section. All five state the
same four steps, retyped, under a section number that was different every time — 6, 8, 6, 9, 7 —
because nothing was shared. **All five state them in an order that is wrong**, and the last three
each discovered that at run time and regenerated a second time.

**A project can now keep a procedure at `htmlspec/runbooks/<name>/`**, a fourth place beside its
accepted specifications, its active changes, and its archive. A procedure that is run repeatedly is
not a change and does not archive, so `changes/` would destroy it at the end of its first run.
`htmlspec list --runbooks` enumerates what is there, beside the existing `--specs`, and a
bootstrapped project gets the place whether or not it ever puts anything in it.

**A runbook is two documents of kinds that already existed, and no fifth kind was added.** The set
of four stays closed. A kind answers *which stage of one change's argument is this*, and a runbook
is not a stage — it is a destination. The precedent was already here: an accepted specification is
a top-level artifact outside any change whose documents are of kind `specification`, the same kind
a change produces. **The split into two documents is forced rather than tidy**: a specification may
not carry task units, which is the only placement rule in the vocabulary and an error rather than a
warning, so the procedure cannot drift into the canonical document.

**The release procedure ships as the first real runbook, and its two hazards are requirements
rather than notes.** That is the whole point. At 0.10.0 a release recorded that advancing the
version restales every installed file, because `generatedBy` is written from `CARGO_PKG_VERSION` at
run time. Three weeks later, at 0.12.0, a release predicted regeneration would be a no-op and found
the same thing — then wrote down the general form. Two records of one fact, in two archived
`tasks.md` files that bind nothing and that nobody reads. **This release ran the procedure from the
runbook and both hazards arrived exactly as it states them**: the version bump alone turned
`check-installed` red with no workflow body touched, and regeneration rewrote one line in each of 45
files and nothing else; `htmlspec init` bootstrapped a stray `htmlspec/` and a `spec.nix` at `0.0.0`
which had to be removed.

**A runbook carries no version and no changelog of its own**, and the reason is the analogy this
project already relies on: an accepted specification carries neither, and its history is recovered
from the archived changes that touched it. A second version axis with nothing comparing it against
the project's own is accuracy without authority, which is also how this project's version gate once
came to compare against a value no project ever had. The one case that genuinely needs a version —
a runbook carried to a machine where the project is absent, so there is no archive to derive
from — is deferred rather than dropped.

**The cookie recipe was built before the release runbook, deliberately, and it is what makes the
claim checkable.** The release procedure is made of commands, so a change arguing only from it
invites the reading that a runbook is a shell script with prose around it. Nine steps from creaming
the butter to the wire rack, with the recipes it came from cited: `htmlspec validate`,
`validate --strict`, and `format --check` all accept it on the unmodified binary, and nothing in a
bowl of creamed butter automates. The five citation links survive because of a line somebody drew
before there was a recipe — `href` is checked on `link`, which loads, and deliberately not on `a`,
because a hyperlink is a citation and the document renders offline without it.

**Two things this release got wrong and is saying so rather than quietly fixing.** The first is that
**the runbook this release ships is not where this release says a runbook goes.** Creating
`htmlspec/` at this repository's root makes it resolve as an html-spec project, and the asset check
then compares across the four stores its conformance fixtures deliberately carry — one `mkdir`, no
files, and `self-assets` is red. `openspec/runbooks/` fails a different check, because an html-spec
artifact inside the OpenSpec mirror classifies as no OpenSpec kind. So this repository's release
runbook is at `runbooks/release/`, at the repository root, and the location is dogfooded in a
scratch project instead. That is weaker, it is the agent's choice rather than the project's, and
what closes it is migrating this repository to `htmlspec/` — the unfinished half of 0.11.0.

The second is smaller and has a clean answer: **a runbook's units are `unchanged`, and the template
cannot know that.** Written the way a change's units are written, the release runbook's
specification reported seven warnings, five of them saying a verification recorded `passed` against
a requirement marked `added`. The rule was right and the document was wrong — nothing is adding
those requirements, the hazards were measured three weeks and two releases ago. `unchanged` on
every unit took it to zero, and the contract now says so, because `htmlspec new document` has no way
to tell a runbook's document from a change's.

**What a runbook still has nowhere to put is its inputs.** Building the recipe, the ingredients had
no home, and a task unit is the wrong one: an ingredient is not a step somebody performs, it is what
the procedure consumes. So they are a table in a document section that says, in the document, that
this is a finding rather than a design. The release procedure has the same gap in a plainer form —
it needs `cargo`, `python3`, a clean working tree, and a repository that is not already a project,
and none of those is a step either. No mechanism was invented for it: an input is close to a machine
requirement and close to a task's references, and deciding which needs its own argument.

Also recorded, because the change was asked to test it: **the proposal→design→tasks→specification
rhythm fit a change whose output is a procedure**, and the place it strained was not the one the
proposal expected. And a mutation that failed nothing — removing the filter that keeps a loose file
from being read as a runbook — found a test that looked like a guard and was not, which is the same
shape the previous release found by the same method.

## 0.12.0 — the paragraph the template wrote

A `tasks` document shipped with the template's own words as its masthead: *"Replace this paragraph
with what this work delivers."* It passed `htmlspec validate` with **zero diagnostics**, passed
`format --check`, and would have passed archival. A person found it by opening the file and
noticing that the other two changes in the same directory had real summaries.

**A placeholder summary is now reported.** `review/example-content` gains a second site, at the same
severity and for the same reason as the first: `htmlspec new document` writes the placeholder every
time, so an error would fail a document the program itself just wrote. Archival and `sync` refuse it,
where they already refuse an example identifier.

The gap was exact rather than an oversight. `review/example-content` matched a unit's
**identifier** — `req-example`, `task-dropped-example` — built by scanning the templates for
`id="…"`. A masthead paragraph carries no identifier, so "check the identifier" was not a narrower
rule for the summary; it was no rule at all. The contract's existing sentence saying to check the
identifier rather than the text still stands, now bounded to units, because a half-replaced unit
keeps its identifier while a half-replaced masthead keeps its text.

**The comparison is against the whole summary, never a fragment of it.** A substring search for
"Replace this paragraph" would report an author who wrote *"unlike the template's 'Replace this
paragraph' text, this summary states the argument"* — and would report every document in this
project that discusses the rule, including both explainers and the change that introduced it. A
check that fires on documents about itself is one an agent learns to ignore. The accepted cost is
the other side of the same trade: a masthead half-edited into the placeholder plus a sentence of the
author's own is not reported, because detecting that needs a rule that guesses at intent.

The placeholder text is read out of the templates and cached, not listed beside them, so rewording a
template needs no second edit. All four kinds are covered, driven from the template set rather than
from four kind names.

**It was found by a person reading a file, and the count behind "several times" is in the evidence
rather than in this entry.** Scanning every `.html` ever added on any branch — 104 files, 12 of them
documents — finds the condition surviving into a commit twice, and both are the same document before
and after archival. That is a floor rather than the number: a placeholder noticed on reading and
fixed before committing leaves no trace in `git log`. It is also the wrong number to argue from. One
of those two is `evidence/the-plan-for-this-repository-2026-09-09.html`, which sits in
`openspec/changes/archive/` right now — a document titled *"Migrating html-spec's own OpenSpec
project"* whose entire summary says nothing about it. It stays there and stays reported: editing an
archived change to satisfy a rule introduced afterwards would rewrite history to make a check look
clean.

**Two of this change's own tests did not fail when the fix was reverted, and mutation testing is
what said so.** The conformance fixture for a summary quoting the placeholder guarded nothing: with
the comparison reverted to a substring search it reported the rule, and the suite passed anyway,
because the harness skips a fixture declaring no rules. A fixture whose whole subject is that
*nothing* fires cannot state that through its declaration, so it now has a test that names it. The
second was a redundant guard whose comment claimed to carry a property it did not; the comment now
records the measurement instead. A check whose reach is not what it says is this release's subject,
and it turned up twice inside the change that closes it.

## 0.11.0 — arriving from OpenSpec

Every plausible adopter of this toolchain already has an OpenSpec project, because this project is
a spinoff of OpenSpec and its whole argument is that HTML is the better format for the same
content. Until this release there was no way in. A contributor with 161 markdown documents and
sixteen archived changes could read the argument and had nothing to run.

`htmlspec migrate` is that way in, and it is two exact operations with judgment between them.
`htmlspec migrate survey` enumerates what a source project holds and writes nothing.
`htmlspec migrate promote` checks that a plan accounts for all of it, checks the staged tree, and
moves the migrated project into place in one step. Between them a `migrate` workflow directs an
agent to read every document, record what each one *does in the project*, and write a plan a
contributor reviews before anything exists.

**The migration's central obligation has no mechanical check, and that is stated rather than
implied.** Nothing distinguishes a thoughtfully migrated document from a mechanically converted
one. Both validate. Both format canonically. Both pass every check this project has. The only
thing that catches a conversion is a person reading the plan, which is why the plan comes before
any document is written and why the first four stages deliberately produce nothing a validator can
look at.

So this release ships no converter, and the workflow says an agent must not write one. The reason
is the one `artifact-kinds` already gives: a proposal and a design carry no imposed structure in
either format, so there is no grammar to read. A converter would emit documents that satisfy every
check and lose the reasoning that made the migration worth doing — worse than producing nothing,
because it looks finished.

**The plan is a `tasks` document**, one task unit per surveyed entity, carrying the source path,
the destination, the disposition, and a reason for anything folded or dropped. No fifth artifact
kind: the set of four is closed, and a working record used once per project is not grounds to
reopen it. Each fact is a note paragraph rather than an attribute, because the projection ignores
an unknown `data-*` on a unit and a gate reading one would have to parse the HTML a second way.

**Three faults in this change's own planning artifacts were found by building it**, and each is
recorded where it happened rather than quietly fixed.

The design had migration compose each archive directory as `<date>-<version>-<name>`, citing code
that scanned directory names. Two accepted requirements forbid exactly that: the archival gate
reads the version each change *records* and never derives one from a name, and an archived
directory is named for its date followed by the change's name. The citation was true at
`cf11132^`, before the commit titled *"Finish the archival, and fix the gate that never fired"*
moved it — so the design cited real code from before the fix whose entire subject was this defect.
A migration composing that name would have produced a project passing every check whose next
archival passed without advancing.

The mapping table had six rows and this repository holds four entity kinds none of them named:
documents from an earlier hand conversion, per-change configuration, a `## RENAMED Requirements`
section the change-state vocabulary has no value for, and evidence nested one directory deeper
than the format describes. Fifteen files. A survey built from the table would have reported full
coverage over a set that omitted them, and the coverage gate compares a plan against *that*
enumeration — so they would have gone missing with every check passing.

Every one of the seven counts recorded in this change's first task was stale by the time the work
started, and the count that corrected them went stale twice more while it was being implemented.
So no test here asserts a literal. They assert properties that survive the project growing: the
survey's enumeration equals an independently written walk of the same tree, file for file, and
every file classifies.

**A test written for a requirement is not the same as a test that can fail when the requirement is
violated.** Keying the identifier derivation table on the identifier alone, discarding the
destination document, left all 205 tests green — including the one written for renames, because a
rename produces *distinct* identifiers and a wrong key finds no duplicate among them. The case
that needs the keying is the ordinary one: 17 headings in this repository are stated by more than
one archived change, one of them by five, so a slug legitimately lands in five archived
specifications. An identifier-keyed table reports five collisions the source never contained. The
mutation found that, and the replacement test uses those five real changes.

**The check-last rule got tested by something other than a test.** Migrating this repository's
founding change produced 281 errors in one document, which were two classes rather than 281
problems: 276 from one wrong mapping rule, where scenario steps were projected as paragraphs
instead of a description list. One fix took 276 to zero. Halting at the first would have meant
repairing one scenario by hand and continuing into 137 more, in a tree that was already half
migrated.

**Three defects a review of this pull request found**, each a rule that was right in isolation and
wrong against the ordinary case. Promotion refused a destination that already held a `CHANGELOG.md`
— which is nearly every real destination, since a migration promotes into the repository the source
project lives in — so the requirement that the release record be carried and the whole-or-none rule
deadlocked. The shared-version exemption was a substring test on the whole changelog, so `0.1.1`
was satisfied by an unrelated `0.1.10` entry and a real collision passed the gate silently. And the
staged-tree format check reported a stale asset reference as a formatting fault, sending a
contributor to the serializer for a reference repair — across every document of a kind, since a
migration produces them in bulk.

None of this change's 32 test result lines caught any of the three, and the reason is worth
recording: every test constructed its own scratch destination, and a scratch destination is empty.
The ordinary destination is a repository with a history. `evidence/three-defects-a-review-found-2026-09-11.md`
records all three, and each fix was checked by reverting it and confirming the test fails.


**A fifth defect reached continuous integration**, and it is the one the others could not have
found. `flake.nix` gives the build sandbox a filtered snapshot rather than the working tree, and its
filter kept `.md`, `.html`, `.json`, `.css`, `.js`, `scripts/`, and `VERSION` — not `.yaml`. So
`config.yaml` and the per-change `.openspec.yaml` files never reached the sandbox, the survey
enumerated a project that did not hold them, and two of the eight entity kinds were absent. The
filter's own comment already stated the rule it broke: a file the suite reads must be in the source
or the suite passes locally and fails here.

Fixing it found two more extensions dropped, `.txt` and `.py`, and those are the dangerous shape.
Evidence also arrives as `.md`, so the `evidence` kind stays present and no kind-level check notices
three files missing — the survey simply reports a smaller project with every test green. A new check
asserts every extension the project holds reaches the survey, which catches a dropped file without
needing a whole kind to disappear first. Both diagnostics now name `flake.nix` rather than the
classifier, which was correct the whole time.

**A second review found five more**, and the count is now the finding rather than any individual
repair. The survey skipped three whole levels of a source project — a file loose in `openspec/`,
`openspec/changes/`, or `openspec/changes/archive/` was invisible, which on a seven-file project lost
four files including the `project.md` and `AGENTS.md` that OpenSpec itself writes. The cross-device
copy left the destination partly populated while promotion reported it unchanged. The shared-version
check read a changelog that promotion then discarded, so a migrated project failed its own release
check on arrival. A `derives:` note with the wrong separator vanished silently. And a plan item with
no `destination:` passed coverage while having every one of its derivations skipped by the collision
check.

Nine defects across two reviews, and every one is a rule that was right paired with a check whose
reach fell short. **None was found by the test suite.** The reason is now recorded twice: its
constructed destinations were empty, and its constructed source projects were tidy. A seven-file
project with files loose at three levels is four lines of setup, and no test had one.

The mutation discipline earned its place here. Two of my own attempts at testing the cross-device
repair passed against the unfixed code — locking the destination's parent fails before `rename` runs,
and an occupied destination returns `ENOTEMPTY` rather than `EXDEV`, so neither reached the branch at
all. `EXDEV` needs two real filesystems, so naming the fallback as its own function is what makes its
recovery testable rather than merely asserted.

**What this release does not do.** It does not migrate this repository. Stages 1 through 4 ran in
full, the mechanical carries ran in full, and all sixteen delta collapses ran in full — 120
requirements and 545 scenarios, and the hardest of them matched a version written by hand three
days earlier on all six of that exercise's findings. The 60 prose documents were not written,
because each needs an agent to decide which paragraph states an obligation and what each argument
was for, across 133,000 words. One design was migrated properly to answer whether the reasoning
survives, and it did: every decision, every rejected alternative, every mitigation, and in three
places the format made the argument more legible than the source. **It took about as long as
reading the source did.** There is no fast version of this work, and a project migrating itself
should plan for the same order of effort as writing the documents took.

**One thing this leaves open, named rather than filled.** A change's evidence carried in this
format keeps its `.html` name, and document discovery collects every `.html` beneath a project
without excluding `evidence/`. So a migrated project holding such evidence reports asset
diagnostics it cannot clear, and the formatter will rewrite a file whose whole value is that
nobody touched it. Excluding `evidence/` from discovery is the right shape and it is a contract
question about what a project's documents are — decided in a change whose subject is something
else would be the exact fault the layout requirement exists to prevent. The change that settles it
is one whose subject is document discovery.

## 0.10.0 — name the requirement, or name the silence

An agent writing this project's own migration change put a row in a mapping table saying an
accepted specification lives at `htmlspec/specs/<cap>.html`, and wrote a reason beside it:

> a directory holding one file is a directory OpenSpec needs and HTMLSpec does not

No requirement says that. The claim came from `README.md:81`, a tree inside a document whose
fifth line reads *"**Status:** early design proposal. The name, commands, and file layout are
working ideas."* — 76 lines above the tree that was read. And the invented reason was false about
the thing it described: `<capability-path>` is a path rather than a wrapper, so the flattening
would have discarded hierarchy the source expressed. One invention produced two errors.

**The agent was not careless, and that is the finding.** It was directed to work within the
project's rules, it went looking for a rule, it found a statement of the layout, and it used it.
`grep -ril "normative"` across all eight workflow bodies returned nothing. `archive.md` said to
apply a change's contract changes. `new.md` said to work within the rules. `explore.md` said the
accepted specifications state what is promised. Not one of them said a README is not a contract,
or said what to do when no requirement exists at all.

**The invented justification is the worse half, and the rule is written against it.** A rule
stated bare invites the question "where does that come from?". A rule with reasoning attached
reads as a decision the project already took, so nobody asks. It validated, it read as
authoritative, every check passed, and a convention invented this way arrives with as many
instances as the change had files.

**`contract-authority` states what the contract is: the accepted specifications, and nothing
else.** The non-normative sources are enumerated rather than gestured at, because an agent applies
a list and interprets a gesture — a README, a template and the example content it writes, a
changelog, an archived change's proposal or design, a code comment, and the output of an earlier
run. Accuracy confers no authority, and the reason is stated with it: no check compares a README
against the contract, so treating a correct description as binding relies on a correspondence the
project never promised to keep. A change's own specification becomes binding when archival applies
it, and not before.

**The obligation fires on the assertion and not on every sentence, and the bound ships with it.**
Remove the claim's subject and read what is left: where what remains states something the project
requires, name a requirement; where what remains is the agent's own choice or a fact about a run,
none is owed. Imported without that test the rule fires everywhere, and an agent that meets a rule
firing everywhere learns to skim it. That is exactly what the pointer-gloss rule's own deletion
test was added to prevent one release before this one.

Naming a capability where a requirement is meant is insufficient, because it leaves the reader the
search the citation existed to remove.

**Silence is recorded and not filled, and recording it is a step rather than an exit.** Having said
the contract is silent, an agent takes one of three routes: preserve what already exists, make the
smallest reversible choice and mark it as its own, or name the change that would settle the
question. The asymmetry is why this is about attribution rather than about choosing well — a choice
attributed to the agent gets corrected, and a choice attributed to the project does not get
questioned.

**The project shipped the same hazard inside its own templates.** `template-specification.html`
writes seven example units into every new specification and `template-tasks.html` writes one:
`req-example`, `scenario-example`, `decision-example`, `risk-example`, `evidence-example`,
`verify-example`, `figure-example`, `task-example`. `req-example` reads exactly like a real
requirement identifier, because the convention for a real one is the same minus the suffix.
`new.md` said to replace them and nothing checked that anyone did. None of the 36 diagnostics was
about example content — `safety/live-example` is the near miss and reports something else
entirely, a code sample carrying live markup.

**The sharpest evidence is that the nearest existing report was an instruction to merge it.** Run
against a specification created from the template and left as written, `htmlspec validate` accepted
the document and `htmlspec archive --check` reported itself ready with questions. What sync said
was: *"`req-example` appears in no accepted specification … apply the change's contract"*. An agent
that complied put an obligation nobody wrote into the project's contract, with every check green.

**`review/example-content` reports a unit carrying a template's example identifier, and both
operations that reach the accepted specifications refuse while one survives.** It reads the
identifier rather than the text, which is what catches the likely case: the prose rewritten and the
identifier kept. The identifiers are derived from the compiled templates rather than listed a
second time, so a template that gains an example unit is covered with no second edit.

A warning rather than an error, because `project-bootstrap` requires a freshly created document to
validate and the program itself writes these identifiers — an error there would report a defect the
program committed. The namespace decides that: severity is a property of the namespace in
`diagnostic.rs`, every namespace but `review/` is pinned to an error, and a `review/` rule is also
non-promotable, so `--strict` cannot restore the error and fail a document `htmlspec new` just
wrote.

**Archival was the stated gate and it was half the answer.** A review found `sync` performs the
same comparison with no gate at all, and that its report names each absent requirement and tells an
agent to merge it — so on a template-fresh specification it instructed the hazard rather than
merely permitting it. Both now read one `example_content_in`, and in `sync` the refusal sits
*before* the comparison so it arrives instead of that instruction. The contract states the property
rather than naming one operation: every operation that puts a change's contract into the accepted
specifications refuses.

**The rule shipped absent from the conformance suite's `known_rules()`, whose own comment says a
rule missing from that list has no coverage requirement.** So the coverage gate passed vacuously.
The reason the suite was silent is worth stating exactly: all 49 fixtures already carried
`id="req-example"`, because the generator's base document *is* the specification template. The
suite was not quiet for want of a violating document — it was quiet while every document violated
it. The 50th fixture pins the half-replaced case.

**Every workflow body carries the obligation in one identical wording, and there are eight.** The
tasks, the proposal, and the design all said seven. `sync` shipped in 0.7.0, after they were
written. An obligation stated over "all seven bodies" is satisfied while an eighth carries nothing,
which is the defect this release is about arriving inside its own task list — so the contract, the
task, and the test each state the property and the test iterates the workflow table rather than a
number.

**What has no mechanical check is the assertion rule itself, and the verification is a transcript.**
A test can assert the wording is present in all eight bodies, identical, with its bound and its
prohibition. No test can assert an agent named the right requirement, or that a cited requirement
supports the claim made from it. That is judgment, and a word-matching proxy would pass while the
citation stayed wrong. **This is the fifth consecutive release whose central obligation is verified
by reading a transcript** — 0.3.0, 0.4.0, 0.5.0, 0.9.0, and this one — and the reason is the same
each time: the subject is what an agent does with prose, and the program does not write the prose.

The transcripts are recorded in the change's evidence rather than summarised here. Three runs
produced the results worth naming:

- **The before-state reproduced the original defect from scratch.** An agent at `80558b5`, in a
  worktree with no `contract-authority` and no body carrying the obligation, wrote
  `htmlspec/specs/<capability-path>.html` and justified it with "README shows
  `htmlspec/specs/theme-selection.html`". It had never seen the earlier draft and arrived at the
  same place by the same route. It also kept the directory while dropping `spec.md`, so its
  flattening contradicted itself — one invention, two errors, again.
- **The after-state cited or recorded silence on every row.** All 20 requirement titles it cited
  resolve to real requirements. Four points are recorded as silence rather than filled. Unprompted,
  it found the unarchived `project-migration` specification — which addresses that migration
  directly and **agrees with its answer** — and refused to cite it, because a change's
  specification binds at archival. That is the rule firing on the one case where citing would have
  been convenient and correct on the merits.
- **A run whose claims were about code rather than layout** landed the obligation on the version
  increment, which `project-bootstrap` leaves to judgment: it cited the requirement, then wrote
  *"That choice is mine, and I am marking it as mine."*

**The counter-test matters as much as the test.** A rule that fires everywhere is a rule an agent
skims, so the transcripts were read for citations that were not owed. Facts about runs carry none,
and neither does the agent's reasoning about its own ordering. Citations land on claims about the
project and nowhere else.

**One limit of the test, stated rather than left to be found.** A third run given the current
contract but never told to read the instructions also cited `contract-authority` and recorded
silence — because the capability is now *in* the contract it was told to read. So the transcripts
cannot separate the contract's contribution from the guidance's. Doing that needs the contract
present and the bodies stale, and no revision has that state.

**A drift the release found in its own files.** The instruction bodies are compiled into the
binary, so an agent reads the installed copy under `.claude/`, `.codex/`, or `.pi/` rather than the
file in `crates/htmlspec-assets/instructions/`. Those are two artifacts and nothing compared them.
Regeneration was complete through 0.5.0. Then `cf11132` at 0.6.0 changed `archive.md` by 88 lines
and updated 2 of the 5 copies, and four releases after it changed bodies and regenerated none — so
`sync` shipped in 0.7.0 and existed in the binary and in no file an agent could read until now.
**The two copies `cf11132` did update were hand-edited rather than regenerated**: after that commit
both still read `generatedBy: htmlspec 0.5.0` against a 0.6.0 workspace, and that stamp is written
from `CARGO_PKG_VERSION` at run time, so no real `init` run could have printed it.

`scripts/check` gains an `installed` stage that runs `init` into a scratch directory and diffs
against the committed files — 40 of them. Not a body-by-body string comparison, because that would
restate `install.rs`'s knowledge of frontmatter, field order, and name prefixes, and a second copy
of that knowledge produces a check that passes while the real writer does something else. It was
proven against all three failure modes the history contains: an unregenerated body edit, an absent
file, and a hand-edited copy.

**It caught something on its first release.** Advancing the version to 0.10.0 restaled all 40
installed files, because the stamp comes from `CARGO_PKG_VERSION`, and the stage went red. That is
a case nobody predicted, found by a check added mid-change from a review finding. The files were
regenerated through `init` rather than by editing the stamp, since a hand-edited stamp is precisely
what the history above records.

**What this deliberately leaves open.** Whether an accepted specification lives at
`htmlspec/specs/<cap>.html` or at `htmlspec/specs/<capability-path>/spec.html` is a real question
and it is still unsettled. This release is about how an agent behaves when it finds that question
open, not about answering it — and answering it here would have meant deciding a layout question
inside a change about not deciding layout questions quietly. The README stays non-normative, and
its status line stays honest: a design proposal that quietly became a contract is worse than one
that says it is a working idea. `artifact-kinds`' authority statement is not reopened; this states
the general rule that its specific one is an instance of, and does not restate the specific one.

## 0.9.0 — a change can state what it comes after

A project using this toolchain wrote the ordering between its changes into a proposal, in prose,
and said why it had to:

> The contract governs three changes that follow it, and one that follows them. They are stated
> here because the ordering is the argument, and because htmlspec expresses no dependency between
> changes — the convention is enforced by contributors reading it and by nothing else.

That was accurate on every particular. A change directory held three documents and no
whole-directory facts at all. The enumeration returned names and, since 0.8.0, task counts.
Validation's subject was one document, so a rule could not see a sibling — and a `data-refs`
attribute naming a unit in another change was reported as an error even with both documents in one
run, because `refs/unresolved` resolves against the projection the reference lives in. Nothing
anywhere related one change to another.

**The ordering was load-bearing and unenforced.** In that project the sequence *is* the argument: a
normative contract is written first precisely so the three changes it grades cannot be written to
fit models that already exist. Archive the graded changes first and the contract becomes a
description of the work instead of a constraint on it. Every artifact still validates. The method
is the only thing that breaks, and nothing reports it.

This project had met that shape twice before and named it once. Version 0.6.0's gate validated a
convention it invented across eleven archives without refusing anything. Version 0.7.0 was an
instruction telling a contributor to apply a change's contract, with no operation that did it.
`project-bootstrap` already states the general form: a rule nothing checks is not a rule.

**A change now states the change it follows, in a `FOLLOWS` file beside its documents.**

```
htmlspec/changes/compare-the-guidance-solver-families/FOLLOWS
  contract-the-guidance-solver-comparison
```

One name, at most one. Absent and empty are the same state, and neither is an error — most changes
follow nothing and are complete as they stand. `VERSION` is the precedent: also a fact about a
change as a whole, also a file beside the documents, also readable without version-control history.
The proposal and design both said `.openspec.yaml`, which is OpenSpec's file and does not exist in
an HTMLSpec change; the gap and the reasoning are recorded in the change's evidence rather than
resolved silently.

**It is a declaration and never an inference.** Three inferences are refused, each in the contract
with its reason. From names, because `artifact-kinds` says a change is renamed by moving its
directory "because nothing depends on the first name", and an ordering read out of names would make
that sentence false. From creation dates, because `created` records when a change was written and
not what it depends on. From requirement identifiers, because two changes touching one requirement
may be ordered or may conflict, and deciding which is judgment the contract reserves for an agent.

**The listing carries it, and this is the part that closes the 0.8.0 gap.**

```
compare-the-guidance-solver-families     1/1  done  follows contract-the-guidance-solver-comparison (active)
contract-the-guidance-solver-comparison  1/1  done
measure-the-cost-of-decimation           1/1  done  follows contract-the-guidance-solver-comparison (active)
settle-sensors-and-precision             1/1  done  follows compare-the-guidance-solver-families (active)
```

Four changes reading `1/1 done` — indistinguishable to a selecting agent one release ago — and
three of them now say what they are waiting on. The strongest reason not to select a change is that
the change it follows is unfinished, and a listing built for selection was reporting the opposite.
The order is unchanged and does not follow the sequence: a dependency order is unstable in a way
name order is not, since a cycle has no such order at all.

**Resolution has three outcomes, not two, and the third is why it is a sum type.** A predecessor is
archived, active, or matches nothing. An `Option<bool>` would let a caller reach "unmatched" by
negating "resolved", and the report a contributor got would send them looking for work already done
under another name. So `archived`, `active`, and `unmatched` are separate values in the payload and
separate sentences in the report — one is nothing to do, one is work to wait for, one is a
declaration to correct.

**Matching an archived predecessor strips the leading date, and the contract says so rather than
leaving it to an implementer.** Archival names an archived directory `<date>-<name>`, so a successor
that recorded `carry-task-progress-into-the-listing` cannot find
`2026-09-02-carry-task-progress-into-the-listing` by equality. String equality therefore fails in
the one case this whole feature exists to detect, and fails on the **correct** path: it concludes
`unmatched` for a predecessor whose work is done, on every in-order archival. The date is
recognised by the same test archival uses when it decides whether a name already carries one, so
the two cannot drift. Both forms match, because a change already named with a date keeps the name
it had — this repository's archive holds `2026-08-10-html-spec`, and stripping unconditionally
would miss it. That last case was found by a test, not by review.

**Archival reports and does not refuse.**

```
$ htmlspec archive measure-the-cost-of-decimation --check
note: this change follows contract-the-guidance-solver-comparison, which is not archived.
Archive that one first, or state that this change no longer follows it
ready with questions: this change follows a change that is not archived, and archival would
report the findings above and ask
```

An ordering violation joins the two findings archival already had, through the existing report,
question, and `--yes` path with no new mechanism. A gate would be wrong on a case that will occur:
a sequence gets abandoned, its first change is dropped, its second is completed on its own merits,
and a gate would refuse that archival with no way out except editing a file to lie about the
history. The payload carries `outOfOrder` and `unresolvedPredecessor` as values, because an agent
driving archival cannot answer a prompt. The question names the predecessor ahead of the sync when
both fired — an unsynced contract can be applied a moment later, and a sequence archived out of
order cannot be un-archived.

**A cycle is reported as a contradiction and never resolved.** Two changes that each follow the
other are inconsistent recorded facts, so the program names them and stops. With one relation per
change this is a bounded walk over predecessors rather than a graph algorithm, and there is no edge
to blame — a contributor who wrote both declarations can see both. It is reported once for the set
rather than once per member, and the listing stays in name order while saying so, which is the
check that the program did not try to order what it just called unorderable.

**What this deliberately does not do.** It does not block archival. It does not order the listing by
dependency, because the accepted contract requires the order not follow what the listing carries. It
does not make a change a reference target, because a reference addresses a unit and a change is a
directory. And it carries one relation rather than a dependency graph: the motivating project's
fourth change follows all three of its predecessors in prose, and only one of those edges is
expressible here. That limit is stated in the change's evidence rather than left for a reader to
discover. A general model would be designed against no evidence; the one edge that would have
caught the observed failure is the one that exists.

Verified against five real projects driven through the release binary — a three-change sequence
archived out of order and then in order, a renamed predecessor, a two-change cycle, and the
motivating four-change sequence — as 31 assertions rather than output to read.

## 0.8.0 — the listing carries what you select on

`htmlspec list` printed names. Four of them, sorted alphabetically, and nothing else. The
accepted requirement that put the operation there states its own purpose in its second
paragraph — "This is the operation that lets a workflow select what it acts on" — so the
enumeration was built for selection and then given nothing to select on.

**A list of names cannot answer the question anyone asks it.** "Which change were we working
on?" is not answered by four names in alphabetical order. Neither is "which of these is nearly
done", "which has been abandoned", or "which is ready to archive". Every one is a question about
state, the project already computed the state, and the enumeration discarded it. An agent
holding that payload had to run `status` once per name to recover what one call could have
carried — and an agent that did not think to do that selected on name order, which is to say on
nothing. `archive` offers this same enumeration when no change is named, so a contributor picked
blind at the moment a directory moves and a contract is rewritten.

**Each change now comes back with how much of its work is done.**

```
alpha                        2/2  done
beta                         1/3  in-progress
delta-with-a-long-name       1/4  in-progress
epsilon-no-tasks        no tasks
gamma                        0/2  done
```

**The payload carries a count per task state, not a fraction.** `gamma` above is the reason. It
reads `0/2` — nothing was completed — and it is finished, because both its tasks were dropped. A
fraction cannot separate that from work nobody has started, and it cannot separate `0/24` where
every task is `todo` from `0/24` where every task is `in-progress`. Those are opposite situations
at one number, and a contributor deciding what to archive is deciding between exactly them. So
the structured payload states `todo`, `inProgress`, `done`, `dropped` and `undeclared`, and the
human report renders the fraction and the change's overall state from that breakdown rather than
storing either beside it.

**The denominator counts abandoned work.** Dropping tasks from the total would shrink it as work
was abandoned, so `18/20` could become `18/18` with nothing completed. A denominator that moves
is one nobody can compare two readings of.

**There is one counter, and three callers.** This is the part that was not optional. Version
0.6.0's entry records a gate that never fired because it recomputed, from a convention it
invented, a fact the project recorded elsewhere. A listing reporting `17/31` while `status`
reports `18/31` for the same change would be that fault reintroduced and invisible in the same
way — neither operation wrong on its own terms, no test written against either one alone able to
see it. So `list`, `status`, and archival now derive their numbers from a single function over a
single traversal. The accepted contract already required two of the three to agree; this extends
the obligation to the third rather than inventing a rule for it. As a side effect `status` now
parses a change's tasks once where it previously parsed them twice.

**A change with no tasks document has its completion absent, not zero.** `0/0` reads as a change
with no work outstanding, which is the opposite of what an unwritten tasks document means. The
payload omits the fields; the report says `no tasks` in the column the fraction would occupy.

The order is unchanged and does not follow progress: a listing that reordered itself as work
completed could not be selected from twice.

**Archival now names the sync it was already checking.** When a change states a requirement no
accepted specification holds, archival has reported the identifiers since 0.6.0 and told the
contributor to "apply the change's contract" — the same phrasing that, before 0.7.0, named an
operation the command surface did not have. It has one now, and the note did not say so. It does:
the finding reports that the change has not been synced and names `htmlspec sync --change <name>`
as it is invoked. The question asked before archival proceeds says the same, and says that
declining leaves the project as it was, because "Archive anyway?" invites the answer that archives
a change whose contract never reached the project's specifications. The structured payload carries
`needsSync` as a value, since an agent driving archival cannot answer a prompt and should not have
to infer the condition from an empty array.

Nothing recomputes that condition — it is a predicate over the finding archival already collects
from the same comparison `sync` makes, for the reason the rest of this entry is about. Archival
still does not perform the merge: `artifact-kinds` reserves it for an agent, and naming a command
is where a reader would otherwise expect an offer to run it. The report is silent when the
contract has reached the specifications, because a warning that appears either way is one a reader
learns to skip.

One thing this work surfaced and did not fix. `htmlspec list` cannot be run against this
repository, because this repository keeps its specifications as markdown under `openspec/` and
the counter reads `tasks.html`. The verification therefore ran against a project `htmlspec init`
established, and is recorded in the change's evidence with that limitation stated. The active
change `migrate-an-openspec-project` is where that gap belongs.

**The accepted contract now states all of the above.** The code shipped first and the change
directory sat active for a day, which meant the project's accepted specifications did not describe
the behavior this release carries. Two requirements absorbed it, both as supersets of what they
already held rather than as rewrites: `The command surface reports what a project holds` went from
three scenarios to ten, and `Archival assesses the merge and never performs it` from two to six.
The change is archived under `2026-09-02-carry-task-progress-into-the-listing`, recording 0.8.0 in
the `VERSION` file the release check reads.

Worth naming, because it is the same shape as the fault this release repaired: nothing in the suite
reports a shipped change whose contract has not been applied. `scripts/check-release` asks its
questions of the archive alone, deliberately, because a change under way legitimately has not
advanced the version yet. So the gap between "the code is merged" and "the contract says so" is
visible to a person and to no check. Archival's new report closes it for a contributor who runs
archival. It stays open for one who does not.

## 0.7.0 — the operation archival told you to run

Archival ends by naming a step and, until now, the program offered no way to take it. When a
change stated a requirement no accepted specification held, archival reported the identifiers and
said: *"Apply the change's contract before archiving, or confirm each one was withdrawn
deliberately."* There was no `apply`. There was no command of any name whose subject was the
accepted specifications. Ten subcommands, and the merge that turns a change's contract into the
project's contract belonged to none of them.

**The gap was correct in one direction and unassigned in the other.** `Archival assesses the
merge and never performs it` is an accepted requirement and it is right: archival compares
identifiers, refuses to edit an accepted specification, and refuses to propose the text of a
merge. What no requirement said was who does. So the merge happened in the minutes before
archiving, by whatever method each agent invented, at the one moment the project's contract
changes.

**`htmlspec sync --change <name>` reports what remains.** Three sets: the requirement identifiers
the change states, those an accepted specification already holds, and those absent from them. The
third is derivable from the other two and is reported anyway — an agent that has to compute a set
difference to find its worklist will compute it, and a report that makes its caller do arithmetic
to reach the answer it exists to give is a report that gets used wrong.

**It never writes an accepted specification, and the prohibition is stated rather than implied.**
The name is inherited: `toolchain-packaging` requires that an operation OpenSpec provides carries
OpenSpec's name for it, and OpenSpec has `sync`. That name reads like an operation that writes. It
does not, and a reader who assumed otherwise would be assuming the program edits the project's
contract unattended.

That division was not invented here. Reading `artifact-kinds` rather than the code comment that
paraphrases it turned up something stronger than the proposal had claimed: updating the accepted
specifications is named there as the fourth of four agent-driven operations and **the only true
merge among them**, followed by the sentence that settles the design — *"The project SHALL supply
the rules and guides that direct these merges, as instructions rather than as code. What the
project SHALL implement in code is the checking."* Sync's shape is the shape the contract already
required. What was missing was that neither half existed: no command did the checking, and no
stage carried the rules. Both do now.

**A non-empty absent set is a success.** This is the operation someone runs *during* the merge, and
a change mid-implementation legitimately states requirements that have not landed; a withdrawn one
is absent permanently. Failing on either would fail on the normal state of the work. What sync
refuses on is a question it cannot answer meaningfully — a change that does not resolve, an absent
specification, or one that does not validate, because a comparison against a contract the
project's own tools reject means nothing.

**One comparison, not two.** `unmerged_requirements` became `merge_state`, returning all three
sets, and archival and sync both call it. A contributor who runs sync, sees an empty absent set,
and then meets archival reporting unmerged requirements would have met a defect invisible to any
test written against either operation alone — the same shape as the 0.6.0 version gate that
validated its own invented convention across eleven archives without ever refusing anything.
`archival_and_sync_name_the_same_absent_requirements` pins the agreement.

The rename was caught by the suite rather than by review. `the_archive_check_reads_only_the_
specification` enumerates archival's constituent functions by name and asserts none of them opens
a proposal or a design; renaming one broke the enumeration, which is the test doing its job.

**An eighth workflow stage carries the half that needs judgment**, between `verify` and `archive`
in the lifecycle. It directs an agent to read both sides for meaning, decide which accepted
specification should hold each requirement, apply partial updates rather than replacing wholesale,
and leave a deliberately withdrawn requirement absent — reported by name, not merged to make the
list go quiet.

## 0.6.1 — work reaches main through a pull request

Nothing about the `htmlspec` binary changed in this version. The repository did.

Until now the only thing standing between a local commit and `main` was intent. `check.yml`
ran the suite on pull requests, but nothing required a pull request to exist: `git push origin
main` put an unreviewed, unchecked commit on the branch every clone pulls from, and the only
signal was a red run afterwards, on a commit already merged.

**The correct fix is not available here, and pretending otherwise would be the worse outcome.**
A ruleset on `main` requiring a pull request prevents the push — the commit never lands. GitHub
gates rulesets and branch protection behind a paid plan for private repositories, and this
repository is private on the free plan. Both REST endpoints answer `403 Upgrade to GitHub Pro
or make this repository public`. So what is in place is two partial controls, and each one says
in its own header what it is not:

- **`.githooks/pre-push`** refuses a push to `main` before it leaves the machine, which is the
  only moment the mistake is free to undo. It protects the clones that ran `scripts/install-hooks`
  and yields to `--no-verify`. A guard rail, not a lock.
- **`.github/workflows/main-guard.yml`** asks GitHub, for each commit a push added to `main`,
  which pull request contains it. A commit from a merge is associated with one; a commit pushed
  straight to the branch is associated with nothing, and is reported by name. Detective rather
  than preventative: `main` already holds the commit when the run goes red. It also refuses a
  force-push outright, because no pull request makes rewriting history other clones hold
  acceptable.

Together they stop the accident and record the deliberate act. Neither stops someone who means
it. **If this repository becomes public, or the account moves to Pro, the ruleset replaces
`main-guard` entirely** — a gate that reports a violation after it has landed is a standing
invitation to normalise the violation, and that note is in the workflow, in the hook, and in the
README so that removing it is easier than forgetting it.

**A pull request now has to be well-formed, and the rules are a script rather than workflow YAML.**
`scripts/check-pull-request` checks two things and `.github/workflows/pr-guard.yml` only feeds it
the event's facts, for the reason `check.yml` carries no list of steps: a rule that lives in a
workflow file is a rule nobody can rehearse before pushing.

The first rule is the title. The repository squash-merges, so a pull request's title becomes a
commit subject verbatim, and that subject is what someone reads in `git log` years later. The
check encodes the convention this project already has — imperative prose, under 72 characters,
no Conventional Commits prefix, no trailing period, no work-in-progress marker, no pull request
number GitHub is about to append itself. Run against all 43 subjects in the history it accepts
42; the one it refuses is `initialize html spec`, which is the first commit and does not follow
the convention the other 42 established.

The floor on length was the one rule that had to be rewritten after meeting the history. A
minimum word count looked natural and was wrong: `Archive distinguish-command-names-from-skill-names`
is two words and among the most specific subjects here, while `Dark mode` is two words and names
an area rather than a change. Characters separate them; words call them equivalent.

The check is also honest about what it cannot do. Imperative mood is not mechanically decidable —
"Bring" and "Bed" are not separable from "Added" by suffix, and a part-of-speech tagger is not a
dependency this project is going to grow for a title lint. So it refuses a short list of openers
that are unambiguously reports, and is silent about the rest. A lint that guesses is worse than
one with a stated blind spot.

The second rule is that a change records what it changed: `CHANGELOG.md` must appear in the diff,
with a `skip-changelog` label as the way out for a pull request that genuinely has nothing to
record. This is the question `scripts/check-release` cannot ask. That check reads files and never
git, deliberately, because it runs in a sandbox with no `.git` — so it cannot see a pull request
that shipped a behavior change and touched nothing here, because the changelog is well-formed
either way. Comparing two refs is the only way to see that fault, and a workflow is the only
place that has both.

## 0.6.0 — archival finishes what it starts

`htmlspec archive` was named for an operation it did not perform. It checked a change's
specification, printed that archival *may* proceed, and exited zero. The change was still in
`changes/`. The accepted specifications still said what they said before. Four steps remained —
merge the contract, move the directory, name it, record the version — and the program had no
opinion about whether any of them happened.

The workflow instructions carried those steps as prose, and prose is not a gate. An agent that
ran the command, saw a zero exit, and stopped had done exactly what the command told it.

**The one mechanical check archival did enforce had never fired.** `last_archived_version` found
the previous version by scanning archive *directory names* for a dotted triple. This project
records the version somewhere else: a `VERSION` file inside each archived change, which is what
the release-hygiene requirement asks for and what all 11 archived changes carry. No directory
name in `changes/archive/` holds a version. So the scan found nothing, fell back to `0.0.0`, and
every declared version above that passed. Run against a replica of this repository's archive
with a declaration of `0.0.1`, the gate printed `version 0.0.1 follows 0.0.0` and exited zero.

The suite had been validating the defect. `the_version_gate_refuses_a_version_that_went_backwards`
built its fixture as `2026-08-14-0.9.0-something-earlier` — a name in the shape the scan
expected and the shape nothing in the project ever wrote, because archival did not create
archive directories at all. The test passed against a convention that existed only in the test.
That is how a green suite and eleven correctly-versioned archives coexisted with a gate that had
never refused anything.

**Archival now completes or leaves the project untouched.** It moves the change into the archive,
names it `<YYYY-MM-DD>-<change-name>`, refuses a collision rather than suffixing it, and records
the declared version in a `VERSION` file — the same file the gate now reads, so the convention
read and the convention written have one author and cannot drift.

The contract refuses and the work asks. A missing specification, a wrong declared kind, a
validation error, or a version that does not advance ends the run with no prompt. Incomplete
tasks, and requirements the change states that no accepted specification holds, are reported by
name and offered. Both are legitimate — a dropped task is a finished decision and a withdrawn
requirement is legitimately absent — so neither can be an error, and neither can be silent.

**The merge is still an agent's work, and the program can now tell whether it happened.** The two
questions have different difficulty: deciding which accepted requirement a scenario belongs under
is judgment, and observing that `req-x` appears in one set of documents and not another is a set
membership test. Archival compares identifiers, reads neither side for meaning, and never edits an
accepted specification. `artifact-kinds` is untouched by this release. Worth stating plainly,
because the command this one was measured against advertises "update main specs" and does not do
it either: OpenSpec's own workflow routes its merge to an agent-driven sync.

**One accepted obligation was withdrawn, and narrowed rather than reversed.** *A command whose
stated job is checking SHALL NOT write to the project it checks* was the closing sentence of the
version requirement. It generalised a reservation about the declaration into a prohibition on
every write, and that generalisation is what left archival as half an operation. What it
protected survives word for word — archival still SHALL NOT modify the declaration, and advancing
the version is still the contributor's act — restated on the act rather than on a command's name.
A search of all eight capabilities found no second sentence saying the same thing.

Because a command that changed from checking to performing would otherwise take away the question
it used to answer, `--check` applies every condition and performs none of them, failing exactly
where archival refuses. That shape is the one the format subcommand already uses, and the
precedent surfaced from the search for write obligations rather than from the design.

Four commands arrive alongside it: `list`, `status`, `instructions`, and `store`. They are not
speculative. The OpenSpec workflow files vendored in this repository make 170 command invocations
across 8 commands, and four of the top five — `status` at 52, `instructions` at 48, `store` at 22,
`list` at 14 — had no counterpart here. The HTMLSpec prompts did not paper over the gap; they had
no change-selection step at all, because there was no command to put in one.

**What is checked, and what is not.** Atomicity is asserted by comparing the whole tree before and
after, not the one file the operation was expected to touch — an operation that moved a change and
moved it back would pass a narrower assertion. The consent decision is a pure function of three
inputs, so the declined-offer path is a constructed value rather than a simulated terminal, which
is the same reason `prompt` gives for its own shape. The instructions command is checked against
the installed files for every workflow in the table, so the two copies cannot drift. What no test
covers is whether an agent runs the merge before the archival rather than after; the instructions
say to, and the consequence of ignoring them is a report naming requirements the agent then has to
answer for, which is a nuisance rather than a corruption.

## 0.5.0 — a report is for the person who asked

A contributor described a tally-mark app, received three good documents, and read a report
that told them none of it. Their words: "If I was a user I asked to make a tally book app and
I got a bunch of nothing back."

The report opened on a numbered task order, reported a warning count, and closed by asking
them to settle a collapse threshold of 50 and a reset-per-box behavior they never mentioned.
Meanwhile the proposal contained the sentence that answered them — "Click Robin seven times.
It reads 7, drawn as one struck group of five followed by two upright marks" — sitting unread
in a file.

`new.md` ended with five report obligations whose subjects were a directory, an argument,
decisions, an order, and open questions. Not one named the thing the contributor asked about,
so an agent obeying all five produced exactly that report. This is the same shape as the two
releases before it: guidance that, followed exactly, produces a bad outcome.

It was not one file's defect. All seven workflows carried a report section written about
artifacts, and `apply.md` was worse than the file that produced the complaint —
six artifact-subjected lines against five. `verify.md` asked for a task-by-task accounting
that is a list of bare identifiers by construction.

A new `workflow-reporting` capability states three obligations once, rather than seven bodies
each restating them. A report's first claim is now stated in the vocabulary of what the
contributor asked for, and it is a **fact** rather than reassurance — a report that reads
warmly while hiding that no code exists yet is a worse failure than the bookkeeping it
replaced. Where an artifact already says the thing in the contributor's terms, the report
surfaces that statement rather than describing the file: an agent told to summarize invents,
and an agent told to surface quotes.

The obligation is register rather than a fixed sequence of parts. A two-slot template of
"what changed, then where it lives" breaks on `explore`, which produces no artifact by design
and would be forced into "nothing changed, nothing was written". So each workflow keeps its
own subject — `explore` reports a question and its answer, `verify` whether the change is
sound, `archive` what the project now guarantees — and the register is shared.

Every report now names what its references point at, bounded by the deletion test this
project's own explainer standard already carries: delete the identifiers, and if nothing is
left they were the claim and each needs naming. Imported without that test, the rule fires on
`htmlspec validate` and `--json`, which name themselves. The received report failed it in one
place and passed in another — a ten-item task list named one identifier and left nine bare,
with the numbering restarting so items 6 through 10 were referenced and never shown, while the
warning line already named its kind and reason.

An agent now settles what it is competent to settle. An arbitrary value the description does
not determine is chosen and reported as the agent's own, and scope the agent added is offered
for removal rather than presented as a question about what the contributor wants.

**Two checks, and a limit stated rather than discovered.** One test requires every workflow to
carry report guidance; another pins the pointer rule's wording identical across all seven,
since the divergence that produced this defect is exactly what repeated prose does. Both were
observed failing first. What no test covers is whether a report's first claim is in the
contributor's vocabulary — that is a judgment, a report is not a file this program writes, and
a word-matching proxy would pass while the register stayed wrong. Verification is a
transcript, as it was for 0.3.0 and 0.4.0.

## 0.4.0 — a contributor's description reaches the agent

`/htmlspec-new I want to start a new app that is a tally mark creator...` discarded
everything after the command name. No generated file carried an argument placeholder:
`write_skills` and `write_commands` emitted `description`, `allowed-tools`, and
`generatedBy`, and no instruction body contained `$ARGUMENTS`. The tools that load these
files substitute a contributor's arguments into that token, so with no token the text
vanished before an agent saw it — and the agent then asked what the work was, which is
correct behavior on the inputs it actually had.

All seven workflows had the fault, not just `new`. Every one takes a subject and every one
dropped it. It was invisible from inside the program: `init` wrote valid files, every
generated-file test passed, and the loss happened in a tool this program does not control.
Only a transcript showed it, which is why it survived two changes that were reading the
`new` workflow's text closely.

A workflow now declares what it takes, as a property of the workflow rather than one string
shared by all seven — `new` begins from a description of work that does not exist, `apply`
acts on a change already present. Every hint is optional in form, `[thing]` and never
`<thing>`, because a required hint lets a tool reject a bare invocation and that would
delete the behavior that is correct when a contributor has genuinely described nothing.
Each body opens with the input it was given, before any procedure, and says what an empty
substitution means: nothing was passed rather than lost. Without that an agent cannot tell
a missing argument from a dropped one, and suspecting the latter is what makes it ask again
for what it was already sent.

Two checks, and both were observed failing first. The file-level check reported 49 missing
declarations and 49 missing substitutions — one pair for each of the 7 workflows across the
4 tools with a command surface — because it reads the files an install produces rather than
the table they derive from. Both halves are assembled by a format string at write time, so
a check over the inputs passes while a writer that omitted one ships. That is the same
reasoning the name and version checks already record, and here it was not hypothetical:
0.3.0 shipped with all 49 files missing both halves and a fully green suite.

**A remark on the previous change, which is not reopened.** `accept-a-described-change`
dropped its task 5.2 because it could not observe what an agent does with a described
change. This is the mechanism behind that: no amendment to an instruction body can recover
an argument the invocation already dropped. The prose repair in 0.3.0 was correct and is
unchanged here. That task stays dropped, and this change is verified by its own runs.

`AGENTS.md` is untouched, and by construction rather than by an exception: a marked region
in a shared file has no invocation, so there is no argument to pass, and the field that
holds it is separate from the ones the checks iterate.

## 0.3.0 — a need may be an absence

A contributor described a change completely and the `new` workflow refused it twice. The
instruction demanded the need be stated as "a measurement, an error message, a behavior
somebody hit" — three examples that all presuppose an existing system misbehaving. A
capability the project does not have yet has none of those, so an agent reading the
guidance faithfully concluded the request was underspecified.

The diagnostic finding was sharper than ambiguity. The agent inferred that an absence of
capability is a legitimate need, wrote that inference down, and asked for a problem
anyway. The instruction did not fail to inform it; the instruction outweighed it.

A need now takes one of two forms, neither privileged: something existing misbehaves, or
the project cannot do something yet. The second form gets a concrete shape of its own —
who wants the capability, what they cannot do without it, and what working looks like the
first time you see it — so an agent has something to write rather than only something to
avoid. The warning against stating a need as a category of concern survives, because that
warning was the correct part of the sentence being replaced. Where a contributor describes
the work and names nothing, the agent derives a kebab-case name, reports it, and proceeds
rather than spending an exchange on the cheapest thing in a change to correct. `ff.md`
gains a paragraph so an absence of capability no longer reads as an unsettled idea routed
to `explore`.

`artifact-kinds` already said "need" rather than "problem", so the narrowing was the
instruction's own invention and stricter than the contract it served. That contract now
states the obligation directly: authoring guidance must not narrow a need to a defect, and
a described change starts without a supplied name.

**One task is dropped, and it names a defect this change does not fix.** The amended
guidance was run against the recorded request and does not refuse it — the agent named the
absence form, declined to invent a need, and disclaimed the round trip. No change was
created regardless, because `/htmlspec-new <description>` discards the description: the
generated command files carry no argument placeholder, so an agent receives the
instruction body alone. Asking is then correct, and is the behavior this change
deliberately preserved for a request that carries no description. The repair belongs to
`project-bootstrap`, which owns generated-file structure, and is filed separately. Until
it lands, a prose repair to authoring guidance cannot be verified end to end.

No crate's behavior changed. The instruction bodies are compiled into the binary, so the
35 generated files were regenerated and this repository's own installed copies stopped
carrying text the change had removed.

## 0.2.0 — release hygiene is checked

The project's own suite now checks that a change advanced the version and recorded what
changed in it. Seven changes had been archived under `0.1.0` with no changelog at all, and
none of the eight suite stages read the version or looked for a changelog, so the omission
stayed invisible for seven changes.

`scripts/check-release` asks three questions of the archive rather than of the working
change: every archived change records a version, every recorded version above the baseline
is distinct, and every recorded version has a changelog entry. The suite has no advisory
level — a stage passes or fails — so a rule about work in progress would be red from a
change's first commit to its last. The archive is a settled record, so the question has a
right answer at every moment.

This is the first version that names one change, which is what the entry above committed
to. It is also the first entry written when the work landed rather than reconstructed
afterward.

Two faults surfaced only inside the build sandbox, where continuous integration runs the
suite. The new script was untracked, so it was absent from the flake's source snapshot. The
`VERSION` files carry no extension, so the source filter excluded them. The filter is
widened, and the contract now states that a file a stage reads must be both tracked and
admitted by the filter, with the filter change in the same change as the stage.

No crate changed. This is the repository checking itself, and the obligation is
deliberately not written into the prose the program ships to the projects it bootstraps.

## 0.1.0 — the first release

**This entry is a reconstruction.** Seven changes were archived before the project
recorded a version per change or kept a changelog at all, so this entry was assembled
afterward from what those archives state. It is written from their proposals and delta
specifications rather than from a release note anybody wrote at the time, and the earliest
of them predate my involvement. A history labelled as reconstructed is more useful than
an absence and more honest than a confident account.

All seven shipped under `0.1.0`. That is recorded as the truth rather than split into
seven invented versions, which would describe releases nobody ever installed. From the
next change onward, each version names one change.

The seven, in the order they were archived:

**The document foundation** (`2026-08-10-html-spec`). Established what HTMLSpec is: a
canonical HTML document, a machine-readable schema for its semantic units — requirement,
scenario, decision, task, evidence, risk, verification — a deterministic serialization,
and a validator that rejects unsafe or unparseable documents. Before this the project was
an argument in a README.

**Content-addressed runtime assets** (`2026-08-13-content-address-runtime-assets`). The
browser-side runtime was stored once per document, and one copy had already gone stale: an
archived document shipped skip-link markup against a stylesheet with no rule for it, so
the first control a keyboard user reached did nothing. Assets became content-addressed and
shared, which let two accepted requirements hold at once — a library appears once per
project, and a rendered document still reproduces later.

**Four artifact kinds** (`2026-08-14-separate-artifact-kinds`). One document shape carried
all seven unit types at once, which lost the separation that makes the workflow worth
having. A change became a directory of four kinds — proposal, design, tasks,
specification — each with its own job and its own degree of formality. A proposal argues
value and is shaped by nothing; a specification states the contract and is checked.

**The project bootstrap** (`2026-08-17-project-bootstrap-cli`). `htmlspec` had no
equivalent of `openspec init`, so the first thing anyone did with it was the thing it did
worst. `htmlspec init` now detects which AI coding tools a project uses, offers them for
selection, writes the directory structure, installs instructions per tool, and closes by
naming the command to type next.

**The full workflow set** (`2026-08-18-install-the-full-workflow-set`). Bootstrapping
installed one workflow, `archive`, so a contributor saw only the last stage of the
lifecycle and improvised every earlier one. Seven workflows now install in lifecycle
order: explore, new, continue, ff, apply, verify, archive. A rule held in a code comment
was replaced by a test that resolves every invocation the instructions name against the
program's own command tree.

**Conforming generated names** (`2026-08-18-conform-generated-frontmatter-names`). Every
generated command file declared the invocation a contributor types as its `name:` —
values carrying `/`, `:`, and `$`, which a loader restricting names to lowercase letters,
digits, and hyphens rejects. Generated names became the workflow's identity, and a test
now reads back every file an install writes rather than checking the table it wrote them
from.

**One name per workflow per tool**
(`2026-08-21-distinguish-command-names-from-skill-names`). The previous change's fix made
each command file declare the name its sibling skill file already declared, so three tools
listed every workflow twice — fourteen entries for seven workflows. Command files now
declare no name, because the file path is what determines the invocation. The scenario
that had required the duplication was withdrawn from the contract, which is the first
withdrawal the project has made.

Two further merges landed in `0.1.0` without an archived change of their own: a fix that
made the build sandbox run the interactive-prompt check it had been skipping since the
second merge, and the archival record for the change above.
