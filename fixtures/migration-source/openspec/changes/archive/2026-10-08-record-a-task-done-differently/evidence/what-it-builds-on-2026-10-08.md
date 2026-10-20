# What this change builds on, as built

**Date:** 2026-10-08
**Task:** 1.4, and the reading of the code the design's Context and decisions rest on, done before
any edit
**Revision:** `main` at `b768860`, the archival of `show-what-a-modification-changed`

The design was written on 2026-09-30 and 2026-10-01, before any of the six changes ahead of it in
this set landed. Each claim it makes about the code was read again against `b768860`. Three were
wrong, one task's instruction is stale, and one point the design does not reach had to be decided.
`design.md` and `tasks.md` were corrected before any code, and each correction says so where it was
made.

## The contract version (task 1.4)

`CONTRACT_VERSION` in `crates/htmlspec-schema/src/lib.rs` is **`0.4.0`**. The schema's `$id` is
`https://html-spec.dev/schema/0.4.0/document.schema.json`, its `schemaVersion` enumeration is `0.1.0`
to `0.4.0`, every `assets/template-*.html` declares `0.4.0` in its meta element and its masthead, and
the projection specification's header reads `0.4.0`.

**Two earlier changes in this set moved it**, by the contract changelog and the archive:

| Contract | Dated | Moved by | Released as |
| --- | --- | --- | --- |
| `0.3.0` | 2026-10-02 | change 1, `reject-change-markup-inside-a-unit` | 0.19.0 |
| `0.4.0` | 2026-10-07 | change 5, `mark-the-words-a-modification-changed` (PR #46) | 0.23.0 |

Changes 2, 3, 4, and 6 left it alone, and released as 0.20.0, 0.21.0, 0.22.0, and 0.24.0. So changes 1
to 6 shipped as 0.19.0 to 0.24.0, as task 8.2 expects, and **this change moves the contract from
`0.4.0` to `0.5.0`**, the minor version after the one HEAD declares, as the Migration Plan says.

**Correction 1.** The design's Context said "The contract is `0.2.0`" and that every template
declares `0.2.0`, and its Migration Plan named `0.3.0` as the version if nothing moved the contract
first. Both now say `0.4.0` and `0.5.0`.

## `"done"` cannot be the stem the rule reads its state by

Decision 2 said the rule reads `done` with `status_named("done", "definitions/task/properties/state")`,
and that `schema_rust_split.rs` gains `done` as a literal rule code may not restate. The two cannot
both hold. `rule_code_does_not_restate_schema_expressible_values` matches each entry of
`SCHEMA_OWNED_LITERALS` as a whole string literal, `"done"` with its quotes, on every line of rule
code outside a test module and a comment. `status_named("done", …)` is exactly that literal, so the
test the design adds would fail on the rule the design adds.

The existing rules show the intended shape. `status_named`'s own comment says "The stem is a
fragment of the concept's English name and not the contract value": `dropp` finds `dropped`, and
`supersed` finds `superseded`. Among the four task states, `don` is contained in `done` alone, so
`status_named` returns it, and a schema rename that broke the match would still surface through
`every_conditional_rule_finds_its_state` rather than as a rule that stopped firing.

**Correction 2.** Decision 2 now reads `done` through the stem `don`, and says why the whole value
cannot be the stem. `every_conditional_rule_finds_its_state` asserts the stem finds `done`.

The rest of the decision holds as written: `"done"` appears in the validator's sources once, at
`relations.rs:425`, inside the test module, so adding `done` to `SCHEMA_OWNED_LITERALS` flags
nothing today.

## The fixture generator no longer deletes the hand-authored fixtures

Task 2.4 said the generator deletes four hand-authored fixtures and asked for them to be restored.
`reject-change-markup-inside-a-unit` (#39) added `HAND_AUTHORED` to
`conformance/src/bin/generate_fixtures.rs`, naming exactly those four, and the generator now keeps
them, asserts each still has its `input.html` and `expected.json`, and refuses a name in both
tables. Run at HEAD, it wrote 68 fixtures and `git status fixtures/` was empty afterwards; the
directory holds 72, the 68 and the four.

**Correction 3, to task 2.4.** There is nothing to restore. The task now says to check that the four
are untouched after the generator runs.

## `status` cannot be called from a unit test as it stands

Task 2.5 puts the `status` payload's tests in `commands.rs`. `status` resolves its project with
`crate::store::project_for(store)`, which with no store reads the working directory, and a unit
test runs in `crates/htmlspec-cli`, which is not a project. `list` met the same constraint and was
split into `list`, which resolves, and `list_in`, which reports on a resolved project, "so the
enumeration is reachable from a test without registering a store".

**Decided here, because the design does not reach it:** `status` is split the same way, into the
resolution and a `status_of(directory, streams)` that reports on a resolved change directory. The
messages, exit codes, and payload for every existing case are unchanged, which
`status_and_archival_agree_about_open_tasks` and the `status` tests in `tests/cli.rs` hold. Recorded
in design decision 4.

## What holds as the design states it

Each of these was read at `b768860`:

- **`project_task` accepts `reason` and `note`.** `project_paragraphs(unit, &["reason"], …)`, and
  `note` is accepted by every unit. Any other `data-field` is fault 4.0, measured with `deviation`
  in `the-measurement-repeated-2026-10-08.md`.
- **`Paragraphs::single` says "the caller reports the repetition"**, and no caller does: not
  `project_task` for `reason`, not `project_risk` for `mitigation`, not `project_verification` for
  `method`. A second `reason` validates with no diagnostic, measured again today.
- **`dropped_tasks_state_a_reason`** is in `relations.rs` and reads `dropped` through
  `status_named("dropp", "definitions/task/properties/state")`. `relations::validate` is already in
  `rules::ALL`, so a function beside it needs no change to `rules/mod.rs`.
- **`Completion`** in `crates/htmlspec-cli/src/commands.rs` counts `todo`, `in_progress`, `done`,
  `dropped`, and `undeclared`, and carries `open`, filled in one loop in `completion()`.
  `total()`, `resolved()`, and `state()` derive from them. `list` renders
  `{done}/{total}`, the state, and the predecessor, each column widened to its widest member and the
  row trimmed at the end. Its JSON puts the per-state counts under `tasks`, with `state` and `total`
  beside them. `status` renders `N of T task(s) done or dropped`, and its JSON is
  `tasks: {total, open, openTasks}`. Archival's `open_tasks` is `completion(directory)`'s `open`.
- **The tasks template has two example tasks**, `task-example` (`todo`) and `task-dropped-example`
  (`dropped`, with its reason). `htmlspec_assets::example_identifiers()` derives every identifier
  ending `-example` from the compiled templates, and
  `every_identifier_the_templates_write_is_reported` iterates over it, so `task-deviation-example`
  will be covered with no code change.
- **The name is free.** `grep -rni deviation openspec/specs crates assets/template-*.html` prints
  nothing.
- **A verification's `data-outcome`** is `passed`, `failed`, `blocked`, or `not-run`, in the schema's
  `definitions/verification/properties/outcome`.
- **The four test helpers the tasks name exist**: `tasks_document` builds a tasks document from the
  template by replacing everything from its first `<li class="task"` to its last `</li>`, so a third
  example task in the template changes nothing it produces, and
  `the_report_aligns_across_names_of_differing_length`,
  `the_enumeration_and_the_completion_report_agree`, and
  `archival_derives_its_open_tasks_from_the_one_counter` are where the tasks place them.
