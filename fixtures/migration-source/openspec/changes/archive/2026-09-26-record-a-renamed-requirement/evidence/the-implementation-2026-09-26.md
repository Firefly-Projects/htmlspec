# What the implementation checked, and what it found

Tasks 4.1 to 8.2, recorded as they were run. Each section states the check, what it showed, and
anything the task list or the design did not expect.

## 4.1 Six rules, each seen failing before it existed

The six rules live in `crates/htmlspec-validate/src/rules/renames.rs`, in the `schema/*` namespace,
as errors with their own codes:

| Rule | Reports |
| --- | --- |
| `schema/rename-outside-requirement` | a `renamed-from` paragraph in a scenario, decision, task, evidence, risk, or verification |
| `schema/rename-on-added` | one on a requirement whose change state is `added` |
| `schema/rename-on-removed` | one on a requirement whose change state is `removed` |
| `schema/rename-repeated` | more than one in one requirement |
| `schema/rename-empty` | one whose text is empty after normalization |
| `schema/rename-to-same-name` | one stating the requirement's own heading name |

The tests were written first, against a `validate` that did nothing. **Nine of the twelve
failed**: each test asserting a rule fires, and the test asserting the two change states are found
in the schema. The three that passed assert silence: a rename on a `modified` or `unchanged`
requirement, a requirement with no rename, and a rename that changes only case. They pass under a
rule that reports nothing, correctly, which is why they cannot be the whole of the coverage. With
the rules in place, all twelve pass, and so do the other 203 unit tests in the crate.

**A misplaced rename is one diagnostic, not two.** Projection section 4.0 made any `data-field`
naming no field of its unit a `schema/projection` fault, so a scenario carrying a `renamed-from`
paragraph would have reported that fault and `rename-outside-requirement` together. The projection
now records such a paragraph beside the model, in `Projection::misplaced_renames`, and does not
fault it. Section 4.0 states the exception. `a_rename_outside_a_requirement_is_reported_once_per_unit`
asserts exactly one diagnostic for each of the six other units.

**An empty paragraph had to survive the projection.** Every empty paragraph is discarded before its
`data-field` is read, so an empty `renamed-from` would have projected as no rename at all, and
`rename-empty` could never fire. The projection keeps `renamed-from` paragraphs, empty ones
included, as `Vec<RenamedFrom>` on the requirement, with an `Empty` case rather than an empty
string. The model's `renamedFrom` is serialized from that list: the first paragraph with text.

## 4.2 The fixtures, and the generator deleting four rather than two

Twelve fixtures, one valid and one invalid per rule, each valid one differing from its invalid twin
in the one way the rule turns on:

| Rule | Invalid | Valid twin |
| --- | --- | --- |
| outside a requirement | a decision carrying the paragraph | a decision whose *note* mentions its old title |
| on added | renamed, `added` | renamed, `modified` |
| on removed | renamed, `removed` | renamed, `unchanged` |
| repeated | two paragraphs | one, with the older history in a note |
| empty | a whitespace-only paragraph | text inside `<em>` |
| to the same name | the heading's own name | a case-only rename |

Run through the built binary, each invalid fixture reports exactly its one error. Beside it are
only the ten `review/*` warnings that `valid-minimal` also reports, from the template's example
identifiers. The exception is `invalid-schema-rename-on-removed`: it carries one more warning,
`review/unverified-requirement`, because its verification can no longer name a requirement that is
being withdrawn (that would be `refs/removed-target`, an error).

**The generator's base now declares `0.1.0` explicitly.** It is rendered from the specification
template, which now declares `0.2.0`, so regenerating would have rewritten the version in every one
of the 52 committed inputs. `base()` rewrites the template's version back to `0.1.0`, following the
same approach it already uses to keep the pre-addressing asset paths. That kept all 52 byte-identical
(`git status` showed no modified fixture). It also makes the whole conformance suite a standing
check that a `0.1.0` document is still accepted, which is the promise a minor bump makes. The twelve
rename fixtures declare `0.2.0` through their own edit, because they use a field `0.2.0` added.

**The generator deleted four hand-written fixtures, not the two the task anticipated:**

```
removing stale fixture valid-runbook-machine-requirements-unsatisfiable
removing stale fixture warning-review-placeholder-summary
removing stale fixture valid-runbook-machine-requirements
removing stale fixture valid-summary-quoting-the-placeholder
```

The two `valid-runbook-machine-requirements*` fixtures arrived with `5ea25e7` ("Carry a runbook to
another machine", #33), after the evidence that recorded the first two was written. All four were
restored with `git checkout -- fixtures/conformance/`, and the tree then held 68 fixtures. The trap
`log-a-runbook-run/evidence/the-generator-deletes-two-fixtures-2026-09-18.md` describes has doubled
since it was recorded, and it is still not this change's to fix.

## 4.3 Formatting a rename is canonical and idempotent

Five documents carrying `renamed-from` paragraphs were formatted twice with `htmlspec format`: four
conformance fixtures and the projection fixture. The second pass changed nothing (`diff -r` was
empty). A well-formed `renamed-from` paragraph is canonical as written:
`valid-rename-of-a-modified-requirement` passed `format --check` unchanged. The formatter turns a
whitespace-only paragraph into an empty one, and that one still reports `schema/rename-empty` after
formatting. So a rename's fault survives formatting as well as its content does.

## 6.4 The installed copies, regenerated through `init`

`htmlspec init --tools claude,codex,pi` rewrote the fifteen installed files of the three workflows
whose bodies changed: `archive`, `migrate`, and `sync`. That is five files each, a command and a skill
under `.claude/`, a skill under `.codex/`, and a prompt and a skill under `.pi/`. `AGENTS.md` was
untouched. It also bootstrapped a stray `htmlspec/` and a `spec.nix` at
`0.0.0`, as the release runbook's second hazard says it will, and both were removed.
`scripts/check-installed` then reported `ok  45 installed instruction file(s) match what init writes`.

**`scripts/check-installed` cannot run in either environment as it stands.** Outside
`nix develop` there is no `cargo`, and inside it there is no `python3`. It ran with the development
shell's `cargo` put on the system `PATH`. The release runbook lists `python3` as an input the
procedure consumes, so this is the runbook's stated environment rather than a defect in it. It is
still two environments stitched together by hand for one stage.

## 7.2 One identifier through this repository's rename chain

The `toolchain-packaging` requirement titled "The command surface names its operations as OpenSpec
names them" was established as "A single command exposes validate, format, and init". Derived from
the full heading, that is `req-a-single-command-exposes-validate-format-and-init`. The chain across
this repository's archive:

| Document | States | Rename it records | Identifier |
| --- | --- | --- | --- |
| `2026-08-10-html-spec` | A | | `req-a-single-command-exposes-validate-format-and-init` |
| `2026-08-14-separate-artifact-kinds` | A | | same |
| `2026-08-17-project-bootstrap-cli` | B | A → B | same (B alone would derive `…-format-bootstrap-and-create`) |
| `2026-08-30-finish-the-archival` | C | B → C | same (C alone would derive `req-the-command-surface-…`) |
| `2026-08-31-sync-a-change-into-the-accepted-contract` | C | | same |
| accepted `toolchain-packaging` | C | | same |

**The identifier is the same in both renaming changes and in the accepted specification**, as the
task required. No destination holds a second requirement that resolves to it, so the plan's
collision check finds nothing.

**The design did not name one case: `2026-08-31` restates C without renaming it.** It is not part of
the chain, and deriving from its own heading would give the one obligation a second identifier in
exactly that document. The rule the design states, "the identifier its `FROM:` heading already
resolved to", covers the renaming changes. `migrate.md` now also says a later change that restates
a renamed heading resolves through it.

`plan.rs`'s `one_obligation_renamed_across_changes_is_not_a_collision` used invented change names,
and it asserted the rule this change replaces: "one obligation, three identifiers". It now states
the real chain above, five statements with one identifier, and still asserts no collision. That
also makes it catch a collision key that drops the destination, which the test after it says it
could not do while renames gave distinct identifiers. Mutation 19 in the mutation record checks
that claim.

## 8.1 and 8.2 The stylesheets, and what a reader sees

New revisions of both stylesheets, at new addresses: `htmlspec.css` at `sha256-aac8967609f80d9f`
and `htmlspec-print.css` at `sha256-04ac9a24fc436a87`. The screen sheet labels the paragraph with
generated content, "Renamed from" followed by the heading in curly quotes. The print sheet sets its
ink.

**The lockfile had no place for a revision left in place.** `own` records one digest per file, the
build embeds what `own` records, and `init` stores it. Leaving the previous revisions at
`sha256-e4f5de7cecc4f691` and `sha256-0350261bb5666167` would have left committed bytes that nothing
pins. The lockfile's own comment names that as how a stale stylesheet once shipped undetected. The
runbook and several archived documents in this repository name those addresses, so they stay.
The lockfile gains a `retained` list recording them, and
`every_retained_revision_is_committed_and_superseded` checks three things for each entry: its bytes
match its digest at its address, it is not the current revision, and it is not embedded. The
templates moved to the new revisions with no edit, because they name assets by file name and the
address is substituted at render time.

Rendered in headless Chrome 153 from a project `init` created, with a specification carrying the
7.2 requirement renamed and modified:

- **With styles**, the heading, its identifier, and a MODIFIED badge are followed by the line
  `Renamed from “A single command exposes validate, format, bootstrap, and create”` in the muted
  interface face.
- **Printed to PDF**, the same line appears, in ink, under the same heading.
- **With the stylesheet links removed**, the previous heading is a plain paragraph between the
  heading and the statement. It is readable and unlabelled. The design accepts that as the same
  degradation the change-state badges accept.

The browser ran only outside the Bash tool's sandbox. Inside it, headless Chrome hung without
writing a file.
