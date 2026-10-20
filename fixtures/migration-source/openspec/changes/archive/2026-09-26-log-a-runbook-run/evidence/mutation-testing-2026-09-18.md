# Mutation testing: ten mutations, two findings

Every new test verified to fail without the thing it pins, one mutation at a time, reverted between
each. The instruction was explicit about why: **the predecessor proved a naive test passes for the
wrong reason when several unrelated gates refuse first**, and that is exactly what happened here to
the most important test in this change.

`--offline` throughout, against `crates/htmlspec-cli/tests/run.rs` (16 tests),
`crates/htmlspec-validate/src/rules/runs.rs` (8 unit tests), `conformance/tests/runs.rs` (5), and
`crates/htmlspec-cli/src/project.rs` (13).

## The table

| # | Mutation | Failed |
| --- | --- | --- |
| 1 | `recorded_ids` drops its `has_run` filter, so a "not run" verification counts as a recorded outcome | `a_just_started_run_has_recorded_nothing`, `the_payload_names_the_record_and_the_diagnostics` |
| 2 | `refuse_a_runbook` returns `Ok(())` unconditionally | `a_records_directory_is_not_archivable`, `a_nested_project_or_repository_does_not_disable_the_guard` — **see the finding below** |
| 3 | `settle_change_states` returns immediately, so `data-change` is copied as-is | **nothing, first time round** — see the second finding |
| 4 | `record_section` is not appended, so the record copies requirements and writes no verifications | `a_record_enumerates_what_the_run_owes`, `a_run_that_recorded_its_outcomes_reports_nothing`, `a_run_that_recorded_a_failure_reports_nothing` |
| 5 | the `is_a_name` check on the runbook and record names is skipped | `a_record_name_carrying_a_path_is_refused` |
| 6 | the existing-record check is skipped, so `run` overwrites | `starting_a_run_does_not_overwrite_a_record` |
| 7 | `units_of` strips `<code>` and `<strong>` from each cloned unit | `a_record_carries_the_procedure_and_the_copied_requirements` |
| 8 | `conformance` reports only the first short run | `a_just_started_run_has_recorded_nothing`, `the_payload_names_the_record_and_the_diagnostics` |
| 9 | `conformance` returns an empty vector | 3 of the rule's 8 unit tests, 3 of the 5 fixture tests, 2 integration tests |
| 10 | `ProjectRoot::runs` returns the runbook's own directory | `a_runs_directory_lives_beneath_its_runbook` |

Mutation 9 is worth reading as a column rather than a cell: the rule's own tests, the fixture pairs,
and the end-to-end command each failed, and the three that *didn't* in each suite are the ones whose
subject is silence — a runbook with no runs, one stating nothing, and a conforming run. Those pass
under a rule that reports nothing, correctly, which is why they cannot be the whole of the coverage.

## Finding 1: the guard test failed for the wrong reason

**Mutation 2 failed `a_records_directory_is_not_archivable`, and the failure message was not about
the guard:**

```
the refusal by the relative path does not name what the directory is:
htmlspec: htmlspec/runbooks/release/runs has no specification.html. Archival produces it
  from the completed change …
```

`runs/` holds run records and no `specification.html`, so **the absent-specification gate refuses it
whatever the guard does.** The test would have passed against no guard at all — it was asserting that
*something* refused, which is the shape the predecessor's evidence records at
`archival-destroyed-a-runbook-2026-09-17.md`: *"A test that cannot fail for the reason it was written
is worse than no test."* One directory down, same trap, and the only reason I found it is that the
instruction said to mutate rather than to assume.

**The route that can actually destroy a record is the runbook, not `runs/`.** The runbook holds both
documents and real content, so it clears all four gates and reaches the move — and archival renames
the whole directory, taking `runs/` and every record inside it. So the test now:

- probes the **runbook first**, and the three `runs/` spellings after it;
- asserts the record is still on disk **after each route** rather than once at the end, so a route
  that destroys it fails naming itself;
- orders the assertions so the file check comes before the message check, because a destroyed record
  is the finding and a wrong message is a detail.

With the mutation re-applied, it now fails at the real assertion:

```
assertion `left != right` failed: archival accepted the runbook holding the records:
htmlspec/runbooks/release/specification.html
```

`a_nested_project_or_repository_does_not_disable_the_guard` had the same defect for the same reason
and was rewritten the same way: the nested `htmlspec/` and `.git` are planted inside `runs/`, which is
where a record's neighbour would be, and archival is pointed at the runbook.

## Finding 2: one mutation was caught by nothing, and the reason is the fixture

**Mutation 3 — `settle_change_states` neutered, so a runbook's `data-change="added"` is copied
straight into the record — left all fifteen tests green.**

The cause is the test helper. `a_project_with_a_runbook` rewrites the template's `added` into
`unchanged` when it builds the fixture runbook, because otherwise the *runbook* draws
`review/stale-verification` and confounds every test about the record. So the copy had nothing to
settle, and the assertion "the record carries no `added`" was asserting a property of the fixture
rather than of the code.

**The input is not contrived, which is what makes the gap real.** `htmlspec new document` writes
`added` on every unit and cannot know it is writing a runbook —
`runbook-authority` states that explicitly and
`the-canonical-documents-change-state-2026-09-17.md` measured it. So a runbook authored by hand *is*
the runbook that carries `added`, and it is precisely the input the copy exists to fix.

Closed by `a_record_declares_its_copied_units_unchanged_whatever_the_runbook_declares`, which puts
the runbook back into the state the templates actually write and then asserts three things: no `added`
survives, every `data-change=` in the record reads `unchanged` (a count comparison, so a scenario left
`added` inside a settled requirement fails — the half-fixed state the previous change shipped and a
review caught), and validation reports no `review/stale-verification`. Under mutation 3 it fails:

```
the record copied the runbook's `added` state, so its verifications attest to obligations the
record itself declares do not exist yet
```

and under nothing else.

## What this says about the two findings together

Both are the same defect at different scales: **a test whose passing depended on something other
than the code it named.** The guard test depended on an unrelated gate; the change-state assertion
depended on the fixture's own preparation. Neither is visible by reading the test, and both are
visible in one command.

The predecessor found one such case per round by the same method. This change found two, and the
count is not the point — the point is that the method found them and reading did not.
