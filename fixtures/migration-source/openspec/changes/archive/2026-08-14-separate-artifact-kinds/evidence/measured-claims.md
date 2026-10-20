# Evidence for the measured claims

Task 10.6. Each section records a measurement rather than a restatement, with the
command that produced it, so a reader can repeat it. Where a measurement contradicted
something the design or the task list asserted, the contradiction is recorded here
rather than smoothed over.

## 1. The document population, before and after

Counted with `git ls-tree -r --name-only` at each revision, then `git show` on each
`.html` file to test for the metadata field. `9fec61d` is the commit before this
change; `HEAD` is after group 9.

| | before (`9fec61d`) | after |
| --- | --- | --- |
| `.html` files | 80 | 93 |
| declaring `htmlspec-status` | 69 | 0 |
| declaring `htmlspec-kind` | 0 | 81 |
| conformance fixtures | 41 | 48 |
| projection fixtures | 23 | 23 |
| parse-error fixtures | 10 | 10 |
| dogfood documents | 3 | 6 |
| templates | 1 | 4 |

The design claimed "sixty-nine of the project's 82 HTML documents declare a status".
The 69 is exact. The 82 counted three root-level HTML files that are not html-spec
documents; `git ls-tree` finds 80 tracked `.html` files at that revision, so the
denominator was two high. The load-bearing number — how many documents the migration
had to touch — was right.

The 13 new documents are 7 conformance fixtures (the two `kind/*` rules, the advisory
rule, the two positive fixtures pinning the upstream freedom, the remote-script
boundary, and the absent-kind case), 3 dogfood documents from splitting two conflated
ones, and 3 templates from splitting the single one.

**81 rather than 93 declare a kind**: the 10 parse-error fixtures declare none, which
task 6.6 predicted and this confirms — malformed markup never reaches the metadata
rules. The other two are the repository's own README and explainer pages, which are
not html-spec documents.

## 2. Which rules fire from which units

Eleven documents, each built to trip one rule from one unit, validated through the
built binary. This is the claim the design got wrong the first time and corrected;
these are the numbers behind the correction.

| Rule | Fires from | Confirmed by |
| --- | --- | --- |
| `refs/unresolved` | a decision, a task, an evidence, a risk, **or** a verification | five separate documents, one per unit |
| `schema/missing-reason` | a task, and only a task | it reads `task.state == dropped`, and no other unit has a state |
| `schema/prefix-mismatch` | any unit | a risk carrying a `req-` identifier |
| `kind/task-in-specification` | a task, in a `specification` | one document |
| `kind/no-tasks` | the document, when a `tasks` kind has none | one document |
| `review/unverified-requirement` | a requirement, in a `specification` only | two documents: the same requirement fires in a specification and reports nothing in a proposal |

This is what made group 4 six tasks rather than a second generator base. Only
`schema/missing-reason` needed the task back, because it is the only rule whose
subject a task alone can be.

**A correction the task list did not have.** Task 4.2 says
`invalid-schema-missing-reason` is "the only fixture pinning a rule a task alone can
violate". That is true of the *rule* and false of the *fixtures*:
`invalid-a11y-script-supplied-content` also worked by emptying the task's heading.
Found by building the task-free base and running the binary rather than by reading.
It moved to the verification unit, whose `method` field is optional — evidence does
not work, because its required `summary` reports a second fault for one edit.

A second finding from the same probe: removing the tasks section left the contents
list's `#hs-tasks` link dangling, so `refs/broken-link` fired on what was supposed to
be the conforming base. The navigation entry had to go with the section.

## 3. Every rule's defect, reintroduced

Task 10.3. Five mutations, each reverted after measuring. Run with
`cargo test --workspace --all-targets --locked --no-fail-fast`, so every suite reports
rather than only the first that breaks. Full output in `mutation-testing.txt`.

| Mutation | Tests that failed |
| --- | --- |
| Placement rule reports nothing | 5 |
| Completeness rule inverted | 17 |
| Advisory rule scoped to the wrong kinds | 8 |
| `review/*` becomes promotable | 5 |
| `kind` no longer required by the schema | 41 |

Every defect was caught. Two results are worth naming:

**The `review/*` promotability mutation** is the one that matters most, because it is
invisible under default validation — the warning is still reported, and the document
is still accepted. It fails only under `--strict`, and it failed
`rules::tests::strict_mode_does_not_fail_a_specification_awaiting_verification` and
`strict_mode_promotes_every_warning_except_a_review_marker`. Without those two, a
specification legitimately in progress would have started failing a project's CI and
nothing would have said why.

**`--no-fail-fast` changed the answer.** My first run of the placement mutation
reported one failing test, and I nearly recorded that as thin coverage. Cargo had
stopped after the first failing suite. With every suite running, the conformance
fixtures caught it too — `every_fixture_reports_exactly_the_rules_it_declares` and
`every_fixture_is_accepted_exactly_when_it_declares_it_is`, which is what those
fixtures exist for.

## 4. The upstream freedom, measured

A proposal carrying all seven semantic units, validated at both strictness settings:

```
$ htmlspec validate p.html --strict
  no diagnostics: the document is accepted
```

Zero diagnostics, not merely acceptance. This is task 10.2, and it is the freedom the
change is for stated as an executable claim. It is pinned as a test as well —
`the_kind_namespace_has_exactly_two_rules_and_constrains_no_upstream_kind` — which
draws its unit list from the schema, so a unit type added to the contract is covered
without an edit.

An earlier run of the same document reported two warnings, both
`review/stale-reference` from `data-change="added"` markers pointing at a requirement
in the same document. Those are the dependency rules working, not kind rules; the
measurement above removes the change states so the claim is about kind alone.

## 5. Strict mode does not gate an in-progress specification

Task 10.4:

```
$ htmlspec validate fixtures/dogfood/dark-theme-specification.html --strict
  warning: review/unverified-requirement at req-theme-selection: ...
  warning: review/unverified-requirement at req-theme-contrast: ...
  0 errors, 3 warnings: the document is accepted
```

Reported and accepted, under `--strict`. This is the `review/*` placement decision
holding: verification arrives as work completes, so a rule that failed the document
would gate the work that produces the verification.

## 6. The archival round trip

Nine tests in `crates/htmlspec-cli/tests/archive.rs`, driving the built binary over
change directories created by `init --change`:

- A valid specification lets archival proceed.
- A dangling reference stops it, and the message says archival does not proceed.
- A task unit in the specification stops it (`kind/task-in-specification`).
- An unverified requirement does **not** stop it — the advisory placement holding at
  the operation it was decided for.
- A change with no specification is refused, and the message names whose work
  producing it is.
- A specification declaring another kind is refused.
- All four documents survive the check.

And the negative, which is the load-bearing one. A change whose proposal and design
are filled with `SHALL` and `MUST` prose produces no specification:
`the_archive_operation_ships_no_requirement_extractor` asserts the file does not
appear. `the_archive_check_reads_only_the_specification` asserts the same property at
the source level — the command resolves one file by name, names neither upstream
document, and calls `read_to_string` exactly once.

That negative exists because the failure it prevents is invisible to a validator. An
extractor's output would be well-formed and would misstate the contract, and it would
look authoritative doing it.

## 7. Authoring one real change in the project's own format

Task 10.5. The change directory is at
`openspec/changes/separate-artifact-kinds/documents/`, created with
`htmlspec init … --change --assets .` and authored by hand. All three documents
validate, all three are canonically formatted, and the presentation checks discover
them under the change location.

`init --change` reused the project's existing asset store rather than writing a second
copy — the relative reference it computed is `../../../../assets/sha256-…`, resolved
against the store four levels up.

The findings are recorded in the tasks document itself, under "What the vocabulary made
awkward", because that is where a reader of the change will look. The four, in the
order of how much they cost:

1. **A reference resolves within one document, and a change is now four.** The sharpest
   cost, and the one the design did not anticipate. Before the separation a task
   carried `data-refs` naming the requirement it served and the validator resolved it.
   Split apart, that attribute names an identifier no unit in the tasks document
   declares. It had to go. Every task now says in prose which obligation it served,
   which carries the information for a reader and none of it for a tool. The dependency
   rules cannot see across the boundary either.

2. **The tasks kind wants a smaller granularity than is readable.** Fifty-four items in
   ten groups. Fifty-four task units would produce a document nobody reads; ten
   produced one that works. But a state belongs to an item rather than a group, so one
   task unit reading `in-progress` is doing work that four separate states would do
   better.

3. **A design's decisions want to be both prose and units.** The boundary was a
   judgment call at every paragraph. The freedom is real, and it means two authors
   produce differently shaped designs for one change, and a reader who wants the
   decisions has to read rather than query. That trade was deliberate; living inside it
   for one document is what confirms it is real rather than theoretical.

4. **Authoring in HTML costs more than Markdown, and not by a little.** Every paragraph
   is a tag pair, every unit is four lines of attributes before any content, and a
   mistyped identifier prefix is a validation error rather than a typo. The `init`
   shapes help and the validator catches what Markdown would let through. The honest
   summary is that the format costs more to write and pays it back in what it can
   check.

Finding 1 is the one worth acting on, and it is not in scope here. It is a real gap in
the four-document model rather than an inconvenience: the model gives up
machine-checkable traceability between a task and its requirement, and nothing in this
change replaces it.
